#!/usr/bin/env bash
# =============================================================================
# GenFlow v2 — First-Time Server Setup
#
# Run on a fresh Ubuntu 22.04/24.04 server:
#   curl -fsSL https://raw.githubusercontent.com/hadiranweb/GenFlow/main-platform/deploy/setup.sh | bash
# =============================================================================

set -euo pipefail

echo "========================================"
echo " GenFlow v2 — Server Setup"
echo "========================================"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
NC='\033[0m'

ok()  { echo -e "${GREEN}✓${NC} $1"; }
err() { echo -e "${RED}✗${NC} $1"; exit 1; }

# ============================================================
# 1. System dependencies
# ============================================================
echo ""
echo "--- Installing system dependencies..."

sudo apt-get update -qq
sudo apt-get install -y -qq \
    curl \
    git \
    openssl \
    ca-certificates \
    gnupg \
    lsb-release \
    ufw \
    2>/dev/null

ok "System dependencies installed"

# ============================================================
# 2. Docker
# ============================================================
echo ""
echo "--- Installing Docker..."

if ! command -v docker &>/dev/null; then
    sudo install -m 0755 -d /etc/apt/keyrings
    curl -fsSL https://download.docker.com/linux/ubuntu/gpg | \
        sudo gpg --dearmor -o /etc/apt/keyrings/docker.gpg 2>/dev/null
    echo "deb [arch=$(dpkg --print-architecture) signed-by=/etc/apt/keyrings/docker.gpg] \
        https://download.docker.com/linux/ubuntu $(lsb_release -cs) stable" | \
        sudo tee /etc/apt/sources.list.d/docker.list > /dev/null
    sudo apt-get update -qq
    sudo apt-get install -y -qq docker-ce docker-ce-cli containerd.io docker-compose-plugin 2>/dev/null
    sudo usermod -aG docker "$USER"
    ok "Docker installed (log out/in to use without sudo)"
else
    ok "Docker already installed ($(docker --version))"
fi

# ============================================================
# 3. Clone GenFlow
# ============================================================
echo ""
echo "--- Cloning GenFlow..."

DEPLOY_DIR="/opt/genflow"
if [ ! -d "$DEPLOY_DIR" ]; then
    sudo mkdir -p "$DEPLOY_DIR"
    sudo git clone https://github.com/hadiranweb/GenFlow.git "$DEPLOY_DIR"
    sudo chown -R "$USER:$USER" "$DEPLOY_DIR"
    cd "$DEPLOY_DIR"
    git checkout main-platform
    ok "GenFlow cloned to $DEPLOY_DIR (branch: main-platform)"
else
    cd "$DEPLOY_DIR"
    git pull origin main-platform
    ok "GenFlow already cloned — updated to latest"
fi

# ============================================================
# 4. Generate secrets
# ============================================================
echo ""
echo "--- Generating secrets..."

if [ ! -f "$DEPLOY_DIR/deploy/secrets/jwt_secret.txt" ]; then
    openssl rand -hex 32 | sudo tee "$DEPLOY_DIR/deploy/secrets/jwt_secret.txt" > /dev/null
    ok "JWT secret generated"
fi

if [ ! -f "$DEPLOY_DIR/deploy/secrets/db_password.txt" ]; then
    openssl rand -hex 16 | sudo tee "$DEPLOY_DIR/deploy/secrets/db_password.txt" > /dev/null
    ok "Database password generated"
fi

# ============================================================
# 5. Configure .env
# ============================================================
echo ""
echo "--- Configuring environment..."

if [ ! -f "$DEPLOY_DIR/.env" ]; then
    cat > "$DEPLOY_DIR/.env" << 'EOF'
# GenFlow v2 — Production Environment
JWT_SECRET=$(cat /opt/genflow/deploy/secrets/jwt_secret.txt)
API_PORT=3000
WEB_PORT=3001
DB_USER=genflow
DB_PASS=$(cat /opt/genflow/deploy/secrets/db_password.txt)
DB_NAME=genflow
DB_PORT=5432
REDIS_PORT=6379
LOG_LEVEL=info
LOG_FORMAT=json
API_CPU_LIMIT=2
API_MEM_LIMIT=1G
WEB_CPU_LIMIT=1
WEB_MEM_LIMIT=512M
DB_CPU_LIMIT=2
DB_MEM_LIMIT=2G
OPENAI_API_KEY=
ANTHROPIC_API_KEY=
EOF
    ok ".env file created — edit OPENAI_API_KEY and ANTHROPIC_API_KEY if needed"
else
    ok ".env already exists"
fi

# ============================================================
# 6. Firewall
# ============================================================
echo ""
echo "--- Configuring firewall..."

sudo ufw --force reset > /dev/null 2>&1
sudo ufw default deny incoming > /dev/null
sudo ufw default allow outgoing > /dev/null
sudo ufw allow ssh > /dev/null
sudo ufw allow 80/tcp > /dev/null
sudo ufw allow 443/tcp > /dev/null
sudo ufw allow 3000/tcp > /dev/null  # API
sudo ufw allow 3001/tcp > /dev/null  # Web
sudo ufw --force enable > /dev/null
ok "Firewall configured (SSH, HTTP, HTTPS, API, Web)"

# ============================================================
# 7. Done
# ============================================================
echo ""
echo "========================================"
echo " Setup Complete!"
echo "========================================"
echo ""
echo " Next steps:"
echo "   1. Edit /opt/genflow/.env with your API keys"
echo "   2. cd /opt/genflow && ./deploy/deploy.sh --prod"
echo ""
echo " Services will be available at:"
echo "   API:  http://YOUR_SERVER_IP:3000/health"
echo "   Web:  http://YOUR_SERVER_IP:3001"
echo ""
