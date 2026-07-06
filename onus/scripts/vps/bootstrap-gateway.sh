#!/usr/bin/env bash
set -euo pipefail

if [ "$(id -u)" -ne 0 ]; then
  echo "Run as root on the VPS." >&2
  exit 1
fi

REMOTE_DIR="${ONUS_REMOTE_DIR:-/opt/onus-gateway}"
DB_NAME="${ONUS_DB_NAME:-onus}"
DB_USER="${ONUS_DB_USER:-onus_user}"
DB_PASSWORD="${ONUS_DB_PASSWORD:-}"

apt-get update
apt-get install -y ca-certificates curl gnupg tar nginx postgresql postgresql-contrib openssl

NODE_MAJOR="$(node -v 2>/dev/null | sed -E 's/^v([0-9]+).*/\1/' || true)"
if [ -z "$NODE_MAJOR" ] || [ "$NODE_MAJOR" -lt 20 ]; then
  curl -fsSL https://deb.nodesource.com/setup_20.x | bash -
  apt-get install -y nodejs
fi

id onus >/dev/null 2>&1 || useradd --system --home "$REMOTE_DIR" --shell /usr/sbin/nologin onus
mkdir -p "$REMOTE_DIR"
chown -R onus:onus "$REMOTE_DIR"

if [ -n "$DB_PASSWORD" ]; then
  DB_USER_ESCAPED="$(printf "%s" "$DB_USER" | sed "s/'/''/g")"
  DB_NAME_ESCAPED="$(printf "%s" "$DB_NAME" | sed "s/'/''/g")"
  DB_PASSWORD_ESCAPED="$(printf "%s" "$DB_PASSWORD" | sed "s/'/''/g")"
  sudo -u postgres psql <<SQL
DO \$\$
BEGIN
  IF NOT EXISTS (SELECT 1 FROM pg_roles WHERE rolname = '$DB_USER_ESCAPED') THEN
    EXECUTE format('CREATE USER %I WITH PASSWORD %L', '$DB_USER_ESCAPED', '$DB_PASSWORD_ESCAPED');
  ELSE
    EXECUTE format('ALTER USER %I WITH PASSWORD %L', '$DB_USER_ESCAPED', '$DB_PASSWORD_ESCAPED');
  END IF;
END
\$\$;
SELECT format('CREATE DATABASE %I OWNER %I', '$DB_NAME_ESCAPED', '$DB_USER_ESCAPED')
WHERE NOT EXISTS (SELECT 1 FROM pg_database WHERE datname = '$DB_NAME_ESCAPED')
\gexec
SQL
else
  echo "ONUS_DB_PASSWORD was not set; Postgres installed, but gateway DB/user were not created." >&2
fi

echo "bootstrap complete"
echo "node: $(node -v)"
echo "npm: $(npm -v)"
echo "remote_dir: $REMOTE_DIR"
