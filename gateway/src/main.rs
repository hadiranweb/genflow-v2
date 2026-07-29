//! GenFlow Gateway — API Gateway binary (Axum)
//!
//! Hybrid Island Architecture: Gateway routes to all Island services.
//! Single binary, single deploy, workspace-aware.

use std::sync::Arc;
use tokio::net::TcpListener;

mod api;
mod auth_context;
mod error_response;
mod state;

use genflow_shared_infra::auth::JwtAuth;
use genflow_shared_infra::config::AppConfig;
use genflow_shared_infra::db::DatabasePool;
use genflow_shared_infra::health::HealthChecker;
use genflow_shared_infra::redis::RedisPool;
use genflow_shared_infra::telemetry;
use state::AppState;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 1. Load configuration
    let config = AppConfig::from_env();

    // 2. Initialize tracing
    telemetry::init_tracing(&config.logging);

    tracing::info!("GenFlow v2 Gateway starting...");
    tracing::info!("Server: {}:{}", config.server.host, config.server.port);

    // 3. Connect to infrastructure
    let db_pool = DatabasePool::connect(&config.database).await?;
    let redis_pool = Arc::new(RedisPool::connect(&config.redis).await?);

    // 4. Run migrations
    db_pool.run_migrations().await?;

    // 5. Initialize island services
    let pg_pool = db_pool.pool().clone();

    let mcp_repo = Arc::new(genflow_mcp_registry::PgMcpRepository::new(pg_pool.clone()));
    let mcp_cache = Arc::new(genflow_mcp_registry::RedisMcpCache::new(redis_pool.clone()));
    let mcp_builder = Arc::new(genflow_mcp_registry::McpBuilderImpl::new());
    let mcp_resolver = Arc::new(genflow_mcp_registry::McpResolver::new(
        mcp_repo.clone(),
        mcp_cache.clone(),
        mcp_builder.clone(),
    ));

    let position_engine =
        genflow_position_generation::PositionGenerationEngine::new(pg_pool.clone());
    let matching_engine = genflow_candidate_matching::MatchingEngine::new(pg_pool.clone());
    let invitation_manager = genflow_candidate_matching::InvitationManager::new(pg_pool.clone());
    let report_generator = genflow_candidate_matching::ReportGenerator::new(pg_pool.clone());
    let dashboard_engine = genflow_dashboard_analytics::DashboardEngine::new(pg_pool.clone());
    let notification_service =
        genflow_dashboard_analytics::NotificationService::new(pg_pool.clone());

    // 6. Initialize Synaptic Hub
    let synaptic_bus = Arc::new(genflow_synaptic_hub::SynapticBus::new(redis_pool.clone()));

    // 7. Build application state
    let host = config.server.host.clone();
    let port = config.server.port;
    let jwt_auth = JwtAuth::new(config.jwt.clone());
    let health_checker = HealthChecker::new(pg_pool.clone(), redis_pool.clone());

    let state = Arc::new(AppState {
        config,
        db_pool: pg_pool,
        mcp_resolver,
        position_engine,
        matching_engine,
        invitation_manager,
        report_generator,
        dashboard_engine,
        notification_service,
        jwt_auth,
        synaptic_bus,
        health_checker,
        redis_pool,
    });

    // 8. Build Axum router
    let app = api::build_router(state);

    // 9. Start server
    let listener = TcpListener::bind(format!("{host}:{port}")).await?;

    tracing::info!("GenFlow v2 Gateway ready — listening on {}:{}", host, port);

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    tracing::info!("GenFlow v2 Gateway shutting down");
    Ok(())
}

async fn shutdown_signal() {
    tokio::signal::ctrl_c()
        .await
        .expect("Failed to install CTRL+C handler");
}
