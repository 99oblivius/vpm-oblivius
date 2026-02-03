use std::sync::Arc;

#[derive(Clone)]
pub struct CodeUseCases {
    store: Arc<dyn Database>,
}

impl CodeUseCases {
    pub fn new(
        store: Arc<dyn Database>,
    ) -> Self {
        Self {
            store,
        }
    }

    pub async fn create(&self
    ) -> AppResult<()> {
        Ok(())
    }

    pub async fn disable(&self
    ) -> AppResult<()> {
        Ok(())
    }

    pub async fn enable(&self
    ) -> AppResult<()> {
        Ok(())
    }

    pub async fn delete(&self
    ) -> AppResult<()> {
        Ok(())
    }
}
