//! `onus dashboard` — local read-only dashboard backed by the SQLite audit DB.

use crate::audit::AuditTrail;
use clap::Args;
use std::path::{Path, PathBuf};

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
        let request = server.recv()?;
        let url = request.url().to_string();
        if !crate::security::authorized_url(&url, &token) {
            let _ = request.respond(response(401, b"Unauthorized", "text/plain"));
            continue;
        }
        let path = url.split('?').next().unwrap_or("/");
        let response = match path {
            "/" => response(
                200,
                render_index(&db_path, &token)?.as_bytes(),
                "text/html; charset=utf-8",
            ),
            "/api/actions" => response(
                200,
                render_actions_json(&db_path)?.as_bytes(),
                "application/json",
            ),
            _ => response(404, b"Not Found", "text/plain"),
        };
        let _ = request.respond(response);
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

fn response(status_code: u16, body: &[u8], content_type: &str) -> tiny_http::ResponseBox {
    tiny_http::Response::from_data(body)
        .with_status_code(tiny_http::StatusCode(status_code))
        .with_header(
            tiny_http::Header::from_bytes(&b"Content-Type"[..], content_type.as_bytes()).unwrap(),
        )
        .boxed()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Verdict;

    fn temp_db(name: &str) -> PathBuf {
        std::env::temp_dir().join(format!("onus-{}-{}.db", name, uuid::Uuid::new_v4()))
    }

    #[test]
    fn dashboard_actions_json_masks_persisted_payloads() {
        let path = temp_db("dashboard-mask");
        let mut audit = AuditTrail::open(&path).unwrap();
        audit
            .record_action(
                "session-1",
                1,
                "file_write",
                "Write",
                r#"{"content":"password=super-secret","token":"raw-token"}"#,
                &Verdict::Allow,
                None,
                None,
                1,
            )
            .unwrap();

        let body = render_actions_json(&path).unwrap();
        assert!(!body.contains("super-secret"));
        assert!(!body.contains("raw-token"));
        assert!(body.contains(crate::security::REDACTED));
        assert!(body.contains("payload_hash"));

        let _ = std::fs::remove_file(path);
    }
}
