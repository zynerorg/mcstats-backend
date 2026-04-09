pub use crate::entities::{Player, PlayerStats, StatCategorie};
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Deserialize, Debug)]
pub struct StatsFile {
    pub stats: HashMap<String, HashMap<String, i32>>,
}
