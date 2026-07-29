//! Candidate Matching Handlers

use crate::auth_context::TenantAuth;
use crate::error_response::ApiError;
use crate::state::AppState;
use axum::extract::{Path, State};
use axum::Json;
use genflow_shared_infra::{error::AppError, Permission};
use sqlx::Row;
use std::sync::Arc;
use uuid::Uuid;

pub async fn calculate_match(
    auth: TenantAuth,
    State(state): State<Arc<AppState>>,
    Path((position_id, candidate_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<genflow_receptors::JobMatch>, ApiError> {
    auth.require_permission(Permission::CalculateMatch)?;
    require_position_tenant(&state, &auth, position_id).await?;
    let match_result = state
        .matching_engine
        .calculate_match(position_id, candidate_id)
        .await?;

    if let Err(error) = state
        .synaptic_bus
        .publish_event(&genflow_receptors::events::MatchCalculatedEvent {
            match_id: match_result.id,
            position_id: match_result.position_id,
            candidate_id: match_result.candidate_id,
            composite_score: match_result.composite_index.value(),
            human_review_required: match_result.human_review_required,
        })
        .await
    {
        tracing::warn!(
            error = %error,
            match_id = %match_result.id,
            "Failed to publish match calculated event"
        );
    }

    Ok(Json(match_result))
}

#[derive(serde::Deserialize)]
pub struct CreateInvitationRequest {
    pub position_id: Uuid,
    pub invited_by_rep_id: Uuid,
    pub email: Option<String>,
    pub phone: Option<String>,
}

pub async fn create_invitation(
    auth: TenantAuth,
    State(state): State<Arc<AppState>>,
    Json(req): Json<CreateInvitationRequest>,
) -> Result<Json<genflow_receptors::PositionInvite>, ApiError> {
    auth.require_permission(Permission::CreateInvitation)?;
    require_position_tenant(&state, &auth, req.position_id).await?;
    let invite = state
        .invitation_manager
        .create_invitation(req.position_id, req.invited_by_rep_id, req.email, req.phone)
        .await?;

    if let Err(error) = state
        .synaptic_bus
        .publish_event(&genflow_receptors::events::CandidateInvitedEvent {
            invite_id: invite.id,
            position_id: invite.position_id,
            candidate_id: invite.candidate_id,
            email: invite.email.clone(),
        })
        .await
    {
        tracing::warn!(
            error = %error,
            invite_id = %invite.id,
            "Failed to publish candidate invited event"
        );
    }

    Ok(Json(invite))
}

#[derive(serde::Serialize)]
pub struct AcceptInvitationResponse {
    pub status: &'static str,
    pub candidate_id: Uuid,
}

pub async fn accept_invitation(
    State(state): State<Arc<AppState>>,
    Path(code): Path<String>,
) -> Result<Json<AcceptInvitationResponse>, ApiError> {
    let candidate_id = state.invitation_manager.accept_invitation(&code).await?;
    Ok(Json(AcceptInvitationResponse {
        status: "accepted",
        candidate_id,
    }))
}

pub async fn generate_report(
    auth: TenantAuth,
    State(state): State<Arc<AppState>>,
    Path(match_id): Path<Uuid>,
) -> Result<Json<genflow_receptors::MatchReport>, ApiError> {
    auth.require_permission(Permission::GenerateReport)?;
    // Load the match from DB to get its details, casting DECIMAL fields to FLOAT8 to avoid SQLx type mismatch panic
    let row = sqlx::query(
        "SELECT job_match.id, job_match.position_id, job_match.candidate_id, job_match.composite_match_index::FLOAT8 as composite_match_index, job_match.confidence_score::FLOAT8 as confidence_score, job_match.status, job_match.human_review_required, job_match.calculated_at FROM job_matches job_match JOIN job_positions position ON position.id = job_match.position_id WHERE job_match.id = $1 AND position.organization_id = $2"
    )
        .bind(match_id)
        .bind(auth.organization_id())
        .fetch_optional(&state.db_pool)
        .await
        .map_err(|e| ApiError(AppError::Infrastructure(e.to_string())))?;

    match row {
        Some(row) => {
            let job_match = genflow_receptors::JobMatch {
                id: row.get("id"),
                position_id: row.get("position_id"),
                candidate_id: row.get("candidate_id"),
                capability_match: genflow_receptors::AxisMatch {
                    axis_code: "capability".to_string(),
                    match_percentage: genflow_receptors::Score::new_unchecked(0.0), // placeholder — would need JSONB parsing
                    gap_severity: genflow_receptors::GapSeverity::Aligned,
                    details: vec![],
                },
                output_kpi_match: genflow_receptors::AxisMatch {
                    axis_code: "output_kpi".to_string(),
                    match_percentage: genflow_receptors::Score::new_unchecked(0.0),
                    gap_severity: genflow_receptors::GapSeverity::Aligned,
                    details: vec![],
                },
                business_gap_match: genflow_receptors::AxisMatch {
                    axis_code: "business_gap".to_string(),
                    match_percentage: genflow_receptors::Score::new_unchecked(0.0),
                    gap_severity: genflow_receptors::GapSeverity::Aligned,
                    details: vec![],
                },
                work_style_alignment: genflow_receptors::AxisMatch {
                    axis_code: "work_style".to_string(),
                    match_percentage: genflow_receptors::Score::new_unchecked(0.0),
                    gap_severity: genflow_receptors::GapSeverity::Aligned,
                    details: vec![],
                },
                growth_motivation_match: genflow_receptors::AxisMatch {
                    axis_code: "growth_motivation".to_string(),
                    match_percentage: genflow_receptors::Score::new_unchecked(0.0),
                    gap_severity: genflow_receptors::GapSeverity::Aligned,
                    details: vec![],
                },
                composite_index: genflow_receptors::Score::new_unchecked(
                    row.get::<Option<f64>, _>("composite_match_index")
                        .unwrap_or(0.0) as f32,
                ),
                confidence_score: genflow_receptors::Score::new_unchecked(
                    row.get::<Option<f64>, _>("confidence_score").unwrap_or(0.0) as f32,
                ),
                status: genflow_receptors::MatchStatus::from_db_str(
                    &row.get::<String, _>("status"),
                )
                .unwrap_or(genflow_receptors::MatchStatus::PendingReview),
                human_review_required: row.get("human_review_required"),
                calculated_at: row.get("calculated_at"),
            };

            let report = state
                .report_generator
                .generate(&job_match, genflow_receptors::ReportType::ForEmployer)
                .await?;

            if let Err(error) = state
                .synaptic_bus
                .publish_event(&genflow_receptors::events::ReportGeneratedEvent {
                    report_id: report.id,
                    match_id: report.job_match_id,
                    report_type: report.report_type.as_db_str().to_string(),
                })
                .await
            {
                tracing::warn!(
                    error = %error,
                    report_id = %report.id,
                    "Failed to publish report generated event"
                );
            }

            Ok(Json(report))
        }
        None => Err(ApiError(AppError::NotFound(format!(
            "Match {match_id} not found"
        )))),
    }
}

#[derive(serde::Deserialize)]
pub struct RecordDecisionRequest {
    pub decision: String, // "shortlisted" | "not_selected" | "selected" | "under_review" | "withdrawn"
    /// Legacy field retained for a compatibility window. The authenticated JWT
    /// subject is authoritative and is always used for persistence.
    #[serde(default)]
    pub decided_by: Option<Uuid>,
    pub note: Option<String>,
}

pub async fn record_decision(
    auth: TenantAuth,
    State(state): State<Arc<AppState>>,
    Path(match_id): Path<Uuid>,
    Json(payload): Json<RecordDecisionRequest>,
) -> Result<axum::http::StatusCode, ApiError> {
    auth.require_permission(Permission::RecordDecision)?;
    require_match_tenant(&state, &auth, match_id).await?;
    if let Some(decided_by) = payload.decided_by {
        if decided_by != auth.user_id() {
            return Err(ApiError(AppError::Authorization(
                "Decision actor does not match the authenticated user".to_string(),
            )));
        }
    }

    let status = match payload.decision.as_str() {
        "pending_review" => genflow_receptors::MatchStatus::PendingReview,
        "under_review" => genflow_receptors::MatchStatus::UnderReview,
        "shortlisted" => genflow_receptors::MatchStatus::Shortlisted,
        "not_selected" => genflow_receptors::MatchStatus::NotSelected,
        "selected" => genflow_receptors::MatchStatus::Selected,
        "withdrawn" => genflow_receptors::MatchStatus::Withdrawn,
        _ => {
            return Err(ApiError(AppError::Business(
                "Invalid decision status".to_string(),
            )))
        }
    };

    state
        .matching_engine
        .record_decision(match_id, auth.user_id(), status, payload.note)
        .await?;

    Ok(axum::http::StatusCode::OK)
}

async fn require_position_tenant(
    state: &Arc<AppState>,
    auth: &TenantAuth,
    position_id: Uuid,
) -> Result<(), ApiError> {
    let position = state
        .position_engine
        .get_position(position_id)
        .await?
        .ok_or_else(|| {
            ApiError(AppError::NotFound(format!(
                "Position {position_id} not found"
            )))
        })?;
    auth.require_organization(position.organization_id)
}

async fn require_match_tenant(
    state: &Arc<AppState>,
    auth: &TenantAuth,
    match_id: Uuid,
) -> Result<(), ApiError> {
    let row = sqlx::query(
        "SELECT position.organization_id FROM job_matches job_match JOIN job_positions position ON position.id = job_match.position_id WHERE job_match.id = $1",
    )
    .bind(match_id)
    .fetch_optional(&state.db_pool)
    .await
    .map_err(|error| ApiError(AppError::Infrastructure(error.to_string())))?;
    let organization_id = row
        .map(|row| row.get("organization_id"))
        .ok_or_else(|| ApiError(AppError::NotFound(format!("Match {match_id} not found"))))?;
    auth.require_organization(organization_id)
}
