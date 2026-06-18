//! Approval HTTP server for MCP proxy Escalate verdicts.
//!
//! Serves a minimal web UI on localhost:9191 that lists pending approvals
//! and allows Approve/Reject actions via REST API calls.
//!
//! Security:
//! - Token-based authentication via query parameter
//! - CSRF protection via Origin/Referer header validation on POST
//! - Rate limiting (per-IP, 30 req/min)
//! - Security headers (CSP, X-Content-Type-Options, etc.)

use crate::audit::db::AuditTrail;
use std::collections::HashMap;
use std::path::Path;
use std::sync::{Arc, Mutex, OnceLock};

// ── Rate limiter ─────────────────────────────────────────────────────────────

struct RateLimiter {
    requests: HashMap<String, Vec<i64>>,
    max_requests: usize,
    window_secs: i64,
}

impl RateLimiter {
    fn new(max_requests: usize, window_secs: i64) -> Self {
        Self {
            requests: HashMap::new(),
            max_requests,
            window_secs,
        }
    }

    fn allow(&mut self, key: &str) -> bool {
        let now = unix_now();
        let cutoff = now - self.window_secs;
        let entries = self.requests.entry(key.to_string()).or_default();
        entries.retain(|t| *t > cutoff);
        if entries.len() >= self.max_requests {
            false
        } else {
            entries.push(now);
            true
        }
    }
}

fn unix_now() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64
}

fn rate_limiter() -> &'static Mutex<RateLimiter> {
    static RL: OnceLock<Mutex<RateLimiter>> = OnceLock::new();
    RL.get_or_init(|| Mutex::new(RateLimiter::new(30, 60)))
}

fn origin_is_local(origin: &str) -> bool {
    origin == "http://127.0.0.1"
        || origin.starts_with("http://127.0.0.1:")
        || origin.starts_with("http://localhost")
}

fn referer_is_local(referer: &str) -> bool {
    referer.starts_with("http://127.0.0.1") || referer.starts_with("http://localhost")
}

// ── Response helpers ─────────────────────────────────────────────────────────

fn security_headers(resp: tiny_http::ResponseBox, token: &str) -> tiny_http::ResponseBox {
    let csp = "default-src 'self'; script-src 'self' 'unsafe-inline'; style-src 'self' 'unsafe-inline'; connect-src 'self' http://127.0.0.1:*; img-src 'self' data:;".to_string();
    resp.with_header(
        tiny_http::Header::from_bytes(&b"Content-Security-Policy"[..], csp.as_bytes()).unwrap(),
    )
    .with_header(
        tiny_http::Header::from_bytes(&b"X-Content-Type-Options"[..], b"nosniff").unwrap(),
    )
    .with_header(
        tiny_http::Header::from_bytes(&b"X-Frame-Options"[..], b"DENY").unwrap(),
    )
    .with_header(
        tiny_http::Header::from_bytes(&b"Referrer-Policy"[..], b"same-origin").unwrap(),
    )
    .with_header(
        tiny_http::Header::from_bytes(&b"X-Onus-Token"[..], token.as_bytes()).unwrap(),
    )
}

fn response(
    status_code: u16,
    body: &[u8],
    content_type: &str,
    token: &str,
) -> tiny_http::ResponseBox {
    let resp = tiny_http::Response::from_data(body)
        .with_status_code(tiny_http::StatusCode(status_code))
        .with_header(
            tiny_http::Header::from_bytes(&b"Content-Type"[..], content_type.as_bytes()).unwrap(),
        );
    security_headers(resp.boxed(), token)
}

/// Shared state between the approval server and the rest of the application.
pub struct ApprovalState {
    pub audit: Arc<Mutex<AuditTrail>>,
}

impl ApprovalState {
    pub fn new(audit: Arc<Mutex<AuditTrail>>) -> Self {
        Self { audit }
    }

    pub fn open(db_path: &Path) -> anyhow::Result<Self> {
        let audit = AuditTrail::open(db_path)
            .map_err(|e| anyhow::anyhow!("Failed to open audit DB for approval server: {}", e))?;
        Ok(Self {
            audit: Arc::new(Mutex::new(audit)),
        })
    }
}

