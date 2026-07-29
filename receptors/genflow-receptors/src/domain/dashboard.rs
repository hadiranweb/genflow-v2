//! Dashboard Domain

use crate::domain::score::Score;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardOverview {
    pub organization_id: Uuid,
    pub metrics: KeyMetrics,
    pub recent_activity: Vec<ActivityItem>,
    pub alerts: Vec<DashboardAlert>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyMetrics {
    pub total_positions: u32,
    pub active_positions: u32,
    pub filled_positions: u32,

    pub total_candidates_invited: u32,
    pub total_candidates_completed: u32,
    pub candidates_in_pipeline: u32,

    pub average_match_score: Option<Score>,
    pub average_time_to_hire_days: Option<f32>,

    pub positions_expiring_soon: Vec<PositionAlert>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PositionAlert {
    pub position_id: Uuid,
    pub title: String,
    pub days_until_expire: i64,
    pub candidates_count: u32,
    pub urgency: AlertUrgency,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AlertUrgency {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityItem {
    pub id: Uuid,
    pub actor_name: String,
    pub action: ActivityAction,
    pub entity_type: String,
    pub entity_title: String,
    pub timestamp: DateTime<Utc>,
    pub metadata: serde_json::Value,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ActivityAction {
    PositionCreated,
    CandidateInvited,
    AssessmentCompleted,
    MatchCalculated,
    CandidateShortlisted,
    CandidateHired,
    ReportDownloaded,
}

impl ActivityAction {
    pub fn from_db_str(s: &str) -> Self {
        match s {
            "position_created" => Self::PositionCreated,
            "candidate_invited" => Self::CandidateInvited,
            "assessment_completed" => Self::AssessmentCompleted,
            "match_calculated" => Self::MatchCalculated,
            "candidate_shortlisted" => Self::CandidateShortlisted,
            "candidate_hired" => Self::CandidateHired,
            "report_downloaded" => Self::ReportDownloaded,
            _ => Self::PositionCreated,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PositionDashboardDetail {
    pub position: PositionSummary,
    pub pipeline: PipelineStats,
    pub top_matches: Vec<MatchSummary>,
    pub recent_activity: Vec<ActivityItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PositionSummary {
    pub id: Uuid,
    pub title: String,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
    pub requirements_count: u32,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PipelineStats {
    pub invited: u32,
    pub registered: u32,
    pub in_progress: u32,
    pub completed: u32,
    pub shortlisted: u32,
    pub hired: u32,
    pub rejected: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatchSummary {
    pub match_id: Uuid,
    pub candidate_name: String,
    pub candidate_email: String,
    pub overall_score: Score,
    pub work_style_score: Score,
    pub risk_level: RiskLevel,
    pub status: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RiskLevel {
    Low,
    Medium,
    High,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardAlert {
    pub alert_type: AlertType,
    pub message: String,
    pub entity_id: Option<Uuid>,
    pub severity: AlertUrgency,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlertType {
    HighMatchFound,
    PositionExpiring,
    CandidateCompleted,
    SystemNotification,
}

impl AlertType {
    pub fn from_db_str(s: &str) -> Self {
        match s {
            "high_match_found" => Self::HighMatchFound,
            "position_expiring" => Self::PositionExpiring,
            "candidate_completed" => Self::CandidateCompleted,
            _ => Self::SystemNotification,
        }
    }
}
