# =============================================================================
# GenFlow v2 — Production Rust API Dockerfile
#
# Builder: rust:1.88-slim, workspace-aware dependency caching
# Runtime: debian:bookworm-slim, ca-certificates + curl for health check
# =============================================================================

FROM rust:1.88-slim AS builder

WORKDIR /app
RUN apt-get update && apt-get install -y pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*

# -------------------------------------------------------
# Dependency cache layer — copy only Cargo.toml files
# -------------------------------------------------------
COPY Cargo.toml Cargo.lock ./
COPY .cargo/config.toml .cargo/

COPY receptors/genflow-receptors/Cargo.toml receptors/genflow-receptors/
COPY shared-infra/Cargo.toml shared-infra/
COPY synaptic-hub/Cargo.toml synaptic-hub/
COPY islands/mcp-registry/Cargo.toml islands/mcp-registry/
COPY islands/position-generation/Cargo.toml islands/position-generation/
COPY islands/candidate-matching/Cargo.toml islands/candidate-matching/
COPY islands/dashboard-analytics/Cargo.toml islands/dashboard-analytics/
COPY gateway/Cargo.toml gateway/

# Dummy sources to compile only dependencies
RUN mkdir -p receptors/genflow-receptors/src \
  && echo "pub fn dummy() {}" > receptors/genflow-receptors/src/lib.rs \
  && mkdir -p shared-infra/src \
  && echo "pub fn dummy() {}" > shared-infra/src/lib.rs \
  && mkdir -p synaptic-hub/src \
  && echo "pub fn dummy() {}" > synaptic-hub/src/lib.rs \
  && mkdir -p islands/mcp-registry/src/runtime \
  && echo "pub fn dummy() {}" > islands/mcp-registry/src/lib.rs \
  && mkdir -p islands/position-generation/src/services \
  && echo "pub fn dummy() {}" > islands/position-generation/src/lib.rs \
  && mkdir -p islands/candidate-matching/src/services \
  && echo "pub fn dummy() {}" > islands/candidate-matching/src/lib.rs \
  && mkdir -p islands/dashboard-analytics/src/services \
  && echo "pub fn dummy() {}" > islands/dashboard-analytics/src/lib.rs \
  && mkdir -p gateway/src \
  && echo "fn main() {}" > gateway/src/main.rs

RUN cargo build --release -p genflow-gateway --locked

# -------------------------------------------------------
# Real build — copy actual source
# -------------------------------------------------------
COPY receptors/ receptors/
COPY shared-infra/ shared-infra/
COPY synaptic-hub/ synaptic-hub/
COPY islands/ islands/
COPY gateway/ gateway/
COPY migrations/ migrations/

RUN find receptors shared-infra synaptic-hub islands gateway -name "*.rs" -exec touch {} +

RUN cargo build --release -p genflow-gateway --locked
RUN strip /app/target/release/genflow-api

# -------------------------------------------------------
# Runtime stage — minimal debian
# -------------------------------------------------------
FROM debian:bookworm-slim

WORKDIR /app

RUN apt-get update && apt-get install -y ca-certificates curl \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/genflow-api /app/genflow-api
COPY --from=builder /app/migrations /app/migrations

RUN groupadd -r genflow --gid 1000 && useradd -r -g genflow --uid 1000 -d /app genflow \
    && chown -R genflow:genflow /app

USER genflow
EXPOSE 3000

HEALTHCHECK --interval=30s --timeout=5s --start-period=20s --retries=3 \
  CMD curl -sf http://localhost:3000/health || exit 1

ENTRYPOINT ["/app/genflow-api"]
