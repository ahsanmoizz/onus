#!/usr/bin/env python3
"""Independent verifier for the narrow Onus L4 authority proof."""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import sqlite3
import subprocess
import sys
import tempfile
import time
from pathlib import Path


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--onus-bin", default=os.environ.get("ONUS_BIN", "onus"))
    parser.add_argument("--json", action="store_true")
    args = parser.parse_args()

    result = {
        "verifier": "l4_authority",
        "operation": "disposable_sqlite_insert_l4_item",
        "status": "failed",
        "tests": [],
        "limitations": [
            "This verifies one disposable SQLite authority only; it is not a production credential broker."
        ],
    }

    with tempfile.TemporaryDirectory(prefix="onus-l4-authority-") as tmp:
        root = Path(tmp)
        data = root / "onus-data"
        db = root / "staging.sqlite"
        env = os.environ.copy()
        env["ONUS_DATA_DIR"] = str(data)
        authority = "l4-proof"
        environment = "staging-disposable"

        init = run(
            [
                args.onus_bin,
                "authority",
                "init-disposable-db",
                "--authority",
                authority,
                "--db",
                str(db),
                "--environment",
                environment,
            ],
            env,
        )
        metadata = json.loads(init.stdout) if init.returncode == 0 else {}
        secret_path = data / "authority" / authority / "authority.secret"
        raw_secret = secret_path.read_text(encoding="utf-8") if secret_path.exists() else ""
        add(result, "init_disposable_authority", init.returncode == 0 and raw_secret, init)
        add(
            result,
            "raw_credential_not_returned_on_init",
            bool(raw_secret) and raw_secret not in init.stdout and "credential_hash" in init.stdout,
            init,
        )
        add(
            result,
            "verified_environment_identity",
            metadata.get("environment_identity") == environment
            and metadata.get("l4_claim") == "ONUS_CONTROLLED_AUTHORITY_DISPOSABLE_SQLITE",
            init,
        )

        payload = root / "payload.json"
        write_payload(payload, environment, "row-1", "first value")
        altered = root / "altered.json"
        write_payload(altered, environment, "row-1", "altered value")

        no_human = run(
            [
                args.onus_bin,
                "authority",
                "authorize",
                "--authority",
                authority,
                "--session",
                "session-1",
                "--payload",
                str(payload),
                "--approver",
                "reviewer",
            ],
            env,
        )
        add(result, "human_approval_required", no_human.returncode != 0, no_human)

        auth = run(
            [
                args.onus_bin,
                "authority",
                "authorize",
                "--authority",
                authority,
                "--session",
                "session-1",
                "--payload",
                str(payload),
                "--approver",
                "reviewer",
                "--ttl-seconds",
                "60",
                "--human-approved",
            ],
            env,
        )
        auth_body = json.loads(auth.stdout) if auth.returncode == 0 else {}
        token = auth_body.get("capability", "")
        add(
            result,
            "short_lived_scoped_capability_issued",
            auth.returncode == 0
            and token.startswith("onus-cap-")
            and auth_body.get("environment_identity") == environment
            and auth_body.get("approver") == "reviewer",
            auth,
        )
        add(
            result,
            "raw_credential_not_returned_on_authorize",
            bool(raw_secret) and raw_secret not in auth.stdout and token != raw_secret,
            auth,
        )

        altered_attempt = run(
            [
                args.onus_bin,
                "authority",
                "execute",
                "--authority",
                authority,
                "--capability",
                token,
                "--payload",
                str(altered),
            ],
            env,
        )
        add(result, "denied_altered_payload", altered_attempt.returncode != 0, altered_attempt)

        executed = run(
            [
                args.onus_bin,
                "authority",
                "execute",
                "--authority",
                authority,
                "--capability",
                token,
                "--payload",
                str(payload),
            ],
            env,
        )
        receipt = json.loads(executed.stdout) if executed.returncode == 0 else {}
        receipt_id = receipt.get("receipt_id", "")
        add(
            result,
            "broker_executed_exact_authorized_action",
            executed.returncode == 0 and row_value(db, "row-1") == "first value",
            executed,
        )
        add(
            result,
            "audit_receipt_without_raw_credential",
            executed.returncode == 0
            and receipt.get("decision") == "EXECUTED"
            and receipt.get("canonical_payload_hash") == auth_body.get("canonical_payload_hash")
            and raw_secret not in executed.stdout,
            executed,
        )

        reuse = run(
            [
                args.onus_bin,
                "authority",
                "execute",
                "--authority",
                authority,
                "--capability",
                token,
                "--payload",
                str(payload),
            ],
            env,
        )
        add(result, "denied_reuse", reuse.returncode != 0, reuse)

        revoke_payload = root / "revoke.json"
        write_payload(revoke_payload, environment, "row-revoked", "revoked value")
        rev_auth = authorize(args.onus_bin, env, authority, "session-2", revoke_payload, "reviewer", 60)
        rev_token = json.loads(rev_auth.stdout).get("capability", "")
        revoked = run(
            [args.onus_bin, "authority", "revoke", "--authority", authority, "--capability", rev_token],
            env,
        )
        revoked_exec = run(
            [
                args.onus_bin,
                "authority",
                "execute",
                "--authority",
                authority,
                "--capability",
                rev_token,
                "--payload",
                str(revoke_payload),
            ],
            env,
        )
        add(result, "revocation_denies_execution", revoked.returncode == 0 and revoked_exec.returncode != 0, revoked_exec)

        expired_payload = root / "expired.json"
        write_payload(expired_payload, environment, "row-expired", "expired value")
        exp_auth = authorize(args.onus_bin, env, authority, "session-3", expired_payload, "reviewer", 1)
        exp_token = json.loads(exp_auth.stdout).get("capability", "")
        time.sleep(2)
        expired_exec = run(
            [
                args.onus_bin,
                "authority",
                "execute",
                "--authority",
                authority,
                "--capability",
                exp_token,
                "--payload",
                str(expired_payload),
            ],
            env,
        )
        add(result, "expiry_denies_execution", expired_exec.returncode != 0, expired_exec)

        compensation = run(
            [
                args.onus_bin,
                "authority",
                "compensate",
                "--authority",
                authority,
                "--receipt",
                receipt_id,
            ],
            env,
        )
        add(
            result,
            "compensation_deletes_disposable_row",
            compensation.returncode == 0 and row_value(db, "row-1") is None,
            compensation,
        )

        receipts = run([args.onus_bin, "authority", "receipts", "--authority", authority], env)
        receipt_list = json.loads(receipts.stdout) if receipts.returncode == 0 else []
        caps_path = data / "authority" / authority / "capabilities.json"
        meta_path = data / "authority" / authority / "authority.json"
        stored_without_secret = (
            bool(raw_secret)
            and raw_secret not in receipts.stdout
            and raw_secret not in caps_path.read_text(encoding="utf-8")
            and raw_secret not in meta_path.read_text(encoding="utf-8")
        )
        add(result, "stored_receipts_and_capabilities_do_not_contain_raw_credential", stored_without_secret, receipts)
        add(result, "receipt_hash_chain_verifies", verify_receipt_chain(receipt_list), receipts)

    result["status"] = "passed" if all(t["passed"] for t in result["tests"]) else "failed"
    emit(result, args.json)
    return 0 if result["status"] == "passed" else 1


