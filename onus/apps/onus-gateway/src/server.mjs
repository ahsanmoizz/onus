import crypto from 'node:crypto';
import fs from 'node:fs';
import http from 'node:http';
import { URL } from 'node:url';
import pg from 'pg';

const { Pool } = pg;

const JSON_HEADERS = Object.freeze({
  'content-type': 'application/json; charset=utf-8',
  'cache-control': 'no-store',
});

export function loadDotEnv(filePath = '.env', env = process.env) {
  if (!fs.existsSync(filePath)) {
    return;
  }
  const contents = fs.readFileSync(filePath, 'utf8');
  for (const line of contents.split(/\r?\n/)) {
    const trimmed = line.trim();
    if (!trimmed || trimmed.startsWith('#')) {
      continue;
    }
    const assignment = trimmed.startsWith('export ') ? trimmed.slice(7).trim() : trimmed;
    const index = assignment.indexOf('=');
    if (index <= 0) {
      continue;
    }
    const key = assignment.slice(0, index).trim();
    if (!/^ONUS_[A-Z0-9_]+$|^NODE_ENV$/.test(key) || env[key] !== undefined) {
      continue;
    }
    env[key] = unquote(assignment.slice(index + 1).trim());
  }
}

export function loadConfig(env = process.env) {
  const databaseUrl = nonEmpty(env.ONUS_DATABASE_URL);
  return {
    host: nonEmpty(env.ONUS_GATEWAY_HOST) ?? '127.0.0.1',
    port: integer(env.ONUS_GATEWAY_PORT, 8080),
    providerEndpoint: nonEmpty(env.ONUS_PROVIDER_ENDPOINT),
    providerModel: nonEmpty(env.ONUS_PROVIDER_MODEL),
    providerApiKey: nonEmpty(env.ONUS_PROVIDER_API_KEY),
    providerTimeoutMs: integer(env.ONUS_PROVIDER_TIMEOUT_MS, 30_000),
    providerReferer: nonEmpty(env.ONUS_PROVIDER_REFERER),
    providerTitle: nonEmpty(env.ONUS_PROVIDER_TITLE) ?? 'Onus',
    jwtSecret: nonEmpty(env.ONUS_JWT_SECRET),
    adminToken: nonEmpty(env.ONUS_ADMIN_TOKEN),
    requireAuth: bool(env.ONUS_REQUIRE_AUTH, true),
    tokenTtlDays: integer(env.ONUS_TOKEN_TTL_DAYS, 30),
    dailyRequestLimit: integer(env.ONUS_DAILY_REQUEST_LIMIT, 500),
    databaseUrl,
    requireDatabase: bool(env.ONUS_REQUIRE_DATABASE, Boolean(databaseUrl)),
    allowedOrigins: list(env.ONUS_ALLOWED_ORIGINS),
    maxBodyBytes: integer(env.ONUS_MAX_BODY_BYTES, 65_536),
    maxResponseBytes: integer(env.ONUS_MAX_RESPONSE_BYTES, 262_144),
    allowPublicActivation: bool(env.ONUS_ALLOW_PUBLIC_ACTIVATION, false),
  };
}

