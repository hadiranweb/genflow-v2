#!/usr/bin/env bash
# =============================================================================
# GenFlow v2 — Production Deployment Script
#
# Usage:
#   ./deploy/deploy.sh              # Deploy with default env
#   ./deploy/deploy.sh --prod       # Deploy with production override
#   ./deploy/deploy.sh --rollback   # Rollback to previous version
#
# Requirements: docker, docker compose, git
# =============================================================================

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
cd "$PROJECT_DIR"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

log_info()  { echo -e "${BLUE}[INFO]${NC}  $1"; }
log_ok()    { echo -e "${GREEN}[OK]${NC}    $1"; }
log_warn()  { echo -e "${YELLOW}[WARN]${NC}  $1"; }
log_error() { echo -e "${RED}[ERROR]${NC} $1"; }

# ============================================================
# Pre-flight checks
# ============================================================
preflight() {
    log_info "Running pre-flight checks..."

    command -v docker >/dev/null 2>&1 || { log_error "Docker is not installed"; exit 1; }
    command -v git >/dev/null 2>&1 || { log_error "Git is not installed"; exit 1; }

    # Check Docker Compose (v2 plugin or standalone)
    if docker compose version >/dev/null 2>&1; then
        COMPOSE="docker compose"
    elif docker-compose --version >/dev/null 2>&1; then
        COMPOSE="docker-compose"
    else
        log_error "Docker Compose is not installed"
        exit 1
    fi
    log_ok "Using: $COMPOSE"

    # Check .env file
    if [ ! -f .env ]; then
        log_warn ".env file not found — copying from .env.example"
        cp .env.example .env
        log_warn "Edit .env with your production values and re-run"
    fi

    log_ok "Pre-flight checks passed"
}

# ============================================================
# Deploy
# ============================================================
deploy() {
    local PROFILE="${1:-dev}"
    local COMPOSE_FILES="-f docker-compose.yml"

    log_info "Starting deployment (profile: $PROFILE)..."

    # Pull latest code
    log_info "Pulling latest code..."
    git fetch origin
    git checkout main-platform
    git pull origin main-platform

    # Build and start
    if [ "$PROFILE" = "prod" ]; then
        if [ ! -f docker-compose.prod.yml ]; then
            log_warn "docker-compose.prod.yml not found — deploying without production overrides"
        else
            COMPOSE_FILES="$COMPOSE_FILES -f docker-compose.prod.yml"
            if [ ! -f deploy/secrets/jwt_secret.txt ]; then
                log_info "Generating JWT secret..."
                openssl rand -hex 32 > deploy/secrets/jwt_secret.txt
            fi
            if [ ! -f deploy/secrets/db_password.txt ]; then
                log_info "Generating database password..."
                openssl rand -hex 16 > deploy/secrets/db_password.txt
            fi
        fi
    fi

    log_info "Building images..."
    $COMPOSE $COMPOSE_FILES build --pull

    log_info "Starting services..."
    $COMPOSE $COMPOSE_FILES up -d

    # Wait for services
    log_info "Waiting for services to be healthy..."
    sleep 10

    # Check health
    if $COMPOSE $COMPOSE_FILES ps | grep -q "unhealthy"; then
        log_warn "Some services are unhealthy — check logs: docker compose logs"
        $COMPOSE $COMPOSE_FILES ps
    else
        log_ok "All services are healthy"
    fi

    # Clean up old images
    log_info "Cleaning up old images..."
    docker image prune -f

    log_ok "Deployment complete!"
    echo ""
    echo "  API:  http://$(curl -s ifconfig.me 2>/dev/null || echo 'localhost'):${API_PORT:-3000}/health"
    echo "  Web:  http://$(curl -s ifconfig.me 2>/dev/null || echo 'localhost'):${WEB_PORT:-3001}"
    echo ""
    echo "  Logs: $COMPOSE $COMPOSE_FILES logs -f"
}

# ============================================================
# Rollback
# ============================================================
rollback() {
    log_info "Rolling back to previous deployment..."
    local COMPOSE_FILES="-f docker-compose.yml"

    if [ -f docker-compose.prod.yml ]; then
        COMPOSE_FILES="$COMPOSE_FILES -f docker-compose.prod.yml"
    fi

    # Rollback git
    log_info "Rolling back git..."
    git checkout HEAD~1

    # Rebuild and restart
    $COMPOSE $COMPOSE_FILES build --pull
    $COMPOSE $COMPOSE_FILES up -d

    log_ok "Rollback complete"
}

# ============================================================
# Status
# ============================================================
status() {
    local COMPOSE_FILES="-f docker-compose.yml"
    [ -f docker-compose.prod.yml ] && COMPOSE_FILES="$COMPOSE_FILES -f docker-compose.prod.yml"

    echo "=== GenFlow Service Status ==="
    echo ""
    $COMPOSE $COMPOSE_FILES ps
    echo ""
    echo "=== Resource Usage ==="
    docker stats --no-stream --format "table {{.Name}}\t{{.CPUPerc}}\t{{.MemUsage}}\t{{.NetIO}}\t{{.BlockIO}}" $(docker ps --filter "name=genflow-" --format "{{.Name}}") 2>/dev/null || echo "(containers not running)"
}

# ============================================================
# Main
# ============================================================
main() {
    case "${1:-help}" in
        --prod|--production)
            preflight
            deploy "prod"
            ;;
        --dev|--development)
            preflight
            deploy "dev"
            ;;
        --rollback)
            rollback
            ;;
        --status)
            status
            ;;
        --help|-h)
            echo "GenFlow Deployment Script"
            echo ""
            echo "Usage:"
            echo "  ./deploy/deploy.sh --prod      Deploy to production"
            echo "  ./deploy/deploy.sh --dev       Deploy to development"
            echo "  ./deploy/deploy.sh --rollback  Rollback to previous version"
            echo "  ./deploy/deploy.sh --status    Show service status"
            echo "  ./deploy/deploy.sh --help      Show this help"
            ;;
        *)
            preflight
            deploy "dev"
            ;;
    esac
}

main "$@"
