//! PgMcpRepository — PostgreSQL implementation of McpRepository trait

use crate::traits::{McpRepository, McpRuntimeError};
use async_trait::async_trait;
use genflow_receptors::{FragmentRole, McpContext, McpPromptFragment, McpScope, McpType};
use sqlx::{PgPool, Row};
use uuid::Uuid;

pub struct PgMcpRepository {
    pool: PgPool,
}

impl PgMcpRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

/// Helper: map sqlx row to McpContext domain type
fn row_to_context(row: &sqlx::postgres::PgRow) -> McpContext {
    McpContext {
        id: row.get("id"),
        mcp_type: McpType::from_db_str(&row.get::<String, _>("mcp_type"))
            .unwrap_or(McpType::PlatformPolicy),
        scope: McpScope::from_db_str(&row.get::<String, _>("scope")).unwrap_or(McpScope::Global),
        code: row.get("code"),
        title: row.get("title"),
        description: row.get("description"),
        version: row.get("version"),
        status: genflow_receptors::McpStatus::from_db_str(&row.get::<String, _>("status"))
            .unwrap_or(genflow_receptors::McpStatus::Draft),
        content: row.get("content"),
        content_hash: row.get("content_hash"),
        evidence: row.get("evidence"),
        source_refs: row.get("source_refs"),
        source_quality_score: row.get("source_quality_score"),
        organization_id: row.get("organization_id"),
        industry_code: row.get("industry_code"),
        process_code: row.get("process_code"),
        position_code: row.get("position_code"),
        case_id: row.get("case_id"),
        reusable: row.get("reusable"),
        cacheable: row.get("cacheable"),
        deterministic: row.get("deterministic"),
        expires_at: row.get("expires_at"),
        policy_version: row.get("policy_version"),
        schema_version: row.get("schema_version"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

#[async_trait]
impl McpRepository for PgMcpRepository {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<McpContext>, McpRuntimeError> {
        let result = sqlx::query("SELECT * FROM mcp_contexts WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await?;

        Ok(result.map(|row| row_to_context(&row)))
    }

    async fn find_active_by_code(
        &self,
        mcp_type: McpType,
        scope: McpScope,
        code: &str,
    ) -> Result<Option<McpContext>, McpRuntimeError> {
        let result = sqlx::query(
            "SELECT * FROM mcp_contexts WHERE mcp_type = $1 AND scope = $2 AND code = $3 AND status = 'active'"
        )
            .bind(mcp_type.as_db_str())
            .bind(scope.as_db_str())
            .bind(code)
            .fetch_optional(&self.pool)
            .await?;

        Ok(result.map(|row| row_to_context(&row)))
    }

    async fn save(&self, mcp: &McpContext) -> Result<Uuid, McpRuntimeError> {
        sqlx::query(
            "INSERT INTO mcp_contexts (id, mcp_type, scope, code, title, description, version, status, content, content_hash, evidence, source_refs, source_quality_score, organization_id, industry_code, process_code, position_code, case_id, reusable, cacheable, deterministic, expires_at, policy_version, schema_version, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19, $20, $21, $22, $23, $24, $25, $26) RETURNING id"
        )
            .bind(mcp.id)
            .bind(mcp.mcp_type.as_db_str())
            .bind(mcp.scope.as_db_str())
            .bind(&mcp.code)
            .bind(&mcp.title)
            .bind(&mcp.description)
            .bind(&mcp.version)
            .bind(mcp.status.as_db_str())
            .bind(&mcp.content)
            .bind(&mcp.content_hash)
            .bind(&mcp.evidence)
            .bind(&mcp.source_refs)
            .bind(mcp.source_quality_score)
            .bind(mcp.organization_id)
            .bind(&mcp.industry_code)
            .bind(&mcp.process_code)
            .bind(&mcp.position_code)
            .bind(mcp.case_id)
            .bind(mcp.reusable)
            .bind(mcp.cacheable)
            .bind(mcp.deterministic)
            .bind(mcp.expires_at)
            .bind(&mcp.policy_version)
            .bind(&mcp.schema_version)
            .bind(mcp.created_at)
            .bind(mcp.updated_at)
            .fetch_one(&self.pool)
            .await
            .map(|row| row.get("id"))
            .map_err(McpRuntimeError::Database)
    }

    async fn record_usage(
        &self,
        analysis_id: Uuid,
        mcp_id: Uuid,
        usage_role: &str,
        cache_hit: bool,
    ) -> Result<(), McpRuntimeError> {
        sqlx::query(
            "INSERT INTO business_analysis_mcp_usage (business_analysis_id, mcp_context_id, usage_role, cache_hit) VALUES ($1, $2, $3, $4)"
        )
            .bind(analysis_id)
            .bind(mcp_id)
            .bind(usage_role)
            .bind(cache_hit)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    async fn find_prompt_fragments(
        &self,
        mcp_id: Uuid,
        locale: &str,
    ) -> Result<Vec<McpPromptFragment>, McpRuntimeError> {
        let results = sqlx::query(
            "SELECT * FROM mcp_prompt_fragments WHERE mcp_context_id = $1 AND locale = $2 AND active = true"
        )
            .bind(mcp_id)
            .bind(locale)
            .fetch_all(&self.pool)
            .await?;

        Ok(results
            .iter()
            .map(|row| McpPromptFragment {
                id: row.get("id"),
                mcp_context_id: row.get("mcp_context_id"),
                fragment_key: row.get("fragment_key"),
                fragment_role: FragmentRole::from_db_str(&row.get::<String, _>("fragment_role"))
                    .unwrap_or(FragmentRole::PromptInstruction),
                content: row.get("content"),
                token_estimate: row.get::<i32, _>("token_estimate") as usize,
                content_hash: row.get("content_hash"),
                locale: row.get("locale"),
                active: row.get("active"),
            })
            .collect())
    }
}
