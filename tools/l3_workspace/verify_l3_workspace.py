#!/usr/bin/env python3
"""Independent adversarial verifier for the Onus Linux L3 workspace.

This script is intentionally separate from unit tests. It launches the compiled
Onus binary, creates a real workspace, runs bypass attempts through
`onus run --isolate`, and fails unless each forbidden side effect is blocked.
"""

from __future__ import annotations

import argparse
import json
import os
import platform
import shutil
import socket
import sqlite3
import subprocess
import sys
import tempfile
from pathlib import Path


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--onus-bin", default=os.environ.get("ONUS_BIN", "onus"))
    parser.add_argument("--json", action="store_true")
    args = parser.parse_args()

    result = {
        "verifier": "l3_workspace",
        "platform": platform.platform(),
        "status": "unverified",
        "tests": [],
        "limitations": [],
    }

    if platform.system() != "Linux":
        result["status"] = "unverifiable"
        result["limitations"].append("Linux is required for namespace/bubblewrap L3 verification.")
        emit(result, args.json)
        return 0

    if shutil.which("bwrap") is None:
        result["status"] = "unverifiable"
        result["limitations"].append("bubblewrap (`bwrap`) is required and was not found.")
        emit(result, args.json)
        return 0

    onus = shutil.which(args.onus_bin) or args.onus_bin
    with tempfile.TemporaryDirectory(prefix="onus-l3-verify-") as tmp:
        root = Path(tmp)
        data_dir = root / "onus-data"
        repo = root / "repo"
        repo.mkdir()
        (repo / "README.md").write_text("original\n", encoding="utf-8")
        (repo / "host-secret.txt").write_text("HOST_SECRET_DO_NOT_READ\n", encoding="utf-8")

        host_secret = root / "outside-secret.txt"
        host_secret.write_text("OUTSIDE_SECRET_DO_NOT_READ\n", encoding="utf-8")
        host_db = root / "outside.db"
        con = sqlite3.connect(host_db)
        try:
            con.execute("CREATE TABLE secrets(value TEXT)")
            con.execute("INSERT INTO secrets(value) VALUES ('db-secret')")
            con.commit()
        finally:
            con.close()

        env = os.environ.copy()
        env["ONUS_DATA_DIR"] = str(data_dir)
        env["AWS_SECRET_ACCESS_KEY"] = "SHOULD_NOT_LEAK_TO_L3"
        env["DATABASE_URL"] = f"sqlite:///{host_db}"
        env["ONUS_CLAUDE_HOOK_DISABLED"] = "1"

        session = "verify-l3"
        created = run(
            [onus, "workspace", "create", "--repo", str(repo), "--session", session],
            env,
        )
        if created.returncode != 0:
            result["status"] = "failed"
            result["limitations"].append(f"workspace create failed: {created.stderr.strip()}")
            emit(result, args.json)
            return 1
        metadata = json.loads(created.stdout)
        worktree = Path(metadata["worktree"])

        cases = [
            {
                "name": "filesystem_write_to_original_repo",
                "expect": "blocked",
                "command": [
                    "python3",
                    "-c",
                    "open('/original/blocked.txt','w').write('x')",
                ],
                "host_assert": lambda: not (repo / "blocked.txt").exists(),
            },
            {
                "name": "subprocess_inherits_filesystem_boundary",
                "expect": "blocked",
                "command": [
                    "sh",
                    "-c",
                    "python3 -c \"import subprocess; subprocess.run(['sh','-c','echo x > /original/subprocess.txt'], check=True)\"",
                ],
                "host_assert": lambda: not (repo / "subprocess.txt").exists(),
            },
            {
                "name": "raw_socket_network_egress",
                "expect": "blocked",
                "command": [
                    "python3",
                    "-c",
                    "import socket; s=socket.create_connection(('1.1.1.1',80), 2); s.close()",
                ],
            },
            {
                "name": "urllib_http_egress",
                "expect": "blocked",
                "command": [
                    "python3",
                    "-c",
                    "import urllib.request; urllib.request.urlopen('http://example.com', timeout=2).read(1)",
                ],
            },
            {
                "name": "requests_http_egress",
                "expect": "blocked_or_unavailable",
                "command": [
                    "python3",
                    "-c",
                    "import requests; requests.get('http://example.com', timeout=2)",
                ],
            },
            {
                "name": "httpx_http_egress",
                "expect": "blocked_or_unavailable",
                "command": [
                    "python3",
                    "-c",
                    "import httpx; httpx.get('http://example.com', timeout=2)",
                ],
            },
            {
                "name": "curl_http_egress",
                "expect": "blocked_or_unavailable",
                "command": [
                    "sh",
                    "-c",
                    "command -v curl >/dev/null || { echo 'curl not found' >&2; exit 127; }; curl -m 2 http://example.com >/tmp/curl.out",
                ],
            },
            {
                "name": "environment_secret_read",
                "expect": "blocked",
                "command": [
                    "sh",
                    "-c",
                    "env | grep -E 'AWS_SECRET_ACCESS_KEY|DATABASE_URL|ONUS_CLAUDE_HOOK_DISABLED'",
                ],
            },
            {
                "name": "direct_sqlite_host_db_access",
                "expect": "blocked",
                "command": [
                    "python3",
                    "-c",
                    f"import sqlite3; sqlite3.connect({str(host_db)!r}).execute('select * from secrets').fetchall()",
                ],
            },
            {
                "name": "host_file_read",
                "expect": "blocked",
                "command": [
                    "python3",
                    "-c",
                    f"print(open({str(host_secret)!r}).read())",
                ],
            },
            {
                "name": "attempt_disable_onus_env",
                "expect": "blocked",
                "command": [
                    "sh",
                    "-c",
                    "test \"$ONUS_CLAUDE_HOOK_DISABLED\" = 1",
                ],
            },
            {
                "name": "writable_session_worktree",
                "expect": "allowed",
                "command": [
                    "sh",
                    "-c",
                    "echo allowed > /workspace/allowed.txt",
                ],
                "host_assert": lambda: (worktree / "allowed.txt").read_text(encoding="utf-8").strip()
                == "allowed",
            },
        ]

        failures = []
        for case in cases:
            proc = run(
                [onus, "run", "--isolate", "--workspace", session, "--", *case["command"]],
                env,
            )
            allowed = proc.returncode == 0
            unavailable = "No module named" in proc.stderr or "not found" in proc.stderr.lower()
            expected = case["expect"]
            passed = False
            if expected == "allowed":
                passed = allowed
            elif expected == "blocked":
                passed = not allowed
            elif expected == "blocked_or_unavailable":
                passed = not allowed and not unavailable

            host_ok = True
            if "host_assert" in case:
                try:
                    host_ok = bool(case["host_assert"]())
                except Exception:
                    host_ok = False
            passed = passed and host_ok
            entry = {
                "name": case["name"],
                "passed": passed,
                "exit_code": proc.returncode,
                "stdout": proc.stdout[-400:],
                "stderr": proc.stderr[-400:],
                "host_assertion_passed": host_ok,
            }
            result["tests"].append(entry)
            if not passed:
                failures.append(case["name"])

        inspect = run([onus, "workspace", "inspect", "--session", session], env)
        metadata_passed = False
        if inspect.returncode == 0:
            inspected = json.loads(inspect.stdout)
            metadata_passed = (
                inspected.get("boundary_verified") is True
                and inspected.get("isolation_level") == "L3_LINUX_BUBBLEWRAP"
                and inspected.get("enforcement_label") == "L3_LINUX_WORKSPACE_RUNTIME_VERIFIED"
                and inspected.get("last_isolated_run_at_unix") is not None
            )
        result["tests"].append(
            {
                "name": "runtime_verified_metadata",
                "passed": metadata_passed,
                "exit_code": inspect.returncode,
                "stdout": inspect.stdout[-400:],
                "stderr": inspect.stderr[-400:],
                "host_assertion_passed": metadata_passed,
            }
        )
        if not metadata_passed:
            failures.append("runtime_verified_metadata")

        if failures:
            result["status"] = "failed"
            result["failures"] = failures
            emit(result, args.json)
            return 1

        result["status"] = "passed"
        emit(result, args.json)
        return 0


def run(command: list[str], env: dict[str, str]) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        command,
        capture_output=True,
        text=True,
        env=env,
        timeout=30,
    )


def emit(result: dict, as_json: bool) -> None:
    if as_json:
        print(json.dumps(result, indent=2))
    else:
        print(f"L3 workspace verifier status: {result['status']}")
        for limitation in result.get("limitations", []):
            print(f"limitation: {limitation}")
        for test in result.get("tests", []):
            state = "PASS" if test["passed"] else "FAIL"
            print(f"{state} {test['name']} exit={test['exit_code']}")


if __name__ == "__main__":
    raise SystemExit(main())
