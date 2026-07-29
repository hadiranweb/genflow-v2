//! # MCP (Master Context Protocol) Domain — Pure Types
//!
//! Ultra-fine-grained MCP context definitions.
//! Each MCP Type operates as an independent Cell in the Island architecture.
//!
//! ## MCP Types (Cells)
//! - **PlatformPolicy**: Global governance (Legal, Privacy, Fairness)
//! - **Industry**: Industry standards (Retail, SaaS, Healthcare)
//! - **BusinessProcess**: Business process templates
//! - **StandardPosition**: Standard position templates
//! - **OrganizationContext**: Tenant-specific data
//! - **CaseTemporary**: Temporary context for a specific analysis
//!
//! ## Lifecycle
//! Draft → ReviewReady → Approved → Active → Deprecated → Archived

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::domain::mcp::McpError;

// ═══════════════════════════════════════════════════════════
// Enums
// ═══════════════════════════════════════════════════════════

/// نوع MCP — Each type is an independent Cell
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum McpType {
    PlatformPolicy,
    Industry,
    BusinessProcess,
    StandardPosition,
    OrganizationContext,
    CaseTemporary,
}

impl McpType {
    /// آیا این نوع قابل استفاده مجدد است؟
    pub fn is_reusable(&self) -> bool {
        !matches!(self, Self::CaseTemporary)
    }

    /// آیا این نوع باید کش شود؟
    pub fn is_cacheable(&self) -> bool {
        true // همه کش می‌شوند، اما TTL متفاوت
    }

    /// TTL پیش‌فرض در Redis (ثانیه)
    pub fn default_cache_ttl_seconds(&self) -> u64 {
        match self {
            Self::PlatformPolicy => 7 * 24 * 3600, // 7 روز
            Self::Industry => 24 * 3600,           // 24 ساعت
            Self::BusinessProcess => 24 * 3600,    // 24 ساعت
            Self::StandardPosition => 24 * 3600,   // 24 ساعت
            Self::OrganizationContext => 3600,     // 1 ساعت
            Self::CaseTemporary => 1800,           // 30 دقیقه
        }
    }

    /// به رشته DB تبدیل کن
    pub fn as_db_str(&self) -> &'static str {
        match self {
            Self::PlatformPolicy => "platform_policy",
            Self::Industry => "industry",
            Self::BusinessProcess => "business_process",
            Self::StandardPosition => "standard_position",
            Self::OrganizationContext => "organization_context",
            Self::CaseTemporary => "case_temporary",
        }
    }

    pub fn from_db_str(s: &str) -> Option<Self> {
        match s {
            "platform_policy" => Some(Self::PlatformPolicy),
            "industry" => Some(Self::Industry),
            "business_process" => Some(Self::BusinessProcess),
            "standard_position" => Some(Self::StandardPosition),
            "organization_context" => Some(Self::OrganizationContext),
            "case_temporary" => Some(Self::CaseTemporary),
            _ => None,
        }
    }
}

impl std::fmt::Display for McpType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_db_str())
    }
}

/// محدوده MCP
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum McpScope {
    Global,
    Industry,
    Tenant,
    Case,
}

impl McpScope {
    pub fn as_db_str(&self) -> &'static str {
        match self {
            Self::Global => "global",
            Self::Industry => "industry",
            Self::Tenant => "tenant",
            Self::Case => "case",
        }
    }

    pub fn from_db_str(s: &str) -> Option<Self> {
        match s {
            "global" => Some(Self::Global),
            "industry" => Some(Self::Industry),
            "tenant" => Some(Self::Tenant),
            "case" => Some(Self::Case),
            _ => None,
        }
    }
}

impl std::fmt::Display for McpScope {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_db_str())
    }
}

/// وضعیت چرخه عمر MCP
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum McpStatus {
    Draft,
    ReviewReady,
    Approved,
    Active,
    Deprecated,
    Archived,
}

