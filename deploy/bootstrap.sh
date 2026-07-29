#!/usr/bin/env bash
# =============================================================================
# GenFlow v2 — Bootstrap Script for Fresh Ubuntu Server
#
# Usage:
#   On a clean Ubuntu 22.04/24.04 server:
#     sudo ./deploy/bootstrap.sh --domain genflow.example.com
#
# Options:
#   --domain DOMAIN   Required: your domain (for Nginx + SSL)
#   --email EMAIL     Optional: for Let's Encrypt (default: admin@DOMAIN)
#   --ssh-port PORT   Optional: change SSH port for hardening (default: 22)
#   --branch BRANCH   Optional: git branch to deploy (default: main-platform)
#   --help            Show this help
#
# This script is IDEMPOTENT — safe to run multiple times.
# =============================================================================

set -euo pipefail

# ─────────────────────────────────────────────────────────────
# Color output
# ─────────────────────────────────────────────────────────────
RED='\033[0;31m'; GREEN='\033[0;32m'; YELLOW='\033[1;33m'
BLUE='\033[0;34m'; NC='\033[0m'
info()  { echo -e "${GREEN}[INFO]${NC}  $*"; }
warn()  { echo -e "${YELLOW}[WARN]${NC}  $*"; }
error() { echo -e "${RED}[ERROR]${NC} $*" >&2; }
step()  { echo -e "\n${BLUE}════════════════════════════════════════════════════════════${NC}"; echo -e "${BLUE}  ▶ $*${NC}"; echo -e "${BLUE}════════════════════════════════════════════════════════════${NC}"; }

# ─────────────────────────────────────────────────────────────
# Parse arguments
# ─────────────────────────────────────────────────────────────
DOMAIN=""
EMAIL=""
SSH_PORT="22"
BRANCH="main-platform"

while [[ $# -gt 0 ]]; do
  case "$1" in
    --domain)   DOMAIN="$2";   shift 2 ;;
    --email)    EMAIL="$2";    shift 2 ;;
    --ssh-port) SSH_PORT="$2"; shift 2 ;;
    --branch)   BRANCH="$2";   shift 2 ;;
    --help)     grep "^#" "$0" | grep -v "^#!/" | sed 's/^# //'; exit 0 ;;
    *)          error "Unknown option: $1"; exit 1 ;;
  esac
done

if [ -z "$DOMAIN" ]; then
  error "Missing --domain. Usage: sudo ./deploy/bootstrap.sh --domain genflow.example.com"
  exit 1
fi
EMAIL="${EMAIL:-admin@$DOMAIN}"

# ─────────────────────────────────────────────────────────────
# Pre-flight checks
# ─────────────────────────────────────────────────────────────
step "Pre-flight checks"
if [ "$(id -u)" -ne 0 ]; then
  error "This script must be run as root (use sudo)"
  exit 1
fi

if ! command -v curl &>/dev/null; then
  apt-get update -qq && apt-get install -y -qq curl
fi

OS_VERSION=$(lsb_release -rs 2>/dev/null || cat /etc/os-release 2>/dev/null | grep VERSION_ID | cut -d= -f2 | tr -d '"')
info "OS: $(lsb_release -ds 2>/dev/null || cat /etc/os-release | grep PRETTY_NAME | cut -d= -f2 | tr -d '\"')"
info "Domain: $DOMAIN"
info "Branch: $BRANCH"
info "SSH Port: $SSH_PORT"

# ─────────────────────────────────────────────────────────────
# 1. System update & basic packages
# ─────────────────────────────────────────────────────────────
step "1/8 — System update & basic packages"
apt-get update -qq
apt-get upgrade -y -qq
apt-get install -y -qq \
  ca-certificates curl gnupg lsb-release ufw fail2ban \
  git nginx certbot python3-certbot-nginx \
  htop net-tools unattended-upgrades
info "System packages installed ✅"

# ─────────────────────────────────────────────────────────────
# 2. Firewall (UFW)
# ─────────────────────────────────────────────────────────────
step "2/8 — Firewall configuration"
ufw --force reset
ufw default deny incoming
ufw default allow outgoing
ufw allow "$SSH_PORT"/tcp comment "SSH"
ufw allow 80/tcp comment "HTTP"
ufw allow 443/tcp comment "HTTPS"
ufw --force enable
info "UFW activated ✅ — SSH($SSH_PORT), HTTP(80), HTTPS(443)"

# ─────────────────────────────────────────────────────────────
# 3. Deploy user
# ─────────────────────────────────────────────────────────────
step "3/8 — Deploy user"
if id "deploy" &>/dev/null; then
  info "User 'deploy' already exists"
