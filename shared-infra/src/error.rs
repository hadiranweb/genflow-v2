//! Unified Application Error Types
//!
//! AppError is a pure domain error type (no web framework dependency).
//! The gateway crate provides axum IntoResponse integration.

use serde::Serialize;
use sqlx::Error as SqlxError;

/// Application error — unified across all islands
#[derive(Debug)]
pub enum AppError {
    /// Domain validation error
    Validation(String),
    /// Not found
    NotFound(String),
    /// Authentication error (missing, expired or invalid credentials)
    Auth(String),
    /// Authorization error (authenticated actor lacks the required permission)
    Authorization(String),
    /// Infrastructure error (DB, Redis, network)
    Infrastructure(String),
    /// Business logic error
    Business(String),
    /// Internal error (shouldn't happen)
    Internal(String),
}

/// Error response body (for API serialization)
#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub error: String,
    pub code: u16,
    pub details: Option<String>,
}

impl AppError {
    pub fn status_code(&self) -> u16 {
        match self {
            Self::Validation(_) => 400,
            Self::NotFound(_) => 404,
            Self::Auth(_) => 401,
            Self::Authorization(_) => 403,
            Self::Infrastructure(_) => 503,
            Self::Business(_) => 409,
            Self::Internal(_) => 500,
        }
    }

    pub fn error_code(&self) -> &'static str {
        match self {
            Self::Validation(_) => "VALIDATION_ERROR",
            Self::NotFound(_) => "NOT_FOUND",
            Self::Auth(_) => "AUTH_ERROR",
            Self::Authorization(_) => "AUTHORIZATION_ERROR",
            Self::Infrastructure(_) => "INFRASTRUCTURE_ERROR",
            Self::Business(_) => "BUSINESS_ERROR",
            Self::Internal(_) => "INTERNAL_ERROR",
        }
    }

    pub fn to_response_body(&self) -> ErrorResponse {
        ErrorResponse {
            error: self.error_code().to_string(),
            code: self.status_code(),
            details: Some(self.to_string()),
        }
    }
}

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Validation(msg) => write!(f, "Validation: {msg}"),
            Self::NotFound(msg) => write!(f, "Not found: {msg}"),
            Self::Auth(msg) => write!(f, "Auth: {msg}"),
            Self::Authorization(msg) => write!(f, "Authorization: {msg}"),
            Self::Infrastructure(msg) => write!(f, "Infrastructure: {msg}"),
            Self::Business(msg) => write!(f, "Business: {msg}"),
            Self::Internal(msg) => write!(f, "Internal: {msg}"),
        }
    }
}

impl std::error::Error for AppError {}

/// Convenience type alias
pub type AppResult<T> = Result<T, AppError>;

// From conversions (infrastructure errors only)
impl From<SqlxError> for AppError {
    fn from(e: SqlxError) -> Self {
        Self::Infrastructure(format!("Database: {e}"))
    }
}

impl From<redis::RedisError> for AppError {
    fn from(e: redis::RedisError) -> Self {
        Self::Infrastructure(format!("Redis: {e}"))
    }
}

impl From<genflow_receptors::McpError> for AppError {
    fn from(e: genflow_receptors::McpError) -> Self {
        match e {
            genflow_receptors::McpError::Validation(msg) => Self::Validation(msg),
            genflow_receptors::McpError::NotFound(msg) => Self::NotFound(msg),
            genflow_receptors::McpError::Cache(msg) => Self::Infrastructure(msg),
            genflow_receptors::McpError::Serialization(msg) => Self::Internal(msg),
            genflow_receptors::McpError::Builder(msg) => Self::Business(msg),
        }
    }
}
