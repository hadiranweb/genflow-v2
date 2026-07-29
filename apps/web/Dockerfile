# =============================================================================
# GenFlow v2 — Remix Frontend Dockerfile
#
# Builder: node:20-alpine, pnpm install with frozen-lockfile for deterministic
#   builds. The pnpm-lock.yaml must be committed into the monorepo.
# Runtime: node:20-alpine, production-only artifacts
# =============================================================================

FROM node:20-alpine AS builder

WORKDIR /app
RUN corepack enable && corepack prepare pnpm@9.1.0 --activate

# Dependency cache layer — lockfile enables deterministic installs
COPY pnpm-lock.yaml package.json pnpm-workspace.yaml .npmrc ./
COPY apps/web/package.json apps/web/
COPY packages/ui/package.json packages/ui/
COPY packages/db/package.json packages/db/

RUN pnpm install --frozen-lockfile

# Source code + build
COPY . .
RUN pnpm --filter @genflow/web build

# -------------------------------------------------------
# Runtime stage
# -------------------------------------------------------
FROM node:20-alpine AS runner

WORKDIR /app

ENV NODE_ENV=production NODE_TELEMETRY_DISABLED=1 PORT=3000

COPY --from=builder /app/apps/web/build ./build
COPY --from=builder /app/apps/web/public ./public
COPY --from=builder /app/apps/web/package.json ./

EXPOSE 3000

CMD ["node_modules/.bin/remix-serve", "./build/server/index.js"]
