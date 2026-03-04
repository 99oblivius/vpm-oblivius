use std::sync::Arc;

use async_trait::async_trait;

use crate::app_error::{AppError, AppResult};
use crate::domain::{MarketCredentialStore, MarketPort};

pub struct Gumroad {
    credentials: Arc<MarketCredentialStore>,
    base_url: String,
}

impl Gumroad {
    pub fn new(credentials: Arc<MarketCredentialStore>, base_url: String) -> Self {
        Self { credentials, base_url }
    }
}

#[async_trait]
impl MarketPort for Gumroad {
    fn name(&self) -> &'static str {
        "gumroad"
    }

    fn check_format(&self, key: &str) -> bool {
        // Gumroad license keys are 35-char hex strings with dashes: XXXXXXXX-XXXXXXXX-XXXXXXXX-XXXXXXXX
        let parts: Vec<&str> = key.split('-').collect();
        parts.len() == 4
            && parts.iter().all(|p| p.len() == 8 && p.chars().all(|c| c.is_ascii_hexdigit()))
    }

    async fn verify_key(&self, key: &str, linked_product_ids: &[String]) -> AppResult<Option<String>> {
        let creds = self
            .credentials
            .get(self.name())
            .ok_or_else(|| AppError::MarketError("Gumroad not configured".into()))?;

        if !creds.active {
            return Ok(None);
        }

        let client = reqwest::Client::new();

        // Gumroad requires product_id per request, so try each linked product
        for product_id in linked_product_ids {
            let resp = client
                .post(format!("{}/v2/licenses/verify", self.base_url))
                .header("Content-Type", "application/x-www-form-urlencoded")
                .body(format!(
                    "product_id={}&license_key={}&increment_uses_count=false",
                    product_id, key
                ))
                .send()
                .await
                .map_err(|e| AppError::MarketError(e.to_string()))?;

            if !resp.status().is_success() {
                continue;
            }

            let body: serde_json::Value = resp
                .json()
                .await
                .map_err(|e| AppError::MarketError(e.to_string()))?;

            if body["success"].as_bool() == Some(true) {
                let refunded = body["purchase"]["refunded"].as_bool().unwrap_or(false);
                let chargebacked = body["purchase"]["chargebacked"].as_bool().unwrap_or(false);
                if refunded || chargebacked {
                    continue;
                }
                return Ok(Some(product_id.clone()));
            }
        }

        Ok(None)
    }
}
