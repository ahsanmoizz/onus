#!/usr/bin/env bash
set -euo pipefail

GATEWAY_URL="${1:-${ONUS_GATEWAY_PUBLIC_URL:-http://127.0.0.1:8080}}"

health="$(curl -fsS "${GATEWAY_URL%/}/health")"
echo "health: ok"
echo "$health" | grep -q '"ok":true'

ready="$(curl -fsS "${GATEWAY_URL%/}/ready")"
echo "ready: ok"
echo "$ready" | grep -q '"ok":true'

status="$(
  curl -sS -o /tmp/onus-gateway-auth-check.json -w "%{http_code}" \
    -X POST "${GATEWAY_URL%/}/v1/chat/completions" \
    -H "content-type: application/json" \
    -d '{"messages":[{"role":"user","content":"ping"}]}'
)"

if [ "$status" != "401" ]; then
  echo "auth guard failed: expected 401 without client token, got $status" >&2
  cat /tmp/onus-gateway-auth-check.json >&2
  exit 1
fi

echo "auth guard: ok"
