# GenFlow — CHANGELOG

All notable changes to the GenFlow platform are documented here.

## [2.0.0] — 2024-07-25 — Island Architecture Rewrite

### 🏗️ Architecture (Complete Rewrite)

- **Monolith → Island Workspace**: 8 Cargo workspace crates replacing single monolithic `apps/api/src/`
- **Synaptic Hub**: Dual-layer event bus (tokio mpsc broadcast + Redis pub/sub) for cross-island event flow
- **Convergence Tracker**: Multi-source event aggregation (e.g., mcp.resolved + position.generated → pipeline setup)
- **Event Router**: Pattern-matching routing of events to target islands
- **Pure Receptors**: Domain types separated into `genflow-receptors` (zero runtime dependencies)
- **Gateway Bridge**: `ApiError` newtype for orphan-rule-safe `IntoResponse` integration (AppError stays pure, no axum dependency in shared-infra)

### 🔐 Security

- **Real JWT authentication**: `JwtAuth` with proper `Validation` (issuer verification, expiration)
- **Representative Influence Policy**: Only affects Work Style axis — never hard requirements
- **Non-root Docker user**: `appuser` (uid 1000) in runtime stage

### 🧠 MCP (Master Context Protocol)

- **Ultra-fine MCP Cells**: 6 independent MCP Types (PlatformPolicy, Industry, BusinessProcess, StandardPosition, OrganizationContext, CaseTemporary)
- **Resolution fallback**: Cache → DB → Build draft
- **RedisMcpCache**: `Arc<RedisPool>` pattern, `query_async::<_, T>` with explicit type annotations
- **PgMcpRepository**: Full PostgreSQL CRUD with `sqlx::Row` trait import
- **McpBuilderImpl**: Draft generation for Industry, Process, and Case MCPs

### 🎯 Position Generation

- **Business Input Modes**: SWOT, Gap Analysis, Direct Request (all `Serialize/Deserialize`)
- **Position Graph Builder**: 5-axis graph construction (Capability, Output KPI, Business Gap, Work Style, Growth)
- **Representative Calibrator**: Owner/SeniorManager/Manager/Advisor/External with personality toggle
- **Business Need Discovery**: Automatic need extraction from business analysis input
- **Position Generation Engine**: Full pipeline orchestration with evidence tracking

### 🔄 Candidate Matching

- **5-Axis Matching Engine**: Capability, Output KPI, Business Gap, Work Style, Growth Motivation
- **Risk Flags**: work_style_low, stress_sensitivity (with FlagSeverity levels)
- **Human Review Threshold**: composite < 60 or ActionRequired flags trigger review
- **Invitation Manager**: Code generation, accept/expire lifecycle
- **Report Generator**: ForEmployer + ForCandidate views with disclaimers

### 📊 Dashboard

- **Dashboard Engine**: Organization overview with metrics, activity, alerts
- **Notification Service**: Multi-channel notification persistence
- **Activity Actions**: PositionCreated, CandidateInvited, AssessmentCompleted, etc.
- **Alert Types**: HighMatchFound, PositionExpiring, CandidateCompleted, SystemNotification

### 🛠️ Infrastructure

- **RedisPool**: Async `connection()` method, `Arc<RedisPool>` pattern, no stored connections
- **DatabasePool**: PgPool with configurable min/max connections and timeouts
- **AppConfig**: Full env-based configuration (server, database, redis, jwt, logging)
- **HealthChecker**: DB + Redis health checks with component-level reporting
- **Telemetry**: tracing-subscriber with env-filter, target/thread/file/line info

### 🐳 DevOps

- **Workspace-aware Dockerfile**: Proper cache layers (dummy build → dependency cache → real build)
- **docker-compose.yml**: api + db + redis + migrate services with health checks
- **CI/CD**: cargo check → cargo test (unit) → build & push (main only)

### 📝 Domain Types (Serialize/Deserialize additions)

All API response types now have `Serialize/Deserialize`:
- `JobMatch`, `AxisMatch`, `GapSeverity`, `MatchStatus`, `DimensionMatchDetail`
- `MatchReport`, `ReportType`, `RiskFlag`, `FlagSeverity`
- `DashboardOverview`, `KeyMetrics`, `PositionAlert`, `AlertUrgency`
- `ActivityItem`, `ActivityAction`, `DashboardAlert`, `AlertType`
- `PositionInvite`, `InviteStatus` (with new `as_db_str()` method)
- `BusinessInputMode`, `CapabilityLevel`, `RepresentativeContextInput`
- `McpBundle`, `GeneratedPositionProfile`, `JobPosition`
- `PositionRequirement`, `RequirementType`, `RequirementImportance`, `RequirementSource`
- `GenerationWarning`, `WarningSeverity`, `BusinessAnalysisResult`

### 🔧 Compile Error Fixes

- `query_async` never-type-fallback → explicit `::<_, ()>` or `::<_, Option<String>>` type annotations
- `sqlx::Row` trait import added to repository and dashboard_engine
- `AppError` removed axum dependency (pure domain error)
- `Validation::default()` duplicate removed in auth.rs
- `InviteStatus::as_db_str()` method added
- `McpResolver::find_by_id()` public method added (replaces private `repo` field access)
- Axum v0.7 path syntax: `{id}` instead of `:id`
- `Arc<AppState>` pattern for axum's Clone requirement
- `redis::cmd("PUBLISH").query_async::<_, ()>` for void-returning commands

### 📦 Preserved from v1

- All 7 SQL migration files (1,301 lines) — unchanged
- Organization → Position → Candidate flow concepts
- MCP as core context protocol
- Representative calibration policy (Work Style only)

---

## [1.x] — Sprint 1-5 — Monolithic Architecture

### Sprint 1: Foundation
- Initial monolithic Rust API with axum
- PostgreSQL schema design
- Basic CRUD endpoints

### Sprint 2: MCP Context
- MCP domain types
- Redis caching layer
- PostgreSQL MCP storage

### Sprint 3: Position Generation
- Business input modes (SWOT, Gap, Direct)
- Position graph concept
- Representative calibration framework

### Sprint 4: Candidate Pipeline
- Invitation system
- Assessment placeholder
- Basic matching concept

### Sprint 5: Dashboard & Auth
- Dashboard endpoints
- Placeholder auth (Uuid::new_v4())
- CI/CD pipeline setup
