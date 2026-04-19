use std::vec;

use crate::entities::items::Entity as ItemsEntity;
use crate::entities::items::Model as Items;
use crate::entities::player_stats::{
    Column as PlayerStatsColumn, Entity as PlayerStatsEntity, Model as PlayerStats,
};
use crate::server::helpers::{apply_sorting, get_category_id, parse_order_stats, parse_pagination_stats, StatsFilterParams};
use crate::server::run_server::AppState;
use axum::Json;
use axum::extract::Path;
use axum::extract::Query;
use axum::{extract::State, response::IntoResponse};
use reqwest::StatusCode;
use sea_orm::ColumnTrait;
use sea_orm::EntityTrait;
use sea_orm::QueryFilter;
use sea_orm::QuerySelect;

#[utoipa::path(
    get,
    path="/items",
    responses(
        (status = 200, body = Vec<Items>)
    )
)]
pub async fn items(State(app_state): State<AppState>) -> impl IntoResponse {
    match ItemsEntity::find()
        .all(app_state.database_connection.as_ref())
        .await
    {
        Ok(items) => (StatusCode::OK, Json(items)),
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, Json(vec![])),
    }
}

#[utoipa::path(
    get,
    path = "/items/{item}",
    params(
        ("item" = String, Path),
        ("category" = Option<String>, Query),
        ("limit" = Option<u64>, Query),
        ("page" = Option<u64>, Query),
        ("order" = Option<String>, Query)
    ),
    responses(
        (status = 200, body = Vec<PlayerStats>),
        (status = 404),
        (status = 500)
    )
)]
pub async fn item(
    State(app_state): State<AppState>,
    Path(item): Path<String>,
    Query(params): Query<StatsFilterParams>,
) -> impl IntoResponse {
    let (limit, offset) = parse_pagination_stats(&params);
    let order = parse_order_stats(&params);

    let mut query = PlayerStatsEntity::find()
        .filter(PlayerStatsColumn::StatName.eq(&item));

    if let Some(category) = &params.category {
        let category_id = match get_category_id(&app_state.database_connection, category.clone()).await {
            Some(id) => id,
            None => return (StatusCode::NOT_FOUND, Json(vec![])),
        };
        query = query.filter(PlayerStatsColumn::StatCategoriesId.eq(category_id));
    }

    let query = apply_sorting(query, &order);

    match query
        .limit(limit)
        .offset(offset)
        .all(app_state.database_connection.as_ref())
        .await
    {
        Ok(stats) => (StatusCode::OK, Json(stats)),
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, Json(vec![])),
    }
}

#[utoipa::path(
    get,
    path = "/stats",
    params(
        ("item" = Option<String>, Query),
        ("category" = Option<String>, Query),
        ("player_uuid" = Option<String>, Query),
        ("limit" = Option<u64>, Query),
        ("page" = Option<u64>, Query),
        ("order" = Option<String>, Query)
    ),
    responses(
        (status = 200, body = Vec<PlayerStats>),
        (status = 500)
    )
)]
pub async fn stats(
    State(app_state): State<AppState>,
    Query(params): Query<StatsFilterParams>,
) -> impl IntoResponse {
    let (limit, offset) = parse_pagination_stats(&params);
    let order = parse_order_stats(&params);

    let mut query = PlayerStatsEntity::find();

    if let Some(item) = &params.item {
        query = query.filter(PlayerStatsColumn::StatName.eq(item));
    }

    if let Some(category) = &params.category {
        let category_id = get_category_id(&app_state.database_connection, category.clone()).await;
        if let Some(id) = category_id {
            query = query.filter(PlayerStatsColumn::StatCategoriesId.eq(id));
        }
    }

    if let Some(player_uuid) = &params.player_uuid {
        query = query.filter(PlayerStatsColumn::PlayerUuid.eq(player_uuid));
    }

    let query = apply_sorting(query, &order);

    match query
        .limit(limit)
        .offset(offset)
        .all(app_state.database_connection.as_ref())
        .await
    {
        Ok(stats) => (StatusCode::OK, Json(stats)),
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, Json(vec![])),
    }
}
