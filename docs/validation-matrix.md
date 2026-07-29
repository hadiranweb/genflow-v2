# GenFlow v2 Validation Matrix

This document records the executable acceptance criteria for the incremental
architecture hardening work. It intentionally distinguishes static checks from
runtime checks so an unavailable local Docker/PostgreSQL environment is never
mistaken for a passing integration suite.

## 1. Rust baseline

Run from the repository root with Rust `1.88`:

```bash
cargo fmt --all -- --check
cargo check --workspace --locked
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --lib --locked
cargo build --release -p genflow-gateway --locked
```

All commands must pass before merge. The build must produce
`target/release/genflow-api`.

## 2. Migration chain

Start PostgreSQL from an empty database and start the gateway once. The embedded
migrator must apply migrations `001` through `011` in order.

Verify that these objects exist after migration:

- `current_tenant_organization_id()`
- `candidate_organization_access`
- tenant policies from `010_tenant_context_boundaries.sql`
- `tenant_candidate_organization_access`

Re-running startup migrations must be idempotent.

## 3. Tenant isolation scenarios

Use two organizations, **A** and **B**, and a database role that does not own
the protected tables. For each scenario, set the transaction context through
`begin_organization_transaction`.

| Scenario | Expected result |
|---|---|
| A reads A position | Allowed |
| A reads B position | No row / denied |
| A inserts a Position with B's organization ID | Denied by `WITH CHECK` |
| A reads B Match, Report, Risk Flag or Invitation | No row / denied |
| A writes an access relation for B | Denied |
| Missing tenant context | No tenant-owned row is returned |

Do **not** enable `FORCE ROW LEVEL SECURITY` in production until this suite and
the public Invitation Acceptance flow have been exercised with the production
application database role.

## 4. Position and MCP flow

For an authenticated analyst in Organization A:

1. Resolve MCPs and generate a Position.
2. Verify `position_generation_runs.mcp_bundle_snapshot` is populated.
3. Verify `job_positions.generation_evidence` contains MCP context IDs and
   `standards_used`.
4. Simulate MCP resolver failure; Position Generation must still complete with
   deterministic defaults and a warning log.
5. Simulate Event Bus failure; the Position response must succeed and the log
   must contain the relevant correlation ID.

## 5. Candidate Matching flow

For an existing Position/Candidate pair:

1. Calculate a Match twice.
2. Verify there is exactly one `job_matches` row for the pair.
3. Verify risk flags represent only the latest calculation.
4. Force a risk-flag insert failure and verify the prior Match/flag snapshot is
   retained because the transaction rolls back.
5. Record a human decision, recalculate, and verify `status` and decision audit
   fields remain unchanged while analytical scores refresh.

## 6. Invitation and candidate-access flow

1. Create an Invitation under Organization A.
2. Accept it once and verify Candidate, Invitation and
   `candidate_organization_access` are committed together.
3. Attempt concurrent acceptance with the same code; exactly one request must
   succeed.
4. Verify the response contains the server-generated `candidate_id`.
5. Verify the accepted Candidate access relation belongs to Organization A.

## 7. Gateway authorization matrix

All private endpoints require `Authorization: Bearer <JWT>`.

| Role | Position generation | MCP resolve | Invitation creation | Match decision |
|---|---:|---:|---:|---:|
| `admin` | Allowed | Allowed | Allowed | Allowed |
| `analyst` | Allowed | Allowed | Denied | Denied |
| `representative` | Denied | Denied | Allowed | Allowed |

Additionally verify:

- Missing/invalid token returns `401 AUTH_ERROR`.
- Valid token without permission returns `403 AUTHORIZATION_ERROR`.
- A JWT for Organization A cannot address a Position, Match, Dashboard or
  request body organization belonging to B.
- `RecordDecisionRequest.decided_by`, if supplied by a legacy client, must equal
  the JWT subject. Persistence must always use the JWT subject.

## Current environment status

The Arena sandbox used for this work does not currently provide `cargo`,
`rustfmt`, Docker, or PostgreSQL. Static structural checks have been run, but
Sections 1 through 7 require CI or a development environment with those tools.
