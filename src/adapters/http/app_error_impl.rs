use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};

use crate::app_error::AppError;

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        tracing::error!(error = ?self, "Request failed");

        match self {
            AppError::Database(_) => {
                (StatusCode::INTERNAL_SERVER_ERROR, "Database error").into_response()
            }
            AppError::InvalidCredentials => {
                (StatusCode::UNAUTHORIZED, "Invalid credentials").into_response()
            }
            AppError::InvalidLicense => {
                (StatusCode::BAD_REQUEST, "Invalid or expired license").into_response()
            }
            AppError::ProductNotLinked => {
                (StatusCode::BAD_REQUEST, "This product was not registered. If you believe this to be an error, reach out to the seller.").into_response()
            }
            AppError::MarketError(_) => {
                (StatusCode::BAD_GATEWAY, "Market error").into_response()
            }
            AppError::NotFound => {
                (StatusCode::NOT_FOUND, "Not found").into_response()
            }
            AppError::Conflict(msg) => {
                (StatusCode::CONFLICT, msg).into_response()
            }
            AppError::BadRequest(msg) => {
                (StatusCode::BAD_REQUEST, msg).into_response()
            }
            AppError::Internal(_) => {
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal error").into_response()
            }
        }
    }
}
