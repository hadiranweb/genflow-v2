//! Position Generation Handlers

use crate::auth_context::TenantAuth;
use crate::error_response::ApiError;
use crate::state::AppState;
use axum::extract::{Path, State};
use axum::Json;
use genflow_receptors::BusinessAnalysisRequest;
use genflow_shared_infra::error::AppError;
use genflow_shared_infra::Permission;
use std::sync::Arc;
use uuid::Uuid;

#[derive(serde::Deserialize)]
pub struct GeneratePositionRequest {
    pub organization_id: Uuid,
    pub representative_id: Uuid,
    pub input_mode: genflow_receptors::BusinessInputMode,
    pub industry_code: Option<String>,
    pub process_codes: Vec<String>,
    pub position_hints: Vec<String>,
    pub representative_context: Option<genflow_receptors::RepresentativeContextInput>,
}

pub async fn generate_position(
    auth: TenantAuth,
    State(state): State<Arc<AppState>>,
    Json(req): Json<GeneratePositionRequest>,
) -> Result<Json<genflow_receptors::GeneratedPositionProfile>, ApiError> {
    auth.require_permission(Permission::GeneratePosition)?;
    auth.require_organization(req.organization_id)?;
    let analysis_request = BusinessAnalysisRequest {
        analysis_id: Uuid::new_v4(),
        organization_id: req.organization_id,
        representative_id: req.representative_id,
        input_mode: req.input_mode,
        industry_code: req.industry_code,
        process_codes: req.process_codes,
        position_hints: req.position_hints,
        representative_context: req.representative_context,
    };

    // MCPs enrich the generated evidence but are deliberately not a hard
    // availability dependency for position generation. A resolver outage is
    // observable and falls back to the deterministic generation pipeline.
    let mcp_bundle = match state
        .mcp_resolver
        .resolve_for_analysis(
            analysis_request.organization_id,
            analysis_request.industry_code.as_deref(),
            &analysis_request.process_codes,
            &analysis_request.position_hints,
            analysis_request.analysis_id,
        )
        .await
    {
        Ok(bundle) => {
            let mcp_ids: Vec<Uuid> = bundle.all_mcps().into_iter().map(|mcp| mcp.id).collect();
            if let Err(error) = state
                .synaptic_bus
                .publish_event(&genflow_receptors::events::McpResolvedEvent {
                    analysis_id: analysis_request.analysis_id,
                    organization_id: analysis_request.organization_id,
                    mcp_ids,
                    cache_hits: bundle.resolution_metadata.cache_hits,
                    db_lookups: bundle.resolution_metadata.db_lookups,
                    resolution_time_ms: bundle.resolution_metadata.total_time_ms,
                })
                .await
            {
                tracing::warn!(
                    error = %error,
                    analysis_id = %analysis_request.analysis_id,
                    "Failed to publish MCP resolved event during position generation"
                );
            }
            Some(bundle)
        }
        Err(error) => {
            tracing::warn!(
                error = %error,
                analysis_id = %analysis_request.analysis_id,
                "MCP resolution failed during position generation; continuing with deterministic defaults"
            );
            None
        }
    };

    let profile = state
        .position_engine
        .generate_with_mcp_bundle(&analysis_request, mcp_bundle.as_ref())
        .await?;

    if let Err(error) = state
        .synaptic_bus
        .publish_event(&genflow_receptors::events::BusinessAnalysisCompletedEvent {
            analysis_id: analysis_request.analysis_id,
            organization_id: analysis_request.organization_id,
            needs_discovered: profile.evidence.business_needs_used.len() as u32,
            mcp_ids_used: profile.evidence.mcp_contexts_used.clone(),
        })
        .await
    {
        tracing::warn!(
            error = %error,
            analysis_id = %analysis_request.analysis_id,
            "Failed to publish business analysis completed event"
        );
    }

    if let Err(error) = state
        .synaptic_bus
        .publish_event(&genflow_receptors::events::PositionGraphBuiltEvent {
            position_id: profile.position.id,
            axis_count: profile.graph.axes.len() as u32,
            calibration_applied: profile
                .graph
                .axes
                .iter()
                .any(|axis| axis.calibration_applied),
        })
        .await
    {
        tracing::warn!(
            error = %error,
            position_id = %profile.position.id,
            "Failed to publish position graph built event"
        );
    }

    if let Err(error) = state
        .synaptic_bus
        .publish_event(&genflow_receptors::events::PositionGeneratedEvent {
            position_id: profile.position.id,
            organization_id: profile.position.organization_id,
            position_code: profile.position.position_code.clone(),
            title: profile.position.title.clone(),
            generation_method: profile.position.generation_method.as_db_str().to_string(),
        })
        .await
    {
        tracing::warn!(
            error = %error,
            position_id = %profile.position.id,
            "Failed to publish position generated event"
        );
    }

    Ok(Json(profile))
}

pub async fn get_position(
    auth: TenantAuth,
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<genflow_receptors::JobPosition>, ApiError> {
    auth.require_permission(Permission::ReadPosition)?;
    let position = state
        .position_engine
        .get_position(id)
        .await
        .map_err(ApiError::from)?;

    match position {
        Some(pos) => {
            auth.require_organization(pos.organization_id)?;
            Ok(Json(pos))
        }
        None => Err(ApiError(AppError::NotFound(format!(
            "Position {id} not found"
        )))),
    }
}
