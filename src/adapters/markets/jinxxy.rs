use std::sync::Arc;

use async_trait::async_trait;

use crate::app_error::{AppError, AppResult};
use crate::domain::{MarketCredentialStore, MarketPort};

pub struct Jinxxy {
    credentials: Arc<MarketCredentialStore>,
    base_url: String,
}

impl Jinxxy {
    pub fn new(credentials: Arc<MarketCredentialStore>, base_url: String) -> Self {
        Self { credentials, base_url }
    }

    fn is_uuid(s: &str) -> bool {
        let hex_groups: Vec<&str> = s.split('-').collect();
        hex_groups.len() == 5
            && hex_groups[0].len() == 8
            && hex_groups[1].len() == 4
            && hex_groups[2].len() == 4
            && hex_groups[3].len() == 4
            && hex_groups[4].len() == 12
            && hex_groups.iter().all(|g| g.chars().all(|c| c.is_ascii_hexdigit()))
    }
}

#[async_trait]
impl MarketPort for Jinxxy {
    fn name(&self) -> &'static str {
        "jinxxy"
    }

    fn check_format(&self, key: &str) -> bool {
        Self::is_uuid(key)
    }

    async fn verify_key(&self, key: &str, _linked_product_ids: &[String]) -> AppResult<Option<String>> {
        let creds = self
            .credentials
            .get(self.name())
            .ok_or_else(|| AppError::MarketError("Jinxxy not configured".into()))?;

        if !creds.active {
            return Ok(None);
        }

        let client = reqwest::Client::new();

        // Look up license by full key
        let resp = client
            .get(format!("{}/v1/licenses", self.base_url))
            .query(&[("key", key)])
            .header("x-api-key", &creds.api_key)
            .send()
            .await
            .map_err(|e| AppError::MarketError(e.to_string()))?;

        if !resp.status().is_success() {
            return Ok(None);
        }

        let body: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| AppError::MarketError(e.to_string()))?;

        let license_id = match body["results"].as_array().and_then(|r| r.first()) {
            Some(license) => license["id"].as_str(),
            None => return Ok(None),
        };

        let license_id = match license_id {
            Some(id) => id,
            None => return Ok(None),
        };

        // Fetch full license details to get the product ID
        let resp = client
            .get(format!("{}/v1/licenses/{}", self.base_url, license_id))
            .header("x-api-key", &creds.api_key)
            .send()
            .await
            .map_err(|e| AppError::MarketError(e.to_string()))?;

        if !resp.status().is_success() {
            return Ok(None);
        }

        let body: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| AppError::MarketError(e.to_string()))?;

        Ok(body["inventory_item"]["target_id"].as_str().map(String::from))
    }
}
