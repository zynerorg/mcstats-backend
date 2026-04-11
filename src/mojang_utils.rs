use anyhow::{Ok, Result};
use serde::Deserialize;
use std::{collections::HashMap, path::Path};
use uuid::Uuid;

#[derive(Debug, Deserialize)]
struct LookupEntry {
    #[serde(alias = "id")]
    uuid: Uuid,
    name: String,
}

#[derive(Debug, Clone)]
pub struct UsernameCache {
    uuid_to_name: HashMap<Uuid, String>,
    name_to_uuid: HashMap<String, Uuid>,
}

impl UsernameCache {
    pub fn new() -> Self {
        let uuid_to_name = HashMap::new();
        let name_to_uuid = HashMap::new();

        Self {
            uuid_to_name,
            name_to_uuid,
        }
    }

    pub fn from_usercache(path: &Path) -> Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let entries: Vec<LookupEntry> = serde_json::from_str(&content)?;

        let mut object = UsernameCache::new();

        for entry in entries {
            object.insert_to_cache(&entry);
        }

        Ok(object)
    }

    fn insert_to_cache(&mut self, data: &LookupEntry) {
        self.name_to_uuid
            .insert(data.name.clone(), data.uuid.clone());
        self.uuid_to_name
            .insert(data.uuid.clone(), data.name.clone());
    }

    pub async fn username_to_uuid(&mut self, name: &String) -> Option<Uuid> {
        if let Some(cached) = self.name_to_uuid.get(name) {
            return Some(*cached);
        }

        let response = reqwest::get(format!(
            "https://api.mojang.com/users/profiles/minecraft/{}",
            name
        ))
        .await
        .ok()?;

        let profile: LookupEntry = response.json().await.ok()?;
        self.insert_to_cache(&profile);
        Some(profile.uuid)
    }

    pub async fn uuid_to_username(&mut self, uuid: &Uuid) -> Option<String> {
        if let Some(cached) = self.uuid_to_name.get(uuid) {
            return Some(cached.clone());
        }

        let response = reqwest::get(format!(
            "https://api.minecraftservices.com/minecraft/profile/lookup/{}",
            uuid
        ))
        .await
        .ok()?;

        let profile: LookupEntry = response.json().await.ok()?;
        self.insert_to_cache(&profile);
        Some(profile.name)
    }
}
