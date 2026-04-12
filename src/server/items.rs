use std::vec;

use crate::entities::items::Entity as ItemsEntity;
use crate::entities::items::Model as Items;
use crate::entities::player_stats::{
    Column as PlayerStatsColumn, Entity as PlayerStatsEntity, Model as PlayerStats,
};
use crate::server::helpers::SearchParams;
use crate::server::helpers::apply_sorting;
use crate::server::helpers::get_category_id;
use crate::server::helpers::parse_order;
use crate::server::helpers::parse_pagination;
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
    path = "/items/{category}/{item}",
    params(
        ("category" = String, Path),
        ("item" = String, Path),
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
    Path((category, item)): Path<(String, String)>,
    Query(params): Query<SearchParams>,
) -> impl IntoResponse {
    let (limit, offset) = parse_pagination(&params);
    let order = parse_order(&params);

    let category_id = match get_category_id(&app_state.database_connection, category).await {
        Some(id) => id,
        None => return (StatusCode::NOT_FOUND, Json(vec![])),
    };

    let query = apply_sorting(
        PlayerStatsEntity::find()
            .filter(PlayerStatsColumn::StatName.eq(item))
            .filter(PlayerStatsColumn::StatCategoriesId.eq(category_id)),
        &order,
    );

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
