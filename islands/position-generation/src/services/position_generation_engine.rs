//! Position Generation Engine — Orchestrates the full position generation pipeline
//!
//! Pipeline: Business Input → MCP Resolution → Need Discovery → Graph Build →
//!           Representative Calibration → Position Creation → DB Persistence → Event Publish

use crate::services::{BusinessNeedDiscovery, PositionGraphBuilder, RepresentativeCalibrator};
use genflow_receptors::{
    AxisWeights, BusinessAnalysisRequest, GeneratedPositionProfile, JobPosition, McpBundle,
    PositionGenerationEvidence, PositionGenerationMethod, PositionStatus, Score,
};
use genflow_shared_infra::{db::begin_organization_transaction, error::AppError};
use sqlx::{PgPool, Row};

use uuid::Uuid;

pub struct PositionGenerationEngine {
    pool: PgPool,
    need_discovery: BusinessNeedDiscovery,
    graph_builder: PositionGraphBuilder,
    calibrator: RepresentativeCalibrator,
}

impl PositionGenerationEngine {
    pub fn new(pool: PgPool) -> Self {
        Self {
            pool,
            need_discovery: BusinessNeedDiscovery::new(),
            graph_builder: PositionGraphBuilder::new(),
            calibrator: RepresentativeCalibrator::new(),
        }
    }

    /// Generate a position without a pre-resolved MCP bundle.
    ///
    /// This compatibility entry point is useful for isolated development flows.
    /// Gateway requests should prefer `generate_with_mcp_bundle` so the resolved
    /// context is retained as generation evidence.
    pub async fn generate(
        &self,
        request: &BusinessAnalysisRequest,
    ) -> Result<GeneratedPositionProfile, AppError> {
        self.generate_with_mcp_bundle(request, None).await
    }

