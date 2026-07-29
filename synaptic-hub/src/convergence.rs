//! Convergence Tracker — Multi-source event aggregation
//!
//! Tracks correlated events from different islands and triggers
//! composite actions when convergence patterns are detected.
//!
//! Example: When MCP resolved + Position generated + Candidate invited
//! all happen for the same organization, trigger a dashboard metrics update.

use chrono::{DateTime, Utc};
use genflow_receptors::events::EventEnvelope;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

/// Convergence pattern — a set of event types that must occur
/// for a composite action to trigger
#[derive(Debug, Clone)]
pub struct ConvergencePattern {
    pub pattern_id: String,
    pub required_events: Vec<String>,
    pub timeout_seconds: u64,
    pub action: ConvergenceAction,
}

/// What to do when convergence is detected
#[derive(Debug, Clone)]
pub enum ConvergenceAction {
    EmitEvent { event_type: String },
    Notify { channel: String },
    TriggerCalculation { calculation_type: String },
}

/// State of a convergence tracking session
#[derive(Debug)]
struct ConvergenceState {
    #[allow(dead_code)]
    correlation_id: Uuid,
    #[allow(dead_code)]
    organization_id: Option<Uuid>,
    received_events: HashMap<String, DateTime<Utc>>,
    #[allow(dead_code)]
    pattern_id: String,
}

/// Convergence Tracker — tracks multi-source event patterns
pub struct ConvergenceTracker {
    patterns: Vec<ConvergencePattern>,
    states: Arc<RwLock<HashMap<String, ConvergenceState>>>,
}

impl ConvergenceTracker {
    pub fn new(patterns: Vec<ConvergencePattern>) -> Self {
        Self {
            patterns,
            states: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Process an incoming event and check for convergence
    pub async fn process_event(&self, envelope: &EventEnvelope) -> Option<ConvergenceAction> {
        let event_type = &envelope.event_type;
        let correlation_id = envelope.correlation_id.unwrap_or(envelope.event_id);

        // Find matching patterns
        for pattern in &self.patterns {
            if !pattern.required_events.contains(event_type) {
                continue;
            }

            let key = format!("{}:{}", pattern.pattern_id, correlation_id);

            let mut states = self.states.write().await;
            let state = states
                .entry(key.clone())
                .or_insert_with(|| ConvergenceState {
                    correlation_id,
                    organization_id: None,
                    received_events: HashMap::new(),
                    pattern_id: pattern.pattern_id.clone(),
                });

            state
                .received_events
                .insert(event_type.clone(), envelope.timestamp);

            // Check if all required events have been received
            let all_received = pattern
                .required_events
                .iter()
                .all(|req| state.received_events.contains_key(req));

            if all_received {
                // Convergence detected! Trigger action and clean up state
                let action = pattern.action.clone();
                states.remove(&key);
                tracing::info!(
                    pattern_id = %pattern.pattern_id,
                    correlation_id = %correlation_id,
                    "Convergence pattern detected"
                );
                return Some(action);
            }
        }

        None
    }

    /// Clean up expired convergence states
    pub async fn cleanup_expired(&self) {
        let now = Utc::now();
        let mut states = self.states.write().await;
        states.retain(|_, state| {
            // Keep states that are not older than 1 hour
            let oldest = state.received_events.values().min().unwrap_or(&now);
            now.signed_duration_since(*oldest).num_hours() < 1
        });
    }

    /// Register default convergence patterns for GenFlow
    pub fn default_patterns() -> Vec<ConvergencePattern> {
        vec![
            // When position is generated AND MCP resolved → trigger candidate pipeline setup
            ConvergencePattern {
                pattern_id: "position_pipeline_init".to_string(),
                required_events: vec!["mcp.resolved".to_string(), "position.generated".to_string()],
                timeout_seconds: 3600,
                action: ConvergenceAction::TriggerCalculation {
                    calculation_type: "candidate_pipeline_setup".to_string(),
                },
            },
            // When match calculated AND report generated → trigger dashboard notification
            ConvergencePattern {
                pattern_id: "match_complete_notification".to_string(),
                required_events: vec![
                    "match.calculated".to_string(),
                    "report.generated".to_string(),
                ],
                timeout_seconds: 300,
                action: ConvergenceAction::Notify {
                    channel: "dashboard".to_string(),
                },
            },
        ]
    }
}

impl Default for ConvergenceTracker {
    fn default() -> Self {
        Self::new(Self::default_patterns())
    }
}
