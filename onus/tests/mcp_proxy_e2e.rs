use onus_core::audit::AuditTrail;
use onus_core::task_contract::{ChangeBudget, TaskContract};
use serde_json::Value;
use std::io::{BufRead, BufReader, Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};

fn temp_path(name: &str) -> PathBuf {
    std::env::temp_dir().join(format!("onus-{name}-{}", uuid::Uuid::new_v4()))
}

fn write_message(stdin: &mut ChildStdin, value: &Value) {
    let body = value.to_string();
    write!(stdin, "Content-Length: {}\r\n\r\n{}", body.len(), body).unwrap();
    stdin.flush().unwrap();
}

fn read_message(stdout: &mut BufReader<ChildStdout>) -> Value {
    let mut content_length = None;
    loop {
        let mut line = String::new();
        stdout.read_line(&mut line).unwrap();
        let trimmed = line.trim();
        if trimmed.is_empty() {
            break;
        }
        if let Some(value) = trimmed.strip_prefix("Content-Length: ") {
            content_length = Some(value.parse::<usize>().unwrap());
        }
    }
    let len = content_length.expect("missing Content-Length header");
    let mut body = vec![0u8; len];
    stdout.read_exact(&mut body).unwrap();
    serde_json::from_slice(&body).unwrap()
}

fn write_fake_mcp_server(path: &Path) {
    let script = r#"
import json
import sys

side_effect = sys.argv[1]

def read_msg():
    content_length = None
    while True:
        line = sys.stdin.buffer.readline()
        if not line:
            return None
        line = line.decode("utf-8").strip()
        if line == "":
            break
        if line.lower().startswith("content-length:"):
            content_length = int(line.split(":", 1)[1].strip())
    if content_length is None:
        return None
    return json.loads(sys.stdin.buffer.read(content_length).decode("utf-8"))

def write_msg(obj):
    body = json.dumps(obj, separators=(",", ":")).encode("utf-8")
    sys.stdout.buffer.write(f"Content-Length: {len(body)}\r\n\r\n".encode("ascii"))
    sys.stdout.buffer.write(body)
    sys.stdout.buffer.flush()

while True:
    msg = read_msg()
    if msg is None:
        break
    method = msg.get("method")
    if method == "initialize":
        write_msg({
            "jsonrpc": "2.0",
            "id": msg.get("id"),
            "result": {
                "protocolVersion": "2024-11-05",
                "capabilities": {"tools": {}},
                "serverInfo": {"name": "onus-e2e-fake-mcp", "version": "0.0.1"}
            }
        })
    elif method == "tools/list":
        write_msg({
            "jsonrpc": "2.0",
            "id": msg.get("id"),
            "result": {
                "tools": [{
                    "name": "side.write",
                    "description": "Writes a side effect file",
                    "inputSchema": {
                        "type": "object",
                        "properties": {"content": {"type": "string"}}
                    }
                }]
            }
        })
    elif method == "tools/call":
        args = msg.get("params", {}).get("arguments", {})
        with open(side_effect, "w", encoding="utf-8") as handle:
            handle.write(args.get("content", ""))
        write_msg({
            "jsonrpc": "2.0",
            "id": msg.get("id"),
            "result": {"content": [{"type": "text", "text": "side effect written"}]}
        })
    else:
        write_msg({"jsonrpc": "2.0", "id": msg.get("id"), "result": {}})
"#;
    std::fs::write(path, script).unwrap();
}

fn create_session_contract(db_path: &Path, session_id: &str, workspace: &Path) {
    let mut audit = AuditTrail::open(db_path).unwrap();
    audit
        .start_session(
            session_id,
            "mcp-e2e-test",
            Some("0.0.1"),
            "Runtime-test routed MCP calls through Onus",
            &workspace.to_string_lossy(),
        )
        .unwrap();
    audit
        .save_task_contract(&TaskContract {
            schema_version: 1,
            session_id: session_id.to_string(),
            original_prompt: "Use a routed MCP tool safely.".to_string(),
            normalized_objective: "Runtime-test a routed MCP tool through Onus.".to_string(),
            allowed_paths: vec![],
            allowed_resources: vec!["local_workspace".to_string(), "mcp".to_string()],
            protected_paths: vec![".env".to_string(), "**/.env".to_string()],
            protected_resources: vec!["credentials".to_string()],
            required_evidence: vec![],
            forbidden_actions: vec![],
            approval_required_actions: vec![],
            change_budget: ChangeBudget {
                max_files_changed: 10,
                max_actions: 20,
            },
            environment_identity: "mcp-e2e-local".to_string(),
            policy_version: "test".to_string(),
            canonical_hash: String::new(),
        })
        .unwrap();
}

