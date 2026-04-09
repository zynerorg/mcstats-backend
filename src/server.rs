use axum::{
    Json, Router,
    extract::{Path, Query, State},
    http::Method,
    routing::get,
};
use axum::{http::StatusCode, response::IntoResponse};
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, QueryOrder, QuerySelect};
use serde::Deserialize;
use utoipa::{IntoParams, OpenApi};
use uuid::Uuid;

use crate::database::DatabaseConnection;
use crate::entities::player::Entity as PlayerEntity;
use crate::entities::player::Model as Player;
use crate::entities::player_stats::Column as PlayerStatsColumn;
use crate::entities::player_stats::Entity as PlayerStatsEntity;
use crate::entities::stat_categories::Column as StatCategorieColumn;
use crate::entities::stat_categories::Entity as StatCategorieEntity;
use crate::entities::stat_categories::Model as StatCategorie;

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
        Ok(category) => Json::<Vec<StatCategorie>>(category).into_response(),
        _ => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
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
        (status = 200),
        (status = 404),
        (status = 500)
    )
)]
pub async fn category(
    State(app_state): State<AppState>,
    Path(categorie): Path<String>,
    Query(params): Query<SearchParams>,
) -> impl IntoResponse {
    let limit = params.limit.unwrap_or(25);
    let offset = (params.page.unwrap_or(1).max(1) - 1) * limit;
    let sort_by_owned = params
        .sort_by
        .clone()
        .unwrap_or_else(|| "value".to_string());
    let order_owned = params.order.clone().unwrap_or_else(|| "desc".to_string());

    let category = match StatCategorieEntity::find()
        .filter(StatCategorieColumn::Name.eq(&categorie))
        .one(app_state.database_connection.as_ref())
        .await
    {
        Ok(Some(c)) => c,
        _ => return StatusCode::NOT_FOUND.into_response(),
    };

    let mut query =
        PlayerStatsEntity::find().filter(PlayerStatsColumn::StatCategoriesId.eq(category.id));

    if sort_by_owned == "stat_name" {
        if order_owned == "asc" {
            query = query.order_by_asc(PlayerStatsColumn::StatName);
        } else {
            query = query.order_by_desc(PlayerStatsColumn::StatName);
        }
    } else if sort_by_owned == "value" {
        if order_owned == "asc" {
            query = query.order_by_asc(PlayerStatsColumn::Value);
        } else {
            query = query.order_by_desc(PlayerStatsColumn::Value);
        }
    } else {
        query = query.order_by_desc(PlayerStatsColumn::Value);
    }

    match query
        .limit(limit)
        .offset(offset)
        .all(app_state.database_connection.as_ref())
        .await
    {
        Ok(stats) => {
            Json(serde_json::json!({ "category": category, "stats": stats })).into_response()
        }
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
pub async fn players(State(app_state): State<AppState>) -> impl IntoResponse {
    match PlayerEntity::find()
        .all(app_state.database_connection.as_ref())
        .await
    {
        Ok(all_players) => Json::<Vec<Player>>(all_players).into_response(),
        _ => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
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
        (status = 200),
        (status = 500)
    )
)]
pub async fn player(
    State(app_state): State<AppState>,
    Path(player_uuid): Path<Uuid>,
    Query(params): Query<SearchParams>,
) -> impl IntoResponse {
    let limit = params.limit.unwrap_or(25);
    let offset = (params.page.unwrap_or(1).max(1) - 1) * limit;
    let sort_by_owned = params
        .sort_by
        .clone()
        .unwrap_or_else(|| "value".to_string());
    let order_owned = params.order.clone().unwrap_or_else(|| "desc".to_string());
    let player_uuid_str = player_uuid.to_string();

    let mut query =
        PlayerStatsEntity::find().filter(PlayerStatsColumn::PlayerUuid.eq(&player_uuid_str));

    if sort_by_owned == "stat_name" {
        if order_owned == "asc" {
            query = query.order_by_asc(PlayerStatsColumn::StatName);
        } else {
            query = query.order_by_desc(PlayerStatsColumn::StatName);
        }
    } else if sort_by_owned == "value" {
        if order_owned == "asc" {
            query = query.order_by_asc(PlayerStatsColumn::Value);
        } else {
            query = query.order_by_desc(PlayerStatsColumn::Value);
        }
    } else {
        query = query.order_by_desc(PlayerStatsColumn::Value);
    }

    match query
        .limit(limit)
        .offset(offset)
        .all(app_state.database_connection.as_ref())
        .await
    {
        Ok(stats) => Json(serde_json::json!({ "player_uuid": player_uuid_str, "stats": stats }))
            .into_response(),
        _ => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}

pub async fn run_server(database: DatabaseConnection, port: &str) {
    use tower_http::cors::{Any, CorsLayer};

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
        .route("/categories/{categorie}", get(category))
        .route("/players", get(players))
        .route("/players/{player}", get(player))
        .merge(
            utoipa_swagger_ui::SwaggerUi::new("/docs")
                .url("/openapi.json", crate::api_docs::ApiDoc::openapi()),
        )
        .with_state(state)
        .layer(cors);

    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", port))
        .await
        .unwrap();
    axum::serve(listener, app).await.unwrap();
}
