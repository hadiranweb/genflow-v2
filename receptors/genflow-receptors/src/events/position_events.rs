//! Position Generation Events — Published by position-generation island

use crate::events::common::{DomainEvent, EventSource};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Business analysis completed
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BusinessAnalysisCompletedEvent {
    pub analysis_id: Uuid,
    pub organization_id: Uuid,
    pub needs_discovered: u32,
    pub mcp_ids_used: Vec<Uuid>,
}

impl DomainEvent for BusinessAnalysisCompletedEvent {
    fn event_type(&self) -> &'static str {
        "position.analysis_completed"
    }
    fn source(&self) -> EventSource {
        EventSource::PositionGeneration
    }
}

/// New position generated
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PositionGeneratedEvent {
    pub position_id: Uuid,
    pub organization_id: Uuid,
    pub position_code: String,
    pub title: String,
    pub generation_method: String,
}

impl DomainEvent for PositionGeneratedEvent {
    fn event_type(&self) -> &'static str {
        "position.generated"
    }
    fn source(&self) -> EventSource {
        EventSource::PositionGeneration
    }
}

/// Position graph built
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PositionGraphBuiltEvent {
    pub position_id: Uuid,
    pub axis_count: u32,
    pub calibration_applied: bool,
}

impl DomainEvent for PositionGraphBuiltEvent {
    fn event_type(&self) -> &'static str {
        "position.graph_built"
    }
    fn source(&self) -> EventSource {
        EventSource::PositionGeneration
    }
}
