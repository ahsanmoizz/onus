# REAL Frontend Inventory

> Generated: 2026-06-19
> Repository: D:\Onus\onus
> Apps: `apps/onus-console/` (Product Console), `apps/onus-site/` (Public Website), `site/` (Legacy)

---

## Table of Contents

1. [onus-console (Product Console)](#1-onus-console-product-console)
   - [Configuration](#11-configuration)
   - [Source Files](#12-source-files)
   - [Route Map](#13-route-map)
   - [API Endpoints Called](#14-api-endpoints-called)
   - [Middleware](#15-middleware)
   - [Mock/Fake/Simulated Patterns](#16-mockfakesimulated-patterns)
   - [UI Components](#17-ui-components)
2. [onus-site (Public Website)](#2-onus-site-public-website)
   - [Configuration](#21-configuration)
   - [Source Files](#22-source-files)
   - [Route Map](#23-route-map)
   - [API Endpoints Called](#24-api-endpoints-called)
3. [Legacy Site (site/index.html)](#3-legacy-site-siteindexhtml)
4. [Tests, Lint, Typecheck, Build](#4-tests-lint-typecheck-build)
5. [Security Findings](#5-security-findings)

---

## 1. onus-console (Product Console)

**Location:** `D:\Onus\onus\apps\onus-console`
**Framework:** Next.js 14 (App Router), React 18, TypeScript 5.4
**UI:** Tailwind CSS v4, lucide-react, clsx + tailwind-merge, class-variance-authority
**Data Fetching:** @tanstack/react-query v5
**Dev Port:** 3001
**BFF Proxy:** `/api/*` -> `http://127.0.0.1:9090/api/*` via `next.config.js rewrites()`

### 1.1 Configuration

| File | Purpose |
|---|---|
| `package.json` | Dependencies: next@14.2, react@18, @tanstack/react-query@5, lucide-react, clsx, tailwind-merge, class-variance-authority. Dev: typescript@5.4, tailwindcss@4, @tailwindcss/postcss, eslint@8, eslint-config-next. Scripts: `dev` (port 3001), `build`, `start`, `lint`. |
| `next.config.js` | `rewrites()` proxies `/api/:path*` -> `http://127.0.0.1:9090/api/:path*`. `transpilePackages` includes `@onus/ui`, `@onus/api-client`, `@onus/types`. |
| `tsconfig.json` | Path aliases: `@/*` -> `src/*`, `@onus/ui/*` -> `../../packages/ui/*`, `@onus/api-client` -> `../../packages/api-client/src`, `@onus/types` -> `../../packages/types/src`. |
| `.eslintrc.json` | Extends `next/core-web-vitals`. |
| `postcss.config.js` | Uses `@tailwindcss/postcss` plugin. |

### 1.2 Source Files

Total: **29 source files** (excluding node_modules, .next, public)

```
src/
  middleware.ts
  lib/
    api.ts
    utils.ts
  app/
    globals.css
    layout.tsx
    page.tsx
    providers.tsx
    activate/page.tsx
    sessions/page.tsx
    approvals/page.tsx
    actions/page.tsx
    intake/page.tsx
    audit/page.tsx
    checkpoints/page.tsx
    rollback/page.tsx
    memory/page.tsx
    rules/page.tsx
    workspaces/page.tsx
    authority/page.tsx
    integrations/page.tsx
    agents/page.tsx
    providers/page.tsx
    doctor/page.tsx
    settings/page.tsx
  components/
    dashboard-layout.tsx
    status-card.tsx
    stat-card.tsx
    activity-feed.tsx
```

### 1.3 Route Map

| Route | File | Description | API Calls | Polling |
|---|---|---|---|---|
| `/` | `page.tsx` | Dashboard homepage — 4 stat cards, 4 status cards, activity feed | `getStatus()`, `getRecentActions()` | 5s status, 3s actions |
| `/activate` | `activate/page.tsx` | Multi-step activation wizard (8 steps) | **Simulated** — no real API | None |
| `/sessions` | `sessions/page.tsx` | Session list table | `GET /sessions` | None |
| `/approvals` | `approvals/page.tsx` | Approval list with approve/deny | `GET /approvals`, `POST /approvals/:id/approve`, `POST /approvals/:id/deny` | None |
| `/actions` | `actions/page.tsx` | Action stream (50-item limit) | `GET /actions?limit=50` | 3s |
| `/intake` | `intake/page.tsx` | Prompt submission form | `POST /intake` | None |
| `/audit` | `audit/page.tsx` | Action list + chain verification | `GET /actions?limit=100`, `GET /verify?session=` | 5s |
| `/checkpoints` | `checkpoints/page.tsx` | List/create/restore checkpoints | `GET /checkpoints`, `POST /checkpoints`, `POST /checkpoints/restore` | None |
| `/rollback` | `rollback/page.tsx` | Rollback form (action/group/session) | `POST /rollback/{mode}` | None |
| `/memory` | `memory/page.tsx` | Memory list with delete | `GET /memory`, `DELETE /memory/:id` | None |
| `/rules` | `rules/page.tsx` | Rules table | `GET /rules` | None |
| `/workspaces` | `workspaces/page.tsx` | Create/destroy workspaces | `GET /workspaces`, `POST /workspaces`, `DELETE /workspaces/:id` | None |
| `/authority` | `authority/page.tsx` | L4 capabilities list + revocation | `GET /authority`, `POST /authority/revoke` | None |
| `/integrations` | `integrations/page.tsx` | Integration status cards | `GET /integrations` | None |
| `/agents` | `agents/page.tsx` | Agent list + handoff indicators | `GET /agents` | None |
| `/providers` | `providers/page.tsx` | Provider list with configure/connect | `GET /providers` | None |
| `/doctor` | `doctor/page.tsx` | Doctor results with retry | `GET /doctor` (via `runDoctor()`) | None |
| `/settings` | `settings/page.tsx` | Settings form (guardian mode, etc.) | `GET /settings`, `PUT /settings` | None |

### 1.4 API Endpoints Called

All calls go through the BFF proxy (`/api/*` -> `http://127.0.0.1:9090/api/*`). The `src/lib/api.ts` client wraps `fetch()` with automatic CSRF token injection on state-changing methods.

| Endpoint | Method | Used By | Purpose |
|---|---|---|---|
| `/api/status` | GET | Dashboard | Daemon health + guardian mode |
| `/api/sessions` | GET | Sessions page | List sessions |
| `/api/actions?limit=N` | GET | Dashboard, Actions, Audit | Recent actions stream |
| `/api/approvals?status=` | GET | Approvals | Filtered approval list |
| `/api/approvals/:id/approve` | POST | Approvals | Approve a pending action |
| `/api/approvals/:id/deny` | POST | Approvals | Deny a pending action |
| `/api/checkpoints` | GET | Checkpoints | List checkpoints |
| `/api/checkpoints` | POST | Checkpoints | Create checkpoint |
| `/api/checkpoints/restore` | POST | Checkpoints | Restore checkpoint |
| `/api/verify?session=` | GET | Audit | Verify receipt chain |
| `/api/memory` | GET | Memory | List memory entries |
| `/api/memory/:id` | DELETE | Memory | Delete memory entry |
| `/api/rules` | GET | Rules | List rules |
| `/api/workspaces` | GET | Workspaces | List workspaces |
| `/api/workspaces` | POST | Workspaces | Create workspace |
| `/api/workspaces/:id` | DELETE | Workspaces | Destroy workspace |
| `/api/authority` | GET | Authority | List L4 authorities |
| `/api/authority/revoke` | POST | Authority | Revoke authority |
| `/api/integrations` | GET | Integrations | List integration statuses |
| `/api/agents` | GET | Agents | List agents |
| `/api/providers` | GET | Providers | List providers |
| `/api/doctor` | GET | Doctor | Run system diagnostics |
| `/api/settings` | GET | Settings | Get current settings |
| `/api/settings` | PUT | Settings | Update settings |
| `/api/intake` | POST | Intake | Submit prompt for evaluation |

### 1.5 Middleware

**File:** `src/middleware.ts`

The middleware runs on all routes (`matcher: "/:path*"`) and performs:

1. **Origin validation** — Checks `Origin` / `Referer` headers against `ALLOWED_ORIGINS` list (`localhost:3001`, `127.0.0.1:3001`, `127.0.0.1:3001`). Returns 403 for disallowed origins.
2. **CSRF protection** — For POST/PUT/PATCH/DELETE requests, validates that `X-CSRF-Token` header is present and equals `'1'`. Returns 403 for missing/invalid token.
3. **Security headers** — Sets `X-Content-Type-Options: nosniff`, `X-Frame-Options: DENY`, `X-XSS-Protection: 1; mode=block`, `Referrer-Policy: strict-origin-when-cross-origin`, `Permissions-Policy` (restrictive), `Strict-Transport-Security: max-age=31536000; includeSubDomains` (only in production), `Content-Security-Policy` (restrictive: default-src 'self', connect-src 'self' https://api.github.com, script-src 'self' 'unsafe-eval', style-src 'self' 'unsafe-inline').

### 1.6 Mock/Fake/Simulated Patterns

**The `/activate` page (`src/app/activate/page.tsx`) contains simulated validation steps that do not call real API endpoints.**

Specifically:

1. **Provider test (`testProvider` function, lines ~385-390):**
   ```typescript
   const testProvider = async () => {
     await new Promise(r => setTimeout(r, 1500));
     setProviderTested(true);
   };
   ```
   Does not actually call a provider API. Uses a hardcoded 1500ms delay and unconditionally sets `providerTested = true`.

2. **Daemon validation (`runValidation` function, lines ~450-550):**
   ```typescript
   await new Promise(r => setTimeout(r, 800));
   ```
   Iterates through predefined checks with hardcoded pass/fail messages:
   - "Daemon responding on port 9090" — hardcoded success
   - "Connection verified" — hardcoded success
   - "Default policies installed" — hardcoded success
   - "API reachable" — hardcoded success

   These do not actually call the daemon or check real system state. The validation step runs local checks with pre-determined outcomes.

**No other page in the console uses simulated data.** All other pages use real `fetchApi()` calls to the BFF proxy. There are no test mocks in production paths.

### 1.7 UI Components

| Component | File | Usage |
|---|---|---|
| `DashboardLayout` | `components/dashboard-layout.tsx` | Sidebar nav (17 items), responsive collapse, active state highlighting. Wraps all pages. |
| `StatCard` | `components/stat-card.tsx` | Metric card with title, value, icon, status indicator (active/inactive). Used on Dashboard. |
| `StatusCard` | `components/status-card.tsx` | Colored info card with success/warning/error/info variants. Used on Dashboard. |
| `ActivityFeed` | `components/activity-feed.tsx` | Recent actions list with verdict icons and color coding. 3-sec polling. Used on Dashboard. |

---

## 2. onus-site (Public Website)

**Location:** `D:\Onus\onus\apps\onus-site`
**Framework:** Next.js 14 (App Router), React 18, TypeScript 5.4
**UI:** Tailwind CSS v4, framer-motion v12, lucide-react, clsx + tailwind-merge
**Data Fetching:** None (fully static)
**Dev Port:** 3000
**Build Output:** Static export (`output: 'export'`)

### 2.1 Configuration

| File | Purpose |
|---|---|
| `package.json` | Dependencies: next@14.2, react@18, framer-motion@12, lucide-react, clsx, tailwind-merge. No react-query. Scripts: `dev`, `build`, `start`, `lint`. |
| `next.config.js` | `output: 'export'` for static site generation. `transpilePackages` includes `@onus/ui` only. No rewrites/proxy. |
| `tsconfig.json` | Path aliases: `@/*` -> `src/*`, `@onus/ui/*` -> `../../packages/ui/*`. **No** `@onus/api-client` or `@onus/types`. |
| `.eslintrc.json` | Extends `next/core-web-vitals`. |
| `postcss.config.js` | Uses `@tailwindcss/postcss` plugin. |

### 2.2 Source Files

Total: **31 source files** (excluding node_modules, .next, public)

```
src/
  lib/
    utils.ts
  app/
    globals.css
    layout.tsx
    page.tsx
    install/page.tsx
    product/page.tsx
    docs/page.tsx
    security/page.tsx
    changelog/page.tsx
    limitations/page.tsx
    docs/
      introduction/page.tsx
      installation/page.tsx
      quick-start/page.tsx
      guardian-modes/page.tsx
      providers/page.tsx
      prompt-intake/page.tsx
      task-contracts/page.tsx
      running-governed-tasks/page.tsx
      approvals/page.tsx
      checkpoint-rollback/page.tsx
      memory/page.tsx
      rules-policies/page.tsx
      mcp-l2/page.tsx
      l3-workspaces/page.tsx
      l4-authority/page.tsx
      receipts-audit/page.tsx
      integrations/page.tsx
      agent-handoff/page.tsx
      cli-reference/page.tsx
      troubleshooting/page.tsx
      security-model/page.tsx
      limitations/page.tsx
```

### 2.3 Route Map

| Route | File | Content | API Calls |
|---|---|---|---|
| `/` | `page.tsx` | Marketing homepage — Navbar, Hero (framer-motion animated), Problem, Architecture (L1-L4 tiers), Integrations, CTA, Footer | None |
| `/install` | `install/page.tsx` | Platform install instructions (Windows PowerShell, Linux/macOS, prerequisites, quick start) | None |
| `/product` | `product/page.tsx` | 10 feature cards with descriptions | None |
| `/docs` | `docs/page.tsx` | Docs catalog — 6 categories, 22 linked sub-pages | None |
| `/security` | `security/page.tsx` | Security model — protected assets, trust boundaries, threat coverage, L1-L4, residual risks, crypto guarantees | None |
| `/changelog` | `changelog/page.tsx` | v0.1.0 release notes (25+ changes, 5 commits) | None |
| `/limitations` | `limitations/page.tsx` | 6 limitation categories with status badges | None |
| `/docs/introduction` | `docs/introduction/page.tsx` | Product introduction | None |
| `/docs/installation` | `docs/installation/page.tsx` | Installation guide | None |
| `/docs/quick-start` | `docs/quick-start/page.tsx` | Quick start tutorial | None |
| `/docs/guardian-modes` | `docs/guardian-modes/page.tsx` | Guardian mode documentation | None |
| `/docs/providers` | `docs/providers/page.tsx` | Provider documentation | None |
| `/docs/prompt-intake` | `docs/prompt-intake/page.tsx` | Prompt intake documentation | None |
| `/docs/task-contracts` | `docs/task-contracts/page.tsx` | Task contract documentation | None |
| `/docs/running-governed-tasks` | `docs/running-governed-tasks/page.tsx` | Running governed tasks guide | None |
| `/docs/approvals` | `docs/approvals/page.tsx` | Human approval workflow docs | None |
| `/docs/checkpoint-rollback` | `docs/checkpoint-rollback/page.tsx` | Checkpoint & rollback docs | None |
| `/docs/memory` | `docs/memory/page.tsx` | Memory documentation | None |
| `/docs/rules-policies` | `docs/rules-policies/page.tsx` | Rules & policies documentation | None |
| `/docs/mcp-l2` | `docs/mcp-l2/page.tsx` | MCP L2 proxy documentation | None |
| `/docs/l3-workspaces` | `docs/l3-workspaces/page.tsx` | L3 workspace containment docs | None |
| `/docs/l4-authority` | `docs/l4-authority/page.tsx` | L4 disposable credentials docs | None |
| `/docs/receipts-audit` | `docs/receipts-audit/page.tsx` | Receipts & audit documentation | None |
| `/docs/integrations` | `docs/integrations/page.tsx` | Integration documentation | None |
| `/docs/agent-handoff` | `docs/agent-handoff/page.tsx` | Agent handoff documentation | None |
| `/docs/cli-reference` | `docs/cli-reference/page.tsx` | CLI reference | None |
| `/docs/troubleshooting` | `docs/troubleshooting/page.tsx` | Troubleshooting guide | None |
| `/docs/security-model` | `docs/security-model/page.tsx` | Security model docs | None |
| `/docs/limitations` | `docs/limitations/page.tsx` | Limitations documentation | None |

### 2.4 API Endpoints Called

**None.** The site is fully static with zero API calls. All content is hardcoded JSX/TSX. No data fetching, no client-side state management, no react-query.

---

## 3. Legacy Site (site/index.html)

**Location:** `D:\Onus\onus\site\index.html`
**Size:** 9,237 bytes
**Framework:** None (vanilla HTML + inline CSS)
**JavaScript:** None

A single-page static marketing HTML page with:
- Dark theme (black background, white text, orange accents)
- Hero section: "AI Agent Firewall" messaging
- Feature sections: Prompt Intake Guardian, Deterministic Rules Engine, Semantic Analysis, Human Approval Workflow, Completion Verification, Checkpoints & Rollback
- 6-column integration grid
- Footer with links to Product, Docs, GitHub

This is a pre-Next.js version of the marketing site, still present in the repo but superseded by `apps/onus-site/`.

---

## 4. Tests, Lint, Typecheck, Build

### 4.1 Tests

**Neither app has any tests.** There are no test files, no test runner configuration, and no test scripts in either `package.json`.

### 4.2 Lint

Both apps have the `lint` script defined:
- `npm run lint` / `pnpm lint` — runs `next lint` (ESLint with `next/core-web-vitals` config)

### 4.3 Typecheck

**Neither app has a dedicated typecheck script.** There is no `tsc --noEmit` or similar in either `package.json`. TypeScript errors would only surface during `next build`.

### 4.4 Build

- **onus-console:** `next build` — produces a standard Next.js server build (not exported)
- **onus-site:** `next build` with `output: 'export'` — produces a fully static HTML output

---

## 5. Security Findings

### 5.1 Simulated Validation in Activation Wizard (VIOLATION)

**File:** `src/app/activate/page.tsx` (onus-console)

The activation wizard's validation steps use **hardcoded success messages with timer-based delays** instead of making real API calls to verify daemon status, provider connectivity, or policy installation. This violates the AGENTS.md rules:

- **"No fake completion"** — simulated validation presents hardcoded success as runtime evidence
- **Mocks in production paths** — `setTimeout`-based delays are a mock pattern in production code
- **Security invariant** — "Completion requires evidence" is not satisfied

The specific lines:
- Provider test: `await new Promise(r => setTimeout(r, 1500)); setProviderTested(true);` — unconditionally succeeds
- Validation checks: `await new Promise(r => setTimeout(r, 800))` — 6 hardcoded checks that always pass with pre-written success messages

**Severity:** Medium. The activation wizard is a setup flow, not a runtime enforcement path. However, it gives users false confidence that their daemon is properly configured when no verification actually occurs.

### 5.2 No Mock/Fake Patterns Elsewhere

No other console pages use mock data, simulated responses, or hardcoded results. All other pages call the BFF proxy with real `fetchApi()` calls and use server responses directly.

### 5.3 Console Middleware Security

The `src/middleware.ts` provides:
- Origin validation against an allowlist (localhost + 127.0.0.1:3001)
- CSRF token validation on state-changing methods
- Security headers (CSP, HSTS, X-Frame-Options, etc.)

**Note:** The CSRF token is a static value `'1'` rather than a per-session token. This provides basic but not robust CSRF protection.
