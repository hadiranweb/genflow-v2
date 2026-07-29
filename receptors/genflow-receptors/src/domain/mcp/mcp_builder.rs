//! MCP Context Builder — Pure domain builder (no async)

use chrono::Utc;
use uuid::Uuid;

use super::mcp_context::{McpContext, McpScope, McpStatus, McpType};

/// Builder Pattern برای ساخت MCP در کد
pub struct McpContextBuilder {
    mcp_type: McpType,
    scope: McpScope,
    code: String,
    title: String,
    description: Option<String>,
    content: serde_json::Value,
    organization_id: Option<Uuid>,
    industry_code: Option<String>,
}

impl McpContextBuilder {
    pub fn new(mcp_type: McpType, scope: McpScope, code: impl Into<String>) -> Self {
        Self {
            mcp_type,
            scope,
            code: code.into(),
            title: String::new(),
            description: None,
            content: serde_json::json!({}),
            organization_id: None,
            industry_code: None,
        }
    }

    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = title.into();
        self
    }

    pub fn description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }

    pub fn content(mut self, content: serde_json::Value) -> Self {
        self.content = content;
        self
    }

    pub fn organization_id(mut self, org_id: Uuid) -> Self {
        self.organization_id = Some(org_id);
        self
    }

    pub fn industry_code(mut self, code: impl Into<String>) -> Self {
        self.industry_code = Some(code.into());
        self
    }

    pub fn build(self) -> McpContext {
        let content_hash = McpContext::compute_hash(&self.content);

        McpContext {
            id: Uuid::new_v4(),
            mcp_type: self.mcp_type,
            scope: self.scope,
            code: self.code,
            title: self.title,
            description: self.description,
            version: "0.1.0".to_string(),
            status: McpStatus::Draft,
            content: self.content,
            content_hash,
            evidence: serde_json::json!({}),
            source_refs: vec![],
            source_quality_score: None,
            organization_id: self.organization_id,
            industry_code: self.industry_code,
            process_code: None,
            position_code: None,
            case_id: None,
            reusable: self.mcp_type.is_reusable(),
            cacheable: self.mcp_type.is_cacheable(),
            deterministic: false,
            expires_at: None,
            policy_version: None,
            schema_version: "0.1".to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }
}