export function createGatewayServer(config = loadConfig(), deps = {}) {
  const store = deps.store ?? new GatewayStore(config);
  const fetchImpl = deps.fetch ?? globalThis.fetch;
  if (!fetchImpl) {
    throw new Error('global fetch is unavailable; Node.js 20+ is required');
  }

  return http.createServer(async (req, res) => {
    const requestId = crypto.randomUUID();
    const startedAt = Date.now();
    try {
      applyCors(req, res, config);
      if (req.method === 'OPTIONS') {
        res.writeHead(204);
        res.end();
        return;
      }

      const url = new URL(req.url ?? '/', 'http://127.0.0.1');
      if (req.method === 'GET' && url.pathname === '/health') {
        sendJson(res, 200, { ok: true, service: 'onus-gateway', request_id: requestId });
        return;
      }
      if (req.method === 'GET' && url.pathname === '/ready') {
        await handleReady(res, config, store, requestId);
        return;
      }
      if (req.method === 'POST' && url.pathname === '/v1/tokens') {
        await handleTokenIssue(req, res, config, store, requestId);
        return;
      }
      if (req.method === 'POST' && url.pathname === '/v1/activate') {
        await handlePublicActivation(req, res, config, store, requestId);
        return;
      }
      if (req.method === 'POST' && url.pathname === '/v1/chat/completions') {
        await handleChat(req, res, config, store, fetchImpl, requestId);
        logInfo('chat_completion', requestId, startedAt, { status: res.statusCode });
        return;
      }
      sendJson(res, 404, { error: 'not_found', request_id: requestId });
    } catch (error) {
      const message = error instanceof HttpError ? error.message : 'gateway_error';
      const status = error instanceof HttpError ? error.status : 500;
      logWarn(message, requestId, startedAt, { status });
      sendJson(res, status, { error: message, request_id: requestId });
    }
  });
}

export async function startGateway(config = loadConfig()) {
  validateStartupConfig(config);
  const store = new GatewayStore(config);
  await store.init();
  const server = createGatewayServer(config, { store });
  return new Promise((resolve) => {
    server.listen(config.port, config.host, () => {
      console.log(
        JSON.stringify({
          level: 'info',
          event: 'gateway_started',
          host: config.host,
          port: config.port,
          auth: config.requireAuth,
          database: Boolean(config.databaseUrl),
        }),
      );
      resolve(server);
    });
  });
}

export function signGatewayToken(payload, secret) {
  const header = base64urlJson({ alg: 'HS256', typ: 'JWT' });
  const body = base64urlJson(payload);
  const signature = hmac(`${header}.${body}`, secret);
  return `${header}.${body}.${signature}`;
}

export function verifyGatewayToken(token, secret) {
  const parts = token.split('.');
  if (parts.length !== 3) {
    throw new HttpError(401, 'invalid_token');
  }
  const expected = hmac(`${parts[0]}.${parts[1]}`, secret);
  if (!timingSafeEqual(expected, parts[2])) {
    throw new HttpError(401, 'invalid_token');
  }
  const payload = JSON.parse(Buffer.from(parts[1], 'base64url').toString('utf8'));
  if (payload.aud !== 'onus-semantic-gateway') {
    throw new HttpError(401, 'invalid_token');
  }
  if (typeof payload.exp !== 'number' || payload.exp <= Math.floor(Date.now() / 1000)) {
    throw new HttpError(401, 'token_expired');
  }
  return payload;
}

export function hashToken(token) {
  return crypto.createHash('sha256').update(token).digest('hex');
}

export class GatewayStore {
  constructor(config) {
    this.config = config;
    this.pool = config.databaseUrl ? new Pool({ connectionString: config.databaseUrl }) : null;
  }

  async init() {
    if (!this.pool) {
      if (this.config.requireDatabase) {
        throw new Error('database_required');
      }
      return;
    }
    await this.pool.query(`
      create table if not exists gateway_tokens (
        token_hash text primary key,
        subject text not null,
        plan text not null,
        expires_at timestamptz not null,
        revoked_at timestamptz,
        created_at timestamptz not null default now()
      )
    `);
    await this.pool.query(`
      create table if not exists gateway_usage_events (
        id bigserial primary key,
        token_hash text not null,
        status integer not null,
        input_bytes integer not null,
        output_bytes integer not null,
        request_id uuid not null,
        created_at timestamptz not null default now()
      )
    `);
  }

  async ready() {
    if (!this.pool) {
      return !this.config.requireDatabase;
    }
    await this.pool.query('select 1');
    return true;
  }

  async createToken(tokenHash, { subject, plan, expiresAt }) {
    if (!this.pool) {
      if (this.config.requireDatabase) {
        throw new HttpError(503, 'database_required');
      }
      return;
    }
    await this.pool.query(
      `insert into gateway_tokens (token_hash, subject, plan, expires_at)
       values ($1, $2, $3, $4)
       on conflict (token_hash) do update
       set subject = excluded.subject,
           plan = excluded.plan,
           expires_at = excluded.expires_at,
           revoked_at = null`,
      [tokenHash, subject, plan, expiresAt],
    );
  }

