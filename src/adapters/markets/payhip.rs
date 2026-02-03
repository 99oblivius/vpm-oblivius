use super::Market;
use crate::app_error::AppResult;

pub struct Payhip {
    url: String,
    api_key: String,
}

impl Payhip {
    pub fn new(url: String, api_key: String) -> Self {
        Self { url, api_key }
    }
}

impl Market for Payhip {
    fn verify_key(&self, key: &str) -> AppResult<bool> {
        Ok(true)
    }

    fn disable_key(&self, key: &str) -> AppResult<()> {
        Ok(())
    }

    fn enable_key(&self, key: &str) -> AppResult<()> {
        Ok(())
    }

    fn is_online(&self) -> AppResult<bool> {
        Ok(true)
    }
}
