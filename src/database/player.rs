use super::DatabaseConnection;
use crate::entities::players::{
    ActiveModel as PlayerActiveModel, Column as PlayerColumn, Entity as PlayerEntity,
    Model as Player,
};
use anyhow::Result;
use log::debug;
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter};

impl DatabaseConnection {
    pub async fn insert_player(&self, player: Player) -> Result<Player> {
        debug!("Insert player {:?}", player);

        if let Some(existing) = self.find_player(&player.player_uuid).await? {
            return self.update_player(existing, &player).await;
        }

        self.create_player(&player).await
    }

    pub async fn find_player(&self, uuid: &str) -> Result<Option<Player>> {
        Ok(PlayerEntity::find()
            .filter(PlayerColumn::PlayerUuid.eq(uuid))
            .one(self.as_ref())
            .await?)
    }

    pub async fn update_player(&self, existing: Player, new: &Player) -> Result<Player> {
        let mut active: PlayerActiveModel = existing.into();
        active.name = sea_orm::Set(new.name.clone());

        let updated = active.update(self.as_ref()).await?;
        Ok(updated)
    }

    pub async fn create_player(&self, player: &Player) -> Result<Player> {
        let active = PlayerActiveModel {
            player_uuid: sea_orm::Set(player.player_uuid.clone()),
            name: sea_orm::Set(player.name.clone()),
        };

        PlayerEntity::insert(active).exec(self.as_ref()).await?;

        Ok(player.clone())
    }
}