else
  useradd -m -s /bin/bash -G sudo deploy
  passwd -d deploy
  info "User 'deploy' created"
fi

# Allow deploy to sudo without password (for Docker commands)
echo "deploy ALL=(ALL) NOPASSWD: /usr/bin/docker" > /etc/sudoers.d/deploy-docker
chmod 440 /etc/sudoers.d/deploy-docker

# Copy root's SSH authorized_keys to deploy user (if root has them)
if [ -f /root/.ssh/authorized_keys ] && [ ! -f /home/deploy/.ssh/authorized_keys ]; then
  mkdir -p /home/deploy/.ssh
  cp /root/.ssh/authorized_keys /home/deploy/.ssh/
  chown -R deploy:deploy /home/deploy/.ssh
  chmod 700 /home/deploy/.ssh
  chmod 600 /home/deploy/.ssh/authorized_keys
  info "SSH keys copied from root to deploy user"
fi

# ─────────────────────────────────────────────────────────────
# 4. Docker Engine
# ─────────────────────────────────────────────────────────────
step "4/8 — Docker Engine"
if command -v docker &>/dev/null; then
  info "Docker already installed: $(docker --version)"
else
  install -m 0755 -d /etc/apt/keyrings
  curl -fsSL https://download.docker.com/linux/ubuntu/gpg | gpg --dearmor -o /etc/apt/keyrings/docker.gpg
  echo "deb [arch=$(dpkg --print-architecture) signed-by=/etc/apt/keyrings/docker.gpg] https://download.docker.com/linux/ubuntu $(lsb_release -cs) stable" > /etc/apt/sources.list.d/docker.list
  apt-get update -qq
  apt-get install -y -qq docker-ce docker-ce-cli containerd.io docker-compose-plugin
  info "Docker installed: $(docker --version)"
fi

# Add deploy user to docker group
usermod -aG docker deploy
info "User 'deploy' added to docker group"

# Enable Docker on boot
systemctl enable docker --now
info "Docker service enabled ✅"

# ─────────────────────────────────────────────────────────────
# 5. Clone / Update repository
# ─────────────────────────────────────────────────────────────
step "5/8 — GenFlow repository"
DEPLOY_DIR="/opt/genflow"

if [ -d "$DEPLOY_DIR/.git" ]; then
  info "Repository exists — pulling latest from $BRANCH"
  cd "$DEPLOY_DIR"
  git fetch origin
  git checkout "$BRANCH"
  git pull origin "$BRANCH"
else
  info "Cloning repository (branch: $BRANCH)"
  mkdir -p "$DEPLOY_DIR"
  git clone --depth 1 --branch "$BRANCH" https://github.com/hadiranweb/genflow "$DEPLOY_DIR"
fi
chown -R deploy:deploy "$DEPLOY_DIR"
info "Repository ready at $DEPLOY_DIR ✅"

# ─────────────────────────────────────────────────────────────
# 6. Environment configuration
# ─────────────────────────────────────────────────────────────
step "6/8 — Environment configuration"
ENV_FILE="$DEPLOY_DIR/.env"

if [ -f "$ENV_FILE" ]; then
  info ".env already exists — keeping existing values"
else
  JWT_SECRET=$(openssl rand -hex 64)
  DB_PASS=$(openssl rand -hex 16)
  REDIS_PASS=$(openssl rand -hex 16)

  cat > "$ENV_FILE" << EOF
# ===========================================
# GenFlow — Production Environment
# Auto-generated by bootstrap.sh
# ===========================================

# Server
SERVER_HOST=0.0.0.0
SERVER_PORT=3000

# Database
DB_USER=genflow
DB_PASS=${DB_PASS}
DB_NAME=genflow

# Redis
REDIS_PASS=${REDIS_PASS}

# JWT
JWT_SECRET=${JWT_SECRET}
JWT_EXPIRATION_HOURS=24

# Logging
LOG_LEVEL=info
LOG_FORMAT=json

# Docker compose overrides
COMPOSE_PROJECT_NAME=genflow
EOF
  info "Environment file created with secure passwords ✅"
fi

# ─────────────────────────────────────────────────────────────
# 7. Nginx + SSL reverse proxy
# ─────────────────────────────────────────────────────────────
step "7/8 — Nginx reverse proxy with SSL"

NGINX_CONF="/etc/nginx/sites-available/genflow"

if [ -f "$NGINX_CONF" ]; then
  info "Nginx config exists — skipping"
else
  cat > "$NGINX_CONF" << 'NGINX'
