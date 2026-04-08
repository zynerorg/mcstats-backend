use std::{env, path::PathBuf};

use dotenvy::dotenv;
use notify::RecursiveMode;
use notify_debouncer_mini::{new_debouncer, DebouncedEventKind};
use uuid::Uuid;

mod database;
mod models;
mod mojang_utils;
mod schema;

use crate::database::{establish_connection, insert_stats, populate_database};
use crate::mojang_utils::MojangCache;

async fn handle_stats_file_change(
    database: &mut diesel_async::AsyncPgConnection,
    path: &PathBuf,
    mojang_cache: &MojangCache,
) {
    let file_stem = match path.file_stem().and_then(|s| s.to_str()) {
        Some(s) => s,
        None => {
            log::error!("Failed to get file stem for {:?}", path);
            return;
        }
    };

    let player_uuid = match Uuid::parse_str(file_stem) {
        Ok(uuid) => uuid,
        Err(e) => {
            log::error!("Invalid UUID in filename {}: {:?}", file_stem, e);
            return;
        }
    };

    log::info!("Processing stats file for player: {}", player_uuid);

    let stats_content = match tokio::fs::read_to_string(path).await {
        Ok(c) => c,
        Err(e) => {
            log::error!("Failed to read stats file: {:?}", e);
            return;
        }
    };

    let player_stats: models::StatsFile = match serde_json::from_str(&stats_content) {
        Ok(s) => s,
        Err(e) => {
            log::error!("Failed to parse stats file: {:?}", e);
            return;
        }
    };

    let player_name = mojang_cache
        .uuid_to_username(&player_uuid)
        .unwrap_or_else(|| "Unknown".to_string());

    log::info!("Updating player: {} ({})", player_name, player_uuid);

    if let Err(e) = database::insert_player(
        database,
        models::Player {
            player_uuid,
            name: player_name.clone(),
        },
    )
    .await
    {
        log::error!("Error inserting player {}: {:?}", player_name, e);
    }

    match insert_stats(database, player_uuid, player_stats).await {
        Ok(_) => log::info!(
            "Successfully synced stats for player: {} ({})",
            player_name,
            player_uuid
        ),
        Err(e) => log::error!("Error inserting stats for player {}: {:?}", player_name, e),
    }
}

#[tokio::main]
async fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .format_timestamp_millis()
        .init();

    log::info!("Starting Minecraft Stats Sync");

    let _ = dotenv();
    let stats_env = env::var("WORLD_PATH").expect("WORLD_PATH must be set");
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    log::info!("World path: {}", stats_env);
    log::info!("Database URL: {}", database_url);

    let world_folder = PathBuf::from(&stats_env);
    let usercache_path = world_folder.join("usercache.json");
    let stats_folder = world_folder.join("stats");

    log::info!("Loading usercache from: {:?}", usercache_path);
    let mojang_cache =
        MojangCache::from_usercache(&usercache_path).expect("Failed to load usercache.json");
    log::info!("Loaded {} players from usercache", mojang_cache.len());

    log::info!("Connecting to database...");
    let mut database_connection = establish_connection(&database_url).await;
    log::info!("Connected to database");

    log::info!("Starting initial population of database from stats folder...");
    match populate_database(&mut database_connection, &stats_folder, &mojang_cache).await {
        Ok(_) => log::info!("Initial database population complete"),
        Err(e) => log::error!("Error during initial population: {:?}", e),
    }

    let db_url = database_url.clone();
    let stats_path = stats_folder.clone();
    let cache = mojang_cache.clone();

    tokio::spawn(async move {
        let mut database_connection = establish_connection(&db_url).await;

        let (tx, mut rx) = tokio::sync::mpsc::channel(100);

        let mut debouncer = match new_debouncer(
            std::time::Duration::from_millis(200),
            move |res: Result<Vec<notify_debouncer_mini::DebouncedEvent>, notify::Error>| {
                if let Ok(events) = res {
                    for event in events {
                        if event.kind == DebouncedEventKind::Any {
                            let _ = tx.blocking_send(event);
                        }
                    }
                }
            },
        ) {
            Ok(d) => d,
            Err(e) => {
                log::error!("Failed to create debouncer: {:?}", e);
                return;
            }
        };

        if let Err(e) = debouncer
            .watcher()
            .watch(&stats_path, RecursiveMode::Recursive)
        {
            log::error!("Failed to watch stats folder: {:?}", e);
            return;
        }

        log::info!("Watching for changes in {:?}", stats_path);

        while let Some(event) = rx.recv().await {
            let path = event.path;
            if path.extension().map_or(false, |ext| ext == "json") {
                log::info!("Detected change in: {:?}", path);
                handle_stats_file_change(&mut database_connection, &path, &cache).await;
            }
        }
    });

    log::info!("Application ready and running");
    loop {
        tokio::time::sleep(tokio::time::Duration::from_secs(60)).await;
    }
}
