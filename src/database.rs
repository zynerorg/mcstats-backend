use anyhow::{anyhow, Result};
use diesel::SelectableHelper;
use diesel_async::{AsyncConnection, AsyncPgConnection, RunQueryDsl};
use log::debug;
use std::path::Path;
use uuid::Uuid;

use crate::models::{NewPlayer, Player};
use crate::mojang_utils::MojangCache;

pub async fn establish_connection(url: &str) -> AsyncPgConnection {
    AsyncPgConnection::establish(url)
        .await
        .unwrap_or_else(|_| panic!("Error connecting to {}", url))
}

async fn insert_player(database: &mut AsyncPgConnection, player: NewPlayer) -> Result<Player> {
    use crate::schema::players;
    debug!("Inserted player: {:?}", player);

    diesel::insert_into(players::table)
        .values(&player)
        .returning(Player::as_returning())
        .get_result(database)
        .await
        .map_err(|e| anyhow!(e))
}

pub async fn populate_database(
    database: &mut AsyncPgConnection,
    stats_folder: &Path,
    mojang_cache: &MojangCache,
) {
    debug!("Using stats folder: {:?}", stats_folder);

    for entry in stats_folder
        .read_dir()
        .expect("Failed to read stats directory")
    {
        let entry = entry.expect("Failed to read entry");
        let path = entry.path();

        if path.extension().map_or(false, |ext| ext == "json") {
            let file_stem = path
                .file_stem()
                .and_then(|s| s.to_str())
                .expect("Failed to get file stem");

            let player_uuid = Uuid::parse_str(file_stem)
                .expect(&format!("Invalid UUID in filename: {}", file_stem));

            let player_name = mojang_cache
                .uuid_to_username(&player_uuid)
                .unwrap_or_else(|| "Unknown".to_string());

            if let Err(e) = insert_player(
                database,
                NewPlayer {
                    player_uuid,
                    name: player_name,
                },
            )
            .await
            {
                log::error!("Error inserting player: {:?}", e);
            }
        }
    }
}
