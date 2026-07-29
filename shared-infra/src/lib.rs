//! genflow-shared-infra — Database, Redis, config, error, telemetry utilities
//!
//! Infrastructure shared across all islands and the gateway.
//! This crate provides PgPool setup, Redis connections, JWT auth,
//! unified error types, and tracing/metrics setup.

pub mod auth;
pub mod config;
pub mod db;
pub mod error;
pub mod health;
pub mod redis;
pub mod telemetry;

pub use auth::{AccessRole, AuthClaims, JwtAuth, Permission};
pub use config::AppConfig;
pub use db::DatabasePool;
pub use error::{AppError, AppResult};
pub use health::HealthChecker;
pub use redis::RedisPool;
