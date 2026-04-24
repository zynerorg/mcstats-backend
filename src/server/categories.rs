use crate::server::helpers::{SearchParams, apply_sorting, parse_order, parse_pagination};
use crate::server::run_server::AppState;
use crate::entities::player_stats::{
    Column as PlayerStatsColumn, Entity as PlayerStatsEntity, Model as PlayerStats,
};
use crate::entities::stat_categories::{
    Column as StatCategoryColumn, Entity as StatCategoryEntity, Model as StatCategory,
};
use axum::http::StatusCode;
use axum::{
    Json,
    extract::{Path, Query, State},
    response::IntoResponse,
};
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, QuerySelect};

#[utoipa::path(
    get,
    path="/categories",
    responses(
        (status = 200, body = Vec<StatCategory>),
        (status = 500)
    )
)]
pub async fn categories(State(app_state): State<AppState>) -> impl IntoResponse {
    match StatCategoryEntity::find()
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
        ("order" = Option<String>, Query)
    ),
    responses(
        (status = 200, body = Vec<PlayerStats>),
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
    let order = parse_order(&params);

    let category = match StatCategoryEntity::find()
        .filter(StatCategoryColumn::Name.eq(&categorie))
        .one(app_state.database_connection.as_ref())
        .await
    {
        Ok(Some(c)) => c,
        _ => return (StatusCode::NOT_FOUND, Json(vec![])),
    };

    let query = apply_sorting(
        PlayerStatsEntity::find().filter(PlayerStatsColumn::StatCategoriesId.eq(category.id)),
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