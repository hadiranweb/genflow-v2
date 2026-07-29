//! Report Generator — Generates match reports (employer + candidate)

use genflow_receptors::{JobMatch, MatchReport, ReportType};
use genflow_shared_infra::error::AppError;
use sqlx::PgPool;
use uuid::Uuid;

pub struct ReportGenerator {
    pool: PgPool,
}

impl ReportGenerator {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Generate a match report
    pub async fn generate(
        &self,
        job_match: &JobMatch,
        report_type: ReportType,
    ) -> Result<MatchReport, AppError> {
        let report_id = Uuid::new_v4();

        let (title, summary, strengths, development_areas) = match report_type {
            ReportType::ForEmployer => (
                format!(
                    "Match Report — Position vs Candidate ({})",
                    job_match.position_id
                ),
                format!(
                    "Composite score: {:.1}/100",
                    job_match.composite_index.value()
                ),
                vec![
                    "Capability alignment is strong".to_string(),
                    "Work style shows good fit".to_string(),
                ],
                vec![
                    "Some KPI targets may need calibration".to_string(),
                    "Consider structured onboarding".to_string(),
                ],
            ),
            ReportType::ForCandidate => (
                "Your Match Profile".to_string(),
                "This report highlights how your profile aligns with the position requirements."
                    .to_string(),
                vec!["Your skills match key requirements".to_string()],
                vec!["Opportunities for skill development".to_string()],
            ),
        };

        // Disclaimer (always included)
        let disclaimers = vec![
            "This assessment is for guidance only and should not be the sole basis for hiring decisions.".to_string(),
            "All scores are relative and contextual.".to_string(),
            "No assessment can capture the full complexity of human potential.".to_string(),
        ];

        let report = MatchReport {
            id: report_id,
            job_match_id: job_match.id,
            report_type,
            title,
            summary,
            key_findings: vec![format!(
                "Composite: {:.1}",
                job_match.composite_index.value()
            )],
            strengths,
            development_areas,
            recommendations: vec![
                "Schedule a structured interview for deeper evaluation".to_string()
            ],
            disclaimers,
        };

        // Save to DB
        sqlx::query(
            "INSERT INTO match_reports (id, job_match_id, report_type, title, summary) VALUES ($1, $2, $3, $4, $5)"
        )
            .bind(report_id)
            .bind(job_match.id)
            .bind(report_type.as_db_str())
            .bind(&report.title)
            .bind(&report.summary)
            .execute(&self.pool)
            .await?;

        tracing::info!(report_id = %report_id, "Match report generated");
        Ok(report)
    }
}
