use std::path::PathBuf;

#[derive(Clone)]
pub struct Config {
    pub database_url: String,
    pub database_pool_size: u32,
    pub database_concurrency_limit: usize,
    pub world_folder: PathBuf,
    pub usercache_path: PathBuf,
    pub port: String,
}

impl Config {
    pub fn from_env() -> Self {
        let port = std::env::var("PORT").unwrap_or_else(|_| "80".to_string());
        let database_pool_size = std::env::var("DATABASE_POOL_SIZE")
            .map(|v| v.parse().unwrap_or(30))
            .unwrap_or(30);
        let database_concurrency_limit = std::env::var("DATABASE_CONCURRENCY_LIMIT")
            .map(|v| v.parse().unwrap_or(2))
            .unwrap_or(2);
        let database_url = std::env::var("DATABASE_URL")
            .map(|url| {
                if url.starts_with("sqlite") {
                    url
                } else {
                    format!("sqlite://{}", url)
                }
            })
            .unwrap_or_else(|_| "sqlite://data/mcstats.db".to_string());

        let final_url = if database_url.starts_with("sqlite://") {
            let path = database_url.strip_prefix("sqlite://").unwrap();
            let abs_path = if std::path::Path::new(path).is_absolute() {
                path.to_string()
            } else {
                std::env::current_dir()
                    .map(|cwd| cwd.join(path))
                    .map(|p| p.to_string_lossy().to_string())
                    .unwrap_or_else(|_| path.to_string())
            };

            if let Some(parent) = std::path::Path::new(&abs_path).parent() {
                let _ = std::fs::create_dir_all(parent);
            }
            std::fs::File::create(&abs_path).ok();

            format!("sqlite://{}", abs_path)
        } else {
            database_url
        };
        let world_folder =
            PathBuf::from(std::env::var("WORLD_PATH").unwrap_or_else(|_| "data/world".to_string()));
        let usercache_path = PathBuf::from("data/usercache.json");

        Self {
            database_url: final_url,
            database_pool_size,
            database_concurrency_limit,
            world_folder,
            usercache_path,
            port,
        }
    }

    pub fn stats_folder(&self) -> PathBuf {
        self.world_folder.join("stats")
    }
}
