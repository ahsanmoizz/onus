//! `onus dashboard` — local read-only dashboard backed by the SQLite audit DB.
//!
//! Security features:
//! - Token-based authentication via query parameter
//! - CSRF protection via Origin/Referer header validation
//! - Rate limiting (simple per-IP counter)
//! - Security headers (CSP, X-Content-Type-Options, etc.)

use crate::audit::AuditTrail;
use clap::Args;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

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
        let now = now_secs();
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

fn now_secs() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64
}

// ── Helpers ──────────────────────────────────────────────────────────────────

fn origin_is_local(origin: &str) -> bool {
    origin == "http://127.0.0.1" || origin.starts_with("http://127.0.0.1:") || origin.starts_with("http://localhost")
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
    // Expose token in header for XHR access
    .with_header(
        tiny_http::Header::from_bytes(&b"X-Onus-Token"[..], token.as_bytes()).unwrap(),
    )
}

fn response(status_code: u16, body: &[u8], content_type: &str, token: &str) -> tiny_http::ResponseBox {
    let resp = tiny_http::Response::from_data(body)
        .with_status_code(tiny_http::StatusCode(status_code))
        .with_header(
            tiny_http::Header::from_bytes(&b"Content-Type"[..], content_type.as_bytes()).unwrap(),
        );
    security_headers(resp.boxed(), token)
}

// ── Main server ──────────────────────────────────────────────────────────────

use std::sync::OnceLock;

fn rate_limiter() -> &'static Mutex<RateLimiter> {
    static RL: OnceLock<Mutex<RateLimiter>> = OnceLock::new();
    RL.get_or_init(|| Mutex::new(RateLimiter::new(60, 60)))
}

#[derive(Args)]
pub struct DashboardArgs {
    /// Path to audit database
    #[arg(long)]
    pub db: Option<PathBuf>,

    /// Port to serve the dashboard on
    #[arg(long, default_value_t = 8787)]
    pub port: u16,

    /// Local dashboard token. If omitted, a random token is generated.
    #[arg(long)]
    pub token: Option<String>,
}

pub fn run(args: DashboardArgs) -> anyhow::Result<()> {
    let db_path = args
        .db
        .unwrap_or_else(|| crate::data_dir().join("audit.db"));
    let token = args.token.unwrap_or_else(crate::security::local_token);
    let addr = format!("127.0.0.1:{}", args.port);
    let server = tiny_http::Server::http(&addr)
        .map_err(|e| anyhow::anyhow!("Failed to start dashboard on {}: {}", addr, e))?;

    println!("Onus dashboard: http://{}?token={}", addr, token);
    println!("Reading audit DB: {}", db_path.display());

    loop {
        let request = match server.recv() {
            Ok(r) => r,
            Err(e) => {
                log::error!("Dashboard recv error: {}", e);
                continue;
            }
        };

        let url = request.url().to_string();

        // Rate limit by IP
        let client_ip = request
            .remote_addr()
            .map(|a| a.to_string())
            .unwrap_or_else(|| "unknown".to_string());
        {
            let mut limiter = rate_limiter().lock().unwrap();
            if !limiter.allow(&client_ip) {
                let _ = request.respond(response(429, b"Too Many Requests", "text/plain", &token));
                continue;
            }
        }

        // Authentication
        if !crate::security::authorized_url(&url, &token) {
            let _ = request.respond(response(401, b"Unauthorized", "text/plain", &token));
            continue;
        }

        // CSRF protection for non-GET methods
        if request.method() != &tiny_http::Method::Get {
            let origin = request.headers().iter().find(|h| h.field.to_string() == "Origin").map(|h| h.value.to_string());
            let referer = request.headers().iter().find(|h| h.field.to_string() == "Referer").map(|h| h.value.to_string());

            let csrf_ok = match (&origin, &referer) {
                (Some(o), _) if origin_is_local(o) => true,
                (_, Some(r)) if referer_is_local(r) => true,
                _ => false,
            };
            if !csrf_ok {
                let _ = request.respond(response(403, b"CSRF check failed", "text/plain", &token));
                continue;
            }
        }

        let path = url.split('?').next().unwrap_or("/");
        let resp = match path {
            "/" => response(
                200,
                render_index(&db_path, &token)?.as_bytes(),
                "text/html; charset=utf-8",
                &token,
            ),
            "/api/actions" => response(
                200,
                render_actions_json(&db_path)?.as_bytes(),
                "application/json",
                &token,
            ),
            "/health" => response(
                200,
                b"OK",
                "text/plain",
                &token,
            ),
            _ => response(404, b"Not Found", "text/plain", &token),
        };
        let _ = request.respond(resp);
    }
}

