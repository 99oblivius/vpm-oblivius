use thiserror::Error;

pub mod payhip;
pub use payhip::Payhip;

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
