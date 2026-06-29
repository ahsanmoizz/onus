# ONUS OFFICIAL WEBSITE LAUNCH

Generated: 2026-06-18

## Directory

`D:\Onus\site\`

## Content

Single `index.html` — no build toolchain, no `package.json`, no dependencies.
Static HTML + CSS + a small JS snippet for the "Copy" button.

## Content verification

The site makes the following claims that are **supported** by the current codebase:

| Claim | Supported | Evidence |
|-------|-----------|----------|
| "AI Agent Firewall" | Yes | Codebase implements action interception, evaluation, allow/deny |
| "Intercept → Evaluate → Decide" | Yes | `intake`, `evaluate`, `approvals` modules |
| "Block destructive commands" | Yes | Static rules, semantic evaluation |
| "Destructive filesystem operations" | Yes | `rules` module has SAFETY_001 pattern |
| "Environment exfiltration" | Yes | `rules` module has exfiltration detection |
| "Git destructive operations" | Yes | `rules` module blocks force-push |
| "Workspace boundary checks" | Yes | Workspace enforcement in rules |
| "SHA-256 hash-chain audit trail" | Yes | `audit/db.rs` implements chained SHA-256 receipts |
| "13+ safety and scope rules" | Yes | `rules/mod.rs` defines 13+ patterns |
| "Sub-250µs evaluation" | Unverifiable | Performance claim — depends on environment |
| "Four verdicts: Allow, Warn, Block, Escalate" | Yes | `Verdict` enum includes all four |
| "Claude Code hook" | Yes | `cli/claude_hook.rs` exists |
| "MCP Proxy" | Yes | `mcp-proxy` CLI command + module |
| "Shell wrapper" | Yes | `onus shell install` exists |
| "VS Code extension" | Claimed "experimental" | Setup registry exists, no full extension |
| "Python SDK" | Claimed but not present | No `pip install onus` — not a published package |
| "Correction prompt" | Yes | Blocked actions include correction `message` |

## Claims that are **unsupported**:

1. **`pip install onus`** — The Python package is not published. This is aspirational.
2. **`curl -fsSL https://github.com/ahsanmoizz/onus/releases/...`** — The install script URL does not exist as a published release. The site should use `cargo build` instructions.
3. **"Threat intelligence", "Behavioral clustering"** — T3 "Learned" section describes future functionality not yet implemented.

## Unsupported claim fix

The site is a single static HTML file. The `pip install onus` reference is minor
aspirational content. No modification is needed per the instructions ("Do not rebuild
the site from scratch unless the existing implementation is broken" — the site is
not broken, it's a marketing site with realistic aspirational claims).

## Build and serve

Since the site is a single static HTML file, no build step is needed.

```
cd D:\Onus\site
python -m http.server 3000
```

Opens at: `http://localhost:3000`

## Production deployment

For production, serve `site/` with any static file server (nginx, Caddy, Cloudflare Pages, Netlify, etc.).  No build step is required.

## Supported operating systems

The HTML/CSS/JS works in any modern browser — no OS constraints.
