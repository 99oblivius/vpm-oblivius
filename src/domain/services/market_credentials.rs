use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use crate::app_error::AppResult;
use crate::domain::{MarketConfigRepository, MarketCredentials};

pub struct MarketCredentialStore {
    cache: RwLock<HashMap<String, MarketCredentials>>,
    db: Arc<dyn MarketConfigRepository>,
}

impl MarketCredentialStore {
    pub async fn load(db: Arc<dyn MarketConfigRepository>) -> AppResult<Self> {
        let all = db.list_all().await?;
        let cache: HashMap<String, MarketCredentials> = all
            .into_iter()
            .map(|c| (c.market.clone(), c))
            .collect();

        Ok(Self {
            cache: RwLock::new(cache),
            db,
        })
    }

    pub fn get(&self, market: &str) -> Option<MarketCredentials> {
        self.cache.read().unwrap().get(market).cloned()
    }

    pub fn list(&self) -> Vec<MarketCredentials> {
        self.cache.read().unwrap().values().cloned().collect()
    }

    pub async fn update(&self, creds: MarketCredentials) -> AppResult<()> {
        self.db.upsert(&creds).await?;
        self.cache.write().unwrap().insert(creds.market.clone(), creds);
        Ok(())
    }
}
