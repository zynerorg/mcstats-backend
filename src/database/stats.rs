use super::DatabaseConnection;
use crate::database::StatsFile;
use crate::entities::player_stats::{
    ActiveModel as PlayerStatsActiveModel, Entity as PlayerStatsEntity,
};
use crate::entities::stat_categories::{
    ActiveModel as StatCategoryActiveModel, Column as StatCategoryColumn,
    Entity as StatCategoryEntity,
};
use anyhow::Result;
use log::info;
use sea_orm::{ColumnTrait, DbErr, EntityTrait, QueryFilter, TransactionTrait};
use std::collections::HashMap;

impl DatabaseConnection {
    pub async fn insert_stats(&self, uuid: uuid::Uuid, stats: StatsFile) -> Result<()> {
        let uuid_str = uuid.to_string();
        let total = stats
            .stats
            .values()
            .map(|m: &HashMap<String, i32>| m.len())
            .sum::<usize>();

        info!("Inserting {total} stats for {uuid}");

        let pool = self.as_ref();

        pool.transaction::<_, (), DbErr>(|txn| {
            Box::pin(async move {
                for (cat_name, map) in stats.stats {
                    let category_id = Self::get_or_create_category_in_txn(txn, &cat_name)
                        .await
                        .map_err(|e: anyhow::Error| DbErr::Custom(e.to_string()))?;

                    for (name, value) in map {
                        Self::insert_stat_row(txn, &uuid_str, category_id, name, value)
                            .await
                            .map_err(|e: anyhow::Error| DbErr::Custom(e.to_string()))?;
                    }
                }

                Ok(())
            })
        })
        .await?;

        Ok(())
    }

    async fn get_or_create_category_in_txn(
        txn: &sea_orm::DatabaseTransaction,
        name: &str,
    ) -> Result<i32> {
        if let Some(existing) = StatCategoryEntity::find()
            .filter(StatCategoryColumn::Name.eq(name))
            .one(txn)
            .await?
        {
            return Ok(existing.id);
        }

        let active = StatCategoryActiveModel {
            id: sea_orm::NotSet,
            name: sea_orm::Set(name.to_string()),
        };

        match StatCategoryEntity::insert(active).exec(txn).await {
            Ok(res) => Ok(res.last_insert_id),
            Err(e) if e.to_string().contains("UNIQUE") => {
                let found = StatCategoryEntity::find()
                    .filter(StatCategoryColumn::Name.eq(name))
                    .one(txn)
                    .await?
                    .map(|c| c.id);
                found.ok_or_else(|| anyhow::anyhow!("Category missing after conflict"))
            }
            Err(e) => Err(e.into()),
        }
    }

    async fn insert_stat_row(
        txn: &sea_orm::DatabaseTransaction,
        uuid: &str,
        category_id: i32,
        name: String,
        value: i32,
    ) -> Result<()> {
        let model = PlayerStatsActiveModel {
            player_uuid: sea_orm::Set(uuid.to_string()),
            stat_categories_id: sea_orm::Set(category_id),
            stat_name: sea_orm::Set(name),
            value: sea_orm::Set(value),
        };

        match PlayerStatsEntity::insert(model).exec(txn).await {
            Ok(_) => Ok(()),
            Err(e) if e.to_string().contains("UNIQUE") => Ok(()),
            Err(e) => Err(e.into()),
        }
    }
}
