#!/usr/bin/env bash
set -euo pipefail

REMOTE_DIR="${ONUS_REMOTE_DIR:-/opt/onus-gateway}"
ENV_FILE="${ONUS_GATEWAY_ENV_FILE:-$REMOTE_DIR/.env}"
PROVIDER_ENDPOINT="${ONUS_PROVIDER_ENDPOINT:-https://openrouter.ai/api/v1/chat/completions}"
PROVIDER_MODEL="${ONUS_PROVIDER_MODEL:-qwen/qwen3-coder:free}"
PROVIDER_KEY="${ONUS_PROVIDER_API_KEY:-}"
GATEWAY_PUBLIC_URL="${ONUS_GATEWAY_PUBLIC_URL:-}"
SITE_ORIGIN="${ONUS_SITE_ORIGIN:-https://ahsanmoizz.github.io}"
DB_NAME="${ONUS_DB_NAME:-onus}"
DB_USER="${ONUS_DB_USER:-onus_user}"
DB_PASSWORD="${ONUS_DB_PASSWORD:-}"
DATABASE_URL="${ONUS_DATABASE_URL:-}"
JWT_SECRET="${ONUS_JWT_SECRET:-$(openssl rand -hex 48)}"
ADMIN_TOKEN="${ONUS_ADMIN_TOKEN:-$(openssl rand -hex 48)}"

if [ -z "$PROVIDER_KEY" ]; then
  echo "Set ONUS_PROVIDER_API_KEY before running this script." >&2
  exit 1
fi

if [ -z "$GATEWAY_PUBLIC_URL" ]; then
  echo "Set ONUS_GATEWAY_PUBLIC_URL, for example https://api.example.com." >&2
  exit 1
fi

if [ -z "$DATABASE_URL" ]; then
  if [ -z "$DB_PASSWORD" ]; then
    echo "Set ONUS_DB_PASSWORD or ONUS_DATABASE_URL before running this script." >&2
    exit 1
  fi
  DATABASE_URL="postgresql://${DB_USER}:${DB_PASSWORD}@127.0.0.1:5432/${DB_NAME}"
fi

mkdir -p "$REMOTE_DIR"
umask 077
cat > "$ENV_FILE" <<EOF
NODE_ENV=production
ONUS_GATEWAY_HOST=127.0.0.1
ONUS_GATEWAY_PORT=8080

ONUS_PROVIDER_ENDPOINT=$PROVIDER_ENDPOINT
ONUS_PROVIDER_MODEL=$PROVIDER_MODEL
ONUS_PROVIDER_API_KEY=$PROVIDER_KEY
ONUS_PROVIDER_TIMEOUT_MS=30000
ONUS_PROVIDER_REFERER=$SITE_ORIGIN/onus
ONUS_PROVIDER_TITLE=Onus

ONUS_REQUIRE_AUTH=1
ONUS_JWT_SECRET=$JWT_SECRET
ONUS_ADMIN_TOKEN=$ADMIN_TOKEN
ONUS_TOKEN_TTL_DAYS=30
ONUS_DAILY_REQUEST_LIMIT=500

ONUS_DATABASE_URL=$DATABASE_URL
ONUS_REQUIRE_DATABASE=1

ONUS_ALLOWED_ORIGINS=$SITE_ORIGIN,$SITE_ORIGIN/onus,http://127.0.0.1:3000
ONUS_MAX_BODY_BYTES=65536
ONUS_MAX_RESPONSE_BYTES=262144
ONUS_ALLOW_PUBLIC_ACTIVATION=0
EOF

chmod 600 "$ENV_FILE"
chown onus:onus "$ENV_FILE" 2>/dev/null || true

echo "wrote $ENV_FILE"
echo "gateway_public_url=$GATEWAY_PUBLIC_URL"
echo "admin token and provider key were written to the env file and were not printed"
