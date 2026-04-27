use crate::entities::player_stats::Model as StatModel;
use crate::entities::players::Model as PlayerModel;
use async_graphql::Object;

pub struct Player(pub PlayerModel);
pub struct Stat(pub StatModel);

impl Player {
    pub fn new(model: PlayerModel) -> Self {
        Self(model)
    }
}

impl Stat {
    pub fn new(model: StatModel) -> Self {
        Self(model)
    }
}

#[Object]
impl Player {
    async fn player_uuid(&self) -> &str {
        &self.0.player_uuid
    }

    async fn name(&self) -> &str {
        &self.0.name
    }
}

#[Object]
impl Stat {
    async fn player_uuid(&self) -> &str {
        &self.0.player_uuid
    }

    async fn category(&self) -> &str {
        &self.0.category
    }

    async fn value_name(&self) -> &str {
        &self.0.value_name
    }

    async fn value(&self) -> i64 {
        self.0.value
    }
}
