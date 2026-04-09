use anyhow::{Result, anyhow};
use futures::StreamExt;
use log::{debug, info};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, ConnectOptions, Database, EntityTrait, QueryFilter,
    TransactionTrait,
};
use std::path::Path;
use std::sync::Arc;

use crate::entities::player::ActiveModel as PlayerActiveModel;
use crate::entities::player::Column as PlayerColumn;
use crate::entities::player::Entity as PlayerEntity;
use crate::entities::player::Model as Player;
use crate::entities::player_stats::ActiveModel as PlayerStatsActiveModel;
use crate::entities::player_stats::Entity as PlayerStatsEntity;
use crate::entities::stat_categorie::ActiveModel as StatCategorieActiveModel;
use crate::entities::stat_categorie::Column as StatCategorieColumn;
use crate::entities::stat_categorie::Entity as StatCategorieEntity;
use crate::models::StatsFile;
use crate::mojang_utils::UsernameCache;
use migration::{Migrator, MigratorTrait};

pub type DbPool = sea_orm::DatabaseConnection;

#[derive(Clone)]
pub struct DatabaseConnection {
    conn: Arc<DbPool>,
    concurrency_limit: usize,
}

impl DatabaseConnection {
    pub async fn new(url: &str, pool_size: u32, concurrency_limit: usize) -> Result<Self> {
        info!("Establishing database connection...");
        let mut opt = ConnectOptions::new(url);
        opt.max_connections(pool_size);
        let conn = Database::connect(opt).await.map_err(|e| anyhow!(e))?;
        info!("Database connection established (pool_size: {})", pool_size);

        info!("Running database migrations...");
        Migrator::up(&conn, None).await.map_err(|e| anyhow!(e))?;
        info!("Database migrations complete");

        Ok(Self {
            conn: Arc::new(conn),
            concurrency_limit,
        })
    }

    pub fn as_ref(&self) -> &DbPool {
        &self.conn
    }

    pub async fn insert_player(&self, player: Player) -> Result<Player> {
        debug!("Inserting player: {:?}", player);

        if let Ok(Some(existing)) = PlayerEntity::find()
            .filter(PlayerColumn::PlayerUuid.eq(&player.player_uuid))
            .one(&*self.conn)
            .await
        {
            let mut active: PlayerActiveModel = existing.into();
            active.name = sea_orm::Set(player.name.clone());
            let updated = active.update(&*self.conn).await.map_err(|e| anyhow!(e))?;
            return Ok(updated);
        }

        let active = PlayerActiveModel {
            player_uuid: sea_orm::Set(player.player_uuid.clone()),
            name: sea_orm::Set(player.name.clone()),
        };
        PlayerEntity::insert(active)
            .exec(&*self.conn)
            .await
            .map_err(|e| anyhow!(e))?;

        Ok(Player {
            player_uuid: player.player_uuid,
            name: player.name,
        })
    }

    pub async fn insert_stats(&self, uuid: uuid::Uuid, stats: StatsFile) -> Result<()> {
        let uuid_str = uuid.to_string();

        let stat_count = stats.stats.values().map(|m| m.len()).sum::<usize>();
        info!("Inserting {} stats for player {}", stat_count, uuid);

        let conn = self.conn.as_ref();

        conn.transaction::<_, (), sea_orm::DbErr>(|txn| {
            Box::pin(async move {
                for (category_name, stat_map) in stats.stats {
                    let category_id = if let Ok(Some(existing)) = StatCategorieEntity::find()
                        .filter(StatCategorieColumn::Name.eq(&category_name))
                        .one(txn)
                        .await
                    {
                        existing.id
                    } else {
                        let active = StatCategorieActiveModel {
                            id: sea_orm::NotSet,
                            name: sea_orm::Set(category_name.clone()),
                        };
                        let result = StatCategorieEntity::insert(active).exec(txn).await;
                        match result {
                            Ok(inserted) => inserted.last_insert_id,
                            Err(e) => {
                                if e.to_string().contains("UNIQUE constraint failed") {
                                    StatCategorieEntity::find()
                                        .filter(StatCategorieColumn::Name.eq(&category_name))
                                        .one(txn)
                                        .await
                                        .map_err(|e| sea_orm::DbErr::Custom(e.to_string()))?
                                        .map(|c| c.id)
                                        .unwrap_or_else(|| {
                                            panic!("Failed to find category after insert failed");
                                        })
                                } else {
                                    return Err(e);
                                }
                            }
                        }
                    };

                    for (stat_nm, val) in stat_map {
                        let stat_nm_owned = stat_nm.clone();
                        let player_stat = PlayerStatsActiveModel {
                            player_uuid: sea_orm::Set(uuid_str.clone()),
                            stat_categories_id: sea_orm::Set(category_id),
                            stat_name: sea_orm::Set(stat_nm_owned),
                            value: sea_orm::Set(val),
                        };

                        if let Err(e) = PlayerStatsEntity::insert(player_stat).exec(txn).await {
                            if !e.to_string().contains("UNIQUE constraint failed") {
                                return Err(e);
                            }
                        }
                    }
                }

                info!(
                    "Successfully inserted {} stats for player {}",
                    stat_count, uuid
                );
                Ok(())
            })
        })
        .await
        .map_err(|e| anyhow!(e))?;

        Ok(())
    }

