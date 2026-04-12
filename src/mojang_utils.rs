use anyhow::Result;
use serde::Deserialize;
use std::{collections::HashMap, path::Path};
use tokio::sync::RwLock;
use uuid::Uuid;

#[derive(Debug, Deserialize)]
struct LookupEntry {
    #[serde(alias = "id")]
    uuid: Uuid,
    name: String,
}

#[derive(Debug, Default)]
pub struct UsernameCache {
    uuid_to_name: RwLock<HashMap<Uuid, String>>,
    name_to_uuid: RwLock<HashMap<String, Uuid>>,
}

impl UsernameCache {
    pub fn new() -> Self {
        Self {
            uuid_to_name: RwLock::new(HashMap::new()),
            name_to_uuid: RwLock::new(HashMap::new()),
        }
    }

    pub async fn from_usercache(path: &Path) -> Result<Self> {
        let content = tokio::fs::read_to_string(path).await?;
        let entries: Vec<LookupEntry> = serde_json::from_str(&content)?;

        let cache = UsernameCache::new();

        {
            let mut uuid_to_name = cache.uuid_to_name.write().await;
            let mut name_to_uuid = cache.name_to_uuid.write().await;

            for entry in entries {
                name_to_uuid.insert(entry.name.clone(), entry.uuid);
                uuid_to_name.insert(entry.uuid, entry.name);
            }
        }

        Ok(cache)
    }

    pub async fn username_to_uuid(&self, name: &str) -> Option<Uuid> {
        if let Some(cached) = self.name_to_uuid.read().await.get(name).cloned() {
            return Some(cached);
        }

        let response = reqwest::get(format!(
            "https://api.mojang.com/users/profiles/minecraft/{}",
            name
        ))
        .await
        .ok()?;

        let profile: LookupEntry = response.json().await.ok()?;

        {
            let mut uuid_to_name = self.uuid_to_name.write().await;
            let mut name_to_uuid = self.name_to_uuid.write().await;

            name_to_uuid.insert(profile.name.clone(), profile.uuid);
            uuid_to_name.insert(profile.uuid, profile.name.clone());
        }

        Some(profile.uuid)
    }

    pub async fn uuid_to_username(&self, uuid: &Uuid) -> Option<String> {
        if let Some(cached) = self.uuid_to_name.read().await.get(uuid).cloned() {
            return Some(cached);
        }

        let response = reqwest::get(format!(
            "https://api.minecraftservices.com/minecraft/profile/lookup/{}",
            uuid
        ))
        .await
        .ok()?;

        let profile: LookupEntry = response.json().await.ok()?;

        {
            let mut uuid_to_name = self.uuid_to_name.write().await;
            let mut name_to_uuid = self.name_to_uuid.write().await;

            name_to_uuid.insert(profile.name.clone(), profile.uuid);
            uuid_to_name.insert(profile.uuid, profile.name.clone());
        }

        Some(profile.name)
    }
}
