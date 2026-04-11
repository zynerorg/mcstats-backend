use super::DatabaseConnection;
use crate::database::StatsFile;
use crate::entities::player_stats::{
    ActiveModel as PlayerStatsActiveModel, Entity as PlayerStatsEntity,
};
use anyhow::Result;
use log::info;
use sea_orm::{DbErr, EntityTrait, TransactionTrait};
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
        let db = self.clone();

        pool.transaction::<_, (), DbErr>(|txn| {
            Box::pin(async move {
                for (cat_name, map) in stats.stats {
                    let category_id = db
                        .get_or_create_category(&cat_name)
                        .await
                        .map_err(|e: anyhow::Error| DbErr::Custom(e.to_string()))?;

                    for (name, value) in map {
                        do_insert_stat(txn, &uuid_str, category_id, name, value)
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
}

async fn do_insert_stat(
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
