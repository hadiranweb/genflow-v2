//! McpResolver — Orchestrator for MCP resolution (Cache → DB → Build fallback)

use crate::traits::{McpBuilder, McpCache, McpRepository, McpRuntimeError};
use genflow_receptors::{
    McpBundle, McpContext, McpContextBuilder, McpScope, McpStatus, McpType, ResolutionMetadata,
};
use std::sync::Arc;
use std::time::Instant;
use uuid::Uuid;

pub struct McpResolver<R, C, B>
where
    R: McpRepository,
    C: McpCache,
    B: McpBuilder,
{
    repo: Arc<R>,
    cache: Arc<C>,
    builder: Arc<B>,
}

impl<R, C, B> McpResolver<R, C, B>
where
    R: McpRepository,
    C: McpCache,
    B: McpBuilder,
{
    pub fn new(repo: Arc<R>, cache: Arc<C>, builder: Arc<B>) -> Self {
        Self {
            repo,
            cache,
            builder,
        }
    }

    /// Find MCP by ID (delegates to repository)
    pub async fn find_by_id(&self, id: Uuid) -> Result<Option<McpContext>, McpRuntimeError> {
        self.repo.find_by_id(id).await
    }

    /// Try to resolve from cache → DB (no build fallback)
    async fn resolve_cached(
        &self,
        mcp_type: McpType,
        scope: McpScope,
        code: &str,
        meta: &mut ResolutionMetadata,
    ) -> Result<Option<McpContext>, McpRuntimeError> {
        let cache_key = format!(
            "mcp:ctx:{}:{}:{}:active",
            mcp_type.as_db_str(),
            scope.as_db_str(),
            code
        );

        // 1. Try cache
        if let Some(mcp) = self.cache.get(&cache_key).await? {
            meta.cache_hits += 1;
            tracing::debug!(key = %cache_key, "MCP found in cache");
            return Ok(Some(mcp));
        }

        // 2. Try database
        if let Some(mcp) = self.repo.find_active_by_code(mcp_type, scope, code).await? {
            meta.db_lookups += 1;
            // Populate cache
            let ttl = mcp_type.default_cache_ttl_seconds();
            self.cache.set(&cache_key, &mcp, ttl).await.ok();
            return Ok(Some(mcp));
        }

        Ok(None)
    }

    /// Full resolution for Business Analysis
    pub async fn resolve_for_analysis(
        &self,
        org_id: Uuid,
        industry_code: Option<&str>,
        process_codes: &[String],
        position_hints: &[String],
        _analysis_id: Uuid,
    ) -> Result<McpBundle, McpRuntimeError> {
        let start = Instant::now();
        let mut meta = ResolutionMetadata::default();

        // 1. Industry MCP
        let industry_mcp = if let Some(code) = industry_code {
            match self
                .resolve_cached(McpType::Industry, McpScope::Industry, code, &mut meta)
                .await?
            {
                Some(mcp) => Some(mcp),
                None => {
                    meta.drafts_created += 1;
                    Some(self.builder.build_industry_draft(code).await?)
                }
            }
        } else {
            None
        };

        // 2. Process MCPs
        let mut process_mcps = Vec::new();
        for code in process_codes {
            let mcp = match self
                .resolve_cached(
                    McpType::BusinessProcess,
                    McpScope::Industry,
                    code,
                    &mut meta,
                )
                .await?
            {
                Some(mcp) => mcp,
                None => {
                    meta.drafts_created += 1;
                    self.builder
                        .build_process_draft(code, industry_code)
                        .await?
                }
            };
            process_mcps.push(mcp);
        }

        // 3. Standard Position MCPs
        let mut standard_position_mcps = Vec::new();
        for hint in position_hints {
            let mcp = match self
                .resolve_cached(McpType::StandardPosition, McpScope::Global, hint, &mut meta)
                .await?
            {
                Some(mcp) => mcp,
                None => {
                    meta.drafts_created += 1;
                    // Build a simple standard position MCP (no specific builder method for this)
                    McpContext {
                        status: McpStatus::ReviewReady,
                        ..McpContextBuilder::new(McpType::StandardPosition, McpScope::Global, hint)
                            .title(format!("Position: {hint}"))
                            .build()
                    }
                }
            };
            standard_position_mcps.push(mcp);
        }

        // 4. Organization Context MCP
        let organization_context_mcp = match self
            .resolve_cached(
                McpType::OrganizationContext,
                McpScope::Tenant,
                &org_id.to_string(),
                &mut meta,
            )
            .await?
        {
            Some(mcp) => Some(mcp),
            None => {
                meta.drafts_created += 1;
                Some(self.builder.build_case_draft(org_id).await?)
            }
        };

        // 5. Policy Guardrails (always resolve global platform policies)
        let policy_guardrails = Vec::new(); // TODO: resolve policy MCPs

        // 6. Build Case MCP
        let case_mcp = self.builder.build_case_draft(org_id).await.ok();

        meta.total_time_ms = start.elapsed().as_millis() as u32;
        meta.token_savings_estimate = 0; // Will be calculated based on bundle

        Ok(McpBundle {
            industry_mcp,
            process_mcps,
            standard_position_mcps,
            organization_context_mcp,
            case_mcp,
            policy_guardrails,
            resolution_metadata: meta,
        })
    }
}
