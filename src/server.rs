use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::{
    Json, Router,
    extract::{Path, Query, State},
    http::Method,
    routing::get,
};
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, QueryOrder, QuerySelect};
use serde::Deserialize;
use utoipa::{IntoParams, OpenApi};
use uuid::Uuid;

use crate::config::Config;
use crate::database::DatabaseConnection;
use crate::entities::player::Entity as PlayerEntity;
use crate::entities::player::Model as Player;
use crate::entities::player_stats::Column as PlayerStatsColumn;
use crate::entities::player_stats::Entity as PlayerStatsEntity;
use crate::entities::stat_categories::Column as StatCategorieColumn;
use crate::entities::stat_categories::Entity as StatCategorieEntity;
use crate::entities::stat_categories::Model as StatCategorie;
use crate::entities::{CategoryStatsResponse, PlayerStatsResponse};

const DEFAULT_LIMIT: u64 = 25;
const DEFAULT_PAGE: u64 = 1;
const DEFAULT_SORT_BY: &str = "value";
const DEFAULT_ORDER: &str = "desc";

#[derive(Debug, Deserialize, IntoParams)]
pub struct SearchParams {
    #[serde(default)]
    pub limit: Option<u64>,
    #[serde(default)]
    pub page: Option<u64>,
    #[serde(default)]
    pub sort_by: Option<String>,
    #[serde(default)]
    pub order: Option<String>,
}

#[derive(Clone)]
pub struct AppState {
    pub database_connection: DatabaseConnection,
    pub config: Config,
}

fn parse_pagination(params: &SearchParams) -> (u64, u64) {
    let limit = params.limit.unwrap_or(DEFAULT_LIMIT);
    let offset = (params.page.unwrap_or(DEFAULT_PAGE).max(DEFAULT_PAGE) - 1) * limit;
    (limit, offset)
}

fn parse_sort_order(params: &SearchParams) -> (String, String) {
    let sort_by = params
        .sort_by
        .as_ref()
        .cloned()
        .unwrap_or_else(|| DEFAULT_SORT_BY.to_string());
    let order = params
        .order
        .as_ref()
        .cloned()
        .unwrap_or_else(|| DEFAULT_ORDER.to_string());
    (sort_by, order)
}

fn apply_sorting(
    query: sea_orm::Select<PlayerStatsEntity>,
    sort_by: &str,
    order: &str,
) -> sea_orm::Select<PlayerStatsEntity> {
    if sort_by == "stat_name" {
        if order == "asc" {
            query.order_by_asc(PlayerStatsColumn::StatName)
        } else {
            query.order_by_desc(PlayerStatsColumn::StatName)
        }
    } else if order == "asc" {
        query.order_by_asc(PlayerStatsColumn::Value)
    } else {
        query.order_by_desc(PlayerStatsColumn::Value)
    }
}

#[utoipa::path(
    get,
    path="/categories",
    responses(
        (status = 200, body = Vec<StatCategorie>),
        (status = 500)
    )
)]
pub async fn categories(State(app_state): State<AppState>) -> impl IntoResponse {
    match StatCategorieEntity::find()
        .all(app_state.database_connection.as_ref())
        .await
    {
        Ok(cats) => (StatusCode::OK, Json(cats)),
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, Json(vec![])),
    }
}

#[utoipa::path(
    get,
    path = "/categories/{category}",
    params(
        ("category" = String, Path),
        ("limit" = Option<u64>, Query),
        ("page" = Option<u64>, Query),
        ("sort_by" = Option<String>, Query),
        ("order" = Option<String>, Query)
    ),
    responses(
        (status = 200, body = CategoryStatsResponse),
        (status = 404),
        (status = 500)
    )
)]
pub async fn category(
    State(app_state): State<AppState>,
    Path(categorie): Path<String>,
    Query(params): Query<SearchParams>,
) -> impl IntoResponse {
    let (limit, offset) = parse_pagination(&params);
    let (sort_by, order) = parse_sort_order(&params);

    let category = match StatCategorieEntity::find()
        .filter(StatCategorieColumn::Name.eq(&categorie))
        .one(app_state.database_connection.as_ref())
        .await
    {
        Ok(Some(c)) => c,
        _ => return StatusCode::NOT_FOUND.into_response(),
    };

    let query = apply_sorting(
        PlayerStatsEntity::find().filter(PlayerStatsColumn::StatCategoriesId.eq(category.id)),
        &sort_by,
        &order,
    );

    match query
        .limit(limit)
        .offset(offset)
        .all(app_state.database_connection.as_ref())
        .await
    {
        Ok(stats) => Json(CategoryStatsResponse { category, stats }).into_response(),
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
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
pub async fn players(State(app_state): State<AppState>) -> impl IntoResponse {
    match PlayerEntity::find()
        .all(app_state.database_connection.as_ref())
        .await
    {
        Ok(players) => (StatusCode::OK, Json(players)),
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, Json(vec![])),
    }
}

#[utoipa::path(
    get,
    path = "/players/{player_uuid}",
    params(
        ("player_uuid" = Uuid, Path),
        ("limit" = Option<u64>, Query),
        ("page" = Option<u64>, Query),
        ("sort_by" = Option<String>, Query),
        ("order" = Option<String>, Query)
    ),
    responses(
        (status = 200, body = PlayerStatsResponse),
        (status = 500)
    )
)]
pub async fn player(
    State(app_state): State<AppState>,
    Path(player_uuid): Path<Uuid>,
    Query(params): Query<SearchParams>,
) -> impl IntoResponse {
    let (limit, offset) = parse_pagination(&params);
    let (sort_by, order) = parse_sort_order(&params);
    let player_uuid_str = player_uuid.to_string();

    let query = apply_sorting(
        PlayerStatsEntity::find().filter(PlayerStatsColumn::PlayerUuid.eq(&player_uuid_str)),
        &sort_by,
        &order,
    );

    match query
        .limit(limit)
        .offset(offset)
        .all(app_state.database_connection.as_ref())
        .await
    {
        Ok(stats) => Json(PlayerStatsResponse {
            player_uuid: player_uuid_str,
            stats,
        })
        .into_response(),
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
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
