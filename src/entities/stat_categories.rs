use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize, ToSchema)]
#[sea_orm(table_name = "stat_categories")]
#[schema(as = StatCategory)]
pub struct Model {
    #[sea_orm(column_name = "id", primary_key, auto_increment = true)]
    pub id: i32,
    #[sea_orm(column_name = "name", unique)]
    pub name: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

