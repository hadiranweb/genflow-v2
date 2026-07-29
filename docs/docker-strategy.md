# Docker Image Strategy for GenFlow v2

> **Version**: 1.0 — July 2026
> **Scope**: Production deployment architecture for GenFlow hybrid Rust + Remix platform

---

## 1. Component Breakdown

| Component | Runtime | Build Tool | Dependencies | Scale |
|-----------|---------|------------|-------------|-------|
| **API Gateway** (Rust) | Native binary | `cargo build --release` | None (statically linked) | 1–16 cores |
| **Remix Frontend** (Node.js) | Node.js 20 | `pnpm build` → `remix-serve` | Node.js runtime | 1–8 cores |
| **PostgreSQL 16** | Native | Official image | Persistent volume | Volume-bound |
| **Redis 7** | Native | Official image | Persistent volume | RAM-bound |

**Scale assessment**: This is a **medium-scale** platform. All 4 services can run on a single production server (4–8 vCPU, 16 GB RAM, 100 GB SSD). Each component can also scale horizontally if needed by running behind a load balancer.

---

## 2. Image Count Decision

### ❌ Option A: All-in-one image
API + Web in one container. Benefits: single image to manage.  
**Problems**: Need both Rust binary AND Node.js, different lifecycle, different resource profiles, can't scale independently.  
→ **Rejected**: Loss of architectural clarity, no independent scaling.

### ❌ Option B: Separate images per microservice
Each Rust island (MCP, Position, Matching, Dashboard) as its own image.  
**Problems**: Overkill for current scale, unnecessary network hops, complex orchestration, the islands are library crates not services.  
→ **Rejected**: Adds complexity without benefit at this scale.

### ✅ Option C: Two application images + two data layer images
Three images total (built by us) + two official images:

| Image | Base | Size Target | Source | Scaling |
|-------|------|------------|--------|---------|
| **`genflow/api`** | Distroless/scratch | < 50 MB | `Dockerfile` (Rust) | Horizontal |
| **`genflow/web`** | `node:20-alpine` | < 200 MB | `apps/web/Dockerfile` | Horizontal |
| `postgres:16-alpine` | — | ~400 MB | Official | Volume |
| `redis:7-alpine` | — | ~30 MB | Official | RAM |

**Performance impact**: **ZERO**. Docker containers add no CPU/memory overhead to native binaries. The Rust binary runs at native speed. Node.js runs at native speed. PostgreSQL and Redis run at native speed.

---

## 3. Base Image Selection (Rust API)

### Options Comparison

| Base | Size | Security | Performance | Pros | Cons |
|------|------|----------|------------|------|------|
| **Distroless CC** (`gcr.io/distroless/cc-debian12`) | ~25 MB + binary | ✅ Minimal | ✅ Native | Small, secure, has libc | No shell (debug hard) |
| **Debian slim** (`debian:bookworm-slim`) | ~80 MB + binary | ✅ Good | ✅ Native | Has shell + tools | Larger |
| **Alpine** (`alpine:3.20`) | ~5 MB + binary | ✅ Good | ⚠️ musl (maybe 5-10% slower) | Smallest | musl vs glibc compat, slower |
| **Scratch** | 0 MB | ✅ Best | ✅ Native | Tiny, static binary | Requires fully static binary |

### ✅ Recommendation: Distroless CC (`gcr.io/distroless/cc-debian12`)

- We need libc (glibc) for sqlx + tokio runtime → scratch won't work without fully static musl build
- Distroless gives us glibc with minimal surface area (~25 MB)
- No shell, no package manager → minimal attack surface
- Combined with Rust release binary (LTO + strip + panic=abort) → final image ~35 MB
- Alternative: `debian:bookworm-slim` for debug builds

**Current state**: Uses `debian:bookworm-slim` → good but can be ~50 MB smaller with distroless

---

## 4. Base Image Selection (Remix Frontend)

### ✅ Decision: `node:20-alpine`

