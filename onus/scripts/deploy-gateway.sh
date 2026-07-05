#!/usr/bin/env bash
set -euo pipefail

if [ "${1:-}" = "" ]; then
  echo "Usage: $0 user@server [/opt/onus-gateway] [./.env]"
  exit 1
fi

REMOTE="$1"
REMOTE_DIR="${2:-/opt/onus-gateway}"
ENV_FILE="${3:-}"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../.." && pwd)"
ARCHIVE="/tmp/onus-gateway.tar.gz"

tar -czf "${ARCHIVE}" -C "${REPO_ROOT}/onus/apps" onus-gateway
scp "${ARCHIVE}" "${REMOTE}:/tmp/onus-gateway.tar.gz"
ssh "${REMOTE}" "mkdir -p '${REMOTE_DIR}' && tar -xzf /tmp/onus-gateway.tar.gz -C '${REMOTE_DIR}' --strip-components=1 && cd '${REMOTE_DIR}' && npm ci --omit=dev"

if [ -n "${ENV_FILE}" ]; then
  scp "${ENV_FILE}" "${REMOTE}:${REMOTE_DIR}/.env"
  ssh "${REMOTE}" "chmod 600 '${REMOTE_DIR}/.env'"
fi

cat <<EOF
Gateway uploaded to ${REMOTE}:${REMOTE_DIR}

On the VPS:
  sudo useradd --system --home ${REMOTE_DIR} --shell /usr/sbin/nologin onus || true
  sudo chown -R onus:onus ${REMOTE_DIR}
  sudo cp ${REMOTE_DIR}/systemd/onus-gateway.service /etc/systemd/system/onus-gateway.service
  sudo systemctl daemon-reload
  sudo systemctl enable --now onus-gateway
EOF