def authorize(onus: str, env: dict[str, str], authority: str, session: str, payload: Path, approver: str, ttl: int):
    return run(
        [
            onus,
            "authority",
            "authorize",
            "--authority",
            authority,
            "--session",
            session,
            "--payload",
            str(payload),
            "--approver",
            approver,
            "--ttl-seconds",
            str(ttl),
            "--human-approved",
        ],
        env,
    )


def write_payload(path: Path, environment: str, row_id: str, value: str) -> None:
    path.write_text(
        json.dumps(
            {
                "operation": "insert_l4_item",
                "environment_identity": environment,
                "row_id": row_id,
                "value": value,
            }
        ),
        encoding="utf-8",
    )


def row_value(db: Path, row_id: str) -> str | None:
    con = sqlite3.connect(db)
    try:
        row = con.execute("SELECT value FROM l4_items WHERE row_id = ?", (row_id,)).fetchone()
        return row[0] if row else None
    finally:
        con.close()


def verify_receipt_chain(receipts: list[dict]) -> bool:
    previous = ""
    if not receipts:
        return False
    for receipt in receipts:
        if receipt.get("previous_receipt_hash") != previous:
            return False
        expected = receipt.get("receipt_hash")
        mutable = dict(receipt)
        mutable["receipt_hash"] = ""
        canonical = json.dumps(mutable, sort_keys=True, separators=(",", ":"))
        actual = hashlib.sha256(canonical.encode("utf-8")).hexdigest()
        if actual != expected:
            return False
        previous = expected
    return True


def run(command: list[str], env: dict[str, str]) -> subprocess.CompletedProcess[str]:
    return subprocess.run(command, capture_output=True, text=True, env=env, timeout=30)


def add(result: dict, name: str, passed: bool, proc: subprocess.CompletedProcess[str]) -> None:
    result["tests"].append(
        {
            "name": name,
            "passed": bool(passed),
            "exit_code": proc.returncode,
            "stdout": proc.stdout[-600:],
            "stderr": proc.stderr[-600:],
        }
    )


def emit(result: dict, as_json: bool) -> None:
    if as_json:
        print(json.dumps(result, indent=2))
        return
    print(f"L4 authority verifier status: {result['status']}")
    for test in result["tests"]:
        print(("PASS" if test["passed"] else "FAIL") + " " + test["name"])


if __name__ == "__main__":
    raise SystemExit(main())
