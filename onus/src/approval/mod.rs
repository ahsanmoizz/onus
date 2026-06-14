//! Approval HTTP server for MCP proxy Escalate verdicts.
//!
//! Serves a minimal web UI on localhost:9191 that lists pending approvals
//! and allows Approve/Reject actions via REST API calls.

use crate::audit::db::AuditTrail;
use std::path::Path;
use std::sync::{Arc, Mutex};

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
pub fn serve(state: ApprovalState, port: u16) -> anyhow::Result<()> {
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

        let state = state.clone();

        // Handle each request in a blocking thread (simple enough for local-only UI)
        std::thread::spawn(move || {
            let url = request.url().to_string();
            let method = request.method().as_str().to_string();

            let response = match (method.as_str(), url.as_str()) {
                ("GET", "/") => serve_index(),
                ("GET", "/api/pending") => serve_pending(&state),
                ("POST", url) if url.starts_with("/api/approve/") => {
                    let action_id = &url["/api/approve/".len()..];
                    serve_approve(&state, action_id)
                }
                ("POST", url) if url.starts_with("/api/reject/") => {
                    let action_id = &url["/api/reject/".len()..];
                    serve_reject(&state, action_id)
                }
                _ => response(404u16, "Not Found".as_bytes(), "text/plain"),
            };

            let _ = request.respond(response);
        });
    }
}

fn response(status_code: u16, body: &[u8], content_type: &str) -> tiny_http::ResponseBox {
    let status = tiny_http::StatusCode(status_code);
    let resp = tiny_http::Response::from_data(body)
        .with_status_code(status)
        .with_header(
            tiny_http::Header::from_bytes(&b"Content-Type"[..], content_type.as_bytes()).unwrap(),
        )
        .with_header(
            tiny_http::Header::from_bytes(&b"Access-Control-Allow-Origin"[..], b"*").unwrap(),
        );
    resp.boxed()
}

fn serve_index() -> tiny_http::ResponseBox {
    let html = include_str!("index.html");
    response(200, html.as_bytes(), "text/html; charset=utf-8")
}

fn serve_pending(state: &Arc<ApprovalState>) -> tiny_http::ResponseBox {
    let approvals = {
        let audit = state.audit.lock().unwrap();
        audit.get_pending_approvals().unwrap_or_default()
    };
    let json = serde_json::to_string(&approvals).unwrap_or_else(|_| "[]".to_string());
    response(200, json.as_bytes(), "application/json")
}

fn serve_approve(state: &Arc<ApprovalState>, action_id: &str) -> tiny_http::ResponseBox {
    let result = {
        let mut audit = state.audit.lock().unwrap();
        audit.approve_action(action_id)
    };
    match result {
        Ok(_) => {
            let json = serde_json::json!({"status": "approved", "action_id": action_id}).to_string();
            response(200, json.as_bytes(), "application/json")
        }
        Err(e) => {
            let json = serde_json::json!({"error": format!("{}", e)}).to_string();
            response(500, json.as_bytes(), "application/json")
        }
    }
}

fn serve_reject(state: &Arc<ApprovalState>, action_id: &str) -> tiny_http::ResponseBox {
    let result = {
        let mut audit = state.audit.lock().unwrap();
        audit.reject_action(action_id)
    };
    match result {
        Ok(_) => {
            let json = serde_json::json!({"status": "rejected", "action_id": action_id}).to_string();
            response(200, json.as_bytes(), "application/json")
        }
        Err(e) => {
            let json = serde_json::json!({"error": format!("{}", e)}).to_string();
            response(500, json.as_bytes(), "application/json")
        }
    }
}
