//! Common event types and envelope

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Source of an event (which island produced it)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EventSource {
    McpRegistry,
    PositionGeneration,
    CandidateMatching,
    DashboardAnalytics,
    Gateway,
    External,
}

impl EventSource {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::McpRegistry => "mcp_registry",
            Self::PositionGeneration => "position_generation",
            Self::CandidateMatching => "candidate_matching",
            Self::DashboardAnalytics => "dashboard_analytics",
            Self::Gateway => "gateway",
            Self::External => "external",
        }
    }
}

/// Event envelope — wraps any domain event with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventEnvelope {
    pub event_id: Uuid,
    pub event_type: String,
    pub source: EventSource,
    pub timestamp: DateTime<Utc>,
    pub payload: serde_json::Value,
    pub correlation_id: Option<Uuid>,
}

impl EventEnvelope {
    pub fn new(
        source: EventSource,
        event_type: impl Into<String>,
        payload: serde_json::Value,
    ) -> Self {
        Self {
            event_id: Uuid::new_v4(),
            event_type: event_type.into(),
            source,
            timestamp: Utc::now(),
            payload,
            correlation_id: None,
        }
    }

    pub fn with_correlation_id(mut self, id: Uuid) -> Self {
        self.correlation_id = Some(id);
        self
    }

    /// Redis channel name for pub/sub routing
    pub fn channel_name(&self) -> String {
        format!("genflow:events:{}", self.event_type)
    }
}

/// Trait for domain events that can be serialized into an envelope
pub trait DomainEvent: Serialize {
    fn event_type(&self) -> &'static str;
    fn source(&self) -> EventSource;

    fn to_envelope(&self) -> EventEnvelope {
        EventEnvelope::new(
            self.source(),
            self.event_type(),
            serde_json::to_value(self).unwrap_or(serde_json::json!({})),
        )
    }
}
