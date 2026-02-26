// Post-quantum note: HMAC-SHA256 is a symmetric algorithm and is NOT vulnerable to
// quantum attacks. Grover's algorithm only halves effective security (256→128 bit),
// which remains computationally infeasible. No migration to post-quantum algorithms needed.

use std::time::Duration;

use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};

use crate::app_error::{AppError, AppResult};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum TokenType {
    Access,
    Refresh,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub exp: u64,
    pub iat: u64,
    pub token_type: TokenType,
}

pub fn create_token(
    secret: &str,
    ttl: Duration,
    username: &str,
    token_type: TokenType,
) -> AppResult<String> {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|e| AppError::Internal(e.to_string()))?;

    let claims = Claims {
        sub: username.to_string(),
        iat: now.as_secs(),
        exp: now.as_secs() + ttl.as_secs(),
        token_type,
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .map_err(|e| AppError::Internal(e.to_string()))
}

pub fn validate_token(secret: &str, token: &str) -> AppResult<Claims> {
    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &{
            let mut v = Validation::default();
            v.set_required_spec_claims(&["exp", "sub"]);
            v.leeway = 30;
            v
        },
    )
    .map_err(|_| AppError::InvalidCredentials)?;

    Ok(token_data.claims)
}

pub fn validate_access_token(secret: &str, token: &str, issued_after: u64) -> AppResult<Claims> {
    let claims = validate_token(secret, token)?;

    if claims.token_type != TokenType::Access {
        return Err(AppError::InvalidCredentials);
    }

    if claims.iat < issued_after {
        return Err(AppError::InvalidCredentials);
    }

    Ok(claims)
}

pub fn validate_refresh_token(secret: &str, token: &str, issued_after: u64) -> AppResult<Claims> {
    let claims = validate_token(secret, token)?;

    if claims.token_type != TokenType::Refresh {
        return Err(AppError::InvalidCredentials);
    }

    if claims.iat < issued_after {
        return Err(AppError::InvalidCredentials);
    }

    Ok(claims)
}
