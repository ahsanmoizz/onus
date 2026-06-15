"""DEMO_ONLY end-to-end Onus reality demo.

This script uses the Python Guardian SDK against the Rust core. It performs
real pre-action checks, real file/API/SQLite side effects, real rollback, and
prints the audit log commands to verify the SQLite ledger. It uses a DEMO_ONLY
agent and local demo services; it is not a real LLM or production integration.
"""

from __future__ import annotations

import json
import os
import sqlite3
import sys
import threading
from http.server import BaseHTTPRequestHandler, HTTPServer
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
SDK = ROOT / "bindings" / "python" / "src"
sys.path.insert(0, str(SDK))

from onus import ChangeBudget, Guardian, OnusBlockError, RequiredEvidence, TaskContract  # noqa: E402


class DemoHandler(BaseHTTPRequestHandler):
    def do_GET(self) -> None:
        body = b'{"status":"ok","source":"local-demo-api"}'
        self.send_response(200)
        self.send_header("Content-Type", "application/json")
        self.send_header("Content-Length", str(len(body)))
        self.end_headers()
        self.wfile.write(body)

    def log_message(self, *_args: object) -> None:
        return


class DemoAgent:
    def __init__(self) -> None:
        self.corrections: list[str] = []

    def receive_correction(self, correction: str) -> None:
        self.corrections.append(correction)

    def run(self, guardian: Guardian) -> None:
        try:
            guardian.shell("rm -rf /important", execute=True)
        except OnusBlockError as exc:
            self.receive_correction(str(exc))
            guardian.file_write(
                "agent_output.txt",
                "Agent received correction and wrote a safe file instead.\n",
                tool="DemoAgent.Write",
            )


def find_onus_binary() -> Path:
    candidates = [
        ROOT / "target" / "debug" / "onus.exe",
        ROOT / "target" / "release" / "onus.exe",
        ROOT / "target" / "debug" / "onus",
        ROOT / "target" / "release" / "onus",
    ]
    for candidate in candidates:
        if candidate.exists():
            return candidate
    raise FileNotFoundError("Build Onus first with: cargo build")


def start_api() -> tuple[HTTPServer, int]:
    server = HTTPServer(("127.0.0.1", 0), DemoHandler)
    thread = threading.Thread(target=server.serve_forever, daemon=True)
    thread.start()
    return server, server.server_port


def main() -> None:
    onus_bin = find_onus_binary()
    rules = ROOT / "rules" / "default.toml"
    demo_dir = ROOT.parent / "reality_demo_workspace"
    demo_dir.mkdir(exist_ok=True)
    db = demo_dir / "reality_demo_audit.db"
    if db.exists():
        db.unlink()

    target_file = demo_dir / "agent_output.txt"
    target_file.write_text("original file contents\n", encoding="utf-8")

    sqlite_path = demo_dir / "demo.sqlite"
    if sqlite_path.exists():
        sqlite_path.unlink()
    con = sqlite3.connect(sqlite_path)
    con.execute("CREATE TABLE items (id INTEGER PRIMARY KEY, name TEXT)")
    con.commit()
    con.close()

    api_server, api_port = start_api()
    api_url = f"http://127.0.0.1:{api_port}/status"

    print("ONUS_REALITY_DEMO_START")
    print("demo_mode=DEMO_ONLY")
    print(f"workspace={demo_dir}")
    print(f"audit_db={db}")
    print("import_guardian=ok")

    with Guardian(
        task="Reality demo: intercept shell/file/API/SQLite actions",
        workspace_root=demo_dir,
        agent_name="DemoAgent",
        contract=TaskContract(
            session_id="demo-session",
            original_prompt="Reality demo: intercept shell/file/API/SQLite actions",
            normalized_objective="Demonstrate governed local file, API, SQLite, and shell actions.",
            allowed_paths=["agent_output.txt", "safe_after_correction.txt", "demo.sqlite"],
            allowed_resources=[api_url],
            protected_paths=[".env", "production/**"],
            protected_resources=["production-db"],
            required_evidence=[
                RequiredEvidence(
                    id="demo-output",
                    description="Reality demo output is printed for manual inspection.",
                    kind="demo",
                )
            ],
            forbidden_actions=["file_delete"],
            approval_required_actions=["db_migration"],
            change_budget=ChangeBudget(max_files_changed=3, max_actions=20),
            environment_identity="demo-local",
            policy_version="demo-policy-v1",
        ),
        bin_path=str(onus_bin),
        rules_path=str(rules),
        db_path=str(db),
    ) as guardian:
        before = target_file.read_text(encoding="utf-8")
        result = guardian.file_write(
            "agent_output.txt",
            "new proposed file contents\n",
            tool="DemoAgent.Write",
        )
        after = target_file.read_text(encoding="utf-8")
        print("file_write_verdict=" + result.decision)
        print("file_before=" + json.dumps(before))
        print("file_after=" + json.dumps(after))

        rollback = guardian.rollback_last()
        restored = target_file.read_text(encoding="utf-8")
        print("rollback_action=" + (rollback.action_type if rollback else "none"))
        print("file_restored=" + json.dumps(restored))

        try:
            api_result, api_body = guardian.api_call(api_url, tool="DemoAgent.ApiCall")
            print("api_call_verdict=" + api_result.decision)
            print("api_body=" + api_body.decode("utf-8"))
        except OnusBlockError as exc:
            print("api_call_blocked=" + exc.result.decision)
            print("api_call_approval_decision=" + str(exc.result.approval_decision))
            print("api_call_rule=" + str(exc.result.rule_id))

        db_result = guardian.db_execute(
            sqlite_path,
            "INSERT INTO items (name) VALUES (?)",
            ("real-row",),
            tool="DemoAgent.SQLite",
        )
        print("db_insert_verdict=" + db_result.decision)

        try:
            guardian.db_execute(sqlite_path, "DROP TABLE items;", tool="DemoAgent.SQLite")
        except OnusBlockError as exc:
            print("db_drop_blocked=" + exc.result.decision)
            print("db_drop_rule=" + str(exc.result.rule_id))

        agent = DemoAgent()
        guardian.run_agent(agent)
        print("correction_loop_received=" + json.dumps(agent.corrections))

    api_server.shutdown()
    print("ONUS_REALITY_DEMO_END")
    print(f"verify_command={onus_bin} verify --db {db}")
    print(f"log_command={onus_bin} log --db {db} --limit 20")
    print(f"session_command={onus_bin} session <session_id> --db {db}")


if __name__ == "__main__":
    main()
