use anyhow::{Result, anyhow};
use futures::StreamExt;
use log::{debug, info};
use sea_orm::{ActiveModelTrait, ColumnTrait, Database, EntityTrait, QueryFilter};
use std::path::Path;
use std::sync::Arc;

use crate::entities::player::ActiveModel as PlayerActiveModel;
use crate::entities::player::Column as PlayerColumn;
use crate::entities::player::Entity as PlayerEntity;
use crate::entities::player::Model as Player;
use crate::entities::player_stats::ActiveModel as PlayerStatsActiveModel;
use crate::entities::player_stats::Column as PlayerStatsColumn;
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
}

impl DatabaseConnection {
    pub async fn new(url: &str) -> Result<Self> {
        info!("Establishing database connection...");
        let conn = Database::connect(url).await.map_err(|e| anyhow!(e))?;
        info!("Database connection established");

        info!("Running database migrations...");
        Migrator::up(&conn, None).await.map_err(|e| anyhow!(e))?;
        info!("Database migrations complete");

        Ok(Self {
            conn: Arc::new(conn),
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

        for (category_name, stat_map) in stats.stats {
            let category_id = self.insert_category(&category_name).await?;

            for (stat_nm, val) in stat_map {
                let stat_nm_owned = stat_nm.clone();
                let player_stat = PlayerStatsActiveModel {
                    player_uuid: sea_orm::Set(uuid_str.clone()),
                    stat_categories_id: sea_orm::Set(category_id),
                    stat_name: sea_orm::Set(stat_nm_owned),
                    value: sea_orm::Set(val),
                };

                debug!("Inserting stat: {:?}", player_stat);

                let existing = PlayerStatsEntity::find()
                    .filter(PlayerStatsColumn::PlayerUuid.eq(&uuid_str))
                    .filter(PlayerStatsColumn::StatCategoriesId.eq(category_id))
                    .filter(PlayerStatsColumn::StatName.eq(&stat_nm))
                    .one(&*self.conn)
                    .await?;

                if let Some(existing) = existing {
                    let mut active: PlayerStatsActiveModel = existing.into();
                    active.value = sea_orm::Set(val);
                    active.update(&*self.conn).await.map_err(|e| anyhow!(e))?;
                } else {
                    PlayerStatsEntity::insert(player_stat)
                        .exec(&*self.conn)
                        .await
                        .map_err(|e| anyhow!(e))?;
                }
            }
        }

        info!(
            "Successfully inserted/updated {} stats for player {}",
            stat_count, uuid
        );
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
        let mut tasks = futures::stream::FuturesUnordered::new();

        while let Some(entry) = dir_entries.next_entry().await? {
            let path = entry.path();

            if path.extension().map_or(false, |ext| ext == "json") {
                let db = self.clone();
                let cache = username_cache.clone();
                tasks.push(async move { db.process_stats_file(&path, &cache).await });
            }
        }

        while let Some(result) = tasks.next().await {
            result?;
        }

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
