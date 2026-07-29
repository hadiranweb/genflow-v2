//! MCP Registry Handlers

use crate::auth_context::TenantAuth;
use crate::error_response::ApiError;
use crate::state::AppState;
use axum::extract::{Path, State};
use axum::Json;
use genflow_shared_infra::{error::AppError, Permission};
use std::sync::Arc;
use uuid::Uuid;

pub async fn get_mcp(
    _auth: TenantAuth,
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<genflow_receptors::McpContext>, ApiError> {
    _auth.require_permission(Permission::ReadMcp)?;
    let mcp = state
        .mcp_resolver
        .find_by_id(id)
        .await
        .map_err(|e| ApiError(AppError::Infrastructure(e.to_string())))?;

    match mcp {
        Some(ctx) => Ok(Json(ctx)),
        None => Err(ApiError(AppError::NotFound(format!("MCP {id} not found")))),
    }
}

#[derive(serde::Deserialize)]
pub struct ResolveMcpRequest {
    pub organization_id: Uuid,
    pub industry_code: Option<String>,
    pub process_codes: Vec<String>,
    pub position_hints: Vec<String>,
}

pub async fn resolve_mcp(
    auth: TenantAuth,
    State(state): State<Arc<AppState>>,
    Json(req): Json<ResolveMcpRequest>,
) -> Result<Json<genflow_receptors::McpBundle>, ApiError> {
    auth.require_permission(Permission::ResolveMcp)?;
    auth.require_organization(req.organization_id)?;
    let analysis_id = Uuid::new_v4();

    let bundle = state
        .mcp_resolver
        .resolve_for_analysis(
            req.organization_id,
            req.industry_code.as_deref(),
            &req.process_codes,
            &req.position_hints,
            analysis_id,
        )
        .await
        .map_err(|e| ApiError(AppError::Infrastructure(e.to_string())))?;

    let mcp_ids = bundle.all_mcps().into_iter().map(|mcp| mcp.id).collect();
    if let Err(error) = state
        .synaptic_bus
        .publish_event(&genflow_receptors::events::McpResolvedEvent {
            analysis_id,
            organization_id: req.organization_id,
            mcp_ids,
            cache_hits: bundle.resolution_metadata.cache_hits,
            db_lookups: bundle.resolution_metadata.db_lookups,
            resolution_time_ms: bundle.resolution_metadata.total_time_ms,
        })
        .await
    {
        tracing::warn!(
            error = %error,
            analysis_id = %analysis_id,
            "Failed to publish MCP resolved event"
        );
    }

    Ok(Json(bundle))
}
