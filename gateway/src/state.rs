//! Application State — Shared state for all handlers

use genflow_mcp_registry::McpResolver;
use genflow_shared_infra::{AppConfig, HealthChecker, JwtAuth, RedisPool};
use genflow_synaptic_hub::SynapticBus;
use sqlx::PgPool;
use std::sync::Arc;

/// AppState — holds references to all island services and infrastructure
///
/// Wrapped in Arc for axum's Clone requirement on state.
#[allow(dead_code)]
pub struct AppState {
    pub config: AppConfig,
    pub db_pool: PgPool,

    // MCP Registry Island
    pub mcp_resolver: Arc<
        McpResolver<
            genflow_mcp_registry::PgMcpRepository,
            genflow_mcp_registry::RedisMcpCache,
            genflow_mcp_registry::McpBuilderImpl,
        >,
    >,

    // Position Generation Island
    pub position_engine: genflow_position_generation::PositionGenerationEngine,

    // Candidate Matching Island
    pub matching_engine: genflow_candidate_matching::MatchingEngine,
    pub invitation_manager: genflow_candidate_matching::InvitationManager,
    pub report_generator: genflow_candidate_matching::ReportGenerator,

    // Dashboard Analytics Island
    pub dashboard_engine: genflow_dashboard_analytics::DashboardEngine,
    pub notification_service: genflow_dashboard_analytics::NotificationService,

    // Infrastructure
    pub jwt_auth: JwtAuth,
    pub synaptic_bus: Arc<SynapticBus>,
    pub health_checker: HealthChecker,
    pub redis_pool: Arc<RedisPool>,
}
