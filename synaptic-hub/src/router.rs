//! Event Router — Pattern matching and routing logic
//!
//! Routes events to the appropriate island based on event type.
//! Implements the "convergence patterns" from pema-platform-v2 architecture.

use genflow_receptors::events::EventSource;
use std::collections::HashMap;

/// Event routing table
pub struct EventRouter {
    routes: HashMap<String, Vec<EventSource>>,
}

impl Default for EventRouter {
    fn default() -> Self {
        Self::new()
    }
}

impl EventRouter {
    pub fn new() -> Self {
        let mut routes = HashMap::new();

        // MCP events → Position Generation, Dashboard
        routes.insert(
            "mcp.resolved".to_string(),
            vec![
                EventSource::PositionGeneration,
                EventSource::DashboardAnalytics,
            ],
        );

        // Position events → Candidate Matching, Dashboard
        routes.insert(
            "position.generated".to_string(),
            vec![
                EventSource::CandidateMatching,
                EventSource::DashboardAnalytics,
            ],
        );

        routes.insert(
            "position.analysis_completed".to_string(),
            vec![
                EventSource::CandidateMatching,
                EventSource::DashboardAnalytics,
            ],
        );

        // Candidate events → Dashboard
        routes.insert(
            "candidate.invited".to_string(),
            vec![EventSource::DashboardAnalytics],
        );

        routes.insert(
            "match.calculated".to_string(),
            vec![EventSource::DashboardAnalytics],
        );

        // Dashboard events → all (for cross-cutting concerns)
        routes.insert(
            "dashboard.metrics_updated".to_string(),
            vec![EventSource::Gateway],
        );

        routes.insert(
            "dashboard.alert_triggered".to_string(),
            vec![EventSource::Gateway],
        );

        Self { routes }
    }

    /// Get target islands for an event type
    pub fn route(&self, event_type: &str) -> Vec<EventSource> {
        self.routes.get(event_type).cloned().unwrap_or_default()
    }

    /// Check if an event should be routed to a specific island
    pub fn should_route_to(&self, event_type: &str, target: EventSource) -> bool {
        self.route(event_type).contains(&target)
    }

    /// Add a custom route
    pub fn add_route(&mut self, event_type: String, targets: Vec<EventSource>) {
        self.routes.insert(event_type, targets);
    }
}
