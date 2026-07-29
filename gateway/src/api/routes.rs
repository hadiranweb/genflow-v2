//! Router Composition — Axum router with all island handlers

use axum::routing::{get, post};
use axum::Router;
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;

use crate::api::handlers;
use crate::state::AppState;

/// Build the full Axum router
pub fn build_router(state: Arc<AppState>) -> Router {
    Router::new()
        // Health check
        .route("/health", get(handlers::health::health_check))
        // MCP Registry routes
        .route("/api/v2/mcp/{id}", get(handlers::mcp::get_mcp))
        .route("/api/v2/mcp/resolve", post(handlers::mcp::resolve_mcp))
        // Position Generation routes
        .route(
            "/api/v2/positions/generate",
            post(handlers::position::generate_position),
        )
        .route(
            "/api/v2/positions/{id}",
            get(handlers::position::get_position),
        )
        // Candidate Matching routes
        .route(
            "/api/v2/matches/{position_id}/{candidate_id}",
            get(handlers::candidate::calculate_match),
        )
        .route(
            "/api/v2/invitations",
            post(handlers::candidate::create_invitation),
        )
        .route(
            "/api/v2/invitations/{code}/accept",
            post(handlers::candidate::accept_invitation),
        )
        .route(
            "/api/v2/reports/{match_id}",
            get(handlers::candidate::generate_report),
        )
        .route(
            "/api/v2/matches/{match_id}/decision",
            post(handlers::candidate::record_decision),
        )
        // Dashboard routes
        .route(
            "/api/v2/dashboard/{org_id}",
            get(handlers::dashboard::get_dashboard),
        )
        // Middleware
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods(Any)
                .allow_headers(Any),
        )
        .layer(TraceLayer::new_for_http())
        // State
        .with_state(state)
}
