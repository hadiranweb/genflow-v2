#!/usr/bin/env bash
# =============================================================================
# GenFlow v2 — Database Backup Script
#
# Creates timestamped PostgreSQL backups and prunes old ones (retention: 14 days)
#
# Usage:
#   ./deploy/backup.sh              # Manual backup
#   ./deploy/backup.sh --cron       # Cron mode (quiet, stdout only on error)
#
# Cron: 0 3 * * * /opt/genflow/deploy/backup.sh --cron
# =============================================================================

set -euo pipefail

BACKUP_DIR="/opt/genflow/deploy/backups"
RETENTION_DAYS=14
DB_CONTAINER="genflow-db"
DB_USER="${DB_USER:-genflow}"
DB_NAME="${DB_NAME:-genflow}"
TIMESTAMP=$(date +%Y%m%d_%H%M%S)
BACKUP_FILE="${BACKUP_DIR}/genflow_${TIMESTAMP}.sql.gz"
LATEST_LINK="${BACKUP_DIR}/genflow_latest.sql.gz"

CRON_MODE=false
[ "${1:-}" = "--cron" ] && CRON_MODE=true

error_exit() {
    echo "[ERROR] $(date): $1" >&2
    exit 1
}

info() {
    $CRON_MODE && return 0
    echo "[INFO] $1"
}

# Ensure backup directory exists
mkdir -p "$BACKUP_DIR"

info "Starting backup: ${BACKUP_FILE}"

# Check if container is running
if ! docker ps --format "{{.Names}}" | grep -q "^${DB_CONTAINER}$"; then
    error_exit "Container ${DB_CONTAINER} is not running"
fi

# Run pg_dump inside the container
docker exec "$DB_CONTAINER" \
    pg_dump -U "$DB_USER" -d "$DB_NAME" \
    --clean \
    --if-exists \
    --no-owner \
    --no-acl \
    --verbose \
    2>/dev/null | gzip > "$BACKUP_FILE"

# Verify backup
if [ ! -s "$BACKUP_FILE" ]; then
    error_exit "Backup file is empty or missing"
fi

# Update latest symlink
ln -sf "$BACKUP_FILE" "$LATEST_LINK"

# Prune old backups
find "$BACKUP_DIR" -name "genflow_*.sql.gz" -mtime +${RETENTION_DAYS} -delete 2>/dev/null || true

# Report
BACKUP_SIZE=$(du -h "$BACKUP_FILE" | cut -f1)
REMAINING=$(find "$BACKUP_DIR" -name "genflow_*.sql.gz" | wc -l)
info "Backup complete: ${BACKUP_SIZE} (${REMAINING} backups retained)"
