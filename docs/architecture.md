# GenFlow v2 Architecture Documentation

## Overview

GenFlow v2 is a **Business-First Position Generation & Candidate Matching Platform** built with a **Hybrid Island Architecture** inspired by the pema-platform-v2 ecosystem.

## Core Principles

1. **Rust Performance** — Zero-cost abstractions, async runtime, no GC pauses
2. **Ultra-Fine MCP** — Each MCP Type operates as an independent Cell
3. **Island Architecture** — Business domains as separate crates in a Cargo workspace
4. **Synaptic Hub** — Dual-layer event bus (tokio mpsc + Redis pub/sub)
5. **Single Deploy** — All islands compose into one gateway binary

## Architecture

```
┌──────────────────────────────────────────────────┐
│                    Gateway (Axum)                  │
│  ┌─────────┬──────────┬──────────┬──────────┐    │
│  │ MCP     │ Position │ Candidate│ Dashboard│    │
│  │ Handlers│ Handlers │ Handlers │ Handlers │    │
│  └─────────┴──────────┴──────────┴──────────┘    │
│                      │                             │
│              ┌───────┴───────┐                     │
│              │  AppState     │                     │
│              └───────┬───────┘                     │
└──────────────┼───────┼───────┼──────────────────┘
               │       │       │
┌──────────────┼───────┼───────┼──────────────────┐
│  Islands (lib crates)                          │
│  ┌──────────┐ ┌──────────────┐ ┌──────────────┐│
│  │ MCP Reg  │ │ Position Gen │ │ Candidate Mat││
│  │ Registry │ │ Discovery    │ │ 5-Axis Engine││
│  │ Cache    │ │ Graph Builder│ │ Invitations  ││
│  │ Resolver │ │ Calibrator   │ │ Reports      ││
│  └──────────┘ └──────────────┘ └──────────────┘│
│              │ ┌──────────────┐                  │
│              │ │ Dashboard    │                  │
│              │ │ Metrics      │                  │
│              │ │ Notifications│                  │
│              │ └──────────────┘                  │
└──────────────┼──────────────────────────────────┘
               │
┌──────────────┼──────────────────────────────────┐
│  Synaptic Hub (dual-layer event bus)            │
│  ┌─────────────────┐ ┌─────────────────┐       │
│  │ Layer 1: tokio  │ │ Layer 2: Redis  │       │
│  │ mpsc broadcast  │ │ pub/sub         │       │
│  │ (in-process)    │ │ (cross-container│       │
│  │ (zero-cost)     │ │ (production)    │       │
│  └─────────────────┘ └─────────────────┘       │
│  ┌─────────────────┐ ┌─────────────────┐       │
│  │ Event Router    │ │ Convergence     │       │
│  │ Pattern Match   │ │ Tracker         │       │
│  └─────────────────┘ └─────────────────┘       │
└─────────────────────────────────────────────────┘
               │
┌──────────────┼──────────────────────────────────┐
│  Receptors (shared domain types)                │
│  ┌────────┐ ┌──────┐ ┌──────────┐ ┌──────┐    │
│  │ Score  │ │ MCP  │ │ Position │ │ Match │    │
│  │ Types  │ │ Enums│ │ Types    │ │ Types │    │
│  └────────┘ └──────┘ └──────────┘ └──────┘    │
│  ┌────────┐ ┌──────────┐ ┌──────┐              │
│  │ Events │ │ Assess   │ │ Dash │              │
│  │ Defs   │ │ Types    │ │ Types│              │
│  └────────┘ └──────────┘ └──────┘              │
└─────────────────────────────────────────────────┘
               │
┌──────────────┼──────────────────────────────────┐
│  Shared Infra                                  │
│  ┌──────┐ ┌──────┐ ┌──────┐ ┌──────┐ ┌─────┐ │
│  │ DB   │ │Redis │ │ Auth │ │Error │ │Telem│ │
│  │ Pool │ │ Pool │ │ JWT  │ │Types │ │etry │ │
│  └──────┘ └──────┘ ┌──────┘ └──────┘ ┌─────┘ │
│                   │Config│           │Health│ │
│                   └──────┘           └──────┘ │
└─────────────────────────────────────────────────┘
```

