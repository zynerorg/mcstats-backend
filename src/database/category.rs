use super::DatabaseConnection;
use crate::entities::stat_categories::{
    ActiveModel as StatCategoryActiveModel, Column as StatCategoryColumn,
    Entity as StatCategoryEntity, Model as StatCategory,
};
use anyhow::{Result, anyhow};
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};

impl DatabaseConnection {
    pub async fn get_or_create_category(&self, name: &str) -> Result<i32> {
        if let Some(existing) = StatCategoryEntity::find()
            .filter(StatCategoryColumn::Name.eq(name))
            .one(self.as_ref())
            .await?
        {
            return Ok(existing.id);
        }

        let active = StatCategoryActiveModel {
            id: sea_orm::NotSet,
            name: sea_orm::Set(name.to_string()),
        };

        match StatCategoryEntity::insert(active).exec(self.as_ref()).await {
            Ok(res) => Ok(res.last_insert_id),
            Err(e) if e.to_string().contains("UNIQUE") => {
                let found = StatCategoryEntity::find()
                    .filter(StatCategoryColumn::Name.eq(name))
                    .one(self.as_ref())
                    .await?
                    .map(|c| c.id);
                found.ok_or_else(|| anyhow!("Category missing after conflict"))
            }
            Err(e) => Err(e.into()),
        }
    }

    pub async fn find_category_by_name(&self, name: &str) -> Result<Option<StatCategory>> {
        Ok(StatCategoryEntity::find()
            .filter(StatCategoryColumn::Name.eq(name))
            .one(self.as_ref())
            .await?)
    }

    pub async fn find_category_by_id(&self, id: i32) -> Result<Option<StatCategory>> {
        Ok(StatCategoryEntity::find_by_id(id)
            .one(self.as_ref())
            .await?)
    }
}
