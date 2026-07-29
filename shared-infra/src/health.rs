//! Health Check Utilities

use crate::RedisPool;
use serde::Serialize;
use sqlx::PgPool;
use std::sync::Arc;

#[derive(Debug, Serialize)]
pub struct HealthStatus {
    pub status: String,
    pub database: ComponentHealth,
    pub redis: ComponentHealth,
    pub version: String,
}

#[derive(Debug, Serialize)]
pub struct ComponentHealth {
    pub status: String,
    pub details: Option<String>,
}

pub struct HealthChecker {
    db: PgPool,
    redis: Arc<RedisPool>,
}

impl HealthChecker {
    pub fn new(db: PgPool, redis: Arc<RedisPool>) -> Self {
        Self { db, redis }
    }

    pub async fn check(&self) -> HealthStatus {
        let db_health = self.check_db().await;
        let redis_health = self.check_redis().await;

        let overall = if db_health.status == "healthy" && redis_health.status == "healthy" {
            "healthy"
        } else {
            "degraded"
        };

        HealthStatus {
            status: overall.to_string(),
            database: db_health,
            redis: redis_health,
            version: "2.0.0".to_string(),
        }
    }

    async fn check_db(&self) -> ComponentHealth {
        let result: Result<i32, sqlx::Error> =
            sqlx::query_scalar("SELECT 1").fetch_one(&self.db).await;

        match result {
            Ok(_) => ComponentHealth {
                status: "healthy".to_string(),
                details: None,
            },
            Err(e) => ComponentHealth {
                status: "unhealthy".to_string(),
                details: Some(e.to_string()),
            },
        }
    }

    async fn check_redis(&self) -> ComponentHealth {
        match self.redis.ping().await {
            Ok(_) => ComponentHealth {
                status: "healthy".to_string(),
                details: None,
            },
            Err(e) => ComponentHealth {
                status: "unhealthy".to_string(),
                details: Some(e.to_string()),
            },
        }
    }
}
