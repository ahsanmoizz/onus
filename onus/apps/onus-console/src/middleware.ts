import { NextResponse } from 'next/server';
import type { NextRequest } from 'next/server';

const ALLOWED_ORIGINS = [
  'http://localhost:3001',
  'http://127.0.0.1:3001',
];

const CSRF_METHODS = ['POST', 'PUT', 'PATCH', 'DELETE'];

function isStaticAsset(pathname: string): boolean {
  return (
    pathname.startsWith('/_next/') ||
    pathname.startsWith('/static/') ||
    pathname.startsWith('/favicon') ||
    pathname.endsWith('.js') ||
    pathname.endsWith('.css') ||
    pathname.endsWith('.png') ||
    pathname.endsWith('.jpg') ||
    pathname.endsWith('.svg') ||
    pathname.endsWith('.ico') ||
    pathname.endsWith('.woff2')
  );
}

function isApiRoute(pathname: string): boolean {
  return pathname.startsWith('/api/');
}

function getOrigin(request: NextRequest): string | null {
  return request.headers.get('origin');
}

function getReferer(request: NextRequest): string | null {
  return request.headers.get('referer');
}

function isOriginAllowed(origin: string): boolean {
  return ALLOWED_ORIGINS.some(
    (allowed) => origin === allowed || origin.startsWith(allowed + '/')
  );
}

function addSecurityHeaders(response: NextResponse): NextResponse {
  response.headers.set('X-Content-Type-Options', 'nosniff');
  response.headers.set('X-Frame-Options', 'DENY');
  response.headers.set('Referrer-Policy', 'strict-origin-when-cross-origin');
  response.headers.set('X-XSS-Protection', '0');
  response.headers.set(
    'Permissions-Policy',
    'camera=(), microphone=(), geolocation=()'
  );

  const host = response.headers.get('host') || '';
  if (!host.includes('localhost') && !host.includes('127.0.0.1')) {
    response.headers.set(
      'Strict-Transport-Security',
      'max-age=31536000; includeSubDomains'
    );
  }

  return response;
}

export function middleware(request: NextRequest) {
  const { pathname } = request.nextUrl;

  // Skip middleware for static assets
  if (isStaticAsset(pathname)) {
    return NextResponse.next();
  }

  // Apply security headers to all responses (including API)
  if (isApiRoute(pathname)) {
    // CSRF check for state-changing methods
    if (CSRF_METHODS.includes(request.method)) {
      const origin = getOrigin(request);
      const referer = getReferer(request);

      // If Origin header is present, validate it
      if (origin) {
        if (!isOriginAllowed(origin)) {
          return new NextResponse(
            JSON.stringify({ error: 'CSRF: Origin not allowed' }),
            {
              status: 403,
              headers: { 'Content-Type': 'application/json' },
            }
          );
        }
      } else if (referer) {
        // Fallback to Referer if Origin is absent
        try {
          const refererUrl = new URL(referer);
          if (!isOriginAllowed(refererUrl.origin)) {
            return new NextResponse(
              JSON.stringify({ error: 'CSRF: Referer not allowed' }),
              {
                status: 403,
                headers: { 'Content-Type': 'application/json' },
              }
            );
          }
        } catch {
          return new NextResponse(
            JSON.stringify({ error: 'CSRF: Invalid referer' }),
            {
              status: 403,
              headers: { 'Content-Type': 'application/json' },
            }
          );
        }
      }

      // Require custom header for CSRF protection from browser contexts
      const hasCsrfHeader =
        request.headers.has('x-csrf-token') ||
        request.headers.has('x-requested-by');

      // Only enforce custom header if request appears to be from a browser
      // (has Origin/Referer or common browser headers)
      const userAgent = request.headers.get('user-agent') || '';
      const appearsBrowser =
        !userAgent.includes('curl') &&
        !userAgent.includes('node-fetch') &&
        !userAgent.includes('undici') &&
        (origin !== null || referer !== null);

      if (appearsBrowser && !hasCsrfHeader) {
        return new NextResponse(
          JSON.stringify({
            error: 'CSRF: Missing required anti-forgery header',
          }),
          {
            status: 403,
            headers: { 'Content-Type': 'application/json' },
          }
        );
      }
    }

    // Pass through for API requests (rewrites handle the rest)
    const response = NextResponse.next();
    return addSecurityHeaders(response);
  }

  // Non-API routes
  const response = NextResponse.next();
  return addSecurityHeaders(response);
}

export const config = {
  matcher: ['/((?!_next/static|_next/image|favicon.ico).*)'],
};
