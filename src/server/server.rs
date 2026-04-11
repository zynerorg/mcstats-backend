use axum::{Router, http::Method, routing::get};
use utoipa::OpenApi;

use crate::config::Config;
use crate::database::DatabaseConnection;
use crate::server::categories::{categories, category};
use crate::server::players::{player, players};

#[derive(Clone)]
pub struct AppState {
    pub database_connection: DatabaseConnection,
    pub config: Config,
}

pub async fn run_server(database: DatabaseConnection, config: Config) {
    use tower_http::cors::{Any, CorsLayer};

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([Method::GET, Method::POST])
        .allow_headers(Any);

    let state = AppState {
        database_connection: database,
        config: config.clone(),
    };

    let app = Router::new()
        .route("/", get(|| async { "Hello, from mcstats!" }))
        .route("/categories", get(categories))
        .route("/categories/{categorie}", get(category))
        .route("/players", get(players))
        .route("/players/{player}", get(player))
        .merge(
            utoipa_swagger_ui::SwaggerUi::new("/docs")
                .url("/openapi.json", crate::api_docs::ApiDoc::openapi()),
        )
        .with_state(state)
        .layer(cors);

    let addr = format!("0.0.0.0:{}", config.port);
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
