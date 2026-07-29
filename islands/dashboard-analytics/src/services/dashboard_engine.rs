//! Dashboard Engine — Aggregates metrics and produces dashboard views

use genflow_receptors::{
    ActivityAction, ActivityItem, DashboardAlert, DashboardOverview, KeyMetrics,
};
use genflow_shared_infra::error::AppError;
use sqlx::{PgPool, Row};
use uuid::Uuid;

pub struct DashboardEngine {
    pool: PgPool,
}

impl DashboardEngine {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Get dashboard overview for an organization
    pub async fn get_overview(&self, org_id: Uuid) -> Result<DashboardOverview, AppError> {
        let metrics = self.fetch_metrics(org_id).await?;
        let recent_activity = self.fetch_recent_activity(org_id).await?;
        let alerts = self.fetch_alerts(org_id).await?;

        Ok(DashboardOverview {
            organization_id: org_id,
            metrics,
            recent_activity,
            alerts,
        })
    }

    async fn fetch_metrics(&self, org_id: Uuid) -> Result<KeyMetrics, AppError> {
        let row = sqlx::query(
            r#"
            SELECT 
                COUNT(*) as total_positions,
                COUNT(*) FILTER (WHERE status = 'active') as active_positions,
                (SELECT COUNT(*) FROM position_invites WHERE position_id IN (SELECT id FROM job_positions WHERE organization_id = $1)) as total_candidates_invited
            FROM job_positions 
            WHERE organization_id = $1
            "#
        )
        .bind(org_id)
        .fetch_one(&self.pool)
        .await?;

        let total_positions: i64 = row.get("total_positions");
        let active_positions: i64 = row.get("active_positions");
        let total_invited: i64 = row.get("total_candidates_invited");

        Ok(KeyMetrics {
            total_positions: total_positions as u32,
            active_positions: active_positions as u32,
            filled_positions: 0,
            total_candidates_invited: total_invited as u32,
            total_candidates_completed: 0,
            candidates_in_pipeline: 0,
            average_match_score: None,
            average_time_to_hire_days: None,
            positions_expiring_soon: vec![],
        })
    }

    async fn fetch_recent_activity(&self, org_id: Uuid) -> Result<Vec<ActivityItem>, AppError> {
        let rows = sqlx::query(
            "SELECT * FROM activity_logs WHERE organization_id = $1 ORDER BY created_at DESC LIMIT 20"
        )
            .bind(org_id)
            .fetch_all(&self.pool)
            .await?;

        Ok(rows
            .iter()
            .map(|row| ActivityItem {
                id: row.get("id"),
                actor_name: format!("Representative {}", row.get::<Uuid, _>("actor_id")),
                action: ActivityAction::from_db_str(&row.get::<String, _>("action")),
                entity_type: row.get("entity_type"),
                entity_title: format!(
                    "{} {}",
                    row.get::<String, _>("entity_type"),
                    row.get::<Uuid, _>("entity_id")
                ),
                timestamp: row.get("created_at"),
                metadata: row.get("metadata"),
            })
            .collect())
    }

    async fn fetch_alerts(&self, _org_id: Uuid) -> Result<Vec<DashboardAlert>, AppError> {
        // Simplified for now — would be dynamic based on real data
        Ok(vec![])
    }
}
