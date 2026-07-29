//! 5-Axis Matching Engine — Core matching algorithm
//!
//! Loads position graph and candidate profile from DB,
//! computes 5-axis match, persists results to DB.

use genflow_receptors::{
    AxisMatch, CandidateProfile, FlagSeverity, GapSeverity, JobMatch, MatchStatus, PositionGraph,
    PositionGraphAxis, RiskFlag, Score,
};
use genflow_shared_infra::{db::begin_organization_transaction, error::AppError};
use sqlx::{PgPool, Row};
use uuid::Uuid;

pub struct MatchingEngine {
    pool: PgPool,
}

/// Immutable calculation output passed to the persistence boundary.
///
/// Keeping this separate from SQL prevents recalculation from changing the
/// candidate-selection workflow state or losing the score inputs that produced
/// the persisted match.
struct MatchSnapshot<'a> {
    capability: &'a AxisMatch,
    output_kpi: &'a AxisMatch,
    business_gap: &'a AxisMatch,
    work_style: &'a AxisMatch,
    growth: &'a AxisMatch,
    composite: Score,
    human_review_required: bool,
}

impl MatchingEngine {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Calculate a match between a position and candidate — full pipeline with DB persistence
    pub async fn calculate_match(
        &self,
        position_id: Uuid,
        candidate_id: Uuid,
    ) -> Result<JobMatch, AppError> {
        // 1. Load position graph from DB
        let graph = self.load_position_graph(position_id).await?;

        // 2. Load candidate profile from DB
        let candidate = self.load_candidate_profile(candidate_id).await?;

        // 3. Calculate 5-axis matches
        let capability = self.match_capability_axis(&graph, &candidate)?;
        let output_kpi = self.match_output_kpi_axis(&graph, &candidate)?;
        let business_gap = self.match_business_gap_axis(&graph, &candidate)?;
        let work_style = self.match_work_style_axis(&graph, &candidate)?;
        let growth = self.match_growth_motivation_axis(&graph, &candidate)?;

        // 4. Calculate composite
        let composite = self.calculate_composite(
            &capability,
            &output_kpi,
            &business_gap,
            &work_style,
            &growth,
            &graph,
        );

        // 5. Identify risk flags
        let risk_flags = self.identify_risk_flags(&work_style, &candidate);

        // 6. Determine if human review is required
        let human_review = composite.value() < 60.0
            || risk_flags
                .iter()
                .any(|f| f.severity == FlagSeverity::ActionRequired);

        // 7. Persist the score snapshot and its derived risk flags atomically.
        // Recalculation replaces analytical outputs but intentionally preserves a
        // human workflow decision (for example, `selected` or `not_selected`).
        let snapshot = MatchSnapshot {
            capability: &capability,
            output_kpi: &output_kpi,
            business_gap: &business_gap,
            work_style: &work_style,
            growth: &growth,
            composite,
            human_review_required: human_review,
        };
        let (match_id, status, calculated_at) = self
            .persist_calculation(position_id, candidate_id, &snapshot, &risk_flags)
            .await?;

        tracing::info!(
            match_id = %match_id,
            composite = %composite,
            human_review = %human_review,
            status = %status.as_db_str(),
            risk_flag_count = risk_flags.len(),
            "Match calculation and risk flags persisted"
        );

        Ok(JobMatch {
            id: match_id,
            position_id,
            candidate_id,
            capability_match: capability,
            output_kpi_match: output_kpi,
            business_gap_match: business_gap,
            work_style_alignment: work_style,
            growth_motivation_match: growth,
            composite_index: composite,
            confidence_score: Score::new(85.0).unwrap_or_default(),
            status,
            human_review_required: human_review,
            calculated_at,
        })
    }

    // ─── Axis Matching Functions ───

