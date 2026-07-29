//! MCP Events — Published by mcp-registry island

use crate::events::common::{DomainEvent, EventSource};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// MCP resolved for an analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpResolvedEvent {
    pub analysis_id: Uuid,
    pub organization_id: Uuid,
    pub mcp_ids: Vec<Uuid>,
    pub cache_hits: u32,
    pub db_lookups: u32,
    pub resolution_time_ms: u32,
}

impl DomainEvent for McpResolvedEvent {
    fn event_type(&self) -> &'static str {
        "mcp.resolved"
    }
    fn source(&self) -> EventSource {
        EventSource::McpRegistry
    }
}

/// New MCP context created
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpCreatedEvent {
    pub mcp_id: Uuid,
    pub mcp_type: String,
    pub scope: String,
    pub code: String,
}

impl DomainEvent for McpCreatedEvent {
    fn event_type(&self) -> &'static str {
        "mcp.created"
    }
    fn source(&self) -> EventSource {
        EventSource::McpRegistry
    }
}

/// MCP cache invalidated
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpCacheInvalidatedEvent {
    pub mcp_id: Uuid,
    pub cache_key: String,
}

impl DomainEvent for McpCacheInvalidatedEvent {
    fn event_type(&self) -> &'static str {
        "mcp.cache_invalidated"
    }
    fn source(&self) -> EventSource {
        EventSource::McpRegistry
    }
}
