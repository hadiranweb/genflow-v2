//! Domain Event Definitions — Synaptic Hub payloads
//!
//! These events flow through the dual-layer event bus:
//! - Layer 1: tokio mpsc channels (in-process, zero-cost)
//! - Layer 2: Redis pub/sub (cross-container, production-grade)
//!
//! Each event is a serializable payload that islands publish and consume.

pub mod candidate_events;
pub mod common;
pub mod dashboard_events;
pub mod mcp_events;
pub mod position_events;

pub use candidate_events::*;
pub use common::{DomainEvent, EventEnvelope, EventSource};
pub use dashboard_events::*;
pub use mcp_events::*;
pub use position_events::*;