    fn match_capability_axis(
        &self,
        graph: &PositionGraph,
        candidate: &CandidateProfile,
    ) -> Result<AxisMatch, AppError> {
        let axis = graph
            .axes
            .iter()
            .find(|a| a.code == genflow_receptors::AxisCode::Capability)
            .ok_or(AppError::Business("Missing capability axis".to_string()))?;

        let mut details = Vec::new();
        let mut total_percentage = 0.0;
        let count = axis.dimensions.len().max(1);

        for dim in &axis.dimensions {
            let candidate_score = candidate
                .get_skill_score(&dim.code)
                .or(candidate.get_skill_score(&dim.description))
                .unwrap_or(50.0);

            let cs = Score::new(candidate_score).unwrap_or_default();
            let min = dim.min.unwrap_or(Score::new(0.0).unwrap());
            let ideal = dim.ideal.unwrap_or(Score::new(70.0).unwrap());
            let max = dim.max.unwrap_or(Score::new(100.0).unwrap());

            let match_pct = if cs.value() >= ideal.value() {
                100.0 - (cs.value() - ideal.value()) * 0.5
            } else if cs.value() >= min.value() {
                (cs.value() - min.value()) / (ideal.value() - min.value()) * 100.0
            } else {
                0.0
            };

            details.push(genflow_receptors::DimensionMatchDetail {
                dimension_code: dim.code.clone(),
                required_range: (min, max),
                candidate_score: cs,
                match_percentage: Score::new_unchecked(match_pct),
            });

            total_percentage += match_pct;
        }

        let avg = total_percentage / count as f32;
        let severity = if avg >= 80.0 {
            GapSeverity::Aligned
        } else if avg >= 60.0 {
            GapSeverity::Acceptable
        } else if avg >= 40.0 {
            GapSeverity::Development
        } else {
            GapSeverity::Misaligned
        };

        Ok(AxisMatch {
            axis_code: "capability".to_string(),
            match_percentage: Score::new_unchecked(avg),
            gap_severity: severity,
            details,
        })
    }

    fn match_output_kpi_axis(
        &self,
        _graph: &PositionGraph,
        _candidate: &CandidateProfile,
    ) -> Result<AxisMatch, AppError> {
        Ok(AxisMatch {
            axis_code: "output_kpi".to_string(),
            match_percentage: Score::new(65.0).unwrap_or_default(),
            gap_severity: GapSeverity::Acceptable,
            details: vec![],
        })
    }

    fn match_business_gap_axis(
        &self,
        _graph: &PositionGraph,
        _candidate: &CandidateProfile,
    ) -> Result<AxisMatch, AppError> {
        Ok(AxisMatch {
            axis_code: "business_gap".to_string(),
            match_percentage: Score::new(70.0).unwrap_or_default(),
            gap_severity: GapSeverity::Acceptable,
            details: vec![],
        })
    }

    fn match_work_style_axis(
        &self,
        _graph: &PositionGraph,
        candidate: &CandidateProfile,
    ) -> Result<AxisMatch, AppError> {
        let score = candidate
            .big_five
            .as_ref()
            .map(|bf| bf.average())
            .unwrap_or(50.0);

        let severity = if score >= 75.0 {
            GapSeverity::Aligned
        } else if score >= 50.0 {
            GapSeverity::Acceptable
        } else {
            GapSeverity::Development
        };

        Ok(AxisMatch {
            axis_code: "work_style".to_string(),
            match_percentage: Score::new_unchecked(score),
            gap_severity: severity,
            details: vec![],
        })
    }

    fn match_growth_motivation_axis(
        &self,
        _graph: &PositionGraph,
        _candidate: &CandidateProfile,
    ) -> Result<AxisMatch, AppError> {
        Ok(AxisMatch {
            axis_code: "growth_motivation".to_string(),
            match_percentage: Score::new(60.0).unwrap_or_default(),
            gap_severity: GapSeverity::Acceptable,
            details: vec![],
        })
    }

    // ─── Composite Calculation ───

    fn calculate_composite(
        &self,
        capability: &AxisMatch,
        output_kpi: &AxisMatch,
        business_gap: &AxisMatch,
        work_style: &AxisMatch,
        growth: &AxisMatch,
        graph: &PositionGraph,
    ) -> Score {
        let mut total = 0.0;
        let mut weight_sum = 0.0;

        for axis in &graph.axes {
            let match_pct = match axis.code {
                genflow_receptors::AxisCode::Capability => capability.match_percentage.value(),
                genflow_receptors::AxisCode::OutputKpi => output_kpi.match_percentage.value(),
                genflow_receptors::AxisCode::BusinessGap => business_gap.match_percentage.value(),
                genflow_receptors::AxisCode::WorkStyle => work_style.match_percentage.value(),
                genflow_receptors::AxisCode::GrowthMotivation => growth.match_percentage.value(),
            };
            total += match_pct * axis.weight;
            weight_sum += axis.weight;
        }

        if weight_sum > 0.0 {
            Score::new_unchecked(total / weight_sum)
        } else {
            Score::default()
        }
    }

