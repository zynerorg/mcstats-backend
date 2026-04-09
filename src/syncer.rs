use std::path::PathBuf;

use notify::RecursiveMode;
use notify_debouncer_mini::{DebouncedEventKind, new_debouncer};
use tokio::sync::mpsc;

use crate::database::DatabaseConnection;
use crate::mojang_utils::UsernameCache;

async fn handle_stats_file_change(
    db: &DatabaseConnection,
    path: &PathBuf,
    username_cache: &UsernameCache,
) {
    if let Err(e) = db.process_stats_file(path, username_cache).await {
        log::error!("Error processing stats file {:?}: {:?}", path, e);
    } else {
        log::info!("Successfully synced stats for file: {:?}", path);
    }
}

pub async fn run_syncer(
    database: DatabaseConnection,
    username_cache: UsernameCache,
    stats_folder: PathBuf,
) {
    log::info!("Starting initial population of database from stats folder...");
    database
        .populate(&stats_folder, &username_cache)
        .await
        .expect("Initial population failed");
    log::info!("Initial database population complete");

    let db = database.clone();
    let stats_path = stats_folder.clone();
    let cache = username_cache.clone();

    let (tx, mut rx) = mpsc::channel(100);

    let mut debouncer = new_debouncer(
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
    )
    .expect("Failed to create debouncer");

    debouncer
        .watcher()
        .watch(&stats_path, RecursiveMode::Recursive)
        .expect("Failed to watch stats folder");

    log::info!("Watching for changes in {:?}", stats_path);

    while let Some(event) = rx.recv().await {
        let path = event.path;
        if path.extension().is_some_and(|ext| ext == "json") {
            log::info!("Detected change in: {:?}", path);
            handle_stats_file_change(&db, &path, &cache).await;
        }
    }
}
