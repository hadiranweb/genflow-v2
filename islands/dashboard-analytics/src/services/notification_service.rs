//! Notification Service — Sends notifications via multiple channels

use genflow_shared_infra::error::AppError;
use sqlx::PgPool;
use uuid::Uuid;

pub struct NotificationService {
    pool: PgPool,
}

impl NotificationService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Send a notification to a user
    pub async fn send_notification(
        &self,
        recipient_id: Uuid,
        notification_type: &str,
        title: &str,
        message: &str,
        entity_type: Option<&str>,
        entity_id: Option<Uuid>,
    ) -> Result<Uuid, AppError> {
        let notification_id = Uuid::new_v4();

        sqlx::query(
            "INSERT INTO notifications (id, recipient_id, type, title, message, entity_type, entity_id, is_read) VALUES ($1, $2, $3, $4, $5, $6, $7, false)"
        )
            .bind(notification_id)
            .bind(recipient_id)
            .bind(notification_type)
            .bind(title)
            .bind(message)
            .bind(entity_type)
            .bind(entity_id)
            .execute(&self.pool)
            .await?;

        tracing::info!(
            notification_id = %notification_id,
            recipient_id = %recipient_id,
            type = %notification_type,
            "Notification sent"
        );

        Ok(notification_id)
    }
}
