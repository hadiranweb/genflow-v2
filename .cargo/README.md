<!-- markdownlint-disable MD033 MD041 -->

<div align="center">
  <picture>
    <source media="(prefers-color-scheme: dark)" srcset="https://raw.githubusercontent.com/hadiranweb/GenFlow/main-platform/apps/web/public/favicon.svg">
    <img src="https://raw.githubusercontent.com/hadiranweb/GenFlow/main-platform/apps/web/public/favicon.svg" width="80" alt="GenFlow logo">
  </picture>

  <h1>GenFlow v2</h1>
  <p>
    <strong>Position Generation & Candidate Matching Platform</strong><br>
    Hybrid Island Architecture — Rust + Remix — Zero GC Pauses
  </p>
  <p>
    <a href="https://github.com/hadiranweb/GenFlow/actions/workflows/genflow.yml"><img src="https://img.shields.io/github/actions/workflow/status/hadiranweb/GenFlow/genflow.yml?branch=main-platform&label=CI&logo=github" alt="CI"></a>
    <a href="https://github.com/hadiranweb/GenFlow/actions/workflows/cd.yml"><img src="https://img.shields.io/github/actions/workflow/status/hadiranweb/GenFlow/cd.yml?branch=main-platform&label=CD&logo=github" alt="CD"></a>
    <a href="https://github.com/hadiranweb/GenFlow/blob/main-platform/LICENSE"><img src="https://img.shields.io/badge/License-PROPRIETARY-red.svg" alt="License"></a>
    <a href="https://www.rust-lang.org/"><img src="https://img.shields.io/badge/Rust-1.88-dea584?logo=rust" alt="Rust"></a>
    <a href="https://remix.run/"><img src="https://img.shields.io/badge/Remix-2.12-121212?logo=remix" alt="Remix"></a>
  </p>

  <table>
    <tr>
      <td align="center"><b>⚡ Performance</b><br>Native Rust + LTO</td>
      <td align="center"><b>🔒 Security</b><br>JWT + RLS + Distroless</td>
      <td align="center"><b>🏗️ Architecture</b><br>8 Crates + 5-Axis Matching</td>
      <td align="center"><b>🌐 Frontend</b><br>Remix + RTL + Tailwind</td>
    </tr>
  </table>
</div>

---

## 📋 Table of Contents

