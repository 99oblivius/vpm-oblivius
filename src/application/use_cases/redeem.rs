use std::sync::Arc;

#[derive(Clone)]
pub struct RedeemUseCases {
    services: Arc<dyn Markets>,
    store: Arc<dyn Database>,
}

impl RedeemUseCases {
    pub fn redeem(
        services: Arc<dyn Markets>,
        store: Arc<dyn Database>,
    ) -> Result<(), Box<dyn Error>> {
        Ok(())
    }
}