## Crate Dependency Graph

```
genflow-gateway
├── genflow-receptors (shared domain types + events)
├── genflow-shared-infra (DB, Redis, Auth, Error, Telemetry)
├── genflow-synaptic-hub (event bus)
├── genflow-mcp-registry
│   ├── genflow-receptors
│   ├── genflow-shared-infra
│   ├── genflow-synaptic-hub
├── genflow-position-generation
│   ├── genflow-receptors
│   ├── genflow-shared-infra
│   ├── genflow-synaptic-hub
│   ├── genflow-mcp-registry
├── genflow-candidate-matching
│   ├── genflow-receptors
│   ├── genflow-shared-infra
│   ├── genflow-synaptic-hub
│   ├── genflow-mcp-registry
│   ├── genflow-position-generation
├── genflow-dashboard-analytics
│   ├── genflow-receptors
│   ├── genflow-shared-infra
│   ├── genflow-synaptic-hub
```

## Event Flow (Synaptic Hub)

Events flow between islands through the dual-layer bus:

```
mcp.resolved → Position Generation, Dashboard
position.generated → Candidate Matching, Dashboard
position.analysis_completed → Candidate Matching, Dashboard
candidate.invited → Dashboard
match.calculated → Dashboard
report.generated → Dashboard
dashboard.metrics_updated → Gateway
dashboard.alert_triggered → Gateway
```

## MCP Ultra-Fine Architecture

Each MCP Type is an independent Cell:

| Cell | Scope | TTL | Reusable |
|------|-------|-----|----------|
| PlatformPolicy | Global | 7 days | Yes |
| Industry | Global/Industry | 24h | Yes |
| BusinessProcess | Global/Industry | 24h | Yes |
| StandardPosition | Global/Industry | 24h | Yes |
| OrganizationContext | Tenant | 1h | Yes |
| CaseTemporary | Case | 30m | No |

Resolution flow: Cache → DB → Build (fallback)

## File Structure

```
genflow-v2/
├── Cargo.toml                    # [workspace]
├── .github/workflows/ci-cd.yml
├── Dockerfile                     # Multi-stage, workspace-aware
├── docker-compose.yml
├── migrations/                    # 7 SQL files
├── docs/                          # Architecture docs
│
├── receptors/genflow-receptors/   # Shared domain types
├── shared-infra/                  # DB, Redis, Auth
├── synaptic-hub/                  # Dual-layer event bus
├── islands/
│   ├── mcp-registry/              # MCP Cell runtime
│   ├── position-generation/       # Position services
│   ├── candidate-matching/        # 5-Axis engine
│   ├── dashboard-analytics/       # Dashboard + notifications
├── gateway/                       # Axum API binary
```

## Differences from v1 (Sprint 1-5)

| v1 (Sprint) | v2 (Island) | Change |
|---|---|---|
| Monolithic `apps/api/src/` | Workspace with 8 crates | ✅ Separation of concerns |
| No main.rs | `gateway/src/main.rs` | ✅ Entry point |
| Direct DB queries everywhere | Services with proper error handling | ✅ Clean architecture |
| sqlx::query (runtime only) | sqlx::query (runtime) + proper type mapping | ✅ Consistent |
| No event bus | Synaptic Hub (dual-layer) | ✅ Event-driven |
| Placeholder auth (Uuid::new_v4()) | Real JWT implementation | ✅ Security |
| CI/CD cache issues | Workspace-aware Dockerfile + proper cache layers | ✅ Fixed |
| Domain types mixed with runtime | Receptors (pure types) + Runtime (async traits) | ✅ Clean separation |