  async validateToken(tokenHash) {
    if (!this.pool) {
      if (this.config.requireDatabase) {
        throw new HttpError(503, 'database_required');
      }
      return;
    }
    const result = await this.pool.query(
      `select expires_at, revoked_at
       from gateway_tokens
       where token_hash = $1`,
      [tokenHash],
    );
    if (result.rowCount !== 1) {
      throw new HttpError(401, 'unknown_token');
    }
    const row = result.rows[0];
    if (row.revoked_at) {
      throw new HttpError(401, 'token_revoked');
    }
    if (new Date(row.expires_at).getTime() <= Date.now()) {
      throw new HttpError(401, 'token_expired');
    }
  }

  async countUsageToday(tokenHash) {
    if (!this.pool) {
      return 0;
    }
    const result = await this.pool.query(
      `select count(*)::int as count
       from gateway_usage_events
       where token_hash = $1
         and created_at >= now() - interval '24 hours'`,
      [tokenHash],
    );
    return result.rows[0]?.count ?? 0;
  }

  async recordUsage({ tokenHash, status, inputBytes, outputBytes, requestId }) {
    if (!this.pool) {
      if (this.config.requireDatabase) {
        throw new HttpError(503, 'database_required');
      }
      return;
    }
    await this.pool.query(
      `insert into gateway_usage_events
       (token_hash, status, input_bytes, output_bytes, request_id)
       values ($1, $2, $3, $4, $5)`,
      [tokenHash, status, inputBytes, outputBytes, requestId],
    );
  }

  async close() {
    if (this.pool) {
      await this.pool.end();
    }
  }
}

class HttpError extends Error {
  constructor(status, message) {
    super(message);
    this.status = status;
  }
}

async function handleReady(res, config, store, requestId) {
  const missing = [];
  for (const [name, value] of [
    ['ONUS_PROVIDER_ENDPOINT', config.providerEndpoint],
    ['ONUS_PROVIDER_MODEL', config.providerModel],
    ['ONUS_PROVIDER_API_KEY', config.providerApiKey],
    ['ONUS_JWT_SECRET', config.jwtSecret],
  ]) {
    if (!value) {
      missing.push(name);
    }
  }
  let database = false;
  try {
    database = await store.ready();
  } catch {
    missing.push('ONUS_DATABASE_URL');
  }
  const ok = missing.length === 0 && database === true;
  sendJson(res, ok ? 200 : 503, {
    ok,
    service: 'onus-gateway',
    database,
    missing,
    request_id: requestId,
  });
}

async function handleTokenIssue(req, res, config, store, requestId) {
  requireAdmin(req, config);
  const body = await readJson(req, config.maxBodyBytes);
  const subject = text(body.subject, 'manual-user');
  const plan = text(body.plan, 'launch');
  const ttlDays = Math.min(Math.max(integer(body.ttl_days, config.tokenTtlDays), 1), 90);
  const issued = await issueToken(config, store, { subject, plan, ttlDays });
  sendJson(res, 201, { ...issued, request_id: requestId });
}

async function handlePublicActivation(req, res, config, store, requestId) {
  if (!config.allowPublicActivation) {
    throw new HttpError(403, 'public_activation_disabled');
  }
  const body = await readJson(req, config.maxBodyBytes);
  const subject = text(body.subject, 'anonymous');
  const issued = await issueToken(config, store, {
    subject,
    plan: 'public_activation',
    ttlDays: Math.min(config.tokenTtlDays, 14),
  });
  sendJson(res, 201, { ...issued, request_id: requestId });
}

