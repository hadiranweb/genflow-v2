//! MCP Runtime Traits — async trait definitions (requires sqlx)

use async_trait::async_trait;
use genflow_receptors::{McpContext, McpError, McpPromptFragment, McpScope, McpType};
use uuid::Uuid;

/// MCP Runtime Error — extends domain McpError with infrastructure errors
#[derive(Debug)]
pub enum McpRuntimeError {
    Domain(McpError),
    Database(sqlx::Error),
    Cache(String),
}

impl std::fmt::Display for McpRuntimeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Domain(e) => write!(f, "Domain: {e}"),
            Self::Database(e) => write!(f, "Database: {e}"),
            Self::Cache(msg) => write!(f, "Cache: {msg}"),
        }
    }
}

impl std::error::Error for McpRuntimeError {}

impl From<sqlx::Error> for McpRuntimeError {
    fn from(e: sqlx::Error) -> Self {
        Self::Database(e)
    }
}

impl From<McpError> for McpRuntimeError {
    fn from(e: McpError) -> Self {
        Self::Domain(e)
    }
}

/// Trait Repository برای MCP — Runtime (async, requires DB)
#[async_trait]
pub trait McpRepository: Send + Sync {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<McpContext>, McpRuntimeError>;
    async fn find_active_by_code(
        &self,
        mcp_type: McpType,
        scope: McpScope,
        code: &str,
    ) -> Result<Option<McpContext>, McpRuntimeError>;
    async fn save(&self, mcp: &McpContext) -> Result<Uuid, McpRuntimeError>;
    async fn record_usage(
        &self,
        analysis_id: Uuid,
        mcp_id: Uuid,
        usage_role: &str,
        cache_hit: bool,
    ) -> Result<(), McpRuntimeError>;
    async fn find_prompt_fragments(
        &self,
        mcp_id: Uuid,
        locale: &str,
    ) -> Result<Vec<McpPromptFragment>, McpRuntimeError>;
}

/// Trait Cache برای MCP — Runtime (async, requires Redis)
#[async_trait]
pub trait McpCache: Send + Sync {
    async fn get(&self, key: &str) -> Result<Option<McpContext>, McpRuntimeError>;
    async fn set(
        &self,
        key: &str,
        value: &McpContext,
        ttl_seconds: u64,
    ) -> Result<(), McpRuntimeError>;
    async fn invalidate(&self, key: &str) -> Result<(), McpRuntimeError>;
}

/// Trait Builder برای MCP Draft — Runtime (async)
#[async_trait]
pub trait McpBuilder: Send + Sync {
    async fn build_industry_draft(
        &self,
        industry_code: &str,
    ) -> Result<McpContext, McpRuntimeError>;
    async fn build_process_draft(
        &self,
        process_code: &str,
        industry_code: Option<&str>,
    ) -> Result<McpContext, McpRuntimeError>;
    async fn build_case_draft(&self, org_id: Uuid) -> Result<McpContext, McpRuntimeError>;
}
