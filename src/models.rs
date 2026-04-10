pub use crate::entities::{Player, PlayerStats, StatCategorie};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use utoipa::ToSchema;

#[derive(Deserialize, Debug)]
pub struct StatsFile {
    pub stats: HashMap<String, HashMap<String, i32>>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct CategoryStatsResponse {
    pub category: StatCategorie,
    pub stats: Vec<PlayerStats>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct PlayerStatsResponse {
    pub player_uuid: String,
    pub stats: Vec<PlayerStats>,
}