async function issueToken(config, store, { subject, plan, ttlDays }) {
  if (!config.jwtSecret) {
    throw new HttpError(503, 'jwt_secret_not_configured');
  }
  const now = Math.floor(Date.now() / 1000);
  const exp = now + ttlDays * 24 * 60 * 60;
  const payload = {
    aud: 'onus-semantic-gateway',
    sub: subject.slice(0, 160),
    plan: plan.slice(0, 80),
    jti: crypto.randomUUID(),
    iat: now,
    exp,
  };
  const token = signGatewayToken(payload, config.jwtSecret);
  await store.createToken(hashToken(token), {
    subject: payload.sub,
    plan: payload.plan,
    expiresAt: new Date(exp * 1000),
  });
  return { token, expires_at: new Date(exp * 1000).toISOString() };
}

async function handleChat(req, res, config, store, fetchImpl, requestId) {
  const auth = await authorize(req, config, store);
  const rawBody = await readRaw(req, config.maxBodyBytes);
  let payload;
  try {
    payload = JSON.parse(rawBody);
  } catch {
    throw new HttpError(400, 'malformed_json');
  }
  validateProviderConfig(config);
  if (config.dailyRequestLimit > 0) {
    const used = await store.countUsageToday(auth.tokenHash);
    if (used >= config.dailyRequestLimit) {
      throw new HttpError(429, 'daily_request_limit_exceeded');
    }
  }

  const upstreamPayload = {
    ...payload,
    model: config.providerModel,
    stream: false,
  };
  const controller = new AbortController();
  const timeout = setTimeout(() => controller.abort(), config.providerTimeoutMs);
  let upstreamStatus = 502;
  let outputBytes = 0;
  try {
    const upstream = await fetchImpl(config.providerEndpoint, {
      method: 'POST',
      headers: providerHeaders(config),
      body: JSON.stringify(upstreamPayload),
      signal: controller.signal,
    });
    upstreamStatus = upstream.status;
    const textBody = await limitedResponseText(upstream, config.maxResponseBytes);
    outputBytes = Buffer.byteLength(textBody);
    if (!upstream.ok) {
      await store.recordUsage({
        tokenHash: auth.tokenHash,
        status: upstreamStatus,
        inputBytes: Buffer.byteLength(rawBody),
        outputBytes,
        requestId,
      });
      throw new HttpError(502, 'provider_request_failed');
    }
    await store.recordUsage({
      tokenHash: auth.tokenHash,
      status: upstreamStatus,
      inputBytes: Buffer.byteLength(rawBody),
      outputBytes,
      requestId,
    });
    res.writeHead(200, JSON_HEADERS);
    res.end(textBody);
  } catch (error) {
    clearTimeout(timeout);
    if (error.name === 'AbortError') {
      throw new HttpError(504, 'provider_timeout');
    }
    if (error instanceof HttpError) {
      throw error;
    }
    throw new HttpError(502, 'provider_request_failed');
  } finally {
    clearTimeout(timeout);
  }
}

async function authorize(req, config, store) {
  if (!config.requireAuth) {
    return { tokenHash: 'anonymous' };
  }
  if (!config.jwtSecret) {
    throw new HttpError(503, 'jwt_secret_not_configured');
  }
  const token = bearerToken(req);
  const payload = verifyGatewayToken(token, config.jwtSecret);
  const tokenHash = hashToken(token);
  await store.validateToken(tokenHash);
  return { tokenHash, payload };
}

function requireAdmin(req, config) {
  if (!config.adminToken) {
    throw new HttpError(503, 'admin_token_not_configured');
  }
  const provided = req.headers['x-onus-admin-token'] || bearerToken(req, false);
  if (typeof provided !== 'string' || !timingSafeEqual(provided, config.adminToken)) {
    throw new HttpError(403, 'admin_auth_required');
  }
}

function bearerToken(req, required = true) {
  const header = req.headers.authorization ?? '';
  const match = /^Bearer\s+(.+)$/i.exec(Array.isArray(header) ? header[0] : header);
  if (!match) {
    if (required) {
      throw new HttpError(401, 'authorization_required');
    }
    return undefined;
  }
  return match[1];
}

function providerHeaders(config) {
  const headers = {
    authorization: `Bearer ${config.providerApiKey}`,
    'content-type': 'application/json',
  };
  if (config.providerReferer) {
    headers['http-referer'] = config.providerReferer;
  }
  if (config.providerTitle) {
    headers['x-title'] = config.providerTitle;
  }
  return headers;
}

