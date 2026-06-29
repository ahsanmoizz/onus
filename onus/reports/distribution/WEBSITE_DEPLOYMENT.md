# Onus Website Deployment Preparation

**Date:** 2026-06-19

---

## Application Overview

The public website is a Next.js 14 application at `apps/onus-site/`.

## Build

```bash
cd apps/onus-site
npm ci
npm run lint
npm run build
```

The production build outputs to `.next/` (standard Next.js static + server bundle).

## Deployment Modes

### Static Export (Recommended for initial deployment)

```bash
# In next.config.js, uncomment or add:
#   output: 'export'
#
# Then:
cd apps/onus-site
npm run build
# Output in out/
```

Requires all routes that use server-side features (headers, cookies, etc.) to be disabled or moved to client components.

### Node.js Server (Full SSR)

```bash
cd apps/onus-site
npm run build
npm start -p 3000
```

Requires a process manager (systemd, PM2, or container runtime).

## Environment Variables

| Variable | Purpose | Required |
|---|---|---|
| `NEXT_PUBLIC_SITE_URL` | Canonical site URL | Yes |
| `NEXT_PUBLIC_RELEASE_MANIFEST_URL` | URL to release-manifest.json for download page | Yes |
| `NEXT_PUBLIC_VERSION` | Current Onus version shown on site | Yes |

## Sitemap

Generate sitemap.xml automatically via Next.js or a build-time script:

```typescript
// apps/onus-site/src/app/sitemap.ts
import { MetadataRoute } from 'next'
export default function sitemap(): MetadataRoute.Sitemap {
  const baseUrl = process.env.NEXT_PUBLIC_SITE_URL || 'https://onus.ai'
  return [
    { url: baseUrl, lastModified: new Date() },
    { url: `${baseUrl}/product`, lastModified: new Date() },
    { url: `${baseUrl}/download`, lastModified: new Date() },
    { url: `${baseUrl}/install`, lastModified: new Date() },
    { url: `${baseUrl}/docs`, lastModified: new Date() },
    { url: `${baseUrl}/security`, lastModified: new Date() },
    { url: `${baseUrl}/docs/quick-start`, lastModified: new Date() },
    { url: `${baseUrl}/docs/installation`, lastModified: new Date() },
    { url: `${baseUrl}/docs/cli-reference`, lastModified: new Date() },
    { url: `${baseUrl}/changelog`, lastModified: new Date() },
    { url: `${baseUrl}/limitations`, lastModified: new Date() },
  ]
}
```

## Required Files

- `public/robots.txt` — allow all crawlers on production
- `public/sitemap.xml` — generated at build time

## SEO / Metadata

- All pages should include `<title>` and `<meta name="description">`
- Open Graph tags for `/product`, `/download`, `/docs`
- Canonical URLs on all pages

## Security Headers

Recommended deployment configuration (set in reverse proxy or hosting platform):

```
Content-Security-Policy: default-src 'self'; script-src 'self' 'unsafe-eval' 'unsafe-inline'; style-src 'self' 'unsafe-inline'; img-src 'self' data:; font-src 'self' data:; connect-src 'self';
X-Content-Type-Options: nosniff
X-Frame-Options: DENY
Strict-Transport-Security: max-age=31536000; includeSubDomains
Referrer-Policy: strict-origin-when-cross-origin
Permissions-Policy: camera=(), microphone=(), geolocation=()
```

## Deployment Platforms

| Platform | Compatibility | Notes |
|---|---|---|
| **Vercel** | Native Next.js | Recommended; automatic SSR, ISR |
| **Cloudflare Pages** | Static export only | No SSR without Pages Functions |
| **GitHub Pages** | Static export only | No SSR |
| **Self-hosted** | Full SSR | Requires Node.js runtime, reverse proxy |

## Release Manifest Integration

The download page reads from `release-manifest.json`. Configure via:

```bash
NEXT_PUBLIC_RELEASE_MANIFEST_URL=https://github.com/ahsanmoizz/onus/releases/latest/download/release-manifest.json
```

Fallback to hardcoded version data when the manifest URL is unavailable.

## Pre-Deployment Checklist

- [ ] `npm run build` passes without errors
- [ ] All links are valid (no broken internal links)
- [ ] Release manifest URL is configured and reachable
- [ ] Version number in site matches binary version
- [ ] Download links point to actual release artifacts
- [ ] SHA-256 checksums are displayed accurately
- [ ] Open Graph metadata renders correctly
- [ ] Sitemap is generated and accessible
- [ ] robots.txt allows indexing
- [ ] CSP headers are applied
- [ ] 404 page exists and is functional
- [ ] `NEXT_PUBLIC_SITE_URL` is set to production URL
- [ ] Assets are optimized (images, fonts, scripts)
- [ ] Lighthouse audit passes (performance, accessibility, SEO)

## External Requirements

| Requirement | Status | Notes |
|---|---|---|
| Domain name (onus.ai) | Available | Configure DNS |
| TLS certificate | Available | Let's Encrypt or Cloudflare |
| CDN | Available | Cloudflare or Vercel Edge |
| CI/CD | Available | GitHub Actions |
| Analytics | OPTIONAL | Not yet configured |
