pub mod types;

use crate::database::DatabaseConnection;
use crate::entities::player_stats::{Column as StatColumn, Entity as StatEntity};
use crate::entities::players::Entity as PlayerEntity;
use async_graphql::{Context, EmptySubscription, InputObject, Schema};
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, QueryOrder, QuerySelect};

pub type AppSchema = Schema<QueryRoot, MutationRoot, EmptySubscription>;

#[derive(InputObject)]
pub struct StatFilterInput {
    pub category: Option<String>,
    pub limit: Option<u64>,
    pub page: Option<u64>,
    pub order: Option<String>,
}

#[derive(InputObject)]
pub struct StatsFilterInput {
    pub item: Option<String>,
    pub category: Option<String>,
    pub player_uuid: Option<String>,
    pub limit: Option<u64>,
    pub page: Option<u64>,
    pub order: Option<String>,
}

pub struct QueryRoot;

#[async_graphql::Object]
impl QueryRoot {
    async fn players(&self, ctx: &Context<'_>) -> async_graphql::Result<Vec<types::Player>> {
        let db = ctx.data::<DatabaseConnection>()?;
        let pool = db.as_ref();
        let players = PlayerEntity::find()
            .all(pool)
            .await
            .map_err(|e| async_graphql::Error::new(e.to_string()))?;
        Ok(players.into_iter().map(types::Player::new).collect())
    }

    async fn player(
        &self,
        ctx: &Context<'_>,
        player_uuid: String,
        filter: Option<StatFilterInput>,
    ) -> async_graphql::Result<Vec<types::Stat>> {
        let db = ctx.data::<DatabaseConnection>()?;
        let pool = db.as_ref();
        let filter = filter.unwrap_or(StatFilterInput {
            category: None,
            limit: Some(10),
            page: Some(0),
            order: None,
        });

        let limit = filter.limit.unwrap_or(10);
        let offset = filter.page.unwrap_or(0) * limit;
        let order = filter.order.as_deref().unwrap_or("DESC");

        let mut query = StatEntity::find().filter(StatColumn::PlayerUuid.eq(&player_uuid));

        if let Some(category) = &filter.category {
            query = query.filter(StatColumn::Category.eq(category));
        }

        if order.to_lowercase().starts_with("asc") {
            query = query.order_by_asc(StatColumn::Value);
        } else {
            query = query.order_by_desc(StatColumn::Value);
        }

        let stats = query
            .limit(limit)
            .offset(offset)
            .all(pool)
            .await
            .map_err(|e| async_graphql::Error::new(e.to_string()))?;

        Ok(stats.into_iter().map(types::Stat::new).collect())
    }

    async fn stats(
        &self,
        ctx: &Context<'_>,
        filter: Option<StatsFilterInput>,
    ) -> async_graphql::Result<Vec<types::Stat>> {
        let db = ctx.data::<DatabaseConnection>()?;
        let pool = db.as_ref();
        let filter = filter.unwrap_or(StatsFilterInput {
            item: None,
            category: None,
            player_uuid: None,
            limit: Some(10),
            page: Some(0),
            order: None,
        });

        let limit = filter.limit.unwrap_or(10);
        let offset = filter.page.unwrap_or(0) * limit;
        let order = filter.order.as_deref().unwrap_or("DESC");

        let mut query = StatEntity::find();

        if let Some(item) = &filter.item {
            query = query.filter(StatColumn::ValueName.eq(item));
        }

        if let Some(category) = &filter.category {
            query = query.filter(StatColumn::Category.eq(category));
        }

        if let Some(player_uuid) = &filter.player_uuid {
            query = query.filter(StatColumn::PlayerUuid.eq(player_uuid));
        }

        if order.to_lowercase().starts_with("asc") {
            query = query.order_by_asc(StatColumn::Value);
        } else {
            query = query.order_by_desc(StatColumn::Value);
        }

        let stats = query
            .limit(limit)
            .offset(offset)
            .all(pool)
            .await
            .map_err(|e| async_graphql::Error::new(e.to_string()))?;

        Ok(stats.into_iter().map(types::Stat::new).collect())
    }

    async fn categories(&self, ctx: &Context<'_>) -> async_graphql::Result<Vec<String>> {
        let db = ctx.data::<DatabaseConnection>()?;
        let pool = db.as_ref();

        let result: Vec<String> = StatEntity::find()
            .select_only()
            .column(StatColumn::Category)
            .distinct()
            .into_tuple()
            .all(pool)
            .await
            .map_err(|e| async_graphql::Error::new(e.to_string()))?;

        Ok(result)
    }

    async fn category(
        &self,
        ctx: &Context<'_>,
        name: String,
        filter: Option<StatFilterInput>,
    ) -> async_graphql::Result<Vec<types::Stat>> {
        let db = ctx.data::<DatabaseConnection>()?;
        let pool = db.as_ref();
        let filter = filter.unwrap_or(StatFilterInput {
            category: None,
            limit: Some(10),
            page: Some(0),
            order: None,
        });

        let limit = filter.limit.unwrap_or(10);
        let offset = filter.page.unwrap_or(0) * limit;
        let order = filter.order.as_deref().unwrap_or("dsc");

        let mut query = StatEntity::find().filter(StatColumn::Category.eq(&name));

        if order.to_lowercase().starts_with("asc") {
            query = query.order_by_asc(StatColumn::Value);
        } else {
            query = query.order_by_desc(StatColumn::Value);
        }

        let stats = query
            .limit(limit)
            .offset(offset)
            .all(pool)
            .await
            .map_err(|e| async_graphql::Error::new(e.to_string()))?;

        Ok(stats.into_iter().map(types::Stat::new).collect())
    }
}

pub struct MutationRoot;

#[async_graphql::Object]
impl MutationRoot {
    async fn _ping(&self) -> async_graphql::Result<String> {
        Ok("pong".to_string())
    }
}

pub fn create_schema(database: DatabaseConnection) -> AppSchema {
    Schema::build(QueryRoot, MutationRoot, EmptySubscription)
        .data(database)
        .finish()
}
