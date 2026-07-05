import assert from 'node:assert/strict';
import http from 'node:http';
import { after, before, describe, it } from 'node:test';
import {
  createGatewayServer,
  hashToken,
  loadConfig,
  signGatewayToken,
} from '../src/server.mjs';

function listen(server) {
  return new Promise((resolve) => {
    server.listen(0, '127.0.0.1', () => {
      const address = server.address();
      resolve(`http://127.0.0.1:${address.port}`);
    });
  });
}

function close(server) {
  return new Promise((resolve, reject) => {
    server.close((error) => (error ? reject(error) : resolve()));
  });
}

describe('Onus managed semantic gateway', () => {
  let provider;
  let providerUrl;
  let upstreamAuth;
  let upstreamModel;
  let upstreamBody;

  before(async () => {
    provider = http.createServer((req, res) => {
      const chunks = [];
      req.on('data', (chunk) => chunks.push(chunk));
      req.on('end', () => {
        upstreamAuth = req.headers.authorization;
        upstreamBody = Buffer.concat(chunks).toString('utf8');
        upstreamModel = JSON.parse(upstreamBody).model;
        res.writeHead(200, { 'content-type': 'application/json' });
        res.end(
          JSON.stringify({
            choices: [
              {
                message: {
                  content: JSON.stringify({
                    schema_version: 1,
                    aligned_with_task: true,
                    proportionate: true,
                    quality_problems: [],
                    hidden_side_effects: [],
                    confidence: 0.8,
                    recommended_decision: 'allow',
                    rationale: 'fixture provider response',
                  }),
                },
              },
            ],
          }),
        );
      });
    });
    providerUrl = await listen(provider);
  });

  after(async () => {
    await close(provider);
  });

  function config(overrides = {}) {
    return loadConfig({
      ONUS_GATEWAY_HOST: '127.0.0.1',
      ONUS_GATEWAY_PORT: '0',
      ONUS_PROVIDER_ENDPOINT: providerUrl,
      ONUS_PROVIDER_MODEL: 'server-forced-model',
      ONUS_PROVIDER_API_KEY: 'provider-secret',
      ONUS_JWT_SECRET: 'test-jwt-secret-with-enough-length',
      ONUS_ADMIN_TOKEN: 'admin-secret',
      ONUS_REQUIRE_AUTH: '1',
      ONUS_REQUIRE_DATABASE: '0',
      ONUS_MAX_BODY_BYTES: '2048',
      ONUS_MAX_RESPONSE_BYTES: '4096',
      ONUS_ALLOWED_ORIGINS: 'http://127.0.0.1:3000',
      ...overrides,
    });
  }

  function token(jwtSecret = 'test-jwt-secret-with-enough-length') {
    const now = Math.floor(Date.now() / 1000);
    return signGatewayToken(
      {
        aud: 'onus-semantic-gateway',
        sub: 'test-user',
        plan: 'test',
        jti: 'test-jti',
        iat: now,
        exp: now + 3600,
      },
      jwtSecret,
    );
  }

  it('reports health without exposing configuration', async () => {
    const server = createGatewayServer(config());
    const base = await listen(server);
    const response = await fetch(`${base}/health`);
    const body = await response.json();
    assert.equal(response.status, 200);
    assert.equal(body.ok, true);
    assert.equal(JSON.stringify(body).includes('provider-secret'), false);
    await close(server);
  });

  it('requires a valid Onus client token before forwarding model requests', async () => {
    const server = createGatewayServer(config());
    const base = await listen(server);
    const response = await fetch(`${base}/v1/chat/completions`, {
      method: 'POST',
      headers: { 'content-type': 'application/json' },
      body: JSON.stringify({ messages: [{ role: 'user', content: 'hello' }] }),
    });
    assert.equal(response.status, 401);
    await close(server);
  });

  it('issues client tokens only with the admin token', async () => {
    const issuedHashes = [];
    const store = {
      createToken: async (tokenHash) => issuedHashes.push(tokenHash),
      validateToken: async () => {},
      countUsageToday: async () => 0,
      recordUsage: async () => {},
      ready: async () => true,
    };
    const server = createGatewayServer(config(), { store });
    const base = await listen(server);
    const denied = await fetch(`${base}/v1/tokens`, { method: 'POST' });
    assert.equal(denied.status, 403);

    const accepted = await fetch(`${base}/v1/tokens`, {
      method: 'POST',
      headers: {
        authorization: 'Bearer admin-secret',
        'content-type': 'application/json',
      },
      body: JSON.stringify({ subject: 'user@example.com', ttl_days: 1 }),
    });
    const body = await accepted.json();
    assert.equal(accepted.status, 201);
    assert.equal(typeof body.token, 'string');
    assert.equal(issuedHashes[0], hashToken(body.token));
    await close(server);
  });

  it('forwards chat completions with only the server-side provider key', async () => {
    const clientToken = token();
    const store = {
      validateToken: async (tokenHash) => {
        assert.equal(tokenHash, hashToken(clientToken));
      },
      countUsageToday: async () => 0,
      recordUsage: async () => {},
      ready: async () => true,
    };
    const server = createGatewayServer(config(), { store });
    const base = await listen(server);
    const response = await fetch(`${base}/v1/chat/completions`, {
      method: 'POST',
      headers: {
        authorization: `Bearer ${clientToken}`,
        'content-type': 'application/json',
      },
      body: JSON.stringify({
        model: 'client-requested-model',
        messages: [{ role: 'user', content: 'review this' }],
      }),
    });
    const body = await response.json();
    assert.equal(response.status, 200);
    assert.equal(upstreamAuth, 'Bearer provider-secret');
    assert.equal(upstreamModel, 'server-forced-model');
    assert.equal(upstreamBody.includes(clientToken), false);
    assert.equal(body.choices[0].message.content.includes('schema_version'), true);
    await close(server);
  });

  it('rejects oversized request bodies before provider forwarding', async () => {
    const server = createGatewayServer(config({ ONUS_MAX_BODY_BYTES: '24' }));
    const base = await listen(server);
    const response = await fetch(`${base}/v1/chat/completions`, {
      method: 'POST',
      headers: {
        authorization: `Bearer ${token()}`,
        'content-type': 'application/json',
      },
      body: JSON.stringify({ messages: [{ role: 'user', content: 'x'.repeat(200) }] }),
    });
    assert.equal(response.status, 413);
    await close(server);
  });
});
