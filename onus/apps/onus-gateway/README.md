# Onus Managed Semantic Gateway

This service lets Onus clients use semantic review without receiving the raw provider key. The VPS owns `ONUS_PROVIDER_API_KEY`; installed clients receive only an Onus-issued short-lived token.

## Runtime

```bash
npm ci --omit=dev
cp .env.example .env
nano .env
npm start
```

Do not put `.env` in git.

## Required VPS Environment

```bash
ONUS_PROVIDER_ENDPOINT=https://openrouter.ai/api/v1/chat/completions
ONUS_PROVIDER_MODEL=qwen/qwen3-coder:free
ONUS_PROVIDER_API_KEY=...
ONUS_JWT_SECRET=...
ONUS_ADMIN_TOKEN=...
ONUS_DATABASE_URL=postgresql://...
ONUS_REQUIRE_DATABASE=1
```

Rotate any value that was pasted into a chat or terminal transcript before real production.

## Issue a Client Token

```bash
curl -sS -X POST "https://YOUR-GATEWAY/v1/tokens" \
  -H "authorization: Bearer $ONUS_ADMIN_TOKEN" \
  -H "content-type: application/json" \
  -d '{"subject":"first-user","plan":"launch","ttl_days":30}'
```

The returned token is what a client puts in `ONUS_SEMANTIC_API_KEY`. It is not the provider key.

## Client Config

```bash
ONUS_SEMANTIC_PROVIDER=cloud
ONUS_SEMANTIC_ENDPOINT=https://YOUR-GATEWAY/v1/chat/completions
ONUS_SEMANTIC_MODEL=onus-managed
ONUS_SEMANTIC_API_KEY=CLIENT_TOKEN_FROM_GATEWAY
ONUS_SEMANTIC_FALLBACK=fail_closed
ONUS_SEMANTIC_FAIL_CLOSED_CRITICAL=1
ONUS_SEMANTIC_PRIVACY_MODE=strict
ONUS_SEMANTIC_REDACT=1
```

## Health

```bash
curl -sS http://127.0.0.1:8080/health
curl -sS http://127.0.0.1:8080/ready
```

## Important Limits

The gateway is not an execution boundary. It only brokers semantic-review model calls. Deterministic Onus policy remains authoritative, and deterministic denial cannot be overridden by model output.
