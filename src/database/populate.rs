use super::DatabaseConnection;
use crate::database::StatsFile;
use crate::entities::players::Model as Player;
use crate::mojang_utils::UsernameCache;

use anyhow::{Result, anyhow};
use futures::StreamExt;
use log::error;

use std::path::Path;
use std::sync::Arc;

impl DatabaseConnection {
    pub async fn populate(&self, stats_folder: &Path, cache: Arc<UsernameCache>) -> Result<()> {
        let files = self.collect_json_files(stats_folder).await?;

        let db = self.clone();

        futures::stream::iter(files)
            .map(|path| {
                let db = db.clone();
                let cache = cache.clone();

                async move { db.process_stats_file(&path, cache).await }
            })
            .buffer_unordered(self.concurrency_limit())
            .for_each(|r| async {
                if let Err(e) = r {
                    error!("File processing error: {e}");
                }
            })
            .await;

        Ok(())
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
}
