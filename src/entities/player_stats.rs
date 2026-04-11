use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize, ToSchema)]
#[sea_orm(table_name = "player_stats")]
#[schema(as = PlayerStats)]
pub struct Model {
    #[sea_orm(column_name = "player_uuid", primary_key)]
    pub player_uuid: String,
    #[sea_orm(column_name = "stat_categories_id", primary_key)]
    pub stat_categories_id: i32,
    #[sea_orm(column_name = "stat_name", primary_key)]
    pub stat_name: String,
    #[sea_orm(column_name = "value")]
    pub value: i32,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::player::Entity",
        from = "Column::PlayerUuid",
        to = "super::player::Column::PlayerUuid"
    )]
    Player,
    #[sea_orm(
        belongs_to = "super::stat_categories::Entity",
        from = "Column::StatCategoriesId",
        to = "super::stat_categories::Column::Id"
    )]
    StatCategorie,
}

impl ActiveModelBehavior for ActiveModel {}
