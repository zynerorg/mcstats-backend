use crate::entities::{
    Player, PlayerEntity, PlayerStats, PlayerStatsColumn, PlayerStatsEntity, StatCategoryColumn,
    StatCategoryEntity,
};
use axum::http::StatusCode;
use axum::{
    Json,
    extract::{Path, Query, State},
    response::IntoResponse,
};
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, QuerySelect};
use uuid::Uuid;

use super::helpers::{SearchParams, apply_sorting, parse_order, parse_pagination};
use super::server::AppState;

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
    Query(params): Query<SearchParams>,
) -> impl IntoResponse {
    let (limit, offset) = parse_pagination(&params);
    let order = parse_order(&params);
    let player_uuid_str = player_uuid.to_string();

    let query = apply_sorting(
        PlayerStatsEntity::find().filter(PlayerStatsColumn::PlayerUuid.eq(&player_uuid_str)),
        &order,
    );

    match query
        .limit(limit)
        .offset(offset)
        .all(app_state.database_connection.as_ref())
        .await
    {
        Ok(stats) => Json(stats).into_response(),
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}

#[utoipa::path(
    get,
    path = "/players/{player_uuid}/{category_name}",
    params(
        ("player_uuid" = Uuid, Path),
        ("category_name" = String, Path),
        ("limit" = Option<u64>, Query),
        ("page" = Option<u64>, Query),
        ("order" = Option<String>, Query)
    ),
    responses(
        (status = 200 )
    )
)]
pub async fn player_by_category(
    State(app_state): State<AppState>,
    Path((player_uuid, category_name)): Path<(Uuid, String)>,
    Query(params): Query<SearchParams>,
) -> impl IntoResponse {
    let (limit, offset) = parse_pagination(&params);
    let order = parse_order(&params);

    let player_uuid_str = player_uuid.to_string();

    let category = StatCategoryEntity::find()
        .filter(StatCategoryColumn::Name.eq(&category_name))
        .one(app_state.database_connection.as_ref())
        .await;

    let category = match category {
        Ok(Some(c)) => c,
        Ok(None) => return StatusCode::NOT_FOUND.into_response(),
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    };

    let query = apply_sorting(
        PlayerStatsEntity::find().filter(
            PlayerStatsColumn::PlayerUuid
                .eq(&player_uuid_str)
                .and(PlayerStatsColumn::StatCategoriesId.eq(category.id)),
        ),
        &order,
    );

    let results = query
        .limit(limit)
        .offset(offset)
        .all(app_state.database_connection.as_ref())
        .await;

    match results {
        Ok(data) => (StatusCode::OK, Json(data)).into_response(),
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}
