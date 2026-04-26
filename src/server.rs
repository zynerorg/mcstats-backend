use crate::config::Config;
use crate::database::DatabaseConnection;
use crate::graphql::create_schema;
use crate::graphql::AppSchema;
use async_graphql::http::GraphiQLSource;
use async_graphql_axum::GraphQL;
use axum::{http::Method, response::IntoResponse, routing::get, Router};
use log::info;
use std::net::SocketAddr;
use std::str::FromStr;
use tower_http::cors::{Any, CorsLayer};

#[derive(Clone)]
pub struct AppState {
    pub database_connection: DatabaseConnection,
    pub config: Config,
    pub schema: AppSchema,
}

pub async fn run_server(database: DatabaseConnection, config: Config) {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([Method::GET, Method::POST])
        .allow_headers(Any);

    let schema = create_schema(database.clone());

    let state = AppState {
        database_connection: database,
        config: config.clone(),
        schema,
    };

    let app = Router::new()
        .route("/", get(|| async { "Hello, from mcstats!" }))
        .route(
            "/graphql",
            get(graphql_playground).post_service(GraphQL::new(state.schema.clone())),
        )
        .with_state(state)
        .layer(cors);

    let addr = SocketAddr::from_str(&format!("0.0.0.0:{}", config.port)).unwrap();
    info!("Listening on: {}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn graphql_playground() -> impl IntoResponse {
    let html = GraphiQLSource::build().endpoint("/graphql").finish();
    axum::response::Html(html)
}
