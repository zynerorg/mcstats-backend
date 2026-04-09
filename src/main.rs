use std::env;
use std::path::PathBuf;

use axum::extract::State;
use axum::http::Method;
use axum::{Json, Router, extract::Path, extract::Query, routing::get};
use axum::{http::StatusCode, response::IntoResponse};
use clap::Parser;
use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl};
use dotenvy::dotenv;
use minecraft_stats::api_docs::ApiDoc;
use minecraft_stats::models::{Player, PlayerStats, StatCategorie};
use minecraft_stats::mojang_utils::UsernameCache;
use minecraft_stats::schema::players;
use minecraft_stats::{
    database::DatabaseConnection,
    schema::{player_stats, stat_categories},
};
use notify::RecursiveMode;
use notify_debouncer_mini::{DebouncedEventKind, new_debouncer};
use serde::Deserialize;
use tokio::sync::mpsc;
use tower_http::cors::{Any, CorsLayer};
use utoipa::IntoParams;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;
use uuid::Uuid;

const DEFAULT_LIMIT: i64 = 25;

#[derive(Debug, Deserialize, IntoParams)]
struct SearchParams {
    #[serde(default)]
    limit: Option<i64>,
    #[serde(default)]
    page: Option<i64>,
    #[serde(default)]
    sort_by: Option<String>,
    #[serde(default)]
    order: Option<String>,
}

#[derive(Clone)]
struct AppState {
    pub database_connection: DatabaseConnection,
}

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(long, default_value = "false")]
    server_only: bool,
    #[arg(long, default_value = "false")]
    sync_only: bool,
}

