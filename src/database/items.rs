use super::DatabaseConnection;
use crate::entities::items::{ActiveModel as ItemsActiveModel, Entity as ItemsEntity};
use anyhow::Result;
use sea_orm::EntityTrait;

impl DatabaseConnection {
    pub async fn insert_items(&self, items: Vec<String>) -> Result<()> {
        if items.is_empty() {
            return Ok(());
        }

        let pool = self.as_ref();

        for name in items {
            let active = ItemsActiveModel {
                id: sea_orm::NotSet,
                name: sea_orm::Set(name),
            };

            match ItemsEntity::insert(active).exec(pool).await {
                Ok(_) => {}
                Err(e) if e.to_string().contains("UNIQUE") => {}
                Err(e) => return Err(e.into()),
            }
        }

        Ok(())
    }
}