impl McpStatus {
    pub fn as_db_str(&self) -> &'static str {
        match self {
            Self::Draft => "draft",
            Self::ReviewReady => "review_ready",
            Self::Approved => "approved",
            Self::Active => "active",
            Self::Deprecated => "deprecated",
            Self::Archived => "archived",
        }
    }

    pub fn from_db_str(s: &str) -> Option<Self> {
        match s {
            "draft" => Some(Self::Draft),
            "review_ready" => Some(Self::ReviewReady),
            "approved" => Some(Self::Approved),
            "active" => Some(Self::Active),
            "deprecated" => Some(Self::Deprecated),
            "archived" => Some(Self::Archived),
            _ => None,
        }
    }

    pub fn is_usable(&self) -> bool {
        matches!(self, Self::Active | Self::Approved)
    }
}

/// نوع پیوند بین MCPها
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum McpLinkType {
    Uses,
    Extends,
    Overrides,
    Composes,
    DerivedFrom,
}

impl McpLinkType {
    pub fn as_db_str(&self) -> &'static str {
        match self {
            Self::Uses => "uses",
            Self::Extends => "extends",
            Self::Overrides => "overrides",
            Self::Composes => "composes",
            Self::DerivedFrom => "derived_from",
        }
    }
}

/// نوع قطعه Prompt (Fragment) — Ultra-fine MCP granularity
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FragmentRole {
    IndustrySummary,
    CommonProcesses,
    CommonRoles,
    StandardKpis,
    CommonBottlenecks,
    PositionRequirements,
    PromptInstruction,
    ComplianceWarning,
}

impl FragmentRole {
    pub fn as_db_str(&self) -> &'static str {
        match self {
            Self::IndustrySummary => "industry_summary",
            Self::CommonProcesses => "common_processes",
            Self::CommonRoles => "common_roles",
            Self::StandardKpis => "standard_kpis",
            Self::CommonBottlenecks => "common_bottlenecks",
            Self::PositionRequirements => "position_requirements",
            Self::PromptInstruction => "prompt_instruction",
            Self::ComplianceWarning => "compliance_warning",
        }
    }

    pub fn from_db_str(s: &str) -> Option<Self> {
        match s {
            "industry_summary" => Some(Self::IndustrySummary),
            "common_processes" => Some(Self::CommonProcesses),
            "common_roles" => Some(Self::CommonRoles),
            "standard_kpis" => Some(Self::StandardKpis),
            "common_bottlenecks" => Some(Self::CommonBottlenecks),
            "position_requirements" => Some(Self::PositionRequirements),
            "prompt_instruction" => Some(Self::PromptInstruction),
            "compliance_warning" => Some(Self::ComplianceWarning),
            _ => None,
        }
    }
}

// ═══════════════════════════════════════════════════════════
// Structs
// ═══════════════════════════════════════════════════════════

