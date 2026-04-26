use crate::entities::player_stats::Entity as PlayerStatsEntity;
use crate::entities::players::{
    ActiveModel as PlayerActiveModel, Column as PlayerColumn, Entity as PlayerEntity,
    Model as Player,
};
use crate::username_cache::UsernameCache;
use anyhow::{anyhow, Result};
use log::{debug, error, info};
use migration::{Migrator, MigratorTrait};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, ConnectOptions, Database, DbErr, EntityTrait, QueryFilter,
    TransactionTrait,
};
use serde::Deserialize;
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;

pub type DbPool = sea_orm::DatabaseConnection;

#[derive(Deserialize, Debug)]
pub struct StatsFile {
    pub stats: HashMap<String, HashMap<String, i64>>,
}

#[derive(Clone)]
pub struct DatabaseConnection {
    conn: Arc<DbPool>,
}

impl AsRef<DbPool> for DatabaseConnection {
    fn as_ref(&self) -> &DbPool {
        &self.conn
    }
}

impl DatabaseConnection {
    pub async fn new(url: &str) -> Result<Self> {
        info!("Connecting to database...");

        let mut opt = ConnectOptions::new(url);
        opt.sqlx_logging(false);

        let conn = Database::connect(opt).await?;
        Migrator::up(&conn, None).await?;

        info!("Database ready");

        Ok(Self {
            conn: Arc::new(conn),
        })
    }

    pub async fn insert_player(&self, player: Player) -> Result<Player> {
        debug!("Insert player {:?}", player);

        if let Some(existing) = self.find_player(&player.player_uuid).await? {
            return self.update_player(existing, &player).await;
        }

        self.create_player(&player).await
    }

    pub async fn find_player(&self, uuid: &str) -> Result<Option<Player>> {
        Ok(PlayerEntity::find()
            .filter(PlayerColumn::PlayerUuid.eq(uuid))
            .one(self.as_ref())
            .await?)
    }

    pub async fn update_player(&self, existing: Player, new: &Player) -> Result<Player> {
        let mut active: PlayerActiveModel = existing.into();
        active.name = sea_orm::Set(new.name.clone());

        let updated = active.update(self.as_ref()).await?;
        Ok(updated)
    }

    pub async fn create_player(&self, player: &Player) -> Result<Player> {
        let active = PlayerActiveModel {
            player_uuid: sea_orm::Set(player.player_uuid.clone()),
            name: sea_orm::Set(player.name.clone()),
        };

        PlayerEntity::insert(active).exec(self.as_ref()).await?;

        Ok(player.clone())
    }

    pub async fn insert_stats(&self, uuid: uuid::Uuid, stats: StatsFile) -> Result<()> {
        let uuid_str = uuid.to_string();
        let total = stats.stats.values().map(|m| m.len()).sum::<usize>();

        info!("Inserting {total} stats for {uuid}");

        let pool = self.as_ref();

        pool.transaction::<_, (), DbErr>(|txn| {
            Box::pin(async move {
                for (category, map) in stats.stats {
                    for (name, value) in map {
                        Self::insert_stat_row(txn, &uuid_str, &category, name.clone(), value)
                            .await
                            .map_err(|e: anyhow::Error| DbErr::Custom(e.to_string()))?;
                    }
                }

                Ok(())
            })
        })
        .await?;

        Ok(())
    }

    async fn insert_stat_row(
        txn: &sea_orm::DatabaseTransaction,
        uuid: &str,
        category: &str,
        name: String,
        value: i64,
    ) -> Result<()> {
        let model = crate::entities::player_stats::ActiveModel {
            player_uuid: sea_orm::Set(uuid.to_string()),
            category: sea_orm::Set(category.to_string()),
            value_name: sea_orm::Set(name),
            value: sea_orm::Set(value),
        };

        match PlayerStatsEntity::insert(model).exec(txn).await {
            Ok(_) => Ok(()),
            Err(e) if e.to_string().contains("UNIQUE") => Ok(()),
            Err(e) => Err(e.into()),
        }
    }

    pub async fn populate(&self, stats_folder: &Path, cache: Arc<UsernameCache>) -> Result<()> {
        let files = self.collect_json_files(stats_folder).await?;

        for path in files {
            if let Err(e) = self.process_stats_file(&path, cache.clone()).await {
                error!("File processing error: {e}");
            }
        }

        Ok(())
    }

    async fn collect_json_files(&self, folder: &Path) -> Result<Vec<std::path::PathBuf>> {
        let mut entries = tokio::fs::read_dir(folder).await?;
        let mut files = Vec::new();

        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) == Some("json") {
                files.push(path);
            }
        }

        Ok(files)
    }

    fn extract_uuid(&self, path: &Path) -> Result<uuid::Uuid> {
        let stem = path
            .file_stem()
            .and_then(|s| s.to_str())
            .ok_or_else(|| anyhow!("Invalid file name"))?;

        Ok(uuid::Uuid::parse_str(stem)?)
    }

    async fn load_stats(&self, path: &Path) -> Result<StatsFile> {
        let data = tokio::fs::read_to_string(path).await?;
        Ok(serde_json::from_str(&data)?)
    }

    pub async fn process_stats_file(&self, path: &Path, cache: Arc<UsernameCache>) -> Result<()> {
        let uuid = self.extract_uuid(path)?;
        let stats_data = self.load_stats(path).await?;

        let name = cache
            .uuid_to_username(&uuid)
            .await
            .unwrap_or_else(|| "Unknown".to_string());

        self.insert_player(Player {
            player_uuid: uuid.to_string(),
            name,
        })
        .await?;

        self.insert_stats(uuid, stats_data).await?;

        Ok(())
    }
}