    /// Generate a position and persist the resolved MCP context as auditable evidence.
    pub async fn generate_with_mcp_bundle(
        &self,
        request: &BusinessAnalysisRequest,
        mcp_bundle: Option<&McpBundle>,
    ) -> Result<GeneratedPositionProfile, AppError> {
        // 1. Discover business needs from input
        let needs = self.need_discovery.discover(request);

        // 2. Determine axis weights (with representative context adjustment)
        let weights = request
            .representative_context
            .as_ref()
            .map(|ctx| {
                let mut w = AxisWeights::default();
                if ctx.use_personality {
                    w.work_style += ctx.requested_weight * 0.10;
                    w.capability -= ctx.requested_weight * 0.05;
                }
                w
            })
            .unwrap_or_default();

        // 3. Build position graph
        let position_id = Uuid::new_v4();
        let mut graph = self.graph_builder.build(position_id, &weights);

        // 4. Apply representative calibration (if provided)
        if let Some(ctx) = &request.representative_context {
            self.calibrator
                .calibrate(
                    &mut graph,
                    genflow_receptors::RepresentativeRelation::Manager,
                    ctx.requested_weight,
                    ctx.use_personality,
                )
                .unwrap_or_else(|error| {
                    tracing::warn!(
                        error = %error,
                        organization_id = %request.organization_id,
                        "Representative calibration was rejected; continuing without calibration"
                    );
                });
        }

        // 5. Create position record
        let generation_method = match &request.input_mode {
            genflow_receptors::BusinessInputMode::DirectRequest { .. } => {
                PositionGenerationMethod::DirectRequest
            }
            genflow_receptors::BusinessInputMode::GapAnalysis { .. } => {
                PositionGenerationMethod::GapDriven
            }
            _ => PositionGenerationMethod::BusinessAnalysis,
        };

        let title = self.infer_title(&needs);

        // 6. Build evidence. MCP resolution is optional for resilience, but when
        // available it is persisted both in the immutable run snapshot and the
        // position evidence returned to API consumers.
        let mcp_contexts_used = mcp_bundle
            .map(|bundle| bundle.all_mcps().into_iter().map(|mcp| mcp.id).collect())
            .unwrap_or_default();
        let standards_used = mcp_bundle
            .map(|bundle| {
                bundle
                    .standard_position_mcps
                    .iter()
                    .map(|mcp| mcp.code.clone())
                    .collect()
            })
            .unwrap_or_default();
        let mut rationale: Vec<String> =
            needs.iter().map(|need| need.description.clone()).collect();
        if let Some(bundle) = mcp_bundle {
            rationale.push(format!(
                "MCP resolution retained {} contexts (cache_hits={}, db_lookups={}, drafts_created={})",
                bundle.all_mcps().len(),
                bundle.resolution_metadata.cache_hits,
                bundle.resolution_metadata.db_lookups,
                bundle.resolution_metadata.drafts_created,
            ));
        }
        let evidence = PositionGenerationEvidence {
            generation_method: generation_method.as_db_str().to_string(),
            business_needs_used: needs.iter().map(|need| need.need_id.clone()).collect(),
            mcp_contexts_used,
            standards_used,
            representative_calibration_used: request.representative_context.is_some(),
            representative_effective_weight: request
                .representative_context
                .as_ref()
                .map(|ctx| ctx.requested_weight)
                .unwrap_or(0.0),
            rationale,
        };

        // 7. Build requirements from graph dimensions
        let requirements: Vec<genflow_receptors::PositionRequirement> = graph
            .axes
            .iter()
            .flat_map(|axis| {
                axis.dimensions
                    .iter()
                    .map(|dim| genflow_receptors::PositionRequirement {
                        axis_code: axis.code,
                        requirement_type: genflow_receptors::RequirementType::Skill,
                        description: dim.description.clone(),
                        importance: if dim.is_mandatory {
                            genflow_receptors::RequirementImportance::Critical
                        } else {
                            genflow_receptors::RequirementImportance::Important
                        },
                        source: genflow_receptors::RequirementSource::Generated,
                        rationale: format!("Derived from {} axis", axis.code.as_str()),
                        score_range: dim.min.map(|m| {
                            (
                                m,
                                dim.ideal.unwrap_or_default(),
                                dim.max.unwrap_or(Score::max()),
                            )
                        }),
                    })
            })
            .collect();

        // ─────────────────────────────────────────────────
        // 8. PERSIST TO DATABASE
        // ─────────────────────────────────────────────────
        // Fail explicitly rather than dropping audit evidence if the MCP snapshot
        // cannot be serialized for the immutable generation-run record.
        let mcp_bundle_snapshot = serde_json::to_value(mcp_bundle).map_err(|error| {
            AppError::Internal(format!("Could not serialize MCP bundle snapshot: {error}"))
        })?;

        // Multi-table position generation must be atomic: either the analysis,
        // needs, run, position, graph, and requirements all commit together or
        // none of them do.
        let mut tx = begin_organization_transaction(&self.pool, request.organization_id).await?;

        // 8a. Create business_analysis record
        sqlx::query(
            "INSERT INTO business_analyses (id, organization_id, created_by_rep_id, title, analysis_type, status, input_data) VALUES ($1, $2, $3, $4, $5, 'completed', $6)"
        )
            .bind(request.analysis_id)
            .bind(request.organization_id)
            .bind(request.representative_id)
            .bind(&title)
            .bind(generation_method.as_db_str())
            .bind(serde_json::to_value(&request.input_mode).unwrap_or(serde_json::json!({})))
            .execute(&mut *tx)
            .await?;

        // 8b. Create business_needs records
        for need in &needs {
            sqlx::query(
                "INSERT INTO business_needs (id, business_analysis_id, need_id, need_type, description, urgency) VALUES ($1, $2, $3, $4, $5, $6)"
            )
                .bind(Uuid::new_v4())
                .bind(request.analysis_id)
                .bind(&need.need_id)
                .bind(need.need_type.as_db_str())
                .bind(&need.description)
                .bind(need.urgency.as_db_str())
                .execute(&mut *tx)
                .await?;
        }

        // 8c. Create position_generation_run record (audit trail)
        let run_id = Uuid::new_v4();
        sqlx::query(
            "INSERT INTO position_generation_runs (id, business_analysis_id, input_mode, mcp_bundle_snapshot, discovered_needs_count, selected_hypothesis_title, status, rep_calibration_applied, rep_effective_weight, completed_at) VALUES ($1, $2, $3, $4, $5, $6, 'completed', $7, $8, NOW())"
        )
            .bind(run_id)
            .bind(request.analysis_id)
            .bind(generation_method.as_db_str())
            .bind(&mcp_bundle_snapshot)
            .bind(needs.len() as i32)
            .bind(&title)
            .bind(request.representative_context.is_some())
            .bind(request.representative_context.as_ref().map(|ctx| ctx.requested_weight))
            .execute(&mut *tx)
            .await?;

        // 8d. Insert job_position — the core entity
        let position_code = format!("POS-{}", &position_id.to_string()[..8]);
        sqlx::query(
            "INSERT INTO job_positions (id, organization_id, created_by_rep_id, generation_run_id, position_code, title, description, generation_method, status, generation_evidence, standards_used) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)"
        )
            .bind(position_id)
            .bind(request.organization_id)
            .bind(request.representative_id)
            .bind(run_id)
            .bind(&position_code)
            .bind(&title)
            .bind(None::<String>) // description
            .bind(generation_method.as_db_str())
            .bind(PositionStatus::Draft.as_db_str())
            .bind(serde_json::to_value(&evidence).unwrap_or(serde_json::json!({})))
            .bind(serde_json::to_value(&evidence.standards_used).unwrap_or(serde_json::json!([])))
            .execute(&mut *tx)
            .await?;

        // 8e. Insert position_graph
        sqlx::query(
            "INSERT INTO position_graphs (id, job_position_id, graph_version, capability_axis, output_kpi_axis, business_gap_axis, work_style_axis, growth_motivation_axis, calibration_applied, calibration_notes) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)"
        )
            .bind(Uuid::new_v4())
            .bind(position_id)
            .bind(&graph.version)
            .bind(serde_json::to_value(graph.axes.iter().find(|a| a.code == genflow_receptors::AxisCode::Capability)).unwrap_or(serde_json::json!({})))
            .bind(serde_json::to_value(graph.axes.iter().find(|a| a.code == genflow_receptors::AxisCode::OutputKpi)).unwrap_or(serde_json::json!({})))
            .bind(serde_json::to_value(graph.axes.iter().find(|a| a.code == genflow_receptors::AxisCode::BusinessGap)).unwrap_or(serde_json::json!({})))
            .bind(serde_json::to_value(graph.axes.iter().find(|a| a.code == genflow_receptors::AxisCode::WorkStyle)).unwrap_or(serde_json::json!({})))
            .bind(serde_json::to_value(graph.axes.iter().find(|a| a.code == genflow_receptors::AxisCode::GrowthMotivation)).unwrap_or(serde_json::json!({})))
            .bind(graph.axes.iter().any(|a| a.calibration_applied))
            .bind(&graph.calibration_notes)
            .execute(&mut *tx)
            .await?;

        // 8f. Insert position_requirements
        for req in &requirements {
            sqlx::query(
                "INSERT INTO position_requirements (id, job_position_id, axis_code, requirement_type, description, importance, is_mandatory, source_type, rationale) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)"
            )
                .bind(Uuid::new_v4())
                .bind(position_id)
                .bind(req.axis_code.as_str())
                .bind(match req.requirement_type {
                    genflow_receptors::RequirementType::Knowledge => "knowledge",
                    genflow_receptors::RequirementType::Skill => "skill",
                    genflow_receptors::RequirementType::Ability => "ability",
                    genflow_receptors::RequirementType::PersonalityTrait => "personality_trait",
                    genflow_receptors::RequirementType::Experience => "experience",
                    genflow_receptors::RequirementType::Certification => "certification",
                })
                .bind(&req.description)
                .bind(match req.importance {
                    genflow_receptors::RequirementImportance::Critical => "critical",
                    genflow_receptors::RequirementImportance::Important => "important",
                    genflow_receptors::RequirementImportance::NiceToHave => "nice_to_have",
                })
                .bind(matches!(req.source, genflow_receptors::RequirementSource::Generated))
                .bind("generated")
                .bind(&req.rationale)
                .execute(&mut *tx)
                .await?;
        }

        tx.commit().await?;

        let position = JobPosition {
            id: position_id,
            organization_id: request.organization_id,
            created_by_rep_id: request.representative_id,
            position_code,
            title,
            description: None,
            generation_method,
            status: PositionStatus::Draft,
        };

        tracing::info!(
            position_id = %position_id,
            title = %position.title,
            "Position generated and persisted to database"
        );

        Ok(GeneratedPositionProfile {
            position,
            graph,
            requirements,
            evidence,
            warnings: vec![],
        })
    }

