use sea_orm_migration::prelude::*;
use sea_query::Iden;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Players::Table)
                    .col(
                        ColumnDef::new(Players::PlayerUuid)
                            .text()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Players::Name).text().not_null())
                    .if_not_exists()
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(StatCategories::Table)
                    .col(
                        ColumnDef::new(StatCategories::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(StatCategories::Name)
                            .text()
                            .not_null()
                            .unique_key(),
                    )
                    .if_not_exists()
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(PlayerStats::Table)
                    .col(ColumnDef::new(PlayerStats::PlayerUuid).text().not_null())
                    .col(ColumnDef::new(PlayerStats::StatCategoriesId).integer().not_null())
                    .col(ColumnDef::new(PlayerStats::StatName).text().not_null())
                    .col(ColumnDef::new(PlayerStats::Value).integer().not_null())
                    .if_not_exists()
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, _manager: &SchemaManager) -> Result<(), DbErr> {
        Ok(())
    }
}

#[derive(Iden)]
enum Players {
    Table,
    PlayerUuid,
    Name,
}

#[derive(Iden)]
enum StatCategories {
    Table,
    Id,
    Name,
}

#[derive(Iden)]
enum PlayerStats {
    Table,
    PlayerUuid,
    StatCategoriesId,
    StatName,
    Value,
}