use crate::database::DatabaseConnection;
use crate::entities::player_stats::{Column as PlayerStatsColumn, Entity as PlayerStatsEntity};
use crate::entities::stat_categories::{
    Column as StatCategoryColumn, Entity as StatCategoryEntity,
};
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, QueryOrder};
use serde::Deserialize;
use utoipa::IntoParams;

const DEFAULT_LIMIT: u64 = 25;
const DEFAULT_PAGE: u64 = 1;
const DEFAULT_ORDER: &str = "desc";

#[derive(Debug, Deserialize, IntoParams)]
pub struct SearchParams {
    #[serde(default)]
    pub limit: Option<u64>,
    #[serde(default)]
    pub page: Option<u64>,
    #[serde(default)]
    pub order: Option<String>,
}

#[derive(Debug, Deserialize, IntoParams)]
pub struct FilterParams {
    #[serde(default)]
    pub limit: Option<u64>,
    #[serde(default)]
    pub page: Option<u64>,
    #[serde(default)]
    pub order: Option<String>,
    #[serde(default)]
    pub category: Option<String>,
    #[serde(default)]
    pub player_uuid: Option<String>,
}

#[derive(Debug, Deserialize, IntoParams)]
pub struct StatsFilterParams {
    #[serde(default)]
    pub limit: Option<u64>,
    #[serde(default)]
    pub page: Option<u64>,
    #[serde(default)]
    pub order: Option<String>,
    #[serde(default)]
    pub item: Option<String>,
    #[serde(default)]
    pub category: Option<String>,
    #[serde(default)]
    pub player_uuid: Option<String>,
}

pub fn parse_pagination(params: &SearchParams) -> (u64, u64) {
    let limit = params.limit.unwrap_or(DEFAULT_LIMIT);
    let offset = (params.page.unwrap_or(DEFAULT_PAGE).max(DEFAULT_PAGE) - 1) * limit;
    (limit, offset)
}

pub fn parse_pagination_stats(params: &StatsFilterParams) -> (u64, u64) {
    let limit = params.limit.unwrap_or(DEFAULT_LIMIT);
    let offset = (params.page.unwrap_or(DEFAULT_PAGE).max(DEFAULT_PAGE) - 1) * limit;
    (limit, offset)
}

pub fn parse_order(params: &SearchParams) -> String {
    params
        .order
        .clone()
        .unwrap_or_else(|| DEFAULT_ORDER.to_string())
}

pub fn parse_order_stats(params: &StatsFilterParams) -> String {
    params
        .order
        .clone()
        .unwrap_or_else(|| DEFAULT_ORDER.to_string())
}

pub fn apply_sorting(
    query: sea_orm::Select<PlayerStatsEntity>,
    order: &str,
) -> sea_orm::Select<PlayerStatsEntity> {
    if order == "asc" {
        query.order_by_asc(PlayerStatsColumn::Value)
    } else {
        query.order_by_desc(PlayerStatsColumn::Value)
    }
}

pub async fn get_category_id(database: &DatabaseConnection, category: String) -> Option<i32> {
    StatCategoryEntity::find()
        .filter(StatCategoryColumn::Name.eq(&category))
        .one(database.as_ref())
        .await
        .ok()
        .flatten()
        .map(|c| c.id)
}