/// Start the approval HTTP server on the given port, blocking forever.
pub fn serve(state: ApprovalState, port: u16, token: String) -> anyhow::Result<()> {
    let addr = format!("127.0.0.1:{}", port);
    let server = tiny_http::Server::http(&addr)
        .map_err(|e| anyhow::anyhow!("Failed to start approval server on {}: {}", addr, e))?;
    log::info!("Approval UI listening on http://{}", addr);

    let state = Arc::new(state);

    loop {
        let request = match server.recv() {
            Ok(r) => r,
            Err(e) => {
                log::error!("Approval server recv error: {}", e);
                continue;
            }
        };

        let url = request.url().to_string();
        let method = request.method().as_str().to_string();

        // Rate limit by IP
        let client_ip = request
            .remote_addr()
            .map(|a| a.to_string())
            .unwrap_or_else(|| "unknown".to_string());
        {
            let mut limiter = rate_limiter().lock().unwrap();
            if !limiter.allow(&client_ip) {
                let _ = request.respond(response(429u16, b"Too Many Requests", "text/plain", &token));
                continue;
            }
        }

        // Authentication
        if !crate::security::authorized_url(&url, &token) {
            let _ = request.respond(response(401u16, b"Unauthorized", "text/plain", &token));
            continue;
        }

        // CSRF protection for POST
        if method == "POST" {
            let origin = request
                .headers()
                .iter()
                .find(|h| h.field.to_string() == "Origin")
                .map(|h| h.value.to_string());
            let referer = request
                .headers()
                .iter()
                .find(|h| h.field.to_string() == "Referer")
                .map(|h| h.value.to_string());

            let csrf_ok = match (&origin, &referer) {
                (Some(o), _) if origin_is_local(o) => true,
                (_, Some(r)) if referer_is_local(r) => true,
                _ => false,
            };
            if !csrf_ok {
                let _ = request.respond(response(403u16, b"CSRF check failed", "text/plain", &token));
                continue;
            }
        }

        let path = url.split('?').next().unwrap_or("/").to_string();

        let state = state.clone();
        let token = token.clone();

        // Handle each request in a blocking thread (simple enough for local-only UI)
        std::thread::spawn(move || {
            let response = match (method.as_str(), path.as_str()) {
                ("GET", "/") => serve_index(&token),
                ("GET", "/api/pending") => serve_pending(&state, &token),
                ("POST", path) if path.starts_with("/api/approve/") => {
                    let action_id = &path["/api/approve/".len()..];
                    serve_approve(
                        &state,
                        action_id,
                        &format!("local-token:{}", &token[..8.min(token.len())]),
                        &token,
                    )
                }
                ("POST", path) if path.starts_with("/api/reject/") => {
                    let action_id = &path["/api/reject/".len()..];
                    serve_reject(&state, action_id, &token)
                }
                _ => response(404u16, b"Not Found", "text/plain", &token),
            };

            let _ = request.respond(response);
        });
    }
}

fn serve_index(token: &str) -> tiny_http::ResponseBox {
    let html = include_str!("index.html").replace("__ONUS_TOKEN__", token);
    response(200, html.as_bytes(), "text/html; charset=utf-8", token)
}

fn serve_pending(state: &Arc<ApprovalState>, token: &str) -> tiny_http::ResponseBox {
    let approvals = {
        let audit = state.audit.lock().unwrap();
        audit.get_pending_approvals().unwrap_or_default()
    };
    let json = serde_json::to_string(&approvals).unwrap_or_else(|_| "[]".to_string());
    response(200, json.as_bytes(), "application/json", token)
}

fn serve_approve(
    state: &Arc<ApprovalState>,
    action_id: &str,
    approver: &str,
    token: &str,
) -> tiny_http::ResponseBox {
    let result = {
        let mut audit = state.audit.lock().unwrap();
        audit.approve_action(action_id, approver)
    };
    match result {
        Ok(_) => {
            let json =
                serde_json::json!({"status": "approved", "action_id": action_id}).to_string();
            response(200, json.as_bytes(), "application/json", token)
        }
        Err(e) => {
            let json = serde_json::json!({"error": format!("{}", e)}).to_string();
            response(500, json.as_bytes(), "application/json", token)
        }
    }
}

fn serve_reject(
    state: &Arc<ApprovalState>,
    action_id: &str,
    token: &str,
) -> tiny_http::ResponseBox {
    let result = {
        let mut audit = state.audit.lock().unwrap();
        audit.reject_action(action_id)
    };
    match result {
        Ok(_) => {
            let json =
                serde_json::json!({"status": "rejected", "action_id": action_id}).to_string();
            response(200, json.as_bytes(), "application/json", token)
        }
        Err(e) => {
            let json = serde_json::json!({"error": format!("{}", e)}).to_string();
            response(500, json.as_bytes(), "application/json", token)
        }
    }
}
