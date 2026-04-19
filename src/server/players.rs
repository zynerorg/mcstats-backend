use super::helpers::{apply_sorting, get_category_id, parse_order_stats, parse_pagination_stats, StatsFilterParams};
use super::run_server::AppState;
use crate::entities::player_stats::{
    Column as PlayerStatsColumn, Entity as PlayerStatsEntity, Model as PlayerStats,
};
use crate::entities::players::{Entity as PlayerEntity, Model as Player};
use axum::http::StatusCode;
use axum::{
    Json,
    extract::{Path, Query, State},
    response::IntoResponse,
};
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, QuerySelect};
use uuid::Uuid;

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
        ("category" = Option<String>, Query),
        ("limit" = Option<u64>, Query),
        ("page" = Option<u64>, Query),
        ("order" = Option<String>, Query)
    ),
    responses(
        (status = 200, body = Vec<PlayerStats>),
        (status = 500)
    )
)]
pub async fn player(
    State(app_state): State<AppState>,
    Path(player_uuid): Path<Uuid>,
    Query(params): Query<StatsFilterParams>,
) -> impl IntoResponse {
    let (limit, offset) = parse_pagination_stats(&params);
    let order = parse_order_stats(&params);
    let player_uuid_str = player_uuid.to_string();

    let mut query = PlayerStatsEntity::find()
        .filter(PlayerStatsColumn::PlayerUuid.eq(&player_uuid_str));

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
