use rand::Rng;
use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};

pub fn verify_token(token: &str) -> bool {
    token.len() == 43 && token.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
}

#[derive(Clone)]
pub struct Token {
    pub token: String,
}

impl Token {
    pub fn generate() -> Self {
        let bytes: [u8; 32] = rand::rng().random();
        let token = URL_SAFE_NO_PAD.encode(bytes);
        Self { token }
    }
}
