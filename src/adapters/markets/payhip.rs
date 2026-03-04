use std::sync::Arc;

use async_trait::async_trait;

use crate::app_error::{AppError, AppResult};
use crate::domain::{MarketCredentialStore, MarketPort};

pub struct Payhip {
    credentials: Arc<MarketCredentialStore>,
    base_url: String,
}

impl Payhip {
    pub fn new(credentials: Arc<MarketCredentialStore>, base_url: String) -> Self {
        Self { credentials, base_url }
    }
}

#[async_trait]
impl MarketPort for Payhip {
    fn name(&self) -> &'static str {
        "payhip"
    }

    fn check_format(&self, key: &str) -> bool {
        let parts: Vec<&str> = key.split('-').collect();
        parts.len() == 4
            && parts
                .iter()
                .all(|p| p.len() == 5 && p.chars().all(|c| c.is_ascii_alphanumeric()))
    }

    async fn verify_key(&self, key: &str) -> AppResult<Option<String>> {
        let creds = self
            .credentials
            .get(self.name())
            .ok_or_else(|| AppError::MarketError("Payhip not configured".into()))?;

        if !creds.active {
            return Ok(None);
        }

        let resp = reqwest::Client::new()
            .get(format!("{}/api/v2/license/verify", self.base_url))
            .query(&[("license_key", key)])
            .header("product-secret-key", &creds.api_key)
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

        match body["enabled"].as_bool() {
            Some(true) => Ok(body["product_link"].as_str().map(String::from)),
            _ => Ok(None),
        }
    }
}
