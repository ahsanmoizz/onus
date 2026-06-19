# ONUS CONSOLE LAUNCH

Generated: 2026-06-18

## Current state

There is **no dedicated web console** in the repository.  No `console/` directory
exists, and no `package.json` for a frontend was found.

## Available dashboard

The CLI-based interactive dashboard can be started with:

```
onus dashboard
```

This provides an in-terminal session viewer and action browser.

## Future console

A web console (React / Next.js) is expected in a future milestone.  When the
`console/` directory is added, use:

```powershell
cd onus/console
npm ci
npm run dev       # development
npm run build     # production build
npm start         # production start
```

## Verification

The console does not exist yet, so no build, health check, or API verification
is possible.  The start script `scripts/start-onus-console.ps1` documents
this state and points users to `onus dashboard`.
