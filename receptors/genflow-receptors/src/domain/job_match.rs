//! Job Match Domain — 5-Axis Matching

use crate::domain::score::Score;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobMatch {
    pub id: Uuid,
    pub position_id: Uuid,
    pub candidate_id: Uuid,

    // ۵ محور تطابق
    pub capability_match: AxisMatch,
    pub output_kpi_match: AxisMatch,
    pub business_gap_match: AxisMatch,
    pub work_style_alignment: AxisMatch,
    pub growth_motivation_match: AxisMatch,

    pub composite_index: Score,
    pub confidence_score: Score,

    pub status: MatchStatus,
    pub human_review_required: bool,
    pub calculated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AxisMatch {
    pub axis_code: String,
    pub match_percentage: Score,
    pub gap_severity: GapSeverity,
    pub details: Vec<DimensionMatchDetail>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GapSeverity {
    Aligned,
    Acceptable,
    Development,
    Misaligned,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DimensionMatchDetail {
    pub dimension_code: String,
    pub required_range: (Score, Score),
    pub candidate_score: Score,
    pub match_percentage: Score,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MatchStatus {
    PendingReview,
    UnderReview,
    Shortlisted,
    NotSelected,
    Selected,
    Withdrawn,
}

impl MatchStatus {
    pub fn as_db_str(&self) -> &'static str {
        match self {
            Self::PendingReview => "pending_review",
            Self::UnderReview => "under_review",
            Self::Shortlisted => "shortlisted",
            Self::NotSelected => "not_selected",
            Self::Selected => "selected",
            Self::Withdrawn => "withdrawn",
        }
    }

    pub fn from_db_str(s: &str) -> Option<Self> {
        match s {
            "pending_review" => Some(Self::PendingReview),
            "under_review" => Some(Self::UnderReview),
            "shortlisted" => Some(Self::Shortlisted),
            "not_selected" => Some(Self::NotSelected),
            "selected" => Some(Self::Selected),
            "withdrawn" => Some(Self::Withdrawn),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatchReport {
    pub id: Uuid,
    pub job_match_id: Uuid,
    pub report_type: ReportType,
    pub title: String,
    pub summary: String,
    pub key_findings: Vec<String>,
    pub strengths: Vec<String>,
    pub development_areas: Vec<String>,
    pub recommendations: Vec<String>,
    pub disclaimers: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReportType {
    ForEmployer,
    ForCandidate,
}

impl ReportType {
    pub fn as_db_str(&self) -> &'static str {
        match self {
            Self::ForEmployer => "for_employer",
            Self::ForCandidate => "for_candidate",
        }
    }
}

/// Risk Flag (غیرانگ‌زننده)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskFlag {
    pub code: String,
    pub severity: FlagSeverity,
    pub description: String,
    pub mitigation: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FlagSeverity {
    Info,
    Attention,
    ActionRequired,
}

impl FlagSeverity {
    pub fn as_db_str(&self) -> &'static str {
        match self {
            Self::Info => "info",
            Self::Attention => "attention",
            Self::ActionRequired => "action_required",
        }
    }
}
