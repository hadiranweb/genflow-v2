//! Redis Pool — Connection and pub/sub helpers

use crate::config::RedisConfig;
use crate::error::AppError;
use redis::aio::MultiplexedConnection;
use redis::Client;

pub struct RedisPool {
    client: Client,
}

impl RedisPool {
    /// Create Redis client from config and verify connection
    pub async fn connect(config: &RedisConfig) -> Result<Self, AppError> {
        let client = Client::open(config.url.clone())
            .map_err(|e| AppError::Infrastructure(format!("Redis client creation failed: {e}")))?;

        let mut conn = client
            .get_multiplexed_async_connection()
            .await
            .map_err(|e| AppError::Infrastructure(format!("Redis connection failed: {e}")))?;

        // Verify connection with PING
        let result: String = redis::cmd("PING")
            .query_async(&mut conn)
            .await
            .map_err(|e| AppError::Infrastructure(format!("Redis ping failed: {e}")))?;

        if result != "PONG" {
            return Err(AppError::Infrastructure(
                "Redis ping did not return PONG".to_string(),
            ));
        }

        tracing::info!("Redis pool connected to {}", config.url);

        Ok(Self { client })
    }

    /// Get a fresh async connection for operations
    pub async fn connection(&self) -> Result<MultiplexedConnection, AppError> {
        self.client
            .get_multiplexed_async_connection()
            .await
            .map_err(|e| AppError::Infrastructure(format!("Redis connection failed: {e}")))
    }

    /// Get the client (for creating new connections)
    pub fn client(&self) -> &Client {
        &self.client
    }

    /// Ping check for health
    pub async fn ping(&self) -> Result<(), AppError> {
        let mut conn = self.connection().await?;
        let _: String = redis::cmd("PING")
            .query_async(&mut conn)
            .await
            .map_err(|e| AppError::Infrastructure(format!("Redis ping failed: {e}")))?;
        Ok(())
    }
}
