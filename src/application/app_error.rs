use thiserror::Error;

#[derive(Error, Debug)]
pub enum MarketErrorKind {
    #[error("Http error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("Request failed with status {status}: {message}")]
    BadStatus { status: u16, message: String },

    #[error("{0}")]
    Other(String),
}

impl MarketErrorKind {
    pub fn bad_status(response: &reqwest::Response, body: String) -> Self {
        Self::BadStatus {
            status: response.status().as_u16(),
            message: body,
        }
    }
}

#[derive(Error, Debug)]
pub enum AppError {
     #[error("Database error: {0}")]
     Database(#[from] sqlx::Error),

     #[error("Store error: {0}")]
     MarketError(#[from] MarketErrorKind),

     #[error("Invalid credentials")]
     InvalidCredentials,

     #[error("Internal error: {0}")]
     Internal(String),
}

pub type AppResult<T> = Result<T, AppError>;
