pub use sea_orm_migration::prelude::*;

mod m20240409_000001_create_initial_tables;

pub use sea_orm_migration::MigratorTrait;

pub struct Migrator;

impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![Box::new(
            m20240409_000001_create_initial_tables::Migration,
        )]
    }
}