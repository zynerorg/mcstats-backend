use anyhow::{anyhow, Result};
use diesel::prelude::*;
use diesel::SqliteConnection;
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
use futures::StreamExt;
use log::{debug, info};
use std::path::Path;
use std::sync::Arc;
use tokio::fs;
use uuid::Uuid;

use crate::models::{Player, PlayerStats, StatsFile};
use crate::mojang_utils::UsernameCache;

const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations");

#[derive(Clone)]
pub struct DatabaseConnection {
    url: String,
    db_lock: Arc<tokio::sync::Mutex<()>>,
}

impl DatabaseConnection {
    pub async fn new(url: &str) -> Result<Self> {
        info!("Establishing database connection...");
        let mut conn = SqliteConnection::establish(url)?;
        
        conn.run_pending_migrations(MIGRATIONS)
            .map_err(|e| anyhow!("Migration error: {}", e))?;
        
        info!("Database tables initialized");
        drop(conn);
        Ok(Self {
            url: url.to_string(),
            db_lock: Arc::new(tokio::sync::Mutex::new(())),
        })
    }

    pub fn get(&self) -> Result<SqliteConnection> {
        SqliteConnection::establish(&self.url).map_err(|e| anyhow!(e))
    }

    pub async fn insert_player(&self, player: Player) -> Result<Player> {
        use crate::schema::players::dsl::*;
        debug!("Inserting player: {:?}", player);

        let _lock = self.db_lock.lock().await;
        let mut conn = self.get()?;
        
        diesel::insert_into(players)
            .values(&player)
            .on_conflict(player_uuid)
            .do_update()
            .set(name.eq(&player.name))
            .execute(&mut conn)?;
        
        players
            .filter(player_uuid.eq(&player.player_uuid))
            .get_result(&mut conn)
            .map_err(|e| anyhow!(e))
    }

    pub async fn insert_stats(&self, uuid: Uuid, stats: StatsFile) -> Result<()> {
        use crate::schema::player_stats::columns;
        use crate::schema::player_stats::dsl::*;

        let uuid_str = uuid.to_string();

        let _lock = self.db_lock.lock().await;
        let mut conn = self.get()?;
        
        let stat_count = stats.stats.values().map(|m| m.len()).sum::<usize>();
        info!("Inserting {} stats for player {}", stat_count, uuid);

        for (category_name, stat_map) in stats.stats {
            let category_id = self.insert_category(&mut conn, &category_name)?;

            for (stat_nm, val) in stat_map {
                let player_stat = PlayerStats {
                    player_uuid: uuid_str.clone(),
                    stat_categories_id: category_id,
                    stat_name: stat_nm,
                    value: val,
                };

                diesel::insert_into(player_stats)
                    .values(&player_stat)
                    .on_conflict((
                        columns::player_uuid,
                        columns::stat_categories_id,
                        columns::stat_name,
                    ))
                    .do_update()
                    .set(columns::value.eq(val))
                    .execute(&mut conn)
                    .map_err(|e| anyhow!(e))?;
            }
        }

        info!(
            "Successfully inserted/updated {} stats for player {}",
            stat_count, uuid
        );
        Ok(())
    }

    pub fn insert_category(&self, database: &mut SqliteConnection, category_name: &str) -> Result<i32> {
        use crate::schema::stat_categories::columns;
        use crate::schema::stat_categories::dsl::*;

        if let Ok(existing_id) = stat_categories
            .filter(columns::name.eq(category_name))
            .select(columns::id)
            .get_result::<i32>(database)
        {
            return Ok(existing_id);
        }

        diesel::insert_into(stat_categories)
            .values(columns::name.eq(category_name))
            .on_conflict(columns::name)
            .do_update()
            .set(columns::name.eq(category_name))
            .execute(database)
            .map_err(|e| anyhow!(e))?;

        let new_id: i32 = stat_categories
            .filter(columns::name.eq(category_name))
            .select(columns::id)
            .get_result(database)
            .map_err(|e| anyhow!(e))?;

        debug!(
            "Inserted stat category: {} with id {}",
            category_name, new_id
        );
        Ok(new_id)
    }

    pub async fn populate(
        &self,
        stats_folder: &Path,
        username_cache: &UsernameCache,
    ) -> Result<()> {
        let mut dir_entries = fs::read_dir(stats_folder).await?;
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

        let player_uuid = Uuid::parse_str(file_stem)?;

        let stats_content = fs::read_to_string(path).await?;
        let player_stats: StatsFile = serde_json::from_str(&stats_content)?;

        let player_name = username_cache
            .uuid_to_username(&player_uuid)
            .unwrap_or_else(|| "Unknown".to_string());

        self.insert_player(Player {
            player_uuid: player_uuid.to_string(),
            name: player_name,
        }).await?;

        self.insert_stats(player_uuid, player_stats).await?;

        Ok(())
    }
}
