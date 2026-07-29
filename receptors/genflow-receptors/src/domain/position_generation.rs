//! Position Generation Domain — Full domain types
//!
//! Business analysis → Need discovery → Graph → Calibration → Position generation

use crate::domain::mcp::ResolutionMetadata;
use crate::domain::score::Score;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// ورودی درخواست تحلیل
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BusinessAnalysisRequest {
    pub analysis_id: Uuid,
    pub organization_id: Uuid,
    pub representative_id: Uuid,
    pub input_mode: BusinessInputMode,
    pub industry_code: Option<String>,
    pub process_codes: Vec<String>,
    pub position_hints: Vec<String>,
    pub representative_context: Option<RepresentativeContextInput>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BusinessInputMode {
    Swot {
        strengths: Vec<String>,
        weaknesses: Vec<String>,
        opportunities: Vec<String>,
        threats: Vec<String>,
    },
    GapAnalysis {
        current_capabilities: Vec<CapabilityLevel>,
        target_capabilities: Vec<CapabilityLevel>,
        pain_points: Vec<String>,
    },
    DirectRequest {
        requested_title: String,
        reason: String,
        description: Option<String>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityLevel {
    pub capability: String,
    pub level: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepresentativeContextInput {
    pub use_personality: bool,
    pub requested_weight: f32,
}

/// فرضیه پوزیشن
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PositionHypothesis {
    pub title: String,
    pub position_code_hint: String,
    pub confidence: Score,
    pub source_need_ids: Vec<String>,
    pub rationale: Vec<String>,
    pub suggested_axis_weights: Option<AxisWeights>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AxisWeights {
    pub capability: f32,
    pub output_kpi: f32,
    pub business_gap: f32,
    pub work_style: f32,
    pub growth_motivation: f32,
}

impl Default for AxisWeights {
    fn default() -> Self {
        Self {
            capability: 0.25,
            output_kpi: 0.25,
            business_gap: 0.20,
            work_style: 0.20,
            growth_motivation: 0.10,
        }
    }
}

/// تطبیق با استاندارد صنعت
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StandardPositionMatch {
    pub mcp_context_id: Option<Uuid>,
    pub position_code: String,
    pub title: String,
    pub confidence: Score,
    pub template_content: serde_json::Value,
}

/// گراف ۵ محوره — The core output of position generation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PositionGraph {
    pub position_id: Uuid,
    pub version: String,
    pub axes: Vec<PositionGraphAxis>,
    pub calibration_notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PositionGraphAxis {
    pub code: AxisCode,
    pub weight: f32,
    pub description: String,
    pub dimensions: Vec<DimensionRequirement>,
    pub calibration_applied: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AxisCode {
    Capability,
    OutputKpi,
    BusinessGap,
    WorkStyle,
    GrowthMotivation,
}

impl AxisCode {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Capability => "capability",
            Self::OutputKpi => "output_kpi",
            Self::BusinessGap => "business_gap",
            Self::WorkStyle => "work_style",
            Self::GrowthMotivation => "growth_motivation",
        }
    }

    pub fn is_work_style(&self) -> bool {
        matches!(self, Self::WorkStyle)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DimensionRequirement {
    pub code: String,
    pub description: String,
    pub min: Option<Score>,
    pub ideal: Option<Score>,
    pub max: Option<Score>,
    pub is_mandatory: bool,
}

/// نیازمندی نهایی پوزیشن
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PositionRequirement {
    pub axis_code: AxisCode,
    pub requirement_type: RequirementType,
    pub description: String,
    pub importance: RequirementImportance,
    pub source: RequirementSource,
    pub rationale: String,
    pub score_range: Option<(Score, Score, Score)>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum RequirementType {
    Knowledge,
    Skill,
    Ability,
    PersonalityTrait,
    Experience,
    Certification,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum RequirementImportance {
    Critical,
    Important,
    NiceToHave,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RequirementSource {
    BusinessNeed { need_id: String },
    IndustryStandard { standard_ref: String },
    RepresentativeContext,
    Generated,
}

/// نتیجه نهایی تولید
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneratedPositionProfile {
    pub position: JobPosition,
    pub graph: PositionGraph,
    pub requirements: Vec<PositionRequirement>,
    pub evidence: PositionGenerationEvidence,
    pub warnings: Vec<GenerationWarning>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobPosition {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub created_by_rep_id: Uuid,
    pub position_code: String,
    pub title: String,
    pub description: Option<String>,
    pub generation_method: PositionGenerationMethod,
    pub status: PositionStatus,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum PositionGenerationMethod {
    BusinessAnalysis,
    DirectRequest,
    GapDriven,
}

impl PositionGenerationMethod {
    pub fn as_db_str(&self) -> &'static str {
        match self {
            Self::BusinessAnalysis => "business_analysis",
            Self::DirectRequest => "direct_request",
            Self::GapDriven => "gap_driven",
        }
    }

    pub fn from_db_str(s: &str) -> Option<Self> {
        match s {
            "business_analysis" => Some(Self::BusinessAnalysis),
            "direct_request" => Some(Self::DirectRequest),
            "gap_driven" => Some(Self::GapDriven),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum PositionStatus {
    Draft,
    Active,
    Paused,
    Filled,
    Archived,
}

impl PositionStatus {
    pub fn as_db_str(&self) -> &'static str {
        match self {
            Self::Draft => "draft",
            Self::Active => "active",
            Self::Paused => "paused",
            Self::Filled => "filled",
            Self::Archived => "archived",
        }
    }

    pub fn from_db_str(s: &str) -> Option<Self> {
        match s {
            "draft" => Some(Self::Draft),
            "active" => Some(Self::Active),
            "paused" => Some(Self::Paused),
            "filled" => Some(Self::Filled),
            "archived" => Some(Self::Archived),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PositionGenerationEvidence {
    pub generation_method: String,
    pub business_needs_used: Vec<String>,
    pub mcp_contexts_used: Vec<Uuid>,
    pub standards_used: Vec<String>,
    pub representative_calibration_used: bool,
    pub representative_effective_weight: f32,
    pub rationale: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerationWarning {
    pub code: String,
    pub severity: WarningSeverity,
    pub message: String,
    pub mitigation: Option<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum WarningSeverity {
    Info,
    Warning,
    Attention,
}

/// نتیجه تحلیل کسب‌وکار (intermediate)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BusinessAnalysisResult {
    pub analysis_id: Uuid,
    pub organization_id: Uuid,
    pub mcp_bundle_metadata: ResolutionMetadata,
    pub resolved_mcps: u32,
    pub case_mcp_id: Option<Uuid>,
}
