//! Database Pool — PgPool setup and migration runner

use crate::config::DatabaseConfig;
use crate::error::AppError;
use sqlx::migrate::Migrator;
use sqlx::postgres::{PgPool, PgPoolOptions};
use sqlx::{Postgres, Transaction};
use uuid::Uuid;

/// Embed migrations at compile time (relative to this crate's Cargo.toml)
/// shared-infra/Cargo.toml → ../migrations
static MIGRATOR: Migrator = sqlx::migrate!("../migrations");

pub struct DatabasePool {
    pool: PgPool,
}

impl DatabasePool {
    /// Create a new PgPool from config
    pub async fn connect(config: &DatabaseConfig) -> Result<Self, AppError> {
        let pool = PgPoolOptions::new()
            .max_connections(config.max_connections)
            .min_connections(config.min_connections)
            .acquire_timeout(std::time::Duration::from_secs(
                config.connect_timeout_seconds,
            ))
            .idle_timeout(std::time::Duration::from_secs(config.idle_timeout_seconds))
            .connect(&config.url)
            .await
            .map_err(|e| AppError::Infrastructure(format!("Database connection failed: {e}")))?;

        tracing::info!(
            "Database pool connected (max={}, min={})",
            config.max_connections,
            config.min_connections
        );

        Ok(Self { pool })
    }

    /// Get a reference to the pool
    pub fn pool(&self) -> &PgPool {
        &self.pool
    }

    /// Run migrations using embedded SQL files
    pub async fn run_migrations(&self) -> Result<(), AppError> {
        MIGRATOR
            .run(&self.pool)
            .await
            .map_err(|e| AppError::Infrastructure(format!("Migration failed: {e}")))?;

        tracing::info!("Database migrations completed successfully");
        Ok(())
    }
}

/// Set the PostgreSQL organization context for the lifetime of a transaction.
///
/// RLS policies read `app.current_org_id`; `set_config(..., true)` gives this
/// value `SET LOCAL` semantics, so a pooled connection cannot leak one tenant's
/// context into another request. Tenant-scoped writes must call this before their
/// first SQL statement.
pub async fn set_transaction_org_context(
    tx: &mut Transaction<'_, Postgres>,
    organization_id: Uuid,
) -> Result<(), AppError> {
    sqlx::query("SELECT set_config('app.current_org_id', $1, true)")
        .bind(organization_id.to_string())
        .execute(&mut **tx)
        .await?;

    Ok(())
}

/// Begin a transaction already bound to one organization.
///
/// Islands use this instead of manually opening a transaction and remembering
/// to set RLS state. It is the shared infrastructure boundary between pooled
/// database connections and tenant-owned bounded contexts.
pub async fn begin_organization_transaction(
    pool: &PgPool,
    organization_id: Uuid,
) -> Result<Transaction<'_, Postgres>, AppError> {
    let mut tx = pool.begin().await?;
    set_transaction_org_context(&mut tx, organization_id).await?;
    Ok(tx)
}
