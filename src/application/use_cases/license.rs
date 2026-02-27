use std::sync::Arc;

use chrono::Utc;
use futures::future::join_all;

use crate::domain::{License, LicenseRepository, MarketPort, Token, VerifyResult, check_gift};
use crate::app_error::{AppError, AppResult};

pub struct Markets {
    markets: Vec<Box<dyn MarketPort>>,
}

impl Markets {
    pub fn new() -> Self {
        Self { markets: vec![] }
    }

    pub fn add(mut self, market: Box<dyn MarketPort>) -> Self {
        if let Some(pos) = self.markets.iter().position(|m| m.name() == market.name()) {
            self.markets[pos] = market;
        } else {
            self.markets.push(market);
        }
        self
    }

    pub fn any_format_match(&self, key: &str) -> bool {
        self.markets.iter().any(|m| m.check_format(key))
    }

    pub async fn decrement_use(&self, market_name: &str, key: &str) {
        if let Some(market) = self.markets.iter().find(|m| m.name() == market_name) {
            let _ = market.decrement_use(key).await;
        }
    }

    pub async fn verify_parallel(&self, key: &str) -> Option<VerifyResult> {
        let futures = self.markets
            .iter()
            .filter(|m| m.check_format(key))
            .map(|market| async move {
                market.verify_key(key).await.ok().flatten().map(|product_id| VerifyResult {
                    market: market.name().to_string(),
                    product_id,
                })
            });

        join_all(futures).await.into_iter().flatten().next()
    }
}

#[derive(Clone)]
pub struct LicenseUseCases {
    db: Arc<dyn LicenseRepository>,
    markets: Arc<Markets>,
}

impl LicenseUseCases {
    pub fn new(
        db: Arc<dyn LicenseRepository>,
        markets: Arc<Markets>,
    ) -> Self {
        Self { db, markets }
    }

    pub async fn redeem(&self, license: &str) -> AppResult<String> {
        let is_gift = check_gift(license);
        let has_market = self.markets.any_format_match(license);

        if !is_gift && !has_market {
            return Err(AppError::InvalidLicense);
        }

        if let Some(token) = self.db.get(license).await? {
            return Ok(token);
        }

        if !has_market {
            return Err(AppError::InvalidLicense);
        }

        let result = self.markets.verify_parallel(license).await
            .ok_or(AppError::InvalidLicense)?;

        let uid = match self.db
            .get_package_uid_by_market_product(&result.market, &result.product_id)
            .await?
        {
            Some(uid) => uid,
            None => {
                self.markets.decrement_use(&result.market, license).await;
                return Err(AppError::ProductNotLinked);
            }
        };

        self.register(license, &uid, &result.market).await
    }

    pub async fn get(&self, license: &str) -> AppResult<Option<String>> {
        self.db.get(license).await
    }

    pub async fn list(&self, cursor: &i64, page_size: &i64) -> AppResult<Vec<License>> {
        self.db.list(cursor, page_size).await
    }

    pub async fn register(&self, license: &str, uid: &str, source: &str) -> AppResult<String> {
        let token = Token::generate().token;
        let created_at = Utc::now();
        self.db.register(license, &token, uid, source, &created_at).await?;
        Ok(token)
    }

    pub async fn enable(&self, license: &str) -> AppResult<()> {
        self.db.update(license, true).await
    }

    pub async fn disable(&self, license: &str) -> AppResult<()> {
        self.db.update(license, false).await
    }

    pub async fn increment_use_count(&self, token: &str) -> AppResult<()> {
        self.db.increment_use_count(token).await
    }

    pub async fn delete(&self, license: &str) -> AppResult<()> {
        self.db.delete(license).await
    }
}