- Smallest Node.js image (~120 MB)
- npm/pnpm pre-installed
- Production use is well-established
- Node.js 20 with long-term support

**Current state**: Uses `node:20-alpine` → ✅ Already optimal

---

## 5. Multi-Architecture Support

### Strategy: Docker Buildx for amd64 + arm64

```yaml
# In CI/CD
- name: Set up Docker Buildx
  uses: docker/setup-buildx-action@v3

- name: Build multi-arch
  uses: docker/build-push-action@v5
  with:
    platforms: linux/amd64,linux/arm64
```

- `linux/amd64`: Primary target (production server)
- `linux/arm64`: Apple Silicon devs, ARM servers (AWS Graviton, etc.)

---

## 6. Performance Optimization

### Rust Binary (no compromise)

```toml
# .cargo/config.toml — ALREADY SET
[profile.release]
lto = true            # Full link-time optimization
codegen-units = 1     # Maximum optimization surface
strip = true          # Remove debug symbols (smaller binary)
panic = "abort"       # No panic unwind tables (smaller,faster)
```

**Effect**: Binary size ~15–25 MB, maximum runtime performance.

### Docker Build Cache

```yaml
# In CI/CD docker build
cache-from: type=gha
cache-to: type=gha,mode=max
```

- Cargo registry/layer caching
- pnpm store caching  
- Docker layer reuse

---

## 7. Network Architecture (Single Server)

```
  Internet :80/:443
       │
       ▼
┌──────────────┐
│   Nginx      │  ← Reverse proxy, SSL termination, static cache
│  (optional)  │     Routes: /api/* → API, /* → Web
└──────┬───────┘
       │
  ┌────┴────┐
  │         │
  ▼         ▼
 API       Web
 :3000     :3000 (internal)
  │         │
  └────┬────┘
       │
  ┌────┴────┐
  │  Network │  ← Internal Docker network
  └────┬────┘
       │
  ┌────┴────────┐
  │             │
  ▼             ▼
 PostgreSQL    Redis
 :5432         :6379
```

---

## 8. Resource Limits

| Service | CPU | Memory | Storage | Priority |
|---------|-----|--------|---------|----------|
| API | 1-2 cores | 512 MB - 1 GB | - | High |
| Web | 0.5-1 core | 256-512 MB | - | Medium |
| PostgreSQL | 1-2 cores | 2-4 GB | 50-100 GB | Critical |
| Redis | 0.5 core | 256 MB - 1 GB | - | High |

---

## 9. Summary: Final Image Strategy

1. **3 images to build**: `genflow/api` (distroless), `genflow/web` (alpine), optional `nginx` (alpine)
2. **2 official images**: `postgres:16-alpine`, `redis:7-alpine`
3. **Multi-stage builds with LTO+strip** → zero performance overhead
4. **Multi-arch (amd64 + arm64)** for dev + prod flexibility
5. **Single `docker-compose up -d`** for deployment
6. **Worst case**: 5 containers, ~2 GB RAM total, ~100 GB storage for DB

---

## 10. If NOT Using Docker

If Docker is rejected for any reason:

**Alternative: Systemd + Binary Drop**
```bash
# Build Rust binary
cargo build --release --bin genflow-gateway
# Create systemd service
cat > /etc/systemd/system/genflow-api.service << 'EOF'
[Unit]
Description=GenFlow API Gateway
After=network.target postgresql.service redis-server.service

[Service]
Type=simple
User=genflow
ExecStart=/opt/genflow/genflow-gateway
Restart=always
EnvironmentFile=/opt/genflow/.env

[Install]
WantedBy=multi-user.target
EOF
# Run Remix with systemd or PM2
npm install -g pm2
pm2 start /opt/genflow/apps/web/build/server/index.js --name genflow-web
```

**Pros**: No Docker overhead, simpler debugging  
**Cons**: Manual dependency management, harder rollback, environment inconsistency  
**Decision**: Docker is recommended — zero performance overhead, easier deployment, consistent environments. But systemd alternative is documented.