fn spawn_proxy(
    onus_bin: &str,
    db_path: &Path,
    session_id: &str,
    fake_server: &Path,
    side_effect: &Path,
) -> (Child, ChildStdin, BufReader<ChildStdout>) {
    let mut child = Command::new(onus_bin)
        .args([
            "mcp-proxy",
            "--experimental",
            "--server",
            "python",
            "--db-path",
            &db_path.to_string_lossy(),
            "--approval-port",
            "0",
            "--session-id",
            session_id,
            "--response-timeout-ms",
            "10000",
            "--max-response-bytes",
            "65536",
            "--",
            &fake_server.to_string_lossy(),
            &side_effect.to_string_lossy(),
        ])
        .env("ONUS_SEMANTIC_PROVIDER", "disabled")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let stdin = child.stdin.take().unwrap();
    let stdout = BufReader::new(child.stdout.take().unwrap());
    (child, stdin, stdout)
}

#[test]
fn routed_mcp_proxy_blocks_denied_side_effect_before_upstream_execution() {
    let root = temp_path("mcp-proxy-e2e");
    std::fs::create_dir_all(&root).unwrap();
    let db_path = root.join("audit.db");
    let fake_server = root.join("fake_mcp_server.py");
    let side_effect = root.join("side-effect.txt");
    let session_id = "mcp-e2e-session";
    write_fake_mcp_server(&fake_server);
    create_session_contract(&db_path, session_id, &root);

    let onus_bin = env!("CARGO_BIN_EXE_onus");
    let (mut child, mut stdin, mut stdout) =
        spawn_proxy(onus_bin, &db_path, session_id, &fake_server, &side_effect);

    write_message(
        &mut stdin,
        &serde_json::json!({"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}),
    );
    let initialize = read_message(&mut stdout);
    assert_eq!(
        initialize["result"]["serverInfo"]["name"],
        "onus-e2e-fake-mcp"
    );
    assert_eq!(
        initialize["result"]["_onus_gateway"]["gateway"],
        "onus-mcp-proxy"
    );

    write_message(
        &mut stdin,
        &serde_json::json!({"jsonrpc":"2.0","id":2,"method":"tools/list","params":{}}),
    );
    let list = read_message(&mut stdout);
    assert_eq!(list["result"]["tools"][0]["name"], "side.write");

    write_message(
        &mut stdin,
        &serde_json::json!({
            "jsonrpc":"2.0",
            "id":3,
            "method":"tools/call",
            "params":{"name":"side.write","arguments":{"content":"safe write"}}
        }),
    );
    let allowed = read_message(&mut stdout);
    assert_eq!(
        allowed["result"]["content"][0]["text"],
        "side effect written"
    );
    assert_eq!(
        allowed["result"]["_onus_receipt"]["decision"], "allow",
        "{allowed}"
    );
    assert_eq!(std::fs::read_to_string(&side_effect).unwrap(), "safe write");
    std::fs::remove_file(&side_effect).unwrap();

    write_message(
        &mut stdin,
        &serde_json::json!({
            "jsonrpc":"2.0",
            "id":4,
            "method":"tools/call",
            "params":{"name":"side.write","arguments":{"content":"please rm -rf /important"}}
        }),
    );
    let blocked = read_message(&mut stdout);
    assert_eq!(blocked["error"]["code"], -32001);
    assert!(
        blocked["error"]["message"]
            .as_str()
            .unwrap()
            .contains("MCP_SAFETY_001"),
        "{blocked}"
    );
    assert_eq!(blocked["error"]["data"]["decision"], "block");
    assert!(
        !side_effect.exists(),
        "blocked MCP call reached upstream server and wrote a side effect"
    );

    drop(stdin);
    let _ = child.kill();
    let _ = child.wait();
    let _ = std::fs::remove_dir_all(root);
}