server {
    listen 80;
    listen [::]:80;
    server_name DOMAIN_PLACEHOLDER;

    # Root health check
    location /health {
        proxy_pass http://127.0.0.1:3000;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_http_version 1.1;
    }

    # API upstream
    location /api/ {
        proxy_pass http://127.0.0.1:3000;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
        proxy_http_version 1.1;
    }

    # Frontend
    location / {
        proxy_pass http://127.0.0.1:3001;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
        proxy_http_version 1.1;
    }
}
NGINX
  sed -i "s/DOMAIN_PLACEHOLDER/$DOMAIN/g" "$NGINX_CONF"
  ln -sf "$NGINX_CONF" /etc/nginx/sites-enabled/
  rm -f /etc/nginx/sites-enabled/default
fi

# Obtain SSL certificate
if [ -d "/etc/letsencrypt/live/$DOMAIN" ]; then
  info "SSL certificate exists for $DOMAIN"
else
  info "Obtaining SSL certificate for $DOMAIN ..."
  certbot --nginx -d "$DOMAIN" --non-interactive --agree-tos --email "$EMAIL" || {
    warn "certbot failed — will try again after DNS propagation"
    warn "Run manually: certbot --nginx -d $DOMAIN"
  }
fi

nginx -t && systemctl reload nginx || true
info "Nginx configured ✅"

# ─────────────────────────────────────────────────────────────
# 8. Deploy services
# ─────────────────────────────────────────────────────────────
step "8/8 — Deploy GenFlow services"
cd "$DEPLOY_DIR"

# Use the production override if it exists
COMPOSE_FILES="-f docker-compose.yml"
if [ -f "docker-compose.prod.yml" ]; then
  COMPOSE_FILES="$COMPOSE_FILES -f docker-compose.prod.yml"
fi

# Pull images
info "Pulling Docker images..."
docker compose $COMPOSE_FILES pull
info "Images pulled ✅"

# Start services
info "Starting services..."
docker compose $COMPOSE_FILES up -d --remove-orphans
info "Services started ✅"

# Health check
info "Running health check..."
sleep 15
HEALTH_URL="http://localhost:3000/health"
for i in $(seq 1 12); do
  STATUS=$(curl -s -o /dev/null -w "%{http_code}" "$HEALTH_URL" 2>/dev/null || echo "000")
  if [ "$STATUS" = "200" ]; then
    info "Health check PASSED (attempt $i/12) ✅"
    break
  fi
  if [ "$i" = "12" ]; then
    warn "Health check did not pass after 12 attempts (status=$STATUS)"
    warn "Check logs: docker compose logs api"
  fi
  sleep 5
done

# ─────────────────────────────────────────────────────────────
# Summary
# ─────────────────────────────────────────────────────────────
step "✅ Deployment complete!"

IP=$(curl -s ifconfig.me 2>/dev/null || echo "unknown")
echo ""
echo "  ┌─────────────────────────────────────────────────────────┐"
echo "  │  GenFlow v2 — Deployment Summary                        │"
echo "  ├─────────────────────────────────────────────────────────┤"
printf "  │  Domain:           ${DOMAIN}%-24s│\n" ""
printf "  │  Server IP:        ${IP}%-24s│\n" ""
printf "  │  API:              https://${DOMAIN}/api/v2/%-12s│\n" ""
printf "  │  Frontend:         https://${DOMAIN}/%-30s│\n" ""
printf "  │  Health:           https://${DOMAIN}/health%-19s│\n" ""
echo "  ├─────────────────────────────────────────────────────────┤"
printf "  │  Deploy user:      deploy%-36s│\n" ""
printf "  │  Deploy dir:       ${DEPLOY_DIR}%-40s│\n" ""
echo "  ├─────────────────────────────────────────────────────────┤"
printf "  │  PostgreSQL:       docker compose exec db psql -U genflow%-3s│\n" ""
printf "  │  Logs (api):       docker compose logs -f api%-17s│\n" ""
printf "  │  Logs (web):       docker compose logs -f web%-17s│\n" ""
printf "  │  Backup:           ./deploy/backup.sh%-23s│\n" ""
echo "  ├─────────────────────────────────────────────────────────┤"
printf "  │  SSH:              ssh deploy@${IP} -p ${SSH_PORT}%-10s│\n" ""
echo "  └─────────────────────────────────────────────────────────┘"
echo ""
info "Next steps:"
echo "  1. Point your DNS A record → $IP"
echo "  2. Set up GitHub secrets for CD: STAGING_HOST, STAGING_USER, STAGING_SSH_KEY"
echo "  3. To redeploy: docker compose pull && docker compose up -d"
echo ""
