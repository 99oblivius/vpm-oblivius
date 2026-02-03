use crate::app_error::AppResult;

mod payhip;
pub use payhip::Payhip;

pub trait Market {
    fn verify_key(&self, key: &str) -> AppResult<bool>;
    fn disable_key(&self, key: &str) -> AppResult<()>;
    fn enable_key(&self, key: &str) -> AppResult<()>;
    fn is_online(&self) -> AppResult<bool>;
}

pub struct Markets {
    markets: Vec<Box<dyn Market + Send + Sync>>,
}

impl Markets {
    pub fn new() -> Self {
        Self { markets: vec![] }
    }

    pub fn add(mut self, market: Box<dyn Market + Send + Sync>) -> Self {
        self.markets.push(market);
        self
    }
}
