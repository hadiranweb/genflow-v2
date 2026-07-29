//! Candidate Matching Events — Published by candidate-matching island

use crate::events::common::{DomainEvent, EventSource};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Candidate invited to a position
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CandidateInvitedEvent {
    pub invite_id: Uuid,
    pub position_id: Uuid,
    pub candidate_id: Option<Uuid>,
    pub email: Option<String>,
}

impl DomainEvent for CandidateInvitedEvent {
    fn event_type(&self) -> &'static str {
        "candidate.invited"
    }
    fn source(&self) -> EventSource {
        EventSource::CandidateMatching
    }
}

/// Match calculated
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatchCalculatedEvent {
    pub match_id: Uuid,
    pub position_id: Uuid,
    pub candidate_id: Uuid,
    pub composite_score: f32,
    pub human_review_required: bool,
}

impl DomainEvent for MatchCalculatedEvent {
    fn event_type(&self) -> &'static str {
        "match.calculated"
    }
    fn source(&self) -> EventSource {
        EventSource::CandidateMatching
    }
}

/// Report generated
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportGeneratedEvent {
    pub report_id: Uuid,
    pub match_id: Uuid,
    pub report_type: String,
}

impl DomainEvent for ReportGeneratedEvent {
    fn event_type(&self) -> &'static str {
        "report.generated"
    }
    fn source(&self) -> EventSource {
        EventSource::CandidateMatching
    }
}
