//! Candidate Domain

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Candidate {
    pub id: Uuid,
    pub user_id: Option<Uuid>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub full_name: Option<String>,
    pub analysis_status: CandidateStatus,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CandidateStatus {
    Pending,
    Invited,
    Registered,
    InProgress,
    Completed,
}

impl CandidateStatus {
    pub fn as_db_str(&self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Invited => "invited",
            Self::Registered => "registered",
            Self::InProgress => "in_progress",
            Self::Completed => "completed",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PositionInvite {
    pub id: Uuid,
    pub position_id: Uuid,
    pub invited_by_rep_id: Uuid,
    pub candidate_id: Option<Uuid>,
    pub invite_code: String,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub status: InviteStatus,
    pub expires_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum InviteStatus {
    Created,
    Sent,
    Viewed,
    Accepted,
    Expired,
    Revoked,
}

impl InviteStatus {
    pub fn as_db_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Sent => "sent",
            Self::Viewed => "viewed",
            Self::Accepted => "accepted",
            Self::Expired => "expired",
            Self::Revoked => "revoked",
        }
    }
}

impl PositionInvite {
    pub fn generate_code() -> String {
        use rand::Rng;
        const CHARSET: &[u8] = b"ABCDEFGHJKLMNPQRSTUVWXYZ23456789";
        let mut rng = rand::thread_rng();
        let code: String = (0..8)
            .map(|_| CHARSET[rng.gen_range(0..CHARSET.len())] as char)
            .collect();
        format!("GF-{}-{}", &code[..4], &code[4..])
    }

    pub fn is_valid(&self) -> bool {
        matches!(self.status, InviteStatus::Sent | InviteStatus::Viewed)
            && self.expires_at > Utc::now()
    }
}
