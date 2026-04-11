use clap::{Parser, Subcommand};
use sea_orm_migration::prelude::*;
use sea_orm::Database;

#[derive(Parser)]
#[command(name = "migration")]
#[command(about = "Database migration CLI", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Apply all pending migrations
    Up,
    /// Rollback last applied migration
    Down,
    /// Rollback all applied migrations
    Reset,
    /// Drop all tables and reapply migrations
    Fresh,
    /// Check migration status
    Status,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    let db_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let db = Database::connect(&db_url).await?;

    match cli.command {
        Commands::Up => {
            migration::Migrator::up(&db, None).await?;
            println!("Applied all migrations");
        }
        Commands::Down => {
            migration::Migrator::down(&db, None).await?;
            println!("Rolled back last migration");
        }
        Commands::Reset => {
            migration::Migrator::reset(&db).await?;
            println!("Rolled back all migrations");
        }
        Commands::Fresh => {
            migration::Migrator::fresh(&db).await?;
            println!("Dropped all tables and re-applied migrations");
        }
        Commands::Status => {
            migration::Migrator::status(&db).await?;
        }
    }

    Ok(())
}