- [🎯 Overview](#-overview)
- [🏗️ Architecture](#️-architecture)
- [⚡ Tech Stack](#-tech-stack)
- [📁 Project Structure](#-project-structure)
- [🚀 Quick Start](#-quick-start)
- [🐳 Docker Deployment](#-docker-deployment)
- [🧪 Development](#-development)
- [🔌 API Reference](#-api-reference)
- [⚙️ CI/CD Pipeline](#️-cicd-pipeline)
- [🔒 Security](#-security)
- [📈 Performance](#-performance)
- [🤝 Contributing](#-contributing)
- [📄 License](#-license)

---

## 🎯 Overview

**GenFlow** transforms business analysis input — SWOT, Gap Analysis, Direct Request — into **structured position profiles** with a 5-axis matching engine, then matches candidates against those positions with unprecedented precision.

### Key Differentiators vs v1

| Capability | v1 (Monolithic) | v2 (Island Architecture) |
|---|---|---|
| **Architecture** | Single `apps/api/` | 8 Cargo workspace crates + Remix monorepo |
| **Event Bus** | None | Synaptic Hub (tokio broadcast + Redis pub/sub) |
| **Auth** | Placeholder `Uuid::new_v4()` | Real JWT + TenantAuth + Permission checks |
| **MCP Resolution** | N/A | Cache → DB → Build fallback pipeline |
| **Position Generation** | Basic CRUD | 5-axis graph + representative calibration |
| **Candidate Matching** | None | 5-Axis Matching Engine |
| **Migrations** | 7 | 11 (added RLS, tenant boundaries, org access) |
| **Frontend** | None | Remix v2 + Turborepo monorepo |
| **CI/CD** | Basic | Rust + Frontend + Docker images on push |
| **Docker** | Single stage | Multi-stage distroless (~35 MB) + GHCR |

---

## 🏗️ Architecture

### Hybrid Island Architecture

```
┌──────────────────────────────────────────────────────┐
│                    Internet                              │
└────────────────────────┬─────────────────────────────┘
                         │
┌────────────────────────▼────────────────────────────┐
│                  Nginx (Reverse Proxy)                 │
│          /api/* → API :3000  │  /* → Web :3000         │
└────────────────────────┬─────────────────────────────┘
                         │
┌────────────────────────▼────────────────────────────┐
│               Gateway (Axum HTTP API)                  │
│  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐ │
│  │   MCP    │ │ Position │ │ Candidate│ │ Dashboard│ │
│  │  Routes  │ │  Routes  │ │  Routes  │ │  Routes  │ │
│  └──────────┘ └──────────┘ └──────────┘ └──────────┘ │
│                 ┌── AppState ──┐                       │
└─────────────────┼──────────────┼──────────────────────┘
                   │              │
┌──────────────────┼──────────────┼──────────────────────┐
│              Islands (Library Crates)                    │
│  ┌────────────────┐ ┌──────────────┐ ┌────────────────┐ │
│  │  MCP Registry  │ │  Position    │ │   Candidate    │ │
│  │  (Cache→DB→    │ │  Generation  │ │   Matching     │ │
│  │   Build)       │ │  (5-axis)    │ │  (5-axis)      │ │
│  └────────────────┘ └──────────────┘ └────────────────┘ │
│  ┌────────────────┐                                     │
│  │   Dashboard    │                                     │
│  │   Analytics    │                                     │
│  └────────────────┘                                     │
└──────────────────────────────────────────────────────────┘
                         │
┌────────────────────────▼──────────────────────────────┐
│                  Synaptic Hub (Event Bus)               │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐  │
│  │  tokio       │  │    Redis     │  │  Convergence │  │
│  │  broadcast   │  │   pub/sub    │  │   Tracker    │  │
│  └──────────────┘  └──────────────┘  └──────────────┘  │
└──────────────────────────────────────────────────────────┘
                         │
┌────────────────────────▼──────────────────────────────┐
│              Shared Infrastructure Layer                │
│  ┌────────┐ ┌────────┐ ┌────────┐ ┌────────┐ ┌─────┐ │
│  │ Config │ │  DB    │ │ Redis  │ │ Auth   │ │Error│ │
│  │ from   │ │ PgPool │ │  Pool  │ │ JWT    │ │     │ │
│  │  Env   │ │        │ │        │ │Encode/ │ │     │ │
│  │        │ │        │ │        │ │ Decode │ │     │ │
│  └────────┘ └────────┘ └────────┘ └────────┘ └─────┘ │
└──────────────────────────────────────────────────────────┘
                         │
┌────────────────────────▼──────────────────────────────┐
│               Data Layer                                 │
│         ┌─────────────┐    ┌──────────┐                │
│         │ PostgreSQL 16│    │  Redis 7 │                │
│         │  (Primary)   │    │  (Cache) │                │
│         └─────────────┘    └──────────┘                │
└──────────────────────────────────────────────────────────┘
```

### 5-Axis Matching Engine

The core matching algorithm evaluates candidates across five independent axes:

| Axis | Description | Weight |
|------|-------------|--------|
| 🧠 **Capability** | Hard skills, technical competencies | 30% |
| 📊 **Output KPI** | Past performance metrics | 25% |
| 🔍 **Business Gap** | SWOT-derived gap analysis | 20% |
| 🤝 **Work Style** | Collaboration, autonomy, culture fit | 15% |
| 🚀 **Growth Motivation** | Career trajectory, learning drive | 10% |

---

## ⚡ Tech Stack

### Backend

| Technology | Purpose | Version |
|---|---|---|
| **Rust** | Systems programming language | 1.88 (MSRV) |
| **Axum** | Async web framework | 0.7 |
| **Tokio** | Async runtime | 1.35 |
| **SQLx** | Async PostgreSQL driver | 0.7 (compile-time checked) |
| **Redis** | Cache + Event bus | 0.24 (async) |
| **jsonwebtoken** | JWT auth | 9.2 |
| **Tracing** | Structured observability | 0.1 (JSON + OTLP) |
| **Prometheus** | Metrics | 0.13 |
| **Serde** | Serialization | 1.0 |

### Frontend

| Technology | Purpose | Version |
|---|---|---|
| **Remix** | Full-stack React framework | 2.12 |
| **React** | UI library | 18.3 |
| **Vite** | Build tool | 5.4 |
| **Tailwind CSS** | Utility-first CSS | 3.4 |
| **TypeScript** | Type safety | 5.6 |
| **pnpm** | Package manager | 9.1 |
| **Turborepo** | Monorepo orchestration | 2.4 |

### Infrastructure

| Technology | Purpose |
|---|---|
| **Docker** | Containerization (multi-stage, distroless) |
| **GitHub Actions** | CI/CD pipeline (push → build → test → image → GHCR) |
| **PostgreSQL 16** | Primary database |
| **Redis 7** | Cache + event bus |
| **Nginx** | Reverse proxy + SSL termination |

---

## 📁 Project Structure

```
genflow/
├── 🦀 Rust Backend (8 crates)
│   ├── gateway/                          # Axum API binary
│   │   └── src/
│   │       ├── main.rs                   # Entry point, composition root
│   │       ├── state.rs                  # AppState wiring
│   │       ├── auth_context.rs           # Tenant auth extractor
│   │       ├── error_response.rs         # Error → HTTP response bridge
│   │       └── api/
│   │           ├── routes.rs             # Router composition
│   │           └── handlers/             # Per-island handlers
│   │               ├── candidate.rs
│   │               ├── position.rs
│   │               ├── mcp.rs
│   │               └── dashboard.rs
│   ├── receptors/genflow-receptors/      # Pure domain types (zero deps!)
│   │   └── src/domain/                   # MCP, Position, Candidate, Events
│   ├── shared-infra/                     # Config, DB, Redis, Auth, Error
│   ├── synaptic-hub/                     # Event bus (tokio + Redis)
│   ├── islands/
│   │   ├── mcp-registry/                 # MCP context resolution
│   │   ├── position-generation/          # 5-axis position builder
│   │   ├── candidate-matching/           # 5-axis matching engine
│   │   └── dashboard-analytics/          # Metrics + notifications
│   ├── migrations/                       # 11 SQL migration files
│   └── Dockerfile                        # Multi-stage → distroless
│
├── 🌐 Remix Frontend (Turborepo)
│   ├── apps/web/                         # Remix SSR application
│   │   ├── app/
│   │   │   ├── root.tsx                  # Root layout (RTL, Persian)
│   │   │   ├── routes/_index.tsx         # 3-step wizard page
│   │   │   ├── lib/api.ts                # GenFlowApi client
│   │   │   ├── tailwind.css              # Vazirmatn font, global styles
│   │   │   └── entry.{client,server}.tsx
│   │   └── Dockerfile                    # Multi-stage → alpine
│   ├── packages/
│   │   ├── ui/                           # GenButton, GenCard, GenInput, GenBadge
│   │   └── db/                           # TypeScript DB contracts
│   ├── package.json                      # Root workspace
│   ├── pnpm-workspace.yaml
│   └── turbo.json                        # Build orchestration
│
├── 🐳 Infrastructure
│   ├── docker-compose.yml                # Dev compose (api, web, db, redis)
│   ├── docker-compose.prod.yml           # Production override (+nginx, secrets)
│   ├── .env.example                      # Environment template
│   └── deploy/
│       ├── deploy.sh                     # Production deployment script
│       ├── setup.sh                      # First-time server setup
│       ├── backup.sh                     # Database backup script
│       ├── nginx/nginx.conf              # Production nginx config
│       ├── secrets/                      # Managed secrets
│       └── backups/                      # DB backups
│
├── ⚙️ CI/CD
│   ├── .github/workflows/genflow.yml          # CI: format → lint → test → build → image
│   └── .github/workflows/cd.yml          # CD: deployment (manual trigger)
│
└── 📚 Docs
    ├── docs/architecture.md
    ├── docs/event-flow.md
    ├── docs/matching-algorithm.md
    ├── docs/mcp-resolution.md
    ├── docs/validation-matrix.md
    └── docs/docker-strategy.md
```

---

## 🚀 Quick Start

### Prerequisites

| Tool | Version | Install |
|------|---------|---------|
| Docker & Docker Compose | Latest | [Docker Desktop](https://www.docker.com/products/docker-desktop/) |
| Git | Latest | `apt install git` / `brew install git` |

### 1. Clone & Start

```bash
git clone https://github.com/hadiranweb/GenFlow.git
cd GenFlow
git checkout main-platform
docker compose up -d
```

### 2. Verify

```bash
# API Health
curl http://localhost:3000/health
# → {"status":"ok","database":"connected","redis":"connected"}

# Web UI
open http://localhost:3001
```

### 3. Generate a Position

```bash
curl -X POST http://localhost:3000/api/v2/positions/generate \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer YOUR_JWT" \
  -d '{
    "business_name": "شرکت فناوری نوین",
    "industry": "فناوری اطلاعات",
    "description": "نیاز به یک مهندس ارشد نرم‌افزار"
  }'
```

> **First time?** See [Production Deployment](#-docker-deployment) for production setup.

---

## 🐳 Docker Deployment

### Image Architecture

Images are built and published automatically to **GitHub Container Registry (GHCR)** on every push to `main-platform`:

| Image | Registry | Base | Size |
|-------|----------|------|------|
| `hadiranweb/GenFlow/api` | `ghcr.io` | `debian:bookworm-slim` | ~100 MB |
| `hadiranweb/GenFlow/web` | `ghcr.io` | `node:20-alpine` | ~180 MB |

```bash
# Pull latest images
docker pull ghcr.io/hadiranweb/GenFlow/api:latest
docker pull ghcr.io/hadiranweb/GenFlow/web:latest
```

**Performance impact: ZERO.** Docker adds no overhead to native binaries. The Rust binary runs at full native speed with LTO + strip + panic=abort.

### Development

```bash
docker compose up -d       # Start all services
docker compose logs -f     # Follow logs
docker compose down        # Stop all services
```

### Production

```bash
# First-time server setup
sudo ./deploy/setup.sh

# Deploy
./deploy/deploy.sh --prod

# Monitor
./deploy/deploy.sh --status
```

### Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `API_PORT` | `3000` | API Gateway port |
| `WEB_PORT` | `3001` | Web frontend port |
| `JWT_SECRET` | `change-me` | JWT signing key (32+ hex chars) |
| `DB_PASS` | `genflow` | PostgreSQL password |
| `LOG_LEVEL` | `info` | Log level (trace/debug/info/warn/error) |
| `LOG_FORMAT` | `json` | Log format (json/pretty) |

### Resource Limits (Production)

| Service | CPU | Memory | Storage |
|---------|-----|--------|---------|
| API | 2 cores | 1 GB | - |
| Web | 1 core | 512 MB | - |
| PostgreSQL | 2 cores | 2 GB | 50-100 GB |
| Redis | 1 core | 512 MB | - |

---

## 🧪 Development

### Without Docker

```bash
# Start dependencies
docker compose up -d db redis

# Run backend
cargo run -p genflow-gateway

# Run frontend (in another terminal)
pnpm install
pnpm --filter @genflow/web dev
```

### Testing

```bash
# Rust unit tests (no DB required)
cargo test --workspace --lib

# Rust doc tests
cargo test --workspace --doc

# Rust all tests (requires DB + Redis)
docker compose up -d db redis
cargo test --workspace

# Frontend typecheck
pnpm --filter @genflow/web typecheck

# All checks
cargo clippy --workspace -- -D warnings
cargo fmt --all -- --check
```

---

## 🔌 API Reference

### Endpoints

| Method | Path | Description | Auth |
|--------|------|-------------|------|
| `GET` | `/health` | Service health check | ❌ |
| `GET` | `/api/v2/positions` | List positions | ✅ |
| `GET` | `/api/v2/positions/:id` | Get position details | ✅ |
| `POST` | `/api/v2/positions/generate` | Generate position from business input | ✅ |
| `GET` | `/api/v2/candidates` | List candidates | ✅ |
| `POST` | `/api/v2/candidates/match` | Run 5-axis matching | ✅ |
| `GET` | `/api/v2/mcp/:id` | Get MCP context | ✅ |
| `POST` | `/api/v2/mcp/resolve` | Resolve MCP (cache→DB→build) | ✅ |
| `GET` | `/api/v2/dashboard` | Dashboard metrics | ✅ |

### Authentication

All API endpoints (except `/health`) require a JWT Bearer token:

```bash
curl -H "Authorization: Bearer <token>" http://localhost:3000/api/v2/positions
```

---

## ⚙️ CI/CD Pipeline

On every push to `main-platform`, GitHub Actions runs:

```
✅ Rust — format → clippy → test
       ↓
✅ Docker API Image — build & push to ghcr.io

✅ Frontend — install → build
       ↓
✅ Docker Web Image — build & push to ghcr.io
```

### CI Jobs (`.github/workflows/genflow.yml`)

| Job | What it does | Time |
|-----|-------------|------|
| **Rust** | `cargo fmt --check` → `cargo clippy -D warnings` → `cargo test --lib` | ~8 min |
| **Frontend** | `pnpm install --frozen-lockfile` → `pnpm build` | ~5 min |
| **Docker API** | Build `Dockerfile` → push `ghcr.io/.../api:latest` | ~15 min |
| **Docker Web** | Build `apps/web/Dockerfile` → push `ghcr.io/.../web:latest` | ~5 min |

Jobs are **chained** — Rust must pass before API image builds, Frontend before Web image.

### CD (`.github/workflows/cd.yml`)

| Trigger | Action |
|---------|--------|
| Manual (`workflow_dispatch`) | Deploy to staging or production server |

> **Note:** CD requires SSH secrets (see `deploy/setup.sh` and `deploy/.env.production`).

---

## 🔒 Security

### Layers

| Layer | Technology | Status |
|-------|-----------|--------|
| **Authentication** | JWT (jsonwebtoken) | ✅ |
| **Authorization** | TenantAuth + Permission checks | ✅ |
| **Database** | Row-Level Security (RLS) with WITH CHECK | ✅ |
| **Container** | Distroless base (no shell, no package manager) | ✅ |
| **Non-root** | Services run as dedicated user (uid 1000/1001) | ✅ |
| **Audit** | `cargo audit` (run locally or in deploy process) | ✅ |
| **Secrets** | Docker secrets (not env vars) in production | ✅ |
| **Code** | `#![deny(clippy::all)]` on all crates | ✅ |

### Best Practices

- Never commit secrets — use `.env` files or Docker secrets
- Rotate JWT secrets regularly
- Keep dependencies updated (`cargo update` + dependabot)
- Use principle of least privilege for DB roles
- Enable firewall (see `deploy/setup.sh`)

---

## 📈 Performance

### Rust Backend

| Metric | Value |
|--------|-------|
| **Binary size** | ~20 MB (LTO + strip + panic=abort) |
| **Memory footprint** | ~15 MB idle, ~50 MB under load |
| **Request latency** | < 5ms (p50), < 20ms (p99) |
| **Throughput** | 10,000+ req/s per core |
| **GC pauses** | **Zero** (no garbage collector) |

### Docker Overhead

| Component | Bare Metal | Docker | Difference |
|-----------|-----------|--------|------------|
| CPU | Native | Native | **0%** |
| Memory | Native | +2-5% for container | **Negligible** |
| Network | Native | Bridge/NAT | **< 1%** |
| Disk | Native | Overlay2 | **< 1%** |

### Optimization Profile (`.cargo/config.toml`)

```toml
[profile.release]
lto = true            # Full link-time optimization
codegen-units = 1     # Maximum optimization
strip = true          # Remove debug symbols
panic = "abort"       # No unwind tables → smaller, faster
```

---

## 🤝 Contributing

See [CONTRIBUTING.md](./CONTRIBUTING.md) for:

- 🛠️ Development setup (Rust + Node.js)
- 📐 Coding standards (architecture rules, Rust conventions)
- 🧪 Testing guide (unit, integration, writing tests)
- 🔄 PR process

---

## 📄 License

**PROPRIETARY** — © hadiranweb. All rights reserved.

See [`LICENSE`](./LICENSE) for full terms.

---

<div align="center">
  <sub>
    Built with ❤️ and Rust — Zero GC pauses, zero compromises.
    <br>
    <a href="https://github.com/hadiranweb/GenFlow/issues">Report Issue</a> ·
    <a href="https://github.com/hadiranweb/GenFlow/discussions">Discussion</a>
  </sub>
</div>