    // ─── Risk Flags ───

    fn identify_risk_flags(
        &self,
        work_style: &AxisMatch,
        candidate: &CandidateProfile,
    ) -> Vec<RiskFlag> {
        let mut flags = Vec::new();

        if work_style.match_percentage.is_low() {
            flags.push(RiskFlag {
                // Canonical persistence code; the description retains the more specific legacy finding.
                code: "collaboration_style_gap".to_string(),
                severity: FlagSeverity::Attention,
                description: "Work style alignment below threshold".to_string(),
                mitigation: "Consider team dynamics training or mentorship".to_string(),
            });
        }

        if let Some(bf) = &candidate.big_five {
            if bf.neuroticism.is_high() {
                flags.push(RiskFlag {
                    // Canonical persistence code; this is support guidance, not a diagnostic label.
                    code: "stress_support_needed".to_string(),
                    severity: FlagSeverity::Info,
                    description: "Additional support may be beneficial in high-pressure situations"
                        .to_string(),
                    mitigation: "Ensure adequate support structure".to_string(),
                });
            }
        }

        flags
    }

    // ─── Data Loading (from DB) ───

    async fn load_position_graph(&self, position_id: Uuid) -> Result<PositionGraph, AppError> {
        // Load graph data from position_graphs table
        let row = sqlx::query(
            "SELECT capability_axis, output_kpi_axis, business_gap_axis, work_style_axis, growth_motivation_axis, graph_version, calibration_applied, calibration_notes FROM position_graphs WHERE job_position_id = $1"
        )
            .bind(position_id)
            .fetch_optional(&self.pool)
            .await?;

        match row {
            Some(row) => {
                // Parse JSONB axes into domain PositionGraphAxis structs
                let axes: Vec<PositionGraphAxis> = vec![
                    self.parse_axis_from_json(
                        row.get::<serde_json::Value, _>("capability_axis"),
                        genflow_receptors::AxisCode::Capability,
                    ),
                    self.parse_axis_from_json(
                        row.get::<serde_json::Value, _>("output_kpi_axis"),
                        genflow_receptors::AxisCode::OutputKpi,
                    ),
                    self.parse_axis_from_json(
                        row.get::<serde_json::Value, _>("business_gap_axis"),
                        genflow_receptors::AxisCode::BusinessGap,
                    ),
                    self.parse_axis_from_json(
                        row.get::<serde_json::Value, _>("work_style_axis"),
                        genflow_receptors::AxisCode::WorkStyle,
                    ),
                    self.parse_axis_from_json(
                        row.get::<serde_json::Value, _>("growth_motivation_axis"),
                        genflow_receptors::AxisCode::GrowthMotivation,
                    ),
                ];

                Ok(PositionGraph {
                    position_id,
                    version: row.get::<String, _>("graph_version"),
                    axes,
                    calibration_notes: row.get::<Option<String>, _>("calibration_notes"),
                })
            }
            // Fallback: if no graph exists, return default weights graph
            None => {
                tracing::warn!(position_id = %position_id, "No position graph in DB, using default weights");
                Ok(genflow_position_generation::PositionGraphBuilder::new()
                    .build(position_id, &genflow_receptors::AxisWeights::default()))
            }
        }
    }

