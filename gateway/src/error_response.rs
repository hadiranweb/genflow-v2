//! Axum IntoResponse integration for AppError
//!
//! AppError is a pure domain error (no axum dependency in shared-infra).
//! The gateway provides this newtype wrapper to satisfy the orphan rule.

use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use genflow_shared_infra::error::AppError;

/// Newtype wrapper around AppError to provide axum IntoResponse
pub struct ApiError(pub AppError);

impl From<AppError> for ApiError {
    fn from(e: AppError) -> Self {
        Self(e)
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let status = match self.0.status_code() {
            400 => StatusCode::BAD_REQUEST,
            401 => StatusCode::UNAUTHORIZED,
            403 => StatusCode::FORBIDDEN,
            404 => StatusCode::NOT_FOUND,
            409 => StatusCode::CONFLICT,
            500 => StatusCode::INTERNAL_SERVER_ERROR,
            503 => StatusCode::SERVICE_UNAVAILABLE,
            code => StatusCode::from_u16(code).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
        };

        let body = self.0.to_response_body();
        (status, Json(body)).into_response()
    }
}
