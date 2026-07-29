//! Dashboard Analytics Handlers

use crate::auth_context::TenantAuth;
use crate::error_response::ApiError;
use crate::state::AppState;
use axum::extract::{Path, State};
use axum::Json;
use genflow_shared_infra::Permission;
use std::sync::Arc;
use uuid::Uuid;

pub async fn get_dashboard(
    auth: TenantAuth,
    State(state): State<Arc<AppState>>,
    Path(org_id): Path<Uuid>,
) -> Result<Json<genflow_receptors::DashboardOverview>, ApiError> {
    auth.require_permission(Permission::ReadDashboard)?;
    auth.require_organization(org_id)?;
    let overview = state.dashboard_engine.get_overview(org_id).await?;
    Ok(Json(overview))
}
