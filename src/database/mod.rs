pub mod category;
pub mod connection;
pub mod items;
pub mod player;
pub mod populate;
pub mod stats;

pub use connection::DatabaseConnection;

use serde::Deserialize;
use std::collections::HashMap;

pub type DbPool = sea_orm::DatabaseConnection;

#[derive(Deserialize, Debug)]
pub struct StatsFile {
    pub stats: HashMap<String, HashMap<String, i64>>,
}
