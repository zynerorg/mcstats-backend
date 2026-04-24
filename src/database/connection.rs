use anyhow::Result;
use log::info;
use migration::{Migrator, MigratorTrait};
use sea_orm::{ConnectOptions, Database};
use std::sync::Arc;

type DbPool = crate::database::DbPool;

#[derive(Clone)]
pub struct DatabaseConnection {
    conn: Arc<DbPool>,
    concurrency_limit: usize,
}

impl AsRef<DbPool> for DatabaseConnection {
    fn as_ref(&self) -> &DbPool {
        &self.conn
    }
}

impl DatabaseConnection {
    pub async fn new(url: &str, pool_size: u32, concurrency_limit: usize) -> Result<Self> {
        info!("Connecting to database...");

        let mut opt = ConnectOptions::new(url);
        opt.sqlx_logging(false);
        opt.max_connections(pool_size);

        let conn = Database::connect(opt).await?;
        Migrator::up(&conn, None).await?;

        info!("Database ready");

        Ok(Self {
            conn: Arc::new(conn),
            concurrency_limit,
        })
    }

    pub fn concurrency_limit(&self) -> usize {
        self.concurrency_limit
    }
}
