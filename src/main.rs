use std::sync::Arc;

use mcstats_backend::config::Config;
use mcstats_backend::database::DatabaseConnection;
use mcstats_backend::server;
use mcstats_backend::syncer;
use mcstats_backend::username_cache::UsernameCache;

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .format_timestamp_millis()
        .init();

    log::info!("Starting Minecraft Stats");

    let config = Config::from_env();

    log::info!("World path: {}", config.world_folder.to_str().unwrap());
    log::info!("Database URL: {}", config.database_url);

    let username_cache = Arc::new(
        UsernameCache::from_usercache(&config.usercache_path)
            .await
            .expect("Failed to load usercache"),
    );

    let database = DatabaseConnection::new(&config.database_url)
        .await
        .expect("Could not connect to database");

    tokio::select! {
        _ = server::run_server(database.clone(), config.clone()) => {},
        _ = syncer::run_syncer(database.clone(), username_cache.clone(), config.stats_folder()) => {},
    }
}
