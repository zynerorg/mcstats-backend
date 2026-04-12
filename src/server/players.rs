use super::helpers::{SearchParams, apply_sorting, parse_order, parse_pagination};
use super::run_server::AppState;
use crate::entities::player_stats::{
    Column as PlayerStatsColumn, Entity as PlayerStatsEntity, Model as PlayerStats,
};
use crate::entities::players::{Entity as PlayerEntity, Model as Player};
use crate::entities::stat_categories::{
    Column as StatCategoryColumn, Entity as StatCategoryEntity,
};
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
        Ok(stats) => (StatusCode::OK, Json(stats)),
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, Json(vec![])),
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
        Ok(None) => return (StatusCode::NOT_FOUND, Json(vec![])),
        Err(_) => return (StatusCode::INTERNAL_SERVER_ERROR, Json(vec![])),
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
        Ok(data) => (StatusCode::OK, Json(data)),
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, Json(vec![])),
    }
}
