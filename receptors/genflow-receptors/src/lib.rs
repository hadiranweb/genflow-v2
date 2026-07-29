//! genflow-receptors — Shared domain types and event definitions
//!
//! Pure Rust types with NO database or async dependencies.
//! Every island and the gateway depends on this crate.
//!
//! Inspired by pema-platform-v2's "rust-commons" receptor concept.
//!
//! ## Module Structure
//! - `domain` — Business domain types (Score, MCP, Position, Candidate, etc.)
//! - `events` — Domain event definitions for the Synaptic Hub

pub mod domain;
pub mod events;

// Convenient re-exports
pub use domain::{
    assessment::{AssessmentMethod, BigFiveScores, CandidateProfile, RiasecScores},
    business_need::{BusinessNeed, BusinessNeedType, NeedUrgency},
    candidate::{Candidate, CandidateStatus, InviteStatus, PositionInvite},
    dashboard::{
        ActivityAction, ActivityItem, AlertType, AlertUrgency, DashboardAlert, DashboardOverview,
        KeyMetrics, MatchSummary, PipelineStats, PositionAlert, PositionDashboardDetail,
        PositionSummary, RiskLevel,
    },
    job_match::{
        AxisMatch, DimensionMatchDetail, FlagSeverity, GapSeverity, JobMatch, MatchReport,
        MatchStatus, ReportType, RiskFlag,
    },
    mcp::{
        FragmentRole, McpBundle, McpContext, McpContextBuilder, McpContextLink, McpError,
        McpLinkType, McpPromptFragment, McpScope, McpStatus, McpType, ResolutionMetadata,
    },
    position_generation::{
        AxisCode, AxisWeights, BusinessAnalysisRequest, BusinessAnalysisResult, BusinessInputMode,
        CapabilityLevel, DimensionRequirement, GeneratedPositionProfile, GenerationWarning,
        JobPosition, PositionGenerationEvidence, PositionGenerationMethod, PositionGraph,
        PositionGraphAxis, PositionHypothesis, PositionRequirement, PositionStatus,
        RepresentativeContextInput, RequirementImportance, RequirementSource, RequirementType,
        StandardPositionMatch, WarningSeverity,
    },
    representative::{PolicyError, RepresentativeInfluencePolicy, RepresentativeRelation},
    score::Score,
};
