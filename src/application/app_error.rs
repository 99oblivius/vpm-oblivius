use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
     #[error("Database error: {0}")]
     Database(String),

     #[error("Market error: {0}")]
     MarketError(String),

     #[error("Invalid credentials")]
     InvalidCredentials,

     #[error("Invalid license")]
     InvalidLicense,

     #[error("Product not linked")]
     ProductNotLinked,

     #[error("Not found")]
     NotFound,

     #[error("Conflict: {0}")]
     Conflict(String),

     #[error("{0}")]
     BadRequest(String),

     #[error("Internal error: {0}")]
     Internal(String),
}

pub type AppResult<T> = Result<T, AppError>;
