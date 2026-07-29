//! Business Analysis Engine — Orchestrator for business analysis pipeline

use genflow_mcp_registry::McpResolver;
use genflow_receptors::{BusinessAnalysisRequest, BusinessAnalysisResult};
use genflow_shared_infra::error::AppError;
use sqlx::PgPool;

pub struct BusinessAnalysisEngine {
    #[allow(dead_code)]
    pool: PgPool,
}

impl BusinessAnalysisEngine {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Run full business analysis pipeline
    pub async fn analyze<R, C, B>(
        &self,
        request: BusinessAnalysisRequest,
        resolver: &McpResolver<R, C, B>,
    ) -> Result<BusinessAnalysisResult, AppError>
    where
        R: genflow_mcp_registry::McpRepository,
        C: genflow_mcp_registry::McpCache,
        B: genflow_mcp_registry::McpBuilder,
    {
        // 1. Resolve MCP bundle
        let bundle = resolver
            .resolve_for_analysis(
                request.organization_id,
                request.industry_code.as_deref(),
                &request.process_codes,
                &request.position_hints,
                request.analysis_id,
            )
            .await
            .map_err(|e| AppError::Infrastructure(e.to_string()))?;

        // 2. Store analysis result
        let result = BusinessAnalysisResult {
            analysis_id: request.analysis_id,
            organization_id: request.organization_id,
            mcp_bundle_metadata: bundle.resolution_metadata.clone(),
            resolved_mcps: bundle.all_mcps().len() as u32,
            case_mcp_id: bundle.case_mcp.as_ref().map(|m| m.id),
        };

        tracing::info!(
            analysis_id = %request.analysis_id,
            resolved_mcps = result.resolved_mcps,
            "Business analysis completed"
        );

        Ok(result)
    }
}
