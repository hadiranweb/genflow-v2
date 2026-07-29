//! MCP Error Types — Pure domain errors (no sqlx dependency)
//!
//! The `Database(sqlx::Error)` variant lives in the mcp-registry island
//! as `McpRuntimeError`. Here we only have domain-level errors.

use serde::{Deserialize, Serialize};

/// خطاهای MCP — Domain-level (no infrastructure coupling)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum McpError {
    Cache(String),
    Serialization(String),
    Validation(String),
    Builder(String),
    NotFound(String),
}

impl std::fmt::Display for McpError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Cache(msg) => write!(f, "Cache error: {msg}"),
            Self::Serialization(msg) => write!(f, "Serialization error: {msg}"),
            Self::Validation(msg) => write!(f, "Validation error: {msg}"),
            Self::Builder(msg) => write!(f, "Builder error: {msg}"),
            Self::NotFound(msg) => write!(f, "Not found: {msg}"),
        }
    }
}

impl std::error::Error for McpError {}

impl From<serde_json::Error> for McpError {
    fn from(e: serde_json::Error) -> Self {
        Self::Serialization(e.to_string())
    }
}
