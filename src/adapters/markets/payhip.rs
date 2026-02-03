use crate::app_error::AppResult;

pub struct Payhip {
    url: String,
}

impl Payhip {
    pub fn new(&self, url: &str) -> Self {
        Self { url: url.to_owned() }
    }

    pub fn validate_key(&self, key: &str) -> AppResult<()> {
        Ok(())
    }
}