/// موجودیت اصلی MCP — یک Cell در Island architecture
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpContext {
    pub id: Uuid,
    pub mcp_type: McpType,
    pub scope: McpScope,

    pub code: String,
    pub title: String,
    pub description: Option<String>,

    pub version: String,
    pub status: McpStatus,

    pub content: serde_json::Value,
    pub content_hash: String,

    pub evidence: serde_json::Value,
    pub source_refs: Vec<String>,
    pub source_quality_score: Option<f32>,

    // مالکیت
    pub organization_id: Option<Uuid>,
    pub industry_code: Option<String>,
    pub process_code: Option<String>,
    pub position_code: Option<String>,
    pub case_id: Option<Uuid>,

    // چرخه حیات
    pub reusable: bool,
    pub cacheable: bool,
    pub deterministic: bool,
    pub expires_at: Option<DateTime<Utc>>,

    pub policy_version: Option<String>,
    pub schema_version: String,

    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl McpContext {
    /// آیا این MCP فعال است؟
    pub fn is_active(&self) -> bool {
        matches!(self.status, McpStatus::Active)
    }

    /// آیا این MCP منقضی شده؟
    pub fn is_expired(&self) -> bool {
        self.expires_at.map(|exp| exp < Utc::now()).unwrap_or(false)
    }

    /// آیا این MCP برای استفاده در تحلیل مناسب است؟
    pub fn is_usable(&self) -> bool {
        self.status.is_usable() && !self.is_expired()
    }

    /// تولید کلید کش Redis
    pub fn cache_key(&self) -> String {
        format!(
            "mcp:ctx:{}:{}:{}:{}",
            self.mcp_type.as_db_str(),
            self.scope.as_db_str(),
            self.code,
            self.version
        )
    }

    /// اعتبارسنجی سازگاری scope با mcp_type
    pub fn validate_scope(&self) -> Result<(), McpError> {
        let valid = match self.mcp_type {
            McpType::PlatformPolicy => self.scope == McpScope::Global,
            McpType::Industry => matches!(self.scope, McpScope::Global | McpScope::Industry),
            McpType::BusinessProcess => matches!(self.scope, McpScope::Global | McpScope::Industry),
            McpType::StandardPosition => {
                matches!(self.scope, McpScope::Global | McpScope::Industry)
            }
            McpType::OrganizationContext => self.scope == McpScope::Tenant,
            McpType::CaseTemporary => self.scope == McpScope::Case,
        };

        if !valid {
            return Err(McpError::Validation(format!(
                "scope {:?} incompatible with mcp_type {:?}",
                self.scope, self.mcp_type
            )));
        }

        // case_temporary نباید reusable باشد
        if self.mcp_type == McpType::CaseTemporary && self.reusable {
            return Err(McpError::Validation(
                "case_temporary MCP must have reusable=false".to_string(),
            ));
        }

        Ok(())
    }

    /// محاسبه SHA-256 hash محتوا
    pub fn compute_hash(content: &serde_json::Value) -> String {
        use sha2::{Digest, Sha256};
        let json_str = serde_json::to_string(content).unwrap_or_default();
        let mut hasher = Sha256::new();
        hasher.update(json_str.as_bytes());
        format!("{:x}", hasher.finalize())
    }
}

/// بسته MCP برای یک تحلیل — Synaptic convergence result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpBundle {
    pub industry_mcp: Option<McpContext>,
    pub process_mcps: Vec<McpContext>,
    pub standard_position_mcps: Vec<McpContext>,
    pub organization_context_mcp: Option<McpContext>,
    pub case_mcp: Option<McpContext>,
    pub policy_guardrails: Vec<McpContext>,
    pub resolution_metadata: ResolutionMetadata,
}

impl McpBundle {
    /// تمام MCPهای استفاده‌شده
    pub fn all_mcps(&self) -> Vec<&McpContext> {
        let mut all = Vec::new();
        if let Some(ref m) = self.industry_mcp {
            all.push(m);
        }
        all.extend(self.process_mcps.iter());
        all.extend(self.standard_position_mcps.iter());
        if let Some(ref m) = self.organization_context_mcp {
            all.push(m);
        }
        if let Some(ref m) = self.case_mcp {
            all.push(m);
        }
        all.extend(self.policy_guardrails.iter());
        all
    }

    /// برآورد صرفه‌جویی توکن
    pub fn estimated_token_savings(&self) -> usize {
        let base = 500usize;
        let industry = self.industry_mcp.as_ref().map(|_| 1000).unwrap_or(0);
        let processes = self.process_mcps.len() * 800;
        let positions = self.standard_position_mcps.len() * 1200;
        let org = self
            .organization_context_mcp
            .as_ref()
            .map(|_| 1500)
            .unwrap_or(0);

        base + industry + processes + positions + org
    }
}

