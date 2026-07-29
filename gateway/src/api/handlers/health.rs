//! Health Check Handler

use crate::state::AppState;
use axum::extract::State;
use axum::Json;
use genflow_shared_infra::health::HealthStatus;
use std::sync::Arc;

pub async fn health_check(State(state): State<Arc<AppState>>) -> Json<HealthStatus> {
    let status = state.health_checker.check().await;
    Json(status)
}