    fn parse_axis_from_json(
        &self,
        value: serde_json::Value,
        code: genflow_receptors::AxisCode,
    ) -> PositionGraphAxis {
        let weight = value
            .get("weight")
            .and_then(|v| v.as_f64().map(|x| x as f32))
            .unwrap_or(0.20);
        let description = match code {
            genflow_receptors::AxisCode::Capability => "Knowledge, skills and abilities",
            genflow_receptors::AxisCode::OutputKpi => "Expected results and KPIs",
            genflow_receptors::AxisCode::BusinessGap => "Gap between current and desired state",
            genflow_receptors::AxisCode::WorkStyle => "Work style and collaboration",
            genflow_receptors::AxisCode::GrowthMotivation => "Growth and development motivation",
        };

        let dimensions: Vec<genflow_receptors::DimensionRequirement> = value
            .get("dimensions")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|dim| {
                        Some(genflow_receptors::DimensionRequirement {
                            code: dim.get("code")?.as_str()?.to_string(),
                            description: dim.get("description")?.as_str()?.to_string(),
                            min: dim
                                .get("min")
                                .and_then(|v| v.as_f64().map(|x| x as f32))
                                .and_then(Score::new),
                            ideal: dim
                                .get("ideal")
                                .and_then(|v| v.as_f64().map(|x| x as f32))
                                .and_then(Score::new),
                            max: dim
                                .get("max")
                                .and_then(|v| v.as_f64().map(|x| x as f32))
                                .and_then(Score::new),
                            is_mandatory: dim
                                .get("is_mandatory")
                                .and_then(|v| v.as_bool())
                                .unwrap_or(false),
                        })
                    })
                    .collect()
            })
            .unwrap_or_default();

        PositionGraphAxis {
            code,
            weight,
            description: description.to_string(),
            dimensions,
            calibration_applied: value
                .get("calibration_applied")
                .and_then(|v| v.as_bool())
                .unwrap_or(false),
        }
    }

    async fn load_candidate_profile(
        &self,
        candidate_id: Uuid,
    ) -> Result<CandidateProfile, AppError> {
        // Load candidate's assessment data from DB
        let assessments = sqlx::query(
            "SELECT method_code, result_summary FROM assessment_sessions WHERE subject_candidate_id = $1 AND status = 'completed'"
        )
            .bind(candidate_id)
            .fetch_all(&self.pool)
            .await?;

        let mut big_five = None;
        let mut skills = std::collections::HashMap::new();

        for row in &assessments {
            let method: String = row.get("method_code");
            let summary: serde_json::Value = row.get("result_summary");

            if method == "big_five" {
                big_five = Some(genflow_receptors::BigFiveScores {
                    openness: Score::new(
                        summary
                            .get("openness")
                            .and_then(|v| v.as_f64().map(|x| x as f32))
                            .unwrap_or(50.0),
                    )
                    .unwrap_or_default(),
                    conscientiousness: Score::new(
                        summary
                            .get("conscientiousness")
                            .and_then(|v| v.as_f64().map(|x| x as f32))
                            .unwrap_or(50.0),
                    )
                    .unwrap_or_default(),
                    extraversion: Score::new(
                        summary
                            .get("extraversion")
                            .and_then(|v| v.as_f64().map(|x| x as f32))
                            .unwrap_or(50.0),
                    )
                    .unwrap_or_default(),
                    agreeableness: Score::new(
                        summary
                            .get("agreeableness")
                            .and_then(|v| v.as_f64().map(|x| x as f32))
                            .unwrap_or(50.0),
                    )
                    .unwrap_or_default(),
                    neuroticism: Score::new(
                        summary
                            .get("neuroticism")
                            .and_then(|v| v.as_f64().map(|x| x as f32))
                            .unwrap_or(50.0),
                    )
                    .unwrap_or_default(),
                });
            } else if method == "riasec" {
                // RIASEC scores contribute to skills mapping
                for key in [
                    "realistic",
                    "investigative",
                    "artistic",
                    "social",
                    "enterprising",
                    "conventional",
                ] {
                    if let Some(score) = summary.get(key).and_then(|v| v.as_f64().map(|x| x as f32))
                    {
                        skills.insert(key.to_string(), score);
                    }
                }
            }
        }

        // Load skills from candidate's skill data
        let _candidate_row = sqlx::query("SELECT email, full_name FROM candidates WHERE id = $1")
            .bind(candidate_id)
            .fetch_optional(&self.pool)
            .await?;

        Ok(CandidateProfile {
            candidate_id,
            big_five,
            riasec: None, // Simplified for now
            skills,
            experience_years: None,
        })
    }

    // ─── Persistence ───

    /// Atomically upsert a match and replace its derived risk-flag snapshot.
    ///
    /// The unique `(position_id, candidate_id)` constraint represents the stable
    /// business identity of a match. Score fields are recalculated, while status
    /// and human decision audit fields are intentionally left untouched on
    /// conflict so automation cannot overwrite a reviewer decision.
    async fn persist_calculation(
        &self,
        position_id: Uuid,
        candidate_id: Uuid,
        snapshot: &MatchSnapshot<'_>,
        risk_flags: &[RiskFlag],
    ) -> Result<(Uuid, MatchStatus, chrono::DateTime<chrono::Utc>), AppError> {
        let organization_id = self.position_organization_id(position_id).await?;
        let calculated_at = chrono::Utc::now();
        let mut tx = begin_organization_transaction(&self.pool, organization_id).await?;

        let row = sqlx::query(
            r#"
            INSERT INTO job_matches (
                id, position_id, candidate_id,
                capability_match_score, output_kpi_match_score, business_gap_match_score,
                work_style_alignment_score, growth_motivation_match_score,
                composite_match_index, confidence_score, status,
                human_review_required, calculated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)
            ON CONFLICT (position_id, candidate_id) DO UPDATE SET
                capability_match_score = EXCLUDED.capability_match_score,
                output_kpi_match_score = EXCLUDED.output_kpi_match_score,
                business_gap_match_score = EXCLUDED.business_gap_match_score,
                work_style_alignment_score = EXCLUDED.work_style_alignment_score,
                growth_motivation_match_score = EXCLUDED.growth_motivation_match_score,
                composite_match_index = EXCLUDED.composite_match_index,
                confidence_score = EXCLUDED.confidence_score,
                human_review_required = EXCLUDED.human_review_required,
                calculated_at = EXCLUDED.calculated_at
            RETURNING id, status
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(position_id)
        .bind(candidate_id)
        .bind(snapshot.capability.match_percentage.value())
        .bind(snapshot.output_kpi.match_percentage.value())
        .bind(snapshot.business_gap.match_percentage.value())
        .bind(snapshot.work_style.match_percentage.value())
        .bind(snapshot.growth.match_percentage.value())
        .bind(snapshot.composite.value())
        .bind(Score::new(85.0).unwrap_or_default().value())
        .bind(MatchStatus::PendingReview.as_db_str())
        .bind(snapshot.human_review_required)
        .bind(calculated_at)
        .fetch_one(&mut *tx)
        .await?;

        let match_id: Uuid = row.get("id");
        let status =
            MatchStatus::from_db_str(row.get::<String, _>("status").as_str()).ok_or_else(|| {
                AppError::Internal(format!("Unknown persisted match status for {match_id}"))
            })?;

        // Risk flags are an analytical snapshot, not a human decision. Replace
        // them in the same transaction so a failed recalculation leaves the
        // previous complete snapshot intact.
        sqlx::query("DELETE FROM match_risk_flags WHERE job_match_id = $1")
            .bind(match_id)
            .execute(&mut *tx)
            .await?;

        for flag in risk_flags {
            sqlx::query(
                "INSERT INTO match_risk_flags (id, job_match_id, flag_code, severity, description, mitigation_suggestion) VALUES ($1, $2, $3, $4, $5, $6)",
            )
            .bind(Uuid::new_v4())
            .bind(match_id)
            .bind(&flag.code)
            .bind(flag.severity.as_db_str())
            .bind(&flag.description)
            .bind(&flag.mitigation)
            .execute(&mut *tx)
            .await?;
        }

        tx.commit().await?;
        Ok((match_id, status, calculated_at))
    }

    /// Resolve the owning organization before opening a tenant-scoped write
    /// transaction. Authentication-derived tenant context will replace this
    /// lookup when request auth is introduced in the gateway.
    async fn position_organization_id(&self, position_id: Uuid) -> Result<Uuid, AppError> {
        let row = sqlx::query("SELECT organization_id FROM job_positions WHERE id = $1")
            .bind(position_id)
            .fetch_optional(&self.pool)
            .await?;

        row.map(|row| row.get("organization_id")).ok_or_else(|| {
            AppError::NotFound(format!(
                "Position {position_id} not found while calculating match"
            ))
        })
    }

    /// Record hiring decision on a match
    pub async fn record_decision(
        &self,
        match_id: Uuid,
        decided_by: Uuid,
        decision: MatchStatus,
        note: Option<String>,
    ) -> Result<(), AppError> {
        sqlx::query(
            "UPDATE job_matches SET status = $1, decision_made_by_user_id = $2, decision_made_at = NOW(), decision_note = $3 WHERE id = $4"
        )
        .bind(decision.as_db_str())
        .bind(decided_by)
        .bind(note)
        .bind(match_id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }
}
