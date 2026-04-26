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
                    .table(PlayerStats::Table)
                    .col(ColumnDef::new(PlayerStats::PlayerUuid).text().not_null())
                    .col(ColumnDef::new(PlayerStats::Category).text().not_null())
                    .col(ColumnDef::new(PlayerStats::ValueName).text().not_null())
                    .col(ColumnDef::new(PlayerStats::Value).big_integer().not_null())
                    .primary_key(
                        Index::create()
                            .name("pk_stats")
                            .col(PlayerStats::PlayerUuid)
                            .col(PlayerStats::Category)
                            .col(PlayerStats::ValueName)
                            .primary(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_stats_player_uuid")
                            .from_col(PlayerStats::PlayerUuid)
                            .to_tbl(Players::Table)
                            .to_col(Players::PlayerUuid)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .if_not_exists()
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(
                Table::drop()
                    .table(PlayerStats::Table)
                    .if_exists()
                    .to_owned(),
            )
            .await?;
        manager
            .drop_table(Table::drop().table(Players::Table).if_exists().to_owned())
            .await?;
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
enum PlayerStats {
    Table,
    PlayerUuid,
    Category,
    ValueName,
    Value,
}
