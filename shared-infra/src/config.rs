//! Application Configuration
//!
//! Environment-based config using sensible defaults.
//! All settings are loaded from environment variables.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AppConfig {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub redis: RedisConfig,
    pub jwt: JwtConfig,
    pub logging: LoggingConfig,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub workers: usize,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            host: "0.0.0.0".to_string(),
            port: 3000,
            workers: 4,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
    pub min_connections: u32,
    pub connect_timeout_seconds: u64,
    pub idle_timeout_seconds: u64,
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            url: "postgres://genflow:genflow@localhost:5432/genflow".to_string(),
            max_connections: 20,
            min_connections: 5,
            connect_timeout_seconds: 30,
            idle_timeout_seconds: 600,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RedisConfig {
    pub url: String,
    pub max_connections: u32,
}

impl Default for RedisConfig {
    fn default() -> Self {
        Self {
            url: "redis://localhost:6379".to_string(),
            max_connections: 10,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct JwtConfig {
    pub secret: String,
    pub issuer: String,
    pub expiration_hours: u64,
}

impl Default for JwtConfig {
    fn default() -> Self {
        Self {
            secret: "genflow-dev-secret-change-in-production".to_string(),
            issuer: "genflow".to_string(),
            expiration_hours: 24,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LoggingConfig {
    pub level: String,
    pub format: LogFormat,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum LogFormat {
    Json,
    Pretty,
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            level: "info".to_string(),
            format: LogFormat::Pretty,
        }
    }
}

impl AppConfig {
    /// Load from environment variables with defaults
    pub fn from_env() -> Self {
        Self {
            server: ServerConfig {
                host: std::env::var("SERVER_HOST").unwrap_or_else(|_| "0.0.0.0".to_string()),
                port: std::env::var("SERVER_PORT")
                    .ok()
                    .and_then(|v| v.parse().ok())
                    .unwrap_or(3000),
                workers: std::env::var("SERVER_WORKERS")
                    .ok()
                    .and_then(|v| v.parse().ok())
                    .unwrap_or(4),
            },
            database: DatabaseConfig {
                url: std::env::var("DATABASE_URL").unwrap_or_else(|_| {
                    "postgres://genflow:genflow@localhost:5432/genflow".to_string()
                }),
                max_connections: std::env::var("DB_MAX_CONNECTIONS")
                    .ok()
                    .and_then(|v| v.parse().ok())
                    .unwrap_or(20),
                min_connections: std::env::var("DB_MIN_CONNECTIONS")
                    .ok()
                    .and_then(|v| v.parse().ok())
                    .unwrap_or(5),
                connect_timeout_seconds: 30,
                idle_timeout_seconds: 600,
            },
            redis: RedisConfig {
                url: std::env::var("REDIS_URL")
                    .unwrap_or_else(|_| "redis://localhost:6379".to_string()),
                max_connections: 10,
            },
            jwt: JwtConfig {
                secret: std::env::var("JWT_SECRET")
                    .unwrap_or_else(|_| "genflow-dev-secret-change-in-production".to_string()),
                issuer: "genflow".to_string(),
                expiration_hours: std::env::var("JWT_EXPIRATION_HOURS")
                    .ok()
                    .and_then(|v| v.parse().ok())
                    .unwrap_or(24),
            },
            logging: LoggingConfig {
                level: std::env::var("LOG_LEVEL").unwrap_or_else(|_| "info".to_string()),
                format: match std::env::var("LOG_FORMAT").as_deref() {
                    Ok("json") => LogFormat::Json,
                    _ => LogFormat::Pretty,
                },
            },
        }
    }
}
