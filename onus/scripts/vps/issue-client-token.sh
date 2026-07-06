#!/usr/bin/env bash
set -euo pipefail

SUBJECT="${1:-first-user}"
GATEWAY_URL="${2:-${ONUS_GATEWAY_PUBLIC_URL:-http://127.0.0.1:8080}}"
ENV_FILE="${ONUS_GATEWAY_ENV_FILE:-/opt/onus-gateway/.env}"

if [ -f "$ENV_FILE" ]; then
  set -a
  # shellcheck disable=SC1090
  . "$ENV_FILE"
  set +a
fi

if [ -z "${ONUS_ADMIN_TOKEN:-}" ]; then
  echo "ONUS_ADMIN_TOKEN is not set and was not found in $ENV_FILE." >&2
  exit 1
fi

curl -sS -X POST "${GATEWAY_URL%/}/v1/tokens" \
  -H "authorization: Bearer ${ONUS_ADMIN_TOKEN}" \
  -H "content-type: application/json" \
  -d "{\"subject\":\"${SUBJECT}\",\"plan\":\"launch\",\"ttl_days\":30}"
echo