#[utoipa::path(
    get,
    path="/categories",
    responses(
        (status = 200, body = Vec<StatCategorie>),
        (status = 500)
    )
)]
async fn categories(State(app_state): State<AppState>) -> impl IntoResponse {
    match tokio::task::spawn_blocking({
        let db = app_state.database_connection.clone();
        move || {
            let mut conn = db.get()?;
            let categories: Vec<StatCategorie> = stat_categories::table.load(&mut conn)?;
            Ok::<Vec<StatCategorie>, Box<dyn std::error::Error + Send + Sync>>(categories)
        }
    })
    .await
    {
        Ok(Ok(categories)) => Json::<Vec<StatCategorie>>(categories).into_response(),
        _ => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}

#[utoipa::path(
    get,
    path = "/categories/{categorie}",
    params(
        ("categorie" = String, Path),
        ("limit" = Option<i64>, Query),
        ("page" = Option<i64>, Query),
        ("sort_by" = Option<String>, Query),
        ("order" = Option<String>, Query)
    ),
    responses(
        (status = 200),
        (status = 404),
        (status = 500)
    )
)]
async fn categorie(
    State(app_state): State<AppState>,
    Path(categorie): Path<String>,
    Query(params): Query<SearchParams>,
) -> impl IntoResponse {
    let limit = params.limit.unwrap_or(DEFAULT_LIMIT);
    let offset = (params.page.unwrap_or(1).max(1) - 1) * limit;
    let sort_by_owned = params
        .sort_by
        .clone()
        .unwrap_or_else(|| "value".to_string());
    let order_owned = params.order.clone().unwrap_or_else(|| "desc".to_string());
    let limit_used = limit;
    let offset_used = offset;

    match tokio::task::spawn_blocking({
        let db = app_state.database_connection.clone();
        let categorie = categorie.clone();
        move || {
            let mut conn = db.get()?;
            let categories: Vec<StatCategorie> = stat_categories::table
                .filter(stat_categories::name.eq(&categorie))
                .limit(1)
                .load(&mut conn)?;

            let category = categories.into_iter().next().ok_or("not found")?;

            let query = player_stats::table
                .filter(player_stats::stat_categories_id.eq(category.id))
                .into_boxed();
            let query = if sort_by_owned == "stat_name" {
                if order_owned == "asc" {
                    query.order(player_stats::stat_name.asc())
                } else {
                    query.order(player_stats::stat_name.desc())
                }
            } else if order_owned == "asc" {
                query.order(player_stats::value.asc())
            } else {
                query.order(player_stats::value.desc())
            };
            let stats: Vec<PlayerStats> = query
                .limit(limit_used)
                .offset(offset_used)
                .load(&mut conn)?;

            Ok((category, stats))
                as Result<
                    (StatCategorie, Vec<PlayerStats>),
                    Box<dyn std::error::Error + Send + Sync>,
                >
        }
    })
    .await
    {
        Ok(Ok((category, stats))) => {
            Json(serde_json::json!({ "category": category, "stats": stats })).into_response()
        }
        Ok(Err(_)) => StatusCode::NOT_FOUND.into_response(),
        _ => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}

#[utoipa::path(
    get,
    path = "/players",
    responses(
        (status = 200, body = Vec<Player>),
        (status = 500)
    )
)]
async fn players(State(app_state): State<AppState>) -> impl IntoResponse {
    match tokio::task::spawn_blocking({
        let db = app_state.database_connection.clone();
        move || {
            let mut conn = db.get()?;
            let players: Vec<Player> = players::table.load(&mut conn)?;
            Ok(players) as Result<Vec<Player>, Box<dyn std::error::Error + Send + Sync>>
        }
    })
    .await
    {
        Ok(Ok(players)) => Json::<Vec<Player>>(players).into_response(),
        _ => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}

#[utoipa::path(
    get,
    path = "/players/{player_uuid}",
    params(
        ("player_uuid" = Uuid, Path),
        ("limit" = Option<i64>, Query),
        ("page" = Option<i64>, Query),
        ("sort_by" = Option<String>, Query),
        ("order" = Option<String>, Query)
    ),
    responses(
        (status = 200),
        (status = 500)
    )
)]
async fn player(
    State(app_state): State<AppState>,
    Path(player_uuid): Path<Uuid>,
    Query(params): Query<SearchParams>,
) -> impl IntoResponse {
    let limit = params.limit.unwrap_or(DEFAULT_LIMIT);
    let offset = (params.page.unwrap_or(1).max(1) - 1) * limit;

    match tokio::task::spawn_blocking({
        let db = app_state.database_connection.clone();
        let player_uuid = player_uuid.to_string();
        move || {
            let mut conn = db.get()?;
            let stats: Vec<PlayerStats> = player_stats::table
                .filter(player_stats::player_uuid.eq(&player_uuid))
                .limit(limit)
                .offset(offset)
                .load(&mut conn)?;
            Ok(stats) as Result<Vec<PlayerStats>, Box<dyn std::error::Error + Send + Sync>>
        }
    })
    .await
    {
        Ok(Ok(stats)) => {
            Json(serde_json::json!({ "player_uuid": player_uuid, "stats": stats })).into_response()
        }
        _ => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}

async fn run_server(database: DatabaseConnection) {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([Method::GET, Method::POST])
        .allow_headers(Any);

    let state = AppState {
        database_connection: database,
    };

    let app = Router::new()
        .route("/", get(|| async { "Hello, World!" }))
        .route("/categories", get(categories))
        .route("/categories/{categorie}", get(categorie))
        .route("/players", get(players))
        .route("/players/{player}", get(player))
        .merge(SwaggerUi::new("/docs").url("/openapi.json", ApiDoc::openapi()))
        .with_state(state)
        .layer(cors);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:80").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

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

async fn run_syncer(database: DatabaseConnection, username_cache: UsernameCache) {
    let stats_env = env::var("WORLD_PATH").expect("WORLD_PATH must be set");
    let world_folder = PathBuf::from(&stats_env);
    let stats_folder = world_folder.join("stats");

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

#[tokio::main]
async fn main() {
    let _ = dotenv();

    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .format_timestamp_millis()
        .init();

    log::info!("Starting Minecraft Stats");

    let args = Args::parse();

    let database_url =
        env::var("DATABASE_URL").unwrap_or_else(|_| "./minecraft_stats.db".to_string());
    let stats_env = env::var("WORLD_PATH").unwrap_or_else(|_| "/world".to_string());

    log::info!("World path: {}", stats_env);
    log::info!("Database URL: {}", database_url);

    let world_folder = PathBuf::from(&stats_env);
    let usercache_path = world_folder.join("usercache.json");

    log::info!("Loading usercache from: {:?}", usercache_path);
    let username_cache =
        UsernameCache::from_usercache(&usercache_path).expect("Failed to load usercache");
    log::info!("Loaded {} players from usercache", username_cache.len());

    let database = DatabaseConnection::new(&database_url)
        .await
        .expect("Could not connect to database");

    if args.server_only {
        log::info!("Running server only");
        run_server(database).await;
    } else if args.sync_only {
        log::info!("Running syncer only");
        run_syncer(database, username_cache).await;
    } else {
        log::info!("Running both server and syncer");
        tokio::select! {
            _ = run_server(database.clone()) => {},
            _ = run_syncer(database.clone(), username_cache.clone()) => {},
        }
    }
}