    async fn insert_category(&self, category_name: &str) -> Result<i32> {
        if let Ok(Some(existing)) = StatCategorieEntity::find()
            .filter(StatCategorieColumn::Name.eq(category_name))
            .one(&*self.conn)
            .await
        {
            return Ok(existing.id);
        }

        let active = StatCategorieActiveModel {
            id: sea_orm::NotSet,
            name: sea_orm::Set(category_name.to_string()),
        };

        let result = StatCategorieEntity::insert(active).exec(&*self.conn).await;

        match result {
            Ok(inserted) => {
                let new_id = inserted.last_insert_id;
                debug!(
                    "Inserted stat category: {} with id {}",
                    category_name, new_id
                );
                Ok(new_id)
            }
            Err(e) => {
                if e.to_string().contains("UNIQUE constraint failed") {
                    if let Ok(Some(existing)) = StatCategorieEntity::find()
                        .filter(StatCategorieColumn::Name.eq(category_name))
                        .one(&*self.conn)
                        .await
                    {
                        return Ok(existing.id);
                    }
                }
                Err(e).map_err(|e| anyhow!(e))
            }
        }
    }

    pub async fn populate(
        &self,
        stats_folder: &Path,
        username_cache: &UsernameCache,
    ) -> Result<()> {
        let mut dir_entries = tokio::fs::read_dir(stats_folder).await?;
        let mut paths = Vec::new();

        while let Some(entry) = dir_entries.next_entry().await? {
            let path = entry.path();
            if path.extension().map_or(false, |ext| ext == "json") {
                paths.push(path);
            }
        }

        let db = self.clone();
        let cache = username_cache.clone();

        futures::stream::iter(paths)
            .map(|path| {
                let db = db.clone();
                let cache = cache.clone();
                async move { db.process_stats_file(&path, &cache).await }
            })
            .buffer_unordered(self.concurrency_limit)
            .for_each(|result| async {
                if let Err(e) = result {
                    log::error!("Error processing stats file: {}", e);
                }
            })
            .await;

        Ok(())
    }

    pub async fn process_stats_file(
        &self,
        path: &Path,
        username_cache: &UsernameCache,
    ) -> Result<()> {
        let file_stem = path
            .file_stem()
            .and_then(|s| s.to_str())
            .ok_or_else(|| anyhow!("Failed to get file stem"))?;

        let player_uuid = uuid::Uuid::parse_str(file_stem)?;

        let stats_content = tokio::fs::read_to_string(path).await?;
        let player_stats: StatsFile = serde_json::from_str(&stats_content)?;

        let player_name = username_cache
            .uuid_to_username(&player_uuid)
            .unwrap_or_else(|| "Unknown".to_string());

        self.insert_player(Player {
            player_uuid: player_uuid.to_string(),
            name: player_name,
        })
        .await?;

        self.insert_stats(player_uuid, player_stats).await?;

        Ok(())
    }
}