/// متادیتای فرآیند Resolution
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ResolutionMetadata {
    pub cache_hits: u32,
    pub db_lookups: u32,
    pub drafts_created: u32,
    pub total_time_ms: u32,
    pub token_savings_estimate: u32,
}

/// قطعه Prompt — Each fragment is a micro-cell in MCP
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpPromptFragment {
    pub id: Uuid,
    pub mcp_context_id: Uuid,
    pub fragment_key: String,
    pub fragment_role: FragmentRole,
    pub content: String,
    pub token_estimate: usize,
    pub content_hash: String,
    pub locale: String,
    pub active: bool,
}

/// پیوند بین دو MCP — Synaptic link
#[derive(Debug, Clone)]
pub struct McpContextLink {
    pub id: Uuid,
    pub parent_mcp_id: Uuid,
    pub child_mcp_id: Uuid,
    pub link_type: McpLinkType,
    pub weight: f32,
}

/// Helper: محاسبه SHA-256 hash محتوا
pub fn compute_content_hash(content: &serde_json::Value) -> String {
    McpContext::compute_hash(content)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::mcp::McpContextBuilder;

    #[test]
    fn test_mcp_type_reusability() {
        assert!(McpType::Industry.is_reusable());
        assert!(McpType::StandardPosition.is_reusable());
        assert!(!McpType::CaseTemporary.is_reusable());
    }

    #[test]
    fn test_mcp_ttl() {
        assert_eq!(
            McpType::PlatformPolicy.default_cache_ttl_seconds(),
            7 * 24 * 3600
        );
        assert_eq!(McpType::Industry.default_cache_ttl_seconds(), 24 * 3600);
        assert_eq!(McpType::CaseTemporary.default_cache_ttl_seconds(), 1800);
    }

    #[test]
    fn test_mcp_cache_key() {
        let mcp = McpContextBuilder::new(McpType::Industry, McpScope::Industry, "retail")
            .title("خرده‌فروشی")
            .build();

        assert_eq!(mcp.cache_key(), "mcp:ctx:industry:industry:retail:0.1.0");
    }

    #[test]
    fn test_bundle_token_savings() {
        let bundle = McpBundle {
            industry_mcp: Some(
                McpContextBuilder::new(McpType::Industry, McpScope::Industry, "retail").build(),
            ),
            process_mcps: vec![McpContextBuilder::new(
                McpType::BusinessProcess,
                McpScope::Industry,
                "inventory",
            )
            .build()],
            standard_position_mcps: vec![],
            organization_context_mcp: None,
            case_mcp: None,
            policy_guardrails: vec![],
            resolution_metadata: Default::default(),
        };

        // 500 (base) + 1000 (industry) + 800 (1 process) = 2300
        assert_eq!(bundle.estimated_token_savings(), 2300);
    }

    #[test]
    fn test_validate_scope() {
        let valid =
            McpContextBuilder::new(McpType::PlatformPolicy, McpScope::Global, "fairness").build();
        assert!(valid.validate_scope().is_ok());

        let valid2 =
            McpContextBuilder::new(McpType::OrganizationContext, McpScope::Tenant, "org-ctx")
                .build();
        assert!(valid2.validate_scope().is_ok());

        // Invalid: platform_policy with tenant scope
        let invalid = McpContext {
            ..McpContextBuilder::new(McpType::PlatformPolicy, McpScope::Tenant, "bad").build()
        };
        assert!(invalid.validate_scope().is_err());
    }

    #[test]
    fn test_compute_hash() {
        let content = serde_json::json!({"key": "value"});
        let hash = McpContext::compute_hash(&content);
        assert!(!hash.is_empty());
        assert_eq!(hash.len(), 64); // SHA-256 hex = 64 chars
    }

    #[test]
    fn test_deterministic_field() {
        let mcp = McpContextBuilder::new(McpType::Industry, McpScope::Industry, "retail").build();
        assert!(!mcp.deterministic); // default false
    }
}
