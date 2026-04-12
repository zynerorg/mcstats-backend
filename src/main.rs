use std::sync::Arc;

use clap::Parser;
use minecraft_stats::config::Config;
use minecraft_stats::database::DatabaseConnection;
use minecraft_stats::mojang_utils::UsernameCache;
use minecraft_stats::server::run_server;
use minecraft_stats::syncer;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(long, default_value = "false")]
    server_only: bool,
    #[arg(long, default_value = "false")]
    sync_only: bool,
}

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .format_timestamp_millis()
        .init();

    log::info!("Starting Minecraft Stats");

    let args = Args::parse();
    let config = Config::from_env();

    log::info!("World path: {}", config.world_folder.to_str().unwrap());
    log::info!("Database URL: {}", config.database_url);

    let username_cache = Arc::new(
        UsernameCache::from_usercache(&config.usercache_path)
            .await
            .expect("Failed to load usercache"),
    );

    let database = DatabaseConnection::new(
        &config.database_url,
        config.database_pool_size,
        config.database_concurrency_limit,
    )
    .await
    .expect("Could not connect to database");

    if args.server_only {
        log::info!("Running server only");
        run_server::run_server(database, config.clone()).await;
    } else if args.sync_only {
        log::info!("Running syncer only");
        syncer::run_syncer(database, username_cache, config.stats_folder()).await;
    } else {
        log::info!("Running both server and syncer");
        tokio::select! {
            _ = run_server::run_server(database.clone(), config.clone()) => {},
            _ = syncer::run_syncer(database.clone(), username_cache.clone(), config.stats_folder()) => {},
        }
    }
}
