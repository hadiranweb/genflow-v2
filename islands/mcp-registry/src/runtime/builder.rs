//! McpBuilderImpl — MCP draft builder implementation

use crate::traits::{McpBuilder, McpRuntimeError};
use async_trait::async_trait;
use genflow_receptors::{McpContext, McpContextBuilder, McpScope, McpStatus, McpType};
use uuid::Uuid;

pub struct McpBuilderImpl;

impl McpBuilderImpl {
    pub fn new() -> Self {
        Self
    }
}

impl Default for McpBuilderImpl {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl McpBuilder for McpBuilderImpl {
    async fn build_industry_draft(
        &self,
        industry_code: &str,
    ) -> Result<McpContext, McpRuntimeError> {
        let mcp = McpContextBuilder::new(McpType::Industry, McpScope::Industry, industry_code)
            .title(format!("Industry: {industry_code}"))
            .content(serde_json::json!({
                "industry_code": industry_code,
                "draft": true,
            }))
            .industry_code(industry_code)
            .build();

        // Set to ReviewReady since it's a draft
        let mcp = McpContext {
            status: McpStatus::ReviewReady,
            ..mcp
        };

        tracing::info!(code = %industry_code, "Industry draft built");
        Ok(mcp)
    }

    async fn build_process_draft(
        &self,
        process_code: &str,
        industry_code: Option<&str>,
    ) -> Result<McpContext, McpRuntimeError> {
        let scope = if industry_code.is_some() {
            McpScope::Industry
        } else {
            McpScope::Global
        };
        let mcp = McpContextBuilder::new(McpType::BusinessProcess, scope, process_code)
            .title(format!("Process: {process_code}"))
            .content(serde_json::json!({
                "process_code": process_code,
                "industry_code": industry_code,
                "draft": true,
            }))
            .industry_code(industry_code.unwrap_or_default())
            .build();

        let mcp = McpContext {
            status: McpStatus::ReviewReady,
            ..mcp
        };

        tracing::info!(code = %process_code, "Process draft built");
        Ok(mcp)
    }

    async fn build_case_draft(&self, org_id: Uuid) -> Result<McpContext, McpRuntimeError> {
        let mcp = McpContextBuilder::new(
            McpType::CaseTemporary,
            McpScope::Case,
            format!("case-{org_id}"),
        )
        .title(format!("Case for org {org_id}"))
        .organization_id(org_id)
        .content(serde_json::json!({
            "organization_id": org_id,
            "temporary": true,
        }))
        .build();

        // case_temporary is not reusable
        let mcp = McpContext {
            reusable: false,
            ..mcp
        };

        tracing::info!(org_id = %org_id, "Case draft built");
        Ok(mcp)
    }
}
