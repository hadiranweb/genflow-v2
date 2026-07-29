//! Dashboard Events — Published by dashboard-analytics island

use crate::events::common::{DomainEvent, EventSource};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Dashboard metrics updated
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardMetricsUpdatedEvent {
    pub organization_id: Uuid,
    pub active_positions: u32,
    pub candidates_in_pipeline: u32,
}

impl DomainEvent for DashboardMetricsUpdatedEvent {
    fn event_type(&self) -> &'static str {
        "dashboard.metrics_updated"
    }
    fn source(&self) -> EventSource {
        EventSource::DashboardAnalytics
    }
}

/// Dashboard alert triggered
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardAlertTriggeredEvent {
    pub organization_id: Uuid,
    pub alert_type: String,
    pub entity_id: Option<Uuid>,
    pub severity: String,
}

impl DomainEvent for DashboardAlertTriggeredEvent {
    fn event_type(&self) -> &'static str {
        "dashboard.alert_triggered"
    }
    fn source(&self) -> EventSource {
        EventSource::DashboardAnalytics
    }
}

/// Notification sent
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationSentEvent {
    pub notification_id: Uuid,
    pub target_user_id: Uuid,
    pub notification_type: String,
    pub channel: String,
}

impl DomainEvent for NotificationSentEvent {
    fn event_type(&self) -> &'static str {
        "dashboard.notification_sent"
    }
    fn source(&self) -> EventSource {
        EventSource::DashboardAnalytics
    }
}