fn render_actions_json(db_path: &Path) -> anyhow::Result<String> {
    if !db_path.exists() {
        return Ok("[]".to_string());
    }
    let audit = AuditTrail::open(db_path)?;
    let actions = audit.get_recent_actions(100)?;
    let rows: Vec<_> = actions
        .into_iter()
        .map(|a| {
            serde_json::json!({
                "id": a.id,
                "session_id": a.session_id,
                "sequence": a.sequence,
                "action_type": a.action_type,
                "tool": a.tool_name,
                "payload": crate::security::mask_text_for_display(&a.payload),
                "payload_hash": a.payload_hash,
                "payload_classification": a.payload_classification,
                "verdict": a.verdict,
                "rule_id": a.rule_id,
                "correction": a.correction,
                "approval_decision": a.approval_decision,
                "guardian_mode": a.guardian_mode,
                "obligations": a.obligations_json
                    .as_deref()
                    .and_then(|raw| serde_json::from_str::<serde_json::Value>(raw).ok())
                    .unwrap_or_else(|| serde_json::json!([])),
                "approval_reason": a.approval_reason,
                "hash": a.hash,
                "created_at": a.created_at,
            })
        })
        .collect();
    Ok(serde_json::to_string(&rows)?)
}

fn render_index(db_path: &Path, token: &str) -> anyhow::Result<String> {
    let actions_json = render_actions_json(db_path)?;
    Ok(format!(
        r#"<!doctype html>
<html lang="en">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <meta http-equiv="Content-Security-Policy" content="default-src 'self'; script-src 'self' 'unsafe-inline'; style-src 'self' 'unsafe-inline'; connect-src 'self' http://127.0.0.1:*;">
  <title>Onus Dashboard</title>
  <style>
    body {{ margin: 0; font-family: Inter, Segoe UI, Arial, sans-serif; background: #f7f8fa; color: #111827; }}
    header {{ padding: 24px 32px; background: #ffffff; border-bottom: 1px solid #e5e7eb; }}
    h1 {{ margin: 0 0 6px; font-size: 24px; }}
    main {{ padding: 24px 32px; }}
    .stats {{ display: grid; grid-template-columns: repeat(4, minmax(120px, 1fr)); gap: 12px; margin-bottom: 20px; }}
    .stat {{ background: white; border: 1px solid #e5e7eb; border-radius: 8px; padding: 14px; }}
    .stat strong {{ display: block; font-size: 24px; }}
    table {{ width: 100%; border-collapse: collapse; background: white; border: 1px solid #e5e7eb; border-radius: 8px; overflow: hidden; }}
    th, td {{ text-align: left; padding: 10px 12px; border-bottom: 1px solid #eef0f3; font-size: 13px; vertical-align: top; }}
    th {{ background: #f3f4f6; font-size: 12px; text-transform: uppercase; letter-spacing: .04em; }}
    code {{ white-space: pre-wrap; word-break: break-word; }}
    .allow {{ color: #047857; font-weight: 700; }}
    .warn {{ color: #b45309; font-weight: 700; }}
    .block {{ color: #b91c1c; font-weight: 700; }}
    .escalate {{ color: #7c3aed; font-weight: 700; }}
  </style>
</head>
<body>
  <header>
    <h1>Onus Dashboard</h1>
    <div>Live local audit data from <code>{}</code></div>
  </header>
  <main>
    <section class="stats">
      <div class="stat"><span>Total actions</span><strong id="total">0</strong></div>
      <div class="stat"><span>Blocked</span><strong id="blocked">0</strong></div>
      <div class="stat"><span>Escalated</span><strong id="escalated">0</strong></div>
      <div class="stat"><span>Sessions</span><strong id="sessions">0</strong></div>
    </section>
    <table>
      <thead><tr><th>Time</th><th>Verdict</th><th>Session</th><th>Type</th><th>Tool</th><th>Payload</th><th>Hash</th></tr></thead>
      <tbody id="rows"></tbody>
    </table>
  </main>
  <script>
    const token = {};
    const actions = {};
    const sessions = new Set(actions.map(a => a.session_id));
    document.getElementById('total').textContent = actions.length;
    document.getElementById('blocked').textContent = actions.filter(a => a.verdict === 'block').length;
    document.getElementById('escalated').textContent = actions.filter(a => a.verdict === 'escalate').length;
    document.getElementById('sessions').textContent = sessions.size;
    document.getElementById('rows').innerHTML = actions.map(a => `
      <tr>
        <td>${{escapeHtml(a.created_at)}}</td>
        <td class="${{escapeHtml(a.verdict)}}">${{escapeHtml(a.verdict)}}</td>
        <td>${{escapeHtml(a.session_id)}}</td>
        <td>${{escapeHtml(a.action_type)}}</td>
        <td>${{escapeHtml(a.tool || '-')}}</td>
        <td><code>${{escapeHtml(a.payload)}}</code>${{a.correction ? `<div>${{escapeHtml(a.correction)}}</div>` : ''}}</td>
        <td><code>${{escapeHtml((a.hash || '').slice(0, 16))}}</code></td>
      </tr>
    `).join('');
    function escapeHtml(value) {{
      return String(value ?? '').replace(/[&<>"']/g, c => ({{'&':'&amp;','<':'&lt;','>':'&gt;','"':'&quot;',"'":'&#39;'}}[c]));
    }}
  </script>
</body>
</html>"#,
        db_path.display(),
        serde_json::to_string(token)?,
        actions_json
    ))
}
