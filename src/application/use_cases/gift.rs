use std::sync::Arc;

use chrono::{DateTime, Utc};

use crate::domain::{GiftCode, GiftRepository, Token};
use crate::app_error::AppResult;

#[derive(Clone)]
pub struct GiftUseCases {
    db: Arc<dyn GiftRepository>,
}

impl GiftUseCases {
    pub fn new(db: Arc<dyn GiftRepository>) -> Self {
        Self { db }
    }

    pub async fn create(&self, uid: &str) -> AppResult<(String, String, DateTime<Utc>)> {
        let code = GiftCode::generate().key;
        let token = Token::generate().token;
        let created_at = Utc::now();
        self.db.code_create(&code, &token, uid, &created_at).await?;
        Ok((code, token, created_at))
    }
}