    /// Infer a position title from the discovered needs
    fn infer_title(&self, needs: &[genflow_receptors::BusinessNeed]) -> String {
        if needs.is_empty() {
            return "General Position".to_string();
        }

        let primary = &needs[0];
        match primary.need_type {
            genflow_receptors::BusinessNeedType::CapabilityGap => {
                format!("{} Specialist", primary.description)
            }
            genflow_receptors::BusinessNeedType::ProcessBottleneck => {
                format!("{} Manager", primary.description)
            }
            genflow_receptors::BusinessNeedType::GrowthOpportunity => {
                format!("{} Lead", primary.description)
            }
            genflow_receptors::BusinessNeedType::DirectPositionRequest => {
                primary.description.clone()
            }
            genflow_receptors::BusinessNeedType::RiskMitigation => {
                format!("{} Analyst", primary.description)
            }
        }
    }

    /// Get a position by ID from database
    pub async fn get_position(&self, id: Uuid) -> Result<Option<JobPosition>, AppError> {
        let row = sqlx::query(
            "SELECT id, organization_id, created_by_rep_id, position_code, title, description, generation_method, status FROM job_positions WHERE id = $1"
        )
            .bind(id)
            .fetch_optional(&self.pool)
            .await?;

        match row {
            Some(row) => Ok(Some(JobPosition {
                id: row.get("id"),
                organization_id: row.get("organization_id"),
                created_by_rep_id: row.get("created_by_rep_id"),
                position_code: row.get("position_code"),
                title: row.get("title"),
                description: row.get("description"),
                generation_method: match row.get::<String, _>("generation_method").as_str() {
                    "business_analysis" => PositionGenerationMethod::BusinessAnalysis,
                    "direct_request" => PositionGenerationMethod::DirectRequest,
                    "gap_driven" => PositionGenerationMethod::GapDriven,
                    _ => PositionGenerationMethod::BusinessAnalysis,
                },
                status: match row.get::<String, _>("status").as_str() {
                    "draft" => PositionStatus::Draft,
                    "active" => PositionStatus::Active,
                    "paused" => PositionStatus::Paused,
                    "filled" => PositionStatus::Filled,
                    "archived" => PositionStatus::Archived,
                    _ => PositionStatus::Draft,
                },
            })),
            None => Ok(None),
        }
    }
}
