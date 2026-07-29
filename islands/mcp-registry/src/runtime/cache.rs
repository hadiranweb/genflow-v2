//! RedisMcpCache — Redis implementation of McpCache trait

use crate::traits::McpRuntimeError;
use async_trait::async_trait;
use genflow_receptors::McpContext;
use genflow_shared_infra::RedisPool;
use std::sync::Arc;

pub struct RedisMcpCache {
    redis: Arc<RedisPool>,
}

impl RedisMcpCache {
    pub fn new(redis: Arc<RedisPool>) -> Self {
        Self { redis }
    }
}

#[async_trait]
impl crate::traits::McpCache for RedisMcpCache {
    async fn get(&self, key: &str) -> Result<Option<McpContext>, McpRuntimeError> {
        let mut conn = self
            .redis
            .connection()
            .await
            .map_err(|e| McpRuntimeError::Cache(e.to_string()))?;

        let result: Option<String> = redis::cmd("GET")
            .arg(key)
            .query_async::<_, Option<String>>(&mut conn)
            .await
            .map_err(|e| McpRuntimeError::Cache(e.to_string()))?;

        match result {
            Some(json_str) => {
                let mcp: McpContext = serde_json::from_str(&json_str)
                    .map_err(|e| McpRuntimeError::Cache(format!("Deserialization: {e}")))?;
                Ok(Some(mcp))
            }
            None => Ok(None),
        }
    }

    async fn set(
        &self,
        key: &str,
        value: &McpContext,
        ttl_seconds: u64,
    ) -> Result<(), McpRuntimeError> {
        let json = serde_json::to_string(value)
            .map_err(|e| McpRuntimeError::Cache(format!("Serialization: {e}")))?;

        let mut conn = self
            .redis
            .connection()
            .await
            .map_err(|e| McpRuntimeError::Cache(e.to_string()))?;

        redis::cmd("SETEX")
            .arg(key)
            .arg(ttl_seconds)
            .arg(&json)
            .query_async::<_, ()>(&mut conn)
            .await
            .map_err(|e| McpRuntimeError::Cache(e.to_string()))?;

        Ok(())
    }

    async fn invalidate(&self, key: &str) -> Result<(), McpRuntimeError> {
        let mut conn = self
            .redis
            .connection()
            .await
            .map_err(|e| McpRuntimeError::Cache(e.to_string()))?;

        redis::cmd("DEL")
            .arg(key)
            .query_async::<_, ()>(&mut conn)
            .await
            .map_err(|e| McpRuntimeError::Cache(e.to_string()))?;

        Ok(())
    }
}