function validateStartupConfig(config) {
  validateProviderConfig(config);
  if (config.requireAuth && !config.jwtSecret) {
    throw new Error('ONUS_JWT_SECRET is required when ONUS_REQUIRE_AUTH=1');
  }
}

function validateProviderConfig(config) {
  if (!config.providerEndpoint || !config.providerModel || !config.providerApiKey) {
    throw new HttpError(503, 'provider_not_configured');
  }
}

async function readJson(req, limit) {
  const raw = await readRaw(req, limit);
  if (!raw.trim()) {
    return {};
  }
  try {
    return JSON.parse(raw);
  } catch {
    throw new HttpError(400, 'malformed_json');
  }
}

async function readRaw(req, limit) {
  const chunks = [];
  let total = 0;
  for await (const chunk of req) {
    total += chunk.length;
    if (total > limit) {
      throw new HttpError(413, 'request_too_large');
    }
    chunks.push(chunk);
  }
  return Buffer.concat(chunks).toString('utf8');
}

async function limitedResponseText(response, limit) {
  const buffer = Buffer.from(await response.arrayBuffer());
  if (buffer.length > limit) {
    throw new HttpError(502, 'provider_response_too_large');
  }
  return buffer.toString('utf8');
}

function sendJson(res, status, body) {
  res.writeHead(status, JSON_HEADERS);
  res.end(JSON.stringify(body));
}

function applyCors(req, res, config) {
  const origin = req.headers.origin;
  if (!origin) {
    return;
  }
  if (config.allowedOrigins.includes('*') || config.allowedOrigins.includes(origin)) {
    res.setHeader('access-control-allow-origin', origin);
    res.setHeader('vary', 'origin');
    res.setHeader('access-control-allow-methods', 'GET,POST,OPTIONS');
    res.setHeader('access-control-allow-headers', 'authorization,content-type,x-onus-admin-token');
  }
}

function logInfo(event, requestId, startedAt, extra = {}) {
  console.log(JSON.stringify({ level: 'info', event, request_id: requestId, ms: Date.now() - startedAt, ...extra }));
}

function logWarn(event, requestId, startedAt, extra = {}) {
  console.warn(JSON.stringify({ level: 'warn', event, request_id: requestId, ms: Date.now() - startedAt, ...extra }));
}

function base64urlJson(value) {
  return Buffer.from(JSON.stringify(value)).toString('base64url');
}

function hmac(input, secret) {
  return crypto.createHmac('sha256', secret).update(input).digest('base64url');
}

function timingSafeEqual(a, b) {
  const left = Buffer.from(a);
  const right = Buffer.from(b);
  return left.length === right.length && crypto.timingSafeEqual(left, right);
}

function unquote(value) {
  if (
    (value.startsWith('"') && value.endsWith('"')) ||
    (value.startsWith("'") && value.endsWith("'"))
  ) {
    return value.slice(1, -1);
  }
  return value;
}

function nonEmpty(value) {
  return typeof value === 'string' && value.trim() ? value.trim() : undefined;
}

function bool(value, fallback) {
  if (value === undefined || value === null || value === '') {
    return fallback;
  }
  return !['0', 'false', 'no', 'off'].includes(String(value).toLowerCase());
}

function integer(value, fallback) {
  const parsed = Number.parseInt(String(value ?? ''), 10);
  return Number.isFinite(parsed) ? parsed : fallback;
}

function list(value) {
  return String(value ?? '')
    .split(',')
    .map((item) => item.trim())
    .filter(Boolean);
}

function text(value, fallback) {
  return typeof value === 'string' && value.trim() ? value.trim() : fallback;
}

if (import.meta.url === `file://${process.argv[1]}`) {
  loadDotEnv();
  const config = loadConfig();
  startGateway(config).catch((error) => {
    console.error(JSON.stringify({ level: 'error', event: 'gateway_start_failed', error: error.message }));
    process.exit(1);
  });
}
