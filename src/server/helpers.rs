use crate::entities::player_stats::{Column as PlayerStatsColumn, Entity as PlayerStatsEntity};
use sea_orm::QueryOrder;
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

pub fn parse_pagination(params: &SearchParams) -> (u64, u64) {
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
