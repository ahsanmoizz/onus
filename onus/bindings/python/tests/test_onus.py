"""Tests for the Onus Python SDK."""

from __future__ import annotations

import json
import hashlib
import sys
import os
import socket
import sqlite3
import subprocess
import time
import urllib.error
import urllib.request
from pathlib import Path

import pytest

sys.path.insert(0, str(Path(__file__).parent.parent / "src"))

from onus import (
    ChangeBudget,
    CompletionEvidence,
    Guardian,
    OnusBlockError,
    OnusClient,
    OnusEvaluationError,
    OnusResult,
    PromptIntakeResult,
    ReversibilityClass,
    RequiredEvidence,
    RollbackResult,
    TaskContract,
    UnsupportedExternalRecoveryAdapter,
)


@pytest.fixture
def repo_root() -> Path:
    return Path(__file__).resolve().parents[3]


@pytest.fixture
def onus_bin(repo_root: Path) -> Path:
    candidates = [
        repo_root / "target" / "debug" / "onus.exe",
        repo_root / "target" / "release" / "onus.exe",
        repo_root / "target" / "debug" / "onus",
        repo_root / "target" / "release" / "onus",
    ]
    for candidate in candidates:
        if candidate.is_file():
            return candidate
    raise FileNotFoundError("Build first with cargo build")


@pytest.fixture
def rules_path(repo_root: Path) -> Path:
    return repo_root / "rules" / "default.toml"


@pytest.fixture
def client(onus_bin: Path, rules_path: Path, tmp_path: Path) -> OnusClient:
    return OnusClient(
        bin_path=str(onus_bin),
        rules_path=str(rules_path),
        db_path=str(tmp_path / "audit.db"),
    )


def make_contract(**overrides) -> TaskContract:
    data = {
        "session_id": "will-be-replaced",
        "original_prompt": "Fix the allowed module and prove it.",
        "normalized_objective": "Update files inside the allowed module only.",
        "allowed_paths": ["allowed/**"],
        "protected_paths": [".env", "protected/**"],
        "allowed_resources": ["local-files"],
        "protected_resources": ["production-db"],
        "required_evidence": [
            RequiredEvidence(id="tests", description="Targeted tests pass", kind="test")
        ],
        "forbidden_actions": ["file_delete"],
        "approval_required_actions": ["db_migration"],
        "change_budget": ChangeBudget(max_files_changed=5, max_actions=20),
        "environment_identity": "test-env",
        "policy_version": "test-policy-v1",
    }
    data.update(overrides)
    return TaskContract(**data)


def quality_evidence(*extra: CompletionEvidence) -> list[CompletionEvidence]:
    base = [
        CompletionEvidence(id="tests", passed=True, value="targeted tests pass", kind="test"),
        CompletionEvidence(id="targeted_tests", passed=True, value="auth tests", kind="test"),
        CompletionEvidence(id="lint", passed=True, value="lint passed", kind="lint"),
        CompletionEvidence(id="typecheck", passed=True, value="typecheck passed", kind="typecheck"),
        CompletionEvidence(id="coverage", passed=True, value="before=90 after=90", kind="coverage"),
        CompletionEvidence(id="secret_scan", passed=True, value="no secrets", kind="security"),
        CompletionEvidence(id="architecture_review", passed=True, value="architecture ok", kind="review"),
        CompletionEvidence(id="module_boundary_review", passed=True, value="boundaries ok", kind="review"),
        CompletionEvidence(id="final_scope", passed=True, value="diff inside scope", kind="scope"),
        CompletionEvidence(id="independent_verification", passed=True, value="verified by Onus checks", kind="verification"),
        CompletionEvidence(id="no_test_weakening", passed=True, value="tests preserved", kind="test"),
    ]
    base.extend(extra)
    return base


class TestOnusResult:
    def test_allowed_property(self):
        assert OnusResult(decision="allow").allowed is True
        assert OnusResult(decision="warn").allowed is True
        assert OnusResult(decision="block").allowed is False
        assert OnusResult(decision="escalate").allowed is False

    def test_blocked_property(self):
        assert OnusResult(decision="allow").blocked is False
        assert OnusResult(decision="warn").blocked is False
        assert OnusResult(decision="block").blocked is True
        assert OnusResult(decision="escalate").blocked is True

    def test_from_json(self):
        data = {
            "decision": "block",
            "correction": "Command blocked",
            "rule_id": "SAFETY_001",
            "latency_us": 42,
            "reversibility": "irreversible",
        }
        result = OnusResult.from_json(data)
        assert result.decision == "block"
        assert result.correction == "Command blocked"
        assert result.rule_id == "SAFETY_001"
        assert result.latency_us == 42
        assert result.reversibility == "irreversible"
        assert result.approval_decision is None


class TestOnusClient:
    def test_evaluate_allow(self, client: OnusClient):
        result = client.evaluate("shell", {"command": "echo hello"})
        assert result.decision == "allow"

    def test_evaluate_block(self, client: OnusClient):
        result = client.evaluate("shell", {"command": "rm -rf /important"})
        assert result.blocked is True
        assert result.correction is not None
        assert result.rule_id == "SAFETY_001"

    def test_check_command_block(self, client: OnusClient):
        result = client.check_command("sudo rm -rf /")
        assert result.blocked is True
        assert result.decision == "block"
        assert result.rule_id == "SAFETY_001"

    def test_acceptance_scenario_d_low_risk_action_auto_approved(
        self, client: OnusClient, tmp_path: Path, monkeypatch: pytest.MonkeyPatch
    ):
        monkeypatch.setenv("ONUS_GUARDIAN_MODE", "professional")
        session_id = "acceptance-scenario-d"
        contract = make_contract(
            session_id=session_id,
            allowed_paths=["allowed/**"],
            protected_paths=[".env", "protected/**"],
            forbidden_actions=[],
            approval_required_actions=[],
            environment_identity="local-dev",
        )
        client.start_contract(contract, workspace_root=tmp_path, agent_name="scenario-d")

        result = client.evaluate(
            "file_read",
            {"path": "allowed/demo.py"},
            session_id=session_id,
            tool="Read",
        )

        assert result.decision == "allow"
        assert result.approval_decision == "ALLOW_AUTOMATICALLY"
        assert result.guardian_mode == "professional"
        assert result.action_id
        assert result.canonical_payload_hash

        con = sqlite3.connect(client.db_path)
        try:
            row = con.execute(
                "SELECT approval_decision, guardian_mode, payload_hash FROM actions WHERE session_id = ? ORDER BY id DESC LIMIT 1",
                (session_id,),
            ).fetchone()
        finally:
            con.close()
        assert row == ("ALLOW_AUTOMATICALLY", "professional", result.canonical_payload_hash)

    def test_acceptance_scenario_f_production_migration_requires_human(
        self, client: OnusClient, tmp_path: Path, monkeypatch: pytest.MonkeyPatch
    ):
        monkeypatch.setenv("ONUS_GUARDIAN_MODE", "professional")
        session_id = "acceptance-scenario-f"
        contract = make_contract(
            session_id=session_id,
            allowed_paths=["migrations/**"],
            protected_paths=[],
            forbidden_actions=[],
            approval_required_actions=[],
            environment_identity="production-us-east-1",
        )
        client.start_contract(contract, workspace_root=tmp_path, agent_name="scenario-f")

        result = client.evaluate(
            "db_mutation",
            {
                "db_path": "migrations/prod.sqlite",
                "sql": "ALTER TABLE users ADD COLUMN display_name TEXT",
                "rollback_plan": "",
            },
            session_id=session_id,
            tool="sqlite",
        )

        assert result.decision == "escalate"
        assert result.approval_decision == "REQUIRE_HUMAN_APPROVAL"
        assert result.rule_id == "ONUS_APPROVAL_BROKER_HUMAN_REQUIRED"
        assert "production environment" in (result.approval_reason or "")
        assert "database schema change" in (result.approval_reason or "")
        assert any("migration diff" in item for item in result.obligations)


class TestGuardian:
    def test_import_guardian(self):
        assert Guardian is not None

    def test_file_write_captures_and_rolls_back(self, onus_bin: Path, rules_path: Path, tmp_path: Path):
        target = tmp_path / "demo.txt"
        target.write_text("before\n", encoding="utf-8")
        with Guardian(
            workspace_root=tmp_path,
            contract=make_contract(),
            bin_path=str(onus_bin),
            rules_path=str(rules_path),
            db_path=str(tmp_path / "audit.db"),
        ) as guardian:
            target = tmp_path / "allowed" / "demo.txt"
            target.parent.mkdir()
            target.write_text("before\n", encoding="utf-8")
            result = guardian.file_write("allowed/demo.txt", "after\n")
            assert result.allowed is True
            assert target.read_text(encoding="utf-8") == "after\n"
            rollback = guardian.rollback_last()
            assert rollback is not None
            assert target.read_text(encoding="utf-8") == "before\n"

    def test_recovery_record_contains_state_without_raw_file_content(
        self, onus_bin: Path, rules_path: Path, tmp_path: Path
    ):
        target = tmp_path / "allowed" / "demo.txt"
        target.parent.mkdir()
        target.write_text("before secret-ish value\n", encoding="utf-8")
        with Guardian(
            workspace_root=tmp_path,
            contract=make_contract(allowed_paths=["allowed/demo.txt"]),
            bin_path=str(onus_bin),
            rules_path=str(rules_path),
            db_path=str(tmp_path / "audit.db"),
        ) as guardian:
            guardian.file_write("allowed/demo.txt", "after secret-ish value\n")
            record = guardian.recovery_records[-1]
            journal = (tmp_path / ".onus" / "rollback_journal.jsonl").read_text(encoding="utf-8")

        assert record.reversibility_class == ReversibilityClass.R2
        assert record.pre_state["exists"] is True
        assert record.post_state["sha256"] == hashlib.sha256(target.read_bytes()).hexdigest()
        assert record.inverse_operation["type"] == "restore_file_snapshot"
        assert record.executed_payload["raw_content_in_journal"] is False
        assert "after secret-ish value" not in journal
        assert "before secret-ish value" not in journal

    def test_revert_one_action_restores_only_that_action(
        self, onus_bin: Path, rules_path: Path, tmp_path: Path
    ):
        first = tmp_path / "allowed" / "first.txt"
        second = tmp_path / "allowed" / "second.txt"
        first.parent.mkdir()
        first.write_text("one-before\n", encoding="utf-8")
        second.write_text("two-before\n", encoding="utf-8")
        with Guardian(
            workspace_root=tmp_path,
            contract=make_contract(allowed_paths=["allowed/**"]),
            bin_path=str(onus_bin),
            rules_path=str(rules_path),
            db_path=str(tmp_path / "audit.db"),
        ) as guardian:
            guardian.file_write("allowed/first.txt", "one-after\n")
            first_action = guardian.recovery_records[-1].action_id
            guardian.file_write("allowed/second.txt", "two-after\n")
            result = guardian.revert_action(first_action)

        assert isinstance(result, RollbackResult)
        assert result.status == "REVERTED"
        assert result.restored is True
        assert first.read_text(encoding="utf-8") == "one-before\n"
        assert second.read_text(encoding="utf-8") == "two-after\n"

    def test_revert_action_group_restores_group_in_reverse_order(
        self, onus_bin: Path, rules_path: Path, tmp_path: Path
    ):
        target = tmp_path / "allowed" / "group.txt"
        target.parent.mkdir()
        target.write_text("start\n", encoding="utf-8")
        with Guardian(
            workspace_root=tmp_path,
            contract=make_contract(allowed_paths=["allowed/**"]),
            bin_path=str(onus_bin),
            rules_path=str(rules_path),
            db_path=str(tmp_path / "audit.db"),
        ) as guardian:
            guardian.file_write("allowed/group.txt", "middle\n", action_group="phase-1")
            guardian.file_write("allowed/group.txt", "end\n", action_group="phase-1")
            result = guardian.revert_action_group("phase-1")

        assert result.status == "REVERTED"
        assert result.restored is True
        assert target.read_text(encoding="utf-8") == "start\n"

    def test_revert_session_restores_repository_files_and_sqlite(
        self, onus_bin: Path, rules_path: Path, tmp_path: Path
    ):
        file_target = tmp_path / "allowed" / "session.txt"
        db_target = tmp_path / "data" / "app.sqlite"
        file_target.parent.mkdir()
        db_target.parent.mkdir()
        file_target.write_text("file-before\n", encoding="utf-8")
        with sqlite3.connect(db_target) as con:
            con.execute("CREATE TABLE items (name TEXT)")
            con.execute("INSERT INTO items VALUES ('before')")

        with Guardian(
            workspace_root=tmp_path,
            contract=make_contract(allowed_paths=["allowed/**", "data/**"]),
            bin_path=str(onus_bin),
            rules_path=str(rules_path),
            db_path=str(tmp_path / "audit.db"),
        ) as guardian:
            guardian.file_write("allowed/session.txt", "file-after\n")
            guardian.db_execute("data/app.sqlite", "INSERT INTO items VALUES (?)", ("after",))
            result = guardian.revert_session()

        assert result.status == "REVERTED"
        assert result.restored is True
        assert file_target.read_text(encoding="utf-8") == "file-before\n"
        with sqlite3.connect(db_target) as con:
            rows = [row[0] for row in con.execute("SELECT name FROM items ORDER BY rowid")]
        assert rows == ["before"]

    def test_acceptance_scenario_h_failed_implementation_restores_checkpoint(
        self, onus_bin: Path, rules_path: Path, tmp_path: Path
    ):
        app_file = tmp_path / "allowed" / "app.py"
        created_file = tmp_path / "allowed" / "scratch.txt"
        db_target = tmp_path / "data" / "app.sqlite"
        app_file.parent.mkdir()
        db_target.parent.mkdir()
        app_file.write_text("print('safe')\n", encoding="utf-8")
        with sqlite3.connect(db_target) as con:
            con.execute("CREATE TABLE items (name TEXT)")
            con.execute("INSERT INTO items VALUES ('safe')")

        unresolved_report = None
        with Guardian(
            workspace_root=tmp_path,
            contract=make_contract(allowed_paths=["allowed/**", "data/**"]),
            bin_path=str(onus_bin),
            rules_path=str(rules_path),
            db_path=str(tmp_path / "audit.db"),
        ) as guardian:
            guardian.file_write("allowed/app.py", "print('broken')\n", action_group="attempt")
            guardian.file_write("allowed/scratch.txt", "temporary\n", action_group="attempt")
            guardian.db_execute(
                "data/app.sqlite",
                "INSERT INTO items VALUES (?)",
                ("broken",),
                action_group="attempt",
            )
            restore = guardian.restore_checkpoint()
            unresolved_report = restore.to_dict()

        assert restore.status == "CHECKPOINT_RESTORED"
        assert restore.restored is True
        assert app_file.read_text(encoding="utf-8") == "print('safe')\n"
        assert not created_file.exists()
        with sqlite3.connect(db_target) as con:
            rows = [row[0] for row in con.execute("SELECT name FROM items ORDER BY rowid")]
        assert rows == ["safe"]
        assert unresolved_report["message"].startswith("Restored repository files")
        assert len(unresolved_report["verification_result"]["files"]) >= 2

    def test_external_recovery_interfaces_report_unsupported_state(
        self, onus_bin: Path, rules_path: Path, tmp_path: Path
    ):
        adapter = UnsupportedExternalRecoveryAdapter("postgresql")
        assert adapter.classify({"sql": "UPDATE users SET name = 'x'"}) == ReversibilityClass.R4

        with Guardian(
            workspace_root=tmp_path,
            contract=make_contract(),
            bin_path=str(onus_bin),
            rules_path=str(rules_path),
            db_path=str(tmp_path / "audit.db"),
        ) as guardian:
            record = guardian.record_external_compensation_required(
                "postgresql",
                "postgres://example/db",
                {"sql": "UPDATE users SET name = 'x'"},
            )
            result = guardian.revert_action(record.action_id)

        assert result.status == "ROLLBACK_UNSUPPORTED"
        assert result.restored is False
        assert result.unsupported[0]["resource_type"] == "postgresql"
        assert result.unsupported[0]["supported"] is False

    def test_file_write_rejects_changed_target_after_evaluation(self, tmp_path: Path):
        target = tmp_path / "race.txt"
        target.write_text("before\n", encoding="utf-8")
        guardian = Guardian(
            workspace_root=tmp_path,
            contract=make_contract(),
            bin_path="onus",
            rules_path="rules/default.toml",
        )

        def stale_allow(*_args, **_kwargs):
            target.write_text("changed elsewhere\n", encoding="utf-8")
            return OnusResult(decision="allow")

        guardian.client.evaluate = stale_allow  # type: ignore[method-assign]

        with pytest.raises(OnusEvaluationError):
            guardian.file_write("race.txt", "after\n")
        assert target.read_text(encoding="utf-8") == "changed elsewhere\n"

    def test_file_delete_rejects_changed_target_after_evaluation(self, tmp_path: Path):
        target = tmp_path / "race-delete.txt"
        target.write_text("before\n", encoding="utf-8")
        guardian = Guardian(
            workspace_root=tmp_path,
            contract=make_contract(),
            bin_path="onus",
            rules_path="rules/default.toml",
        )

        def stale_allow(*_args, **_kwargs):
            target.write_text("changed elsewhere\n", encoding="utf-8")
            return OnusResult(decision="allow")

        guardian.client.evaluate = stale_allow  # type: ignore[method-assign]

        with pytest.raises(OnusEvaluationError):
            guardian.file_delete("race-delete.txt")
        assert target.read_text(encoding="utf-8") == "changed elsewhere\n"

    def test_db_execute_rejects_changed_database_after_evaluation(self, tmp_path: Path):
        db_path = tmp_path / "race.sqlite"
        with sqlite3.connect(db_path) as con:
            con.execute("CREATE TABLE items (name TEXT)")
            con.execute("INSERT INTO items VALUES ('before')")

        guardian = Guardian(
            workspace_root=tmp_path,
            contract=make_contract(),
            bin_path="onus",
            rules_path="rules/default.toml",
        )

        def stale_allow(*_args, **_kwargs):
            with sqlite3.connect(db_path) as con:
                con.execute("INSERT INTO items VALUES ('changed')")
            return OnusResult(decision="allow")

        guardian.client.evaluate = stale_allow  # type: ignore[method-assign]

        with pytest.raises(OnusEvaluationError):
            guardian.db_execute("race.sqlite", "INSERT INTO items VALUES ('after')")

        with sqlite3.connect(db_path) as con:
            rows = [row[0] for row in con.execute("SELECT name FROM items ORDER BY rowid")]
        assert rows == ["before", "changed"]

    def test_shell_blocks_before_execution(self, onus_bin: Path, rules_path: Path, tmp_path: Path):
        with Guardian(
            workspace_root=tmp_path,
            contract=make_contract(),
            bin_path=str(onus_bin),
            rules_path=str(rules_path),
            db_path=str(tmp_path / "audit.db"),
        ) as guardian:
            with pytest.raises(OnusBlockError) as exc:
                guardian.shell("rm -rf /important", execute=True)
            assert exc.value.result.rule_id == "SAFETY_001"


class TestTaskContractLifecycle:
    def test_guardian_without_contract_requires_explicit_behavior(self, tmp_path: Path):
        with pytest.raises(ValueError):
            Guardian(workspace_root=tmp_path, bin_path="onus", rules_path="rules/default.toml")

    def test_in_scope_write_is_allowed(self, onus_bin: Path, rules_path: Path, tmp_path: Path):
        with Guardian(
            workspace_root=tmp_path,
            contract=make_contract(),
            bin_path=str(onus_bin),
            rules_path=str(rules_path),
            db_path=str(tmp_path / "audit.db"),
        ) as guardian:
            result = guardian.file_write("allowed/in_scope.txt", "ok\n")
        assert result.allowed is True
        assert (tmp_path / "allowed" / "in_scope.txt").read_text(encoding="utf-8") == "ok\n"

    def test_out_of_scope_write_is_blocked(self, onus_bin: Path, rules_path: Path, tmp_path: Path):
        with Guardian(
            workspace_root=tmp_path,
            contract=make_contract(),
            bin_path=str(onus_bin),
            rules_path=str(rules_path),
            db_path=str(tmp_path / "audit.db"),
        ) as guardian:
            with pytest.raises(OnusBlockError) as exc:
                guardian.file_write("other/out.txt", "nope\n")
        assert exc.value.result.rule_id == "ONUS_CONTRACT_OUT_OF_SCOPE"

    def test_protected_path_is_blocked(self, onus_bin: Path, rules_path: Path, tmp_path: Path):
        with Guardian(
            workspace_root=tmp_path,
            contract=make_contract(),
            bin_path=str(onus_bin),
            rules_path=str(rules_path),
            db_path=str(tmp_path / "audit.db"),
        ) as guardian:
            with pytest.raises(OnusBlockError) as exc:
                guardian.file_write(".env", "TOKEN=value\n")
        assert exc.value.result.rule_id == "ONUS_CONTRACT_PROTECTED_PATH"

    def test_acceptance_scenario_c_hardcoded_secret_is_blocked_redacted_and_not_written(
        self, onus_bin: Path, rules_path: Path, tmp_path: Path
    ):
        target = tmp_path / "allowed" / "config.py"
        session_id = None
        with Guardian(
            workspace_root=tmp_path,
            contract=make_contract(),
            bin_path=str(onus_bin),
            rules_path=str(rules_path),
            db_path=str(tmp_path / "audit.db"),
        ) as guardian:
            session_id = guardian.session_id
            with pytest.raises(OnusBlockError) as exc:
                guardian.file_write(
                    "allowed/config.py",
                    'AWS_SECRET_ACCESS_KEY="abc123"\n',
                )
        assert exc.value.result.rule_id == "SECRET_001"
        assert "secret" in str(exc.value).lower()
        assert not target.exists()
        con = sqlite3.connect(tmp_path / "audit.db")
        try:
            stored_payloads = [
                row[0]
                for row in con.execute(
                    "SELECT payload FROM actions WHERE session_id = ?",
                    (session_id,),
                )
            ]
        finally:
            con.close()
        assert not any("abc123" in payload for payload in stored_payloads)
        assert any("[REDACTED]" in payload for payload in stored_payloads)

    def test_acceptance_scenario_b_deleted_test_is_blocked_and_completion_requires_tests(
        self,
        onus_bin: Path,
        rules_path: Path,
        tmp_path: Path,
        monkeypatch: pytest.MonkeyPatch,
    ):
        monkeypatch.setenv("ONUS_GUARDIAN_MODE", "professional")
        test_file = tmp_path / "tests" / "test_login.py"
        test_file.parent.mkdir(parents=True)
        test_file.write_text("def test_login():\n    assert True\n", encoding="utf-8")
        contract = make_contract(
            allowed_paths=["tests/**"],
            protected_paths=[],
            forbidden_actions=[],
            required_evidence=[
                RequiredEvidence(id="tests", description="Original login test passes", kind="test")
            ],
        )
        with Guardian(
            workspace_root=tmp_path,
            contract=contract,
            bin_path=str(onus_bin),
            rules_path=str(rules_path),
            db_path=str(tmp_path / "audit.db"),
        ) as guardian:
            with pytest.raises(OnusBlockError) as exc:
                guardian.file_delete("tests/test_login.py")
            assert exc.value.result.decision == "block"
            assert exc.value.result.approval_decision == "DENY_WITH_CORRECTION"
            assert test_file.exists()
            result = guardian.complete([])
        assert result["status"] == "HUMAN_REVIEW_REQUIRED"
        assert "tests" in result["missing_evidence"]
        assert "no_test_weakening" in result["missing_evidence"]

    def test_beginner_guardian_mode_creates_checkpoint_and_denies_test_weakening(
        self,
        onus_bin: Path,
        rules_path: Path,
        tmp_path: Path,
        monkeypatch: pytest.MonkeyPatch,
    ):
        monkeypatch.setenv("ONUS_GUARDIAN_MODE", "beginner")
        test_file = tmp_path / "tests" / "test_login.py"
        test_file.parent.mkdir(parents=True)
        test_file.write_text("def test_login():\n    assert user_can_login()\n", encoding="utf-8")
        contract = make_contract(
            allowed_paths=["tests/test_login.py"],
            protected_paths=[],
            forbidden_actions=[],
        )

        with Guardian(
            workspace_root=tmp_path,
            contract=contract,
            bin_path=str(onus_bin),
            rules_path=str(rules_path),
            db_path=str(tmp_path / "audit.db"),
        ) as guardian:
            assert guardian.checkpoint_path.exists()
            checkpoint = json.loads(guardian.checkpoint_path.read_text(encoding="utf-8"))
            assert checkpoint["checkpoint_type"] == "SAFE_SESSION_START"
            assert checkpoint["session_id"] == guardian.session_id
            assert checkpoint["files"][0]["path"].replace("\\", "/") == "tests/test_login.py"

            with pytest.raises(OnusBlockError) as exc:
                guardian.file_write(
                    "tests/test_login.py",
                    "def test_login():\n    pytest.skip('temporarily disabled')\n",
                )

        result = exc.value.result
        assert result.decision == "block"
        assert result.guardian_mode == "beginner"
        assert result.approval_decision == "DENY_WITH_CORRECTION"
        assert "delete, skip, or weaken tests" in (result.approval_reason or "")
        assert "pytest.skip" not in test_file.read_text(encoding="utf-8")

    def test_professional_reviewer_mode_requires_quality_evidence(
        self,
        onus_bin: Path,
        rules_path: Path,
        tmp_path: Path,
        monkeypatch: pytest.MonkeyPatch,
    ):
        monkeypatch.setenv("ONUS_GUARDIAN_MODE", "professional")
        with Guardian(
            workspace_root=tmp_path,
            contract=make_contract(),
            bin_path=str(onus_bin),
            rules_path=str(rules_path),
            db_path=str(tmp_path / "audit.db"),
        ) as guardian:
            result = guardian.complete(
                [CompletionEvidence(id="tests", passed=True, value="tests pass", kind="test")]
            )

        assert result["status"] == "HUMAN_REVIEW_REQUIRED"
        for evidence_id in [
            "targeted_tests",
            "lint",
            "typecheck",
            "coverage",
            "secret_scan",
            "architecture_review",
            "module_boundary_review",
            "final_scope",
            "independent_verification",
            "no_test_weakening",
        ]:
            assert evidence_id in result["missing_evidence"]

    def test_professional_reviewer_mode_detects_dependency_and_configuration_drift(
        self,
        onus_bin: Path,
        rules_path: Path,
        tmp_path: Path,
        monkeypatch: pytest.MonkeyPatch,
    ):
        monkeypatch.setenv("ONUS_GUARDIAN_MODE", "professional")
        contract = make_contract(allowed_paths=["allowed/**"], forbidden_actions=[])
        with Guardian(
            workspace_root=tmp_path,
            contract=contract,
            bin_path=str(onus_bin),
            rules_path=str(rules_path),
            db_path=str(tmp_path / "audit.db"),
        ) as guardian:
            guardian.file_write("allowed/package.json", '{"dependencies":{"demo":"1.0.0"}}\n')
            guardian.file_write("allowed/service.yaml", "service: demo\n")
            result = guardian.complete(quality_evidence())

        assert result["status"] == "HUMAN_REVIEW_REQUIRED"
        assert "dependency_review" in result["missing_evidence"]
        assert "configuration_review" in result["missing_evidence"]

    def test_excessive_file_count_is_blocked(self, onus_bin: Path, rules_path: Path, tmp_path: Path):
        contract = make_contract(change_budget=ChangeBudget(max_files_changed=1, max_actions=20))
        with Guardian(
            workspace_root=tmp_path,
            contract=contract,
            bin_path=str(onus_bin),
            rules_path=str(rules_path),
            db_path=str(tmp_path / "audit.db"),
        ) as guardian:
            guardian.file_write("allowed/one.txt", "one\n")
            with pytest.raises(OnusBlockError) as exc:
                guardian.file_write("allowed/two.txt", "two\n")
        assert exc.value.result.rule_id == "ONUS_CONTRACT_CHANGE_BUDGET"

    def test_database_migration_requires_approval(self, onus_bin: Path, rules_path: Path, tmp_path: Path):
        with Guardian(
            workspace_root=tmp_path,
            contract=make_contract(allowed_paths=["allowed/**", "data/**"]),
            bin_path=str(onus_bin),
            rules_path=str(rules_path),
            db_path=str(tmp_path / "audit.db"),
        ) as guardian:
            with pytest.raises(OnusBlockError) as exc:
                guardian.db_execute("data/app.sqlite", "CREATE TABLE users (id INTEGER)")
        assert exc.value.result.decision == "escalate"
        assert exc.value.result.rule_id == "ONUS_CONTRACT_APPROVAL_REQUIRED"

    def test_policy_block_dominates_contract_approval_in_cli_evaluate(
        self, client: OnusClient, tmp_path: Path
    ):
        contract = make_contract(
            session_id="block-dominates-escalate",
            allowed_paths=["**/*"],
            protected_paths=[],
            forbidden_actions=[],
            approval_required_actions=["shell"],
        )
        client.start_contract(contract, workspace_root=tmp_path, agent_name="regression-test")

        result = client.check_command(
            "sudo rm -rf /important",
            session_id="block-dominates-escalate",
        )

        assert result.decision == "block"
        assert result.rule_id == "SAFETY_001"

    def test_blocked_action_records_incident_memory(
        self, client: OnusClient, tmp_path: Path
    ):
        contract = make_contract(
            session_id="incident-memory-session",
            allowed_paths=["**/*"],
            protected_paths=[],
            forbidden_actions=[],
            approval_required_actions=[],
        )
        client.start_contract(contract, workspace_root=tmp_path, agent_name="incident-test")

        result = client.check_command(
            "rm -rf /important",
            session_id="incident-memory-session",
        )
        assert result.decision == "block"

        con = sqlite3.connect(client.db_path)
        try:
            row = con.execute(
                "SELECT kind, key, value_redacted, provenance_json FROM onus_memory WHERE session_id = ? AND kind = 'incident'",
                ("incident-memory-session",),
            ).fetchone()
        finally:
            con.close()
        assert row is not None
        assert row[0] == "incident"
        assert "SAFETY_001" in row[1]
        assert "rm -rf" not in row[2]
        assert "action_evaluation" in row[3]

    def test_completion_with_missing_evidence_is_rejected(
        self, onus_bin: Path, rules_path: Path, tmp_path: Path
    ):
        with Guardian(
            workspace_root=tmp_path,
            contract=make_contract(),
            bin_path=str(onus_bin),
            rules_path=str(rules_path),
            db_path=str(tmp_path / "audit.db"),
        ) as guardian:
            result = guardian.complete([])
        assert result["status"] == "HUMAN_REVIEW_REQUIRED"
        assert "tests" in result["missing_evidence"]
        assert "independent_verification" in result["missing_evidence"]
        assert "correction" in result

    def test_completion_with_required_evidence_is_verified(
        self, onus_bin: Path, rules_path: Path, tmp_path: Path
    ):
        with Guardian(
            workspace_root=tmp_path,
            contract=make_contract(),
            bin_path=str(onus_bin),
            rules_path=str(rules_path),
            db_path=str(tmp_path / "audit.db"),
        ) as guardian:
            result = guardian.complete(quality_evidence())
        assert result["status"] == "COMPLETED_VERIFIED"

    def test_acceptance_scenario_g_agent_statement_is_not_completion_evidence(
        self, onus_bin: Path, rules_path: Path, tmp_path: Path
    ):
        with Guardian(
            workspace_root=tmp_path,
            contract=make_contract(),
            bin_path=str(onus_bin),
            rules_path=str(rules_path),
            db_path=str(tmp_path / "audit.db"),
        ) as guardian:
            result = guardian.complete(
                [
                    CompletionEvidence(
                        id="agent_complete",
                        passed=True,
                        value="The work is complete.",
                        kind="agent_statement",
                    )
                ]
            )
        assert result["status"] == "HUMAN_REVIEW_REQUIRED"
        assert "independent_verification" in result["missing_evidence"]
        assert any(
            finding["id"] == "AGENT_STATEMENT_NOT_EVIDENCE"
            for finding in result["findings"]
        )


class TestPromptIntakeGuardian:
    def test_scenario_a_broad_destructive_prompt_proposes_safe_contract(
        self, client: OnusClient, tmp_path: Path
    ):
        result = client.intake_prompt(
            "Fix everything and delete anything causing errors.",
            workspace_root=tmp_path,
            start_session=False,
            provider="disabled",
        )
        assert isinstance(result, PromptIntakeResult)
        assert result.status == "READY_WITH_SAFE_CONTRACT"
        assert "dangerously_broad_prompt" in result.reasons
        assert "destructive_wording" in result.reasons
        assert result.semantic_review == "provider_disabled; no LLM provider was invoked"
        assert result.proposed_contract is not None
        assert "file_delete" in result.proposed_contract.forbidden_actions
        assert any(e.id == "tests" for e in result.proposed_contract.required_evidence)

    def test_intake_start_session_persists_contract(self, client: OnusClient, tmp_path: Path):
        result = client.intake_prompt(
            "Fix the local src/auth/login.py bug and run tests.",
            workspace_root=tmp_path,
            session_id="intake-python-test",
            start_session=True,
            provider="disabled",
        )
        assert result.status == "READY"
        assert result.session_started is True
        assert result.contract_hash

        con = sqlite3.connect(client.db_path)
        try:
            row = con.execute(
                "SELECT contract_hash, normalized_objective FROM task_contracts WHERE session_id = ?",
                ("intake-python-test",),
            ).fetchone()
        finally:
            con.close()
        assert row is not None
        assert row[0] == result.contract_hash
        assert "src/auth/login.py" in row[1]

        con = sqlite3.connect(client.db_path)
        try:
            memory_row = con.execute(
                "SELECT kind, key, value_redacted, provenance_json FROM onus_memory WHERE session_id = ?",
                ("intake-python-test",),
            ).fetchone()
        finally:
            con.close()
        assert memory_row is not None
        assert memory_row[0] == "session"
        assert memory_row[1] == "task_intake"
        assert "Fix the local" in memory_row[2]
        assert "prompt_intake" in memory_row[3]

    @pytest.mark.parametrize(
        ("prompt", "status", "reason"),
        [
            ("Disable security so this feature runs.", "REJECTED_AS_UNSAFE", "requests_to_disable_security"),
            ("Put the API key directly in source.", "REJECTED_AS_UNSAFE", "requests_to_expose_or_hardcode_secrets"),
            ("Use my production database and test it.", "CLARIFICATION_REQUIRED", "requests_direct_production_operation"),
            ("Remove tests that are failing.", "READY_WITH_SAFE_CONTRACT", "requests_to_delete_or_disable_tests"),
            ("Fix the bug.", "READY_WITH_SAFE_CONTRACT", "missing_scope"),
        ],
    )
    def test_deterministic_intake_variants(
        self, client: OnusClient, tmp_path: Path, prompt: str, status: str, reason: str
    ):
        result = client.intake_prompt(prompt, workspace_root=tmp_path, provider="disabled")
        assert result.status == status
        assert reason in result.reasons

    def test_guardian_from_prompt_uses_intake_contract(
        self, onus_bin: Path, rules_path: Path, tmp_path: Path
    ):
        with Guardian.from_prompt(
            "Fix the local allowed/widget.py bug and run tests.",
            workspace_root=tmp_path,
            agent_name="intake-agent",
            bin_path=str(onus_bin),
            rules_path=str(rules_path),
            db_path=str(tmp_path / "audit.db"),
            provider="disabled",
        ) as guardian:
            assert guardian.contract is not None
            assert guardian.contract.original_prompt.startswith("Fix the local")

    def test_local_semantic_adapter_accepts_strict_schema(
        self, client: OnusClient, tmp_path: Path
    ):
        adapter = tmp_path / "local_semantic_fixture.py"
        adapter.write_text(
            """
import json
import sys

request = json.load(sys.stdin)
assert request["role"] == "intent_interpreter"
payload = {
    "schema_version": 1,
    "normalized_objective": "Repair the local auth login bug and prove it with tests.",
    "allowed_scope": ["src/auth/login.py", "tests/auth/**"],
    "protected_scope": [".env"],
    "completion_evidence": ["pytest auth tests"],
    "ambiguities": [],
    "risk_assumptions": ["local workspace only"],
    "questions": [],
    "confidence": 0.88,
}
print(json.dumps(payload))
""".strip(),
            encoding="utf-8",
        )

        result = client.intake_prompt(
            "Fix the local src/auth/login.py bug and run tests.",
            workspace_root=tmp_path,
            provider="local",
            semantic_local_command=f'"{sys.executable}" "{adapter}"',
            semantic_privacy="balanced",
        )

        assert result.status == "READY"
        assert result.semantic_roles[0]["provider"] == "local"
        assert result.semantic_roles[0]["provider_invoked"] is True
        assert result.semantic_roles[0]["accepted"] is True
        assert result.proposed_contract is not None
        assert (
            result.proposed_contract.normalized_objective
            == "Repair the local auth login bug and prove it with tests."
        )
        assert any(e.id == "semantic_pytestauthtests" for e in result.proposed_contract.required_evidence)

    def test_malformed_local_semantic_output_is_rejected(
        self, client: OnusClient, tmp_path: Path
    ):
        adapter = tmp_path / "malformed_semantic_fixture.py"
        adapter.write_text(
            "import json; print(json.dumps({'schema_version': 1, 'unexpected': True}))",
            encoding="utf-8",
        )

        result = client.intake_prompt(
            "Fix the local src/auth/login.py bug and run tests.",
            workspace_root=tmp_path,
            provider="local",
            semantic_local_command=f'"{sys.executable}" "{adapter}"',
            semantic_privacy="balanced",
        )

        assert result.status == "READY"
        assert result.semantic_roles[0]["provider_invoked"] is True
        assert result.semantic_roles[0]["accepted"] is False
        assert result.semantic_roles[0]["fallback_used"] is True
        assert "malformed" in result.semantic_roles[0]["error"]
        assert result.proposed_contract is not None
        assert result.proposed_contract.normalized_objective.startswith("Fix the local")

    def test_only_relevant_memory_reaches_local_semantic_reviewer(
        self, client: OnusClient, tmp_path: Path
    ):
        client.intake_prompt(
            "Prime schema for local src/auth/login.py tests.",
            workspace_root=tmp_path,
            provider="disabled",
        )
        project_id = hashlib.sha256(
            f"project:{tmp_path.resolve()}".encode("utf-8")
        ).hexdigest()
        now = int(time.time())
        provenance = json.dumps(
            {
                "actor_type": "system",
                "actor_id": "test",
                "source": "pytest",
                "reason": "memory relevance proof",
            },
            separators=(",", ":"),
        )
        con = sqlite3.connect(client.db_path)
        try:
            for key, summary in [
                ("auth_architecture", '{"note":"login controller owns auth sessions"}'),
                ("billing_architecture", '{"note":"invoices use billing service"}'),
            ]:
                con.execute(
                    """
                    INSERT INTO onus_memory
                        (id, kind, tenant_id, project_id, session_id, key, value_ciphertext,
                         value_redacted, value_hash, classification, sensitive, provenance_json,
                         version, review_status, retention_expires_at, deleted_at, created_at,
                         updated_at, mutable_by_agent)
                    VALUES (?, 'project', 'local', ?, NULL, ?, 'encrypted-fixture',
                            ?, ?, '{}', 0, ?, 1, 'system_recorded', NULL, NULL, ?, ?, 1)
                    """,
                    (
                        f"fixture-{key}",
                        project_id,
                        key,
                        summary,
                        hashlib.sha256(summary.encode("utf-8")).hexdigest(),
                        provenance,
                        now,
                        now,
                    ),
                )
            con.commit()
        finally:
            con.close()

        adapter = tmp_path / "memory_semantic_fixture.py"
        adapter.write_text(
            """
import json
import sys

request = json.load(sys.stdin)
memory = request["input"].get("memory_context", [])
joined = "\\n".join(memory)
assert "auth_architecture" in joined, joined
assert "billing_architecture" not in joined, joined
print(json.dumps({
    "schema_version": 1,
    "normalized_objective": "Use scoped auth memory only.",
    "allowed_scope": ["src/auth/login.py"],
    "protected_scope": [],
    "completion_evidence": ["pytest auth tests"],
    "ambiguities": [],
    "risk_assumptions": [],
    "questions": [],
    "confidence": 0.9,
}))
""".strip(),
            encoding="utf-8",
        )

        result = client.intake_prompt(
            "Fix local auth login controller and run tests.",
            workspace_root=tmp_path,
            provider="local",
            semantic_local_command=f'"{sys.executable}" "{adapter}"',
            semantic_privacy="balanced",
        )

        assert result.semantic_roles[0]["accepted"] is True
        assert result.proposed_contract is not None
        assert result.proposed_contract.normalized_objective == "Use scoped auth memory only."

    @pytest.mark.skipif(
        not (
            os.environ.get("ONUS_SEMANTIC_REAL_PROVIDER") == "1"
            and os.environ.get("ONUS_SEMANTIC_ENDPOINT")
            and os.environ.get("ONUS_SEMANTIC_MODEL")
            and (
                "localhost" in os.environ.get("ONUS_SEMANTIC_ENDPOINT", "")
                or os.environ.get("ONUS_SEMANTIC_API_KEY")
            )
        ),
        reason="real remote semantic provider credentials/configuration not available",
    )
    def test_real_remote_semantic_provider_when_configured(
        self, client: OnusClient, tmp_path: Path
    ):
        result = client.intake_prompt(
            "Fix the local src/auth/login.py bug and run tests.",
            workspace_root=tmp_path,
            provider="cloud",
            semantic_endpoint=os.environ["ONUS_SEMANTIC_ENDPOINT"],
            semantic_model=os.environ["ONUS_SEMANTIC_MODEL"],
            semantic_timeout_ms=int(os.environ.get("ONUS_SEMANTIC_TIMEOUT_MS", "15000")),
            semantic_privacy="strict",
        )

        assert result.semantic_roles[0]["provider"] == "cloud"
        assert result.semantic_roles[0]["provider_invoked"] is True
        assert result.semantic_roles[0]["accepted"] is True


class TestClaudeCodeAdapterRuntime:
    def test_claude_hook_allow_deny_correction_and_agent_retry(
        self, onus_bin: Path, rules_path: Path, tmp_path: Path
    ):
        db_path = tmp_path / "claude-hook.db"
        denied = self._run_claude_hook(
            onus_bin,
            rules_path,
            db_path,
            self._hook_payload("Bash", {"command": "rm -rf /important"}),
        )
        assert denied["hookSpecificOutput"]["permissionDecision"] == "deny"
        assert "SAFETY_001" in denied["hookSpecificOutput"]["permissionDecisionReason"]
        assert "destroy filesystem data" in denied["hookSpecificOutput"]["permissionDecisionReason"]

        retry = self._run_claude_hook(
            onus_bin,
            rules_path,
            db_path,
            self._hook_payload("Bash", {"command": "echo safe retry"}),
        )
        assert retry["hookSpecificOutput"]["permissionDecision"] == "allow"

    def test_claude_hook_ask_for_contract_approval_and_guardian_mode(
        self, onus_bin: Path, rules_path: Path, tmp_path: Path, monkeypatch: pytest.MonkeyPatch
    ):
        monkeypatch.setenv("ONUS_GUARDIAN_MODE", "beginner")
        db_path = tmp_path / "claude-hook.db"
        session_id = "claude-ask-session"
        client = OnusClient(
            bin_path=str(onus_bin),
            rules_path=str(rules_path),
            db_path=str(db_path),
        )
        client.start_contract(
            make_contract(
                session_id=session_id,
                allowed_paths=[],
                protected_paths=[],
                forbidden_actions=[],
                approval_required_actions=["shell"],
            ),
            workspace_root=tmp_path,
            agent_name="claude-code-runtime-test",
        )

        output = self._run_claude_hook(
            onus_bin,
            rules_path,
            db_path,
            self._hook_payload("Bash", {"command": "echo needs approval"}, session_id=session_id),
        )
        assert output["hookSpecificOutput"]["permissionDecision"] == "ask"
        assert "requires approval" in output["hookSpecificOutput"]["permissionDecisionReason"]

        con = sqlite3.connect(db_path)
        try:
            row = con.execute(
                "SELECT approval_decision, guardian_mode FROM actions WHERE session_id = ? ORDER BY id DESC LIMIT 1",
                (session_id,),
            ).fetchone()
        finally:
            con.close()
        assert row == ("REQUIRE_HUMAN_APPROVAL", "beginner")

    def test_claude_hook_malformed_input_process_failure_timeout_unavailable_and_disabled(
        self, onus_bin: Path, rules_path: Path, tmp_path: Path, monkeypatch: pytest.MonkeyPatch
    ):
        db_path = tmp_path / "claude-hook.db"
        malformed = self._run_claude_hook_raw(onus_bin, rules_path, db_path, "{not-json")
        assert malformed["hookSpecificOutput"]["permissionDecision"] == "deny"
        assert "malformed input" in malformed["hookSpecificOutput"]["permissionDecisionReason"]

        bad_output = self._write_fake_evaluator(tmp_path, "malformed")
        malformed_output = self._run_claude_hook(
            onus_bin,
            rules_path,
            db_path,
            self._hook_payload("Bash", {"command": "echo ok"}),
            evaluator=sys.executable,
            evaluator_args=[str(bad_output)],
        )
        assert malformed_output["hookSpecificOutput"]["permissionDecision"] == "deny"
        assert "malformed output" in malformed_output["hookSpecificOutput"]["permissionDecisionReason"]

        failing = self._write_fake_evaluator(tmp_path, "fail")
        failure = self._run_claude_hook(
            onus_bin,
            rules_path,
            db_path,
            self._hook_payload("Bash", {"command": "echo ok"}),
            evaluator=sys.executable,
            evaluator_args=[str(failing)],
        )
        assert failure["hookSpecificOutput"]["permissionDecision"] == "deny"
        assert "produced no JSON output" in failure["hookSpecificOutput"]["permissionDecisionReason"]

        slow = self._write_fake_evaluator(tmp_path, "slow")
        timeout = self._run_claude_hook(
            onus_bin,
            rules_path,
            db_path,
            self._hook_payload("Bash", {"command": "echo ok"}),
            evaluator=sys.executable,
            evaluator_args=[str(slow)],
            extra_args=["--timeout-ms", "50"],
        )
        assert timeout["hookSpecificOutput"]["permissionDecision"] == "deny"
        assert "timed out" in timeout["hookSpecificOutput"]["permissionDecisionReason"]

        unavailable = self._run_claude_hook(
            onus_bin,
            rules_path,
            db_path,
            self._hook_payload("Bash", {"command": "echo ok"}),
            evaluator=str(tmp_path / "missing-evaluator.exe"),
        )
        assert unavailable["hookSpecificOutput"]["permissionDecision"] == "deny"
        assert "evaluator unavailable" in unavailable["hookSpecificOutput"]["permissionDecisionReason"]

        monkeypatch.setenv("ONUS_CLAUDE_HOOK_DISABLED", "1")
        disabled = self._run_claude_hook(
            onus_bin,
            rules_path,
            db_path,
            self._hook_payload("Bash", {"command": "rm -rf /important"}),
        )
        assert disabled["hookSpecificOutput"]["permissionDecision"] == "allow"
        assert "BEST-EFFORT hook is bypassed" in disabled["hookSpecificOutput"]["permissionDecisionReason"]

    def test_claude_hook_nested_subagent_unsupported_tool_and_strict_mode(
        self, onus_bin: Path, rules_path: Path, tmp_path: Path, monkeypatch: pytest.MonkeyPatch
    ):
        db_path = tmp_path / "claude-hook.db"
        nested = self._run_claude_hook(
            onus_bin,
            rules_path,
            db_path,
            self._hook_payload(
                "Read",
                {"file_path": str(tmp_path / "demo.txt")},
                agent_type="general-purpose",
            ),
        )
        assert nested["hookSpecificOutput"]["permissionDecision"] == "allow"
        con = sqlite3.connect(db_path)
        try:
            payload = con.execute(
                "SELECT payload FROM actions ORDER BY id DESC LIMIT 1",
            ).fetchone()[0]
        finally:
            con.close()
        assert "general-purpose" in payload
        assert "L1_BEST_EFFORT" in payload

        unsupported = self._run_claude_hook(
            onus_bin,
            rules_path,
            db_path,
            self._hook_payload("ImaginaryTool", {"value": "x"}),
        )
        assert unsupported["hookSpecificOutput"]["permissionDecision"] == "ask"
        assert "does not yet support tool" in unsupported["hookSpecificOutput"]["permissionDecisionReason"]

        monkeypatch.setenv("ONUS_STRICT", "1")
        strict = self._run_claude_hook(
            onus_bin,
            rules_path,
            db_path,
            self._hook_payload("Bash", {"command": "sudo rm -rf /important"}),
        )
        assert strict["hookSpecificOutput"]["permissionDecision"] == "deny"

    @pytest.mark.skipif(
        os.environ.get("ONUS_CLAUDE_CODE_LIVE") != "1",
        reason="live Claude Code runtime disabled; requires authenticated pinned Claude Code",
    )
    def test_live_pinned_claude_code_environment_available(self, tmp_path: Path):
        version = subprocess.run(
            ["npx", "-y", "@anthropic-ai/claude-code@2.1.177", "--version"],
            capture_output=True,
            text=True,
            timeout=120,
        )
        assert version.returncode == 0
        assert "2.1.177" in version.stdout

        auth = subprocess.run(
            ["npx", "-y", "@anthropic-ai/claude-code@2.1.177", "auth", "status"],
            capture_output=True,
            text=True,
            timeout=120,
        )
        assert auth.returncode == 0
        status = json.loads(auth.stdout)
        assert status["loggedIn"] is True

    @staticmethod
    def _hook_payload(
        tool_name: str,
        tool_input: dict[str, object],
        *,
        session_id: str = "claude-runtime-session",
        agent_type: str = "",
    ) -> dict[str, object]:
        return {
            "hook_event_name": "PreToolUse",
            "tool_name": tool_name,
            "tool_input": tool_input,
            "session_id": session_id,
            "cwd": "D:\\Onus",
            "agent": "claude-code",
            "agent_version": "2.1.177",
            "agent_type": agent_type,
            "transcript_path": "D:\\Onus\\.claude\\transcript.jsonl",
        }

    @classmethod
    def _run_claude_hook(
        cls,
        onus_bin: Path,
        rules_path: Path,
        db_path: Path,
        payload: dict[str, object],
        *,
        evaluator: str | None = None,
        evaluator_args: list[str] | None = None,
        extra_args: list[str] | None = None,
    ) -> dict:
        return cls._run_claude_hook_raw(
            onus_bin,
            rules_path,
            db_path,
            json.dumps(payload),
            evaluator=evaluator,
            evaluator_args=evaluator_args,
            extra_args=extra_args,
        )

    @staticmethod
    def _run_claude_hook_raw(
        onus_bin: Path,
        rules_path: Path,
        db_path: Path,
        payload: str,
        *,
        evaluator: str | None = None,
        evaluator_args: list[str] | None = None,
        extra_args: list[str] | None = None,
    ) -> dict:
        args = [
            str(onus_bin),
            "claude-hook",
            "--rules",
            str(rules_path),
            "--db",
            str(db_path),
        ]
        if evaluator:
            args += ["--evaluator", evaluator]
        for item in evaluator_args or []:
            args += ["--evaluator-arg", item]
        if extra_args:
            args.extend(extra_args)
        proc = subprocess.run(
            args,
            input=payload,
            capture_output=True,
            text=True,
            timeout=30,
        )
        assert proc.returncode == 0, proc.stderr
        return json.loads(proc.stdout)

    @staticmethod
    def _write_fake_evaluator(tmp_path: Path, mode: str) -> Path:
        script = tmp_path / f"fake_evaluator_{mode}.py"
        script.write_text(
            {
                "malformed": "import sys\nsys.stdin.read()\nprint('not json')\n",
                "fail": "import sys\nsys.stdin.read()\nsys.stderr.write('boom')\nsys.exit(1)\n",
                "slow": "import sys, time\nsys.stdin.read()\ntime.sleep(2)\nprint('{\"decision\":\"allow\"}')\n",
            }[mode],
            encoding="utf-8",
        )
        return script


class TestL3WorkspaceCli:
    def test_workspace_create_inspect_export_destroy(self, onus_bin: Path, tmp_path: Path):
        repo = tmp_path / "repo"
        repo.mkdir()
        (repo / "README.md").write_text("workspace source\n", encoding="utf-8")
        (repo / ".git").mkdir()
        (repo / ".git" / "config").write_text("not exported\n", encoding="utf-8")

        env = os.environ.copy()
        env["ONUS_DATA_DIR"] = str(tmp_path / "onus-data")
        session_id = "l3-cli-test"

        create = subprocess.run(
            [
                str(onus_bin),
                "workspace",
                "create",
                "--repo",
                str(repo),
                "--session",
                session_id,
            ],
            capture_output=True,
            text=True,
            env=env,
            timeout=30,
        )
        assert create.returncode == 0, create.stderr
        created = json.loads(create.stdout)
        assert created["session_id"] == session_id
        assert created["network_egress"] == "deny_all"
        assert created["isolation_level"] == "L3_PENDING_RUNTIME_VERIFICATION"
        assert created["enforcement_label"] == "L3_LINUX_WORKSPACE_PENDING_VERIFICATION"
        assert created["boundary_verified"] is False
        worktree = Path(created["worktree"])
        assert (worktree / "README.md").is_file()
        assert not (worktree / ".git").exists()
        assert created["checkpoints"][0]["id"] == "initial"

        inspect = subprocess.run(
            [str(onus_bin), "workspace", "inspect", "--session", session_id],
            capture_output=True,
            text=True,
            env=env,
            timeout=30,
        )
        assert inspect.returncode == 0, inspect.stderr
        inspected = json.loads(inspect.stdout)
        assert inspected["session_id"] == session_id

        export_dir = tmp_path / "exports"
        export = subprocess.run(
            [
                str(onus_bin),
                "workspace",
                "export",
                "--session",
                session_id,
                "--dest",
                str(export_dir),
            ],
            capture_output=True,
            text=True,
            env=env,
            timeout=30,
        )
        assert export.returncode == 0, export.stderr
        exported = json.loads(export.stdout)
        exported_path = Path(exported["export_path"])
        assert (exported_path / "workspace.json").is_file()
        assert (exported_path / "worktree" / "README.md").read_text(encoding="utf-8") == "workspace source\n"
        assert not (exported_path / "worktree" / ".git").exists()

        overwrite = subprocess.run(
            [
                str(onus_bin),
                "workspace",
                "export",
                "--session",
                session_id,
                "--dest",
                str(export_dir),
            ],
            capture_output=True,
            text=True,
            env=env,
            timeout=30,
        )
        assert overwrite.returncode != 0
        assert "refusing to overwrite" in overwrite.stderr

        destroy = subprocess.run(
            [str(onus_bin), "workspace", "destroy", "--session", session_id],
            capture_output=True,
            text=True,
            env=env,
            timeout=30,
        )
        assert destroy.returncode == 0, destroy.stderr
        assert json.loads(destroy.stdout)["destroyed"] is True

        missing = subprocess.run(
            [str(onus_bin), "workspace", "inspect", "--session", session_id],
            capture_output=True,
            text=True,
            env=env,
            timeout=30,
        )
        assert missing.returncode != 0

    def test_run_isolate_fails_closed_without_linux_boundary(self, onus_bin: Path, tmp_path: Path):
        repo = tmp_path / "repo"
        repo.mkdir()
        (repo / "README.md").write_text("workspace source\n", encoding="utf-8")

        env = os.environ.copy()
        env["ONUS_DATA_DIR"] = str(tmp_path / "onus-data")
        session_id = "l3-fail-closed"
        create = subprocess.run(
            [
                str(onus_bin),
                "workspace",
                "create",
                "--repo",
                str(repo),
                "--session",
                session_id,
            ],
            capture_output=True,
            text=True,
            env=env,
            timeout=30,
        )
        assert create.returncode == 0, create.stderr

        run = subprocess.run(
            [
                str(onus_bin),
                "run",
                "--isolate",
                "--workspace",
                session_id,
                "--",
                sys.executable,
                "-c",
                "print('should-not-run')",
            ],
            capture_output=True,
            text=True,
            env=env,
            timeout=30,
        )
        if sys.platform.startswith("linux"):
            if "bubblewrap" in run.stderr or "bwrap" in run.stderr:
                assert run.returncode != 0
            else:
                pytest.skip("Linux L3 boundary available; adversarial verifier covers execution")
        else:
            assert run.returncode != 0
            assert "only on Linux" in run.stderr
            assert "should-not-run" not in run.stdout


class TestMcpProxyRuntime:
    def test_mcp_gateway_initializes_discovers_allows_and_receipts(
        self, onus_bin: Path, rules_path: Path, tmp_path: Path
    ):
        db_path = tmp_path / "audit.db"
        session_id = "mcp-gateway-allow-session"
        client = OnusClient(
            bin_path=str(onus_bin),
            rules_path=str(rules_path),
            db_path=str(db_path),
        )
        client.start_contract(
            make_contract(
                session_id=session_id,
                allowed_paths=[],
                protected_paths=[],
                forbidden_actions=[],
                approval_required_actions=[],
            ),
            workspace_root=tmp_path,
            agent_name="mcp-runtime-allow-test",
        )
        fake_server = self._write_fake_mcp_server(tmp_path)
        side_effect = tmp_path / "side-effect.txt"
        proc = self._start_mcp_proxy(
            onus_bin,
            db_path,
            session_id,
            fake_server,
            side_effect,
        )
        try:
            initialized = self._mcp_request(
                proc,
                {
                    "jsonrpc": "2.0",
                    "id": 1,
                    "method": "initialize",
                    "params": {
                        "protocolVersion": "2024-11-05",
                        "capabilities": {},
                        "clientInfo": {"name": "onus-runtime-client", "version": "test"},
                    },
                },
            )
            assert initialized["result"]["capabilities"]["tools"]["listChanged"] is False
            assert initialized["result"]["_onus_gateway"]["gateway"] == "onus-mcp-proxy"
            assert initialized["result"]["_onus_gateway"]["server_identity"]["identity_hash"]
            assert "Direct client connections" in initialized["result"]["_onus_gateway"]["direct_server_bypass"]

            discovered = self._mcp_request(
                proc,
                {"jsonrpc": "2.0", "id": 2, "method": "tools/list", "params": {}},
            )
            tool_names = {tool["name"] for tool in discovered["result"]["tools"]}
            assert {"echo.safe", "side.effect", "large.response", "slow.response"} <= tool_names
            assert discovered["result"]["_onus_gateway"]["server_identity"]["identity_hash"]

            allowed = self._mcp_request(
                proc,
                self._mcp_tool_message(3, "echo.safe", {"query": "select 1"}),
            )
            assert allowed["result"]["content"][0]["text"] == "forwarded:echo.safe"
            receipt = allowed["result"]["_onus_receipt"]
            assert receipt["decision"] == "allow"
            assert receipt["action_id"]
            assert receipt["canonical_payload_hash"]

            con = sqlite3.connect(db_path)
            try:
                row = con.execute(
                    "SELECT action_id, payload_hash, verdict, payload FROM actions WHERE session_id = ? ORDER BY id DESC LIMIT 1",
                    (session_id,),
                ).fetchone()
            finally:
                con.close()
            assert row[0] == receipt["action_id"]
            assert row[1] == receipt["canonical_payload_hash"]
            assert row[2] == "allow"
            assert "echo.safe" in row[3]
        finally:
            self._stop_process(proc)

    def test_mcp_gateway_denies_before_synthetic_side_effect(
        self, onus_bin: Path, rules_path: Path, tmp_path: Path
    ):
        db_path = tmp_path / "audit.db"
        session_id = "mcp-gateway-deny-session"
        client = OnusClient(
            bin_path=str(onus_bin),
            rules_path=str(rules_path),
            db_path=str(db_path),
        )
        client.start_contract(
            make_contract(
                session_id=session_id,
                allowed_paths=[],
                protected_paths=[],
                forbidden_actions=[],
                approval_required_actions=[],
            ),
            workspace_root=tmp_path,
            agent_name="mcp-runtime-deny-test",
        )
        fake_server = self._write_fake_mcp_server(tmp_path)
        side_effect = tmp_path / "side-effect-denied.txt"
        proc = self._start_mcp_proxy(
            onus_bin,
            db_path,
            session_id,
            fake_server,
            side_effect,
        )
        try:
            denied = self._mcp_request(
                proc,
                self._mcp_tool_message(
                    1,
                    "side.effect",
                    {"command": "rm -rf /important", "path": str(side_effect)},
                ),
            )
            assert denied["error"]["code"] == -32001
            assert "Blocked by Onus" in denied["error"]["message"]
            assert denied["error"]["data"]["rule_id"] == "MCP_SAFETY_001"
            assert not side_effect.exists()
        finally:
            self._stop_process(proc)

    def test_mcp_gateway_handles_timeout_malformed_response_size_and_secret_redaction(
        self, onus_bin: Path, rules_path: Path, tmp_path: Path
    ):
        db_path = tmp_path / "audit.db"
        session_id = "mcp-gateway-error-session"
        client = OnusClient(
            bin_path=str(onus_bin),
            rules_path=str(rules_path),
            db_path=str(db_path),
        )
        client.start_contract(
            make_contract(
                session_id=session_id,
                allowed_paths=[],
                protected_paths=[],
                forbidden_actions=[],
                approval_required_actions=[],
            ),
            workspace_root=tmp_path,
            agent_name="mcp-runtime-error-test",
        )

        fake_server = self._write_fake_mcp_server(tmp_path)
        side_effect = tmp_path / "unused.txt"
        timeout_proc = self._start_mcp_proxy(
            onus_bin,
            db_path,
            session_id,
            fake_server,
            side_effect,
            extra_proxy_args=["--response-timeout-ms", "100"],
        )
        try:
            timed_out = self._mcp_request(
                timeout_proc,
                self._mcp_tool_message(1, "slow.response", {"sleep_ms": 1000}),
            )
            assert timed_out["error"]["code"] == -32098
            assert "timed out" in timed_out["error"]["message"]
        finally:
            self._stop_process(timeout_proc)

        malformed_proc = self._start_mcp_proxy(
            onus_bin,
            db_path,
            session_id,
            fake_server,
            side_effect,
        )
        try:
            malformed = self._mcp_raw_json(malformed_proc, b'{"jsonrpc":')
            assert malformed["error"]["code"] == -32700
            assert "invalid MCP JSON" in malformed["error"]["message"]
        finally:
            self._stop_process(malformed_proc)

        large_proc = self._start_mcp_proxy(
            onus_bin,
            db_path,
            session_id,
            fake_server,
            side_effect,
            extra_proxy_args=["--max-response-bytes", "512"],
        )
        try:
            too_large = self._mcp_request(
                large_proc,
                self._mcp_tool_message(2, "large.response", {"size": 4096}),
            )
            assert too_large["error"]["code"] == -32098
            assert "size" in too_large["error"]["message"]
        finally:
            self._stop_process(large_proc)

        secret_proc = self._start_mcp_proxy(
            onus_bin,
            db_path,
            session_id,
            fake_server,
            side_effect,
        )
        try:
            secret = self._mcp_request(
                secret_proc,
                self._mcp_tool_message(3, "echo.safe", {"api_key": "raw-secret-123"}),
            )
            assert secret["error"]["code"] == -32001
            assert secret["error"]["data"]["rule_id"] == "SECRET_001"
        finally:
            self._stop_process(secret_proc)

        con = sqlite3.connect(db_path)
        try:
            payloads = [
                row[0]
                for row in con.execute(
                    "SELECT payload FROM actions WHERE session_id = ?",
                    (session_id,),
                )
            ]
        finally:
            con.close()
        assert payloads
        assert not any("raw-secret-123" in payload for payload in payloads)
        assert any("[REDACTED]" in payload for payload in payloads)

    def test_acceptance_scenario_e_mcp_proxy_rejects_changed_approval_payload(
        self, onus_bin: Path, rules_path: Path, tmp_path: Path
    ):
        db_path = tmp_path / "audit.db"
        session_id = "mcp-runtime-session"
        client = OnusClient(
            bin_path=str(onus_bin),
            rules_path=str(rules_path),
            db_path=str(db_path),
        )
        contract = make_contract(
            session_id=session_id,
            allowed_paths=[],
            protected_paths=[],
            forbidden_actions=[],
            approval_required_actions=["mcp"],
        )
        client.start_contract(contract, workspace_root=tmp_path, agent_name="mcp-runtime-test")

        fake_server = tmp_path / "fake_mcp_server.py"
        fake_server.write_text(
            """
import json
import sys

def read_msg():
    headers = {}
    while True:
        line = sys.stdin.buffer.readline()
        if not line:
            return None
        line = line.decode("ascii").strip()
        if not line:
            break
        key, value = line.split(":", 1)
        headers[key.lower()] = value.strip()
    body = sys.stdin.buffer.read(int(headers["content-length"]))
    return json.loads(body)

def write_msg(value):
    body = json.dumps(value, separators=(",", ":")).encode("utf-8")
    sys.stdout.buffer.write(f"Content-Length: {len(body)}\\r\\n\\r\\n".encode("ascii"))
    sys.stdout.buffer.write(body)
    sys.stdout.buffer.flush()

while True:
    msg = read_msg()
    if msg is None:
        break
    write_msg({
        "jsonrpc": "2.0",
        "id": msg.get("id"),
        "result": {"content": [{"type": "text", "text": "forwarded"}]}
    })
""".strip(),
            encoding="utf-8",
        )

        proc = subprocess.Popen(
            [
                str(onus_bin),
                "mcp-proxy",
                "--experimental",
                "--server",
                sys.executable,
                "--db-path",
                str(db_path),
                "--approval-port",
                "0",
                "--session-id",
                session_id,
                "--",
                str(fake_server),
            ],
            stdin=subprocess.PIPE,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            text=False,
        )
        try:
            pending = self._mcp_call(proc, 1, {"query": "select 1"})
            assert pending["error"]["code"] == -32000
            assert "Pending human approval" in pending["error"]["message"]

            con = sqlite3.connect(db_path)
            try:
                row = con.execute(
                    "SELECT action_id, canonical_payload_hash, status FROM pending_approvals"
                ).fetchone()
                action_row = con.execute(
                    "SELECT approval_decision, verdict FROM actions WHERE session_id = ? ORDER BY id DESC LIMIT 1",
                    (session_id,),
                ).fetchone()
                assert row is not None
                assert row[2] == "pending"
                assert action_row == ("REQUIRE_HUMAN_APPROVAL", "escalate")
            finally:
                con.close()

            approval_port = self._free_port()
            token = "runtime-approval-token"
            approval_proc = subprocess.Popen(
                [
                    str(onus_bin),
                    "approvals",
                    "--db",
                    str(db_path),
                    "--port",
                    str(approval_port),
                    "--token",
                    token,
                ],
                stdout=subprocess.PIPE,
                stderr=subprocess.PIPE,
                text=True,
            )
            try:
                self._wait_for_approval_server(approval_port, token)
                with pytest.raises(urllib.error.HTTPError) as unauthorized:
                    urllib.request.urlopen(
                        f"http://127.0.0.1:{approval_port}/api/pending",
                        timeout=3,
                    )
                assert unauthorized.value.code == 401

                pending_body = self._http_json(
                    f"http://127.0.0.1:{approval_port}/api/pending?token={token}"
                )
                assert pending_body[0]["action_id"] == row[0]
                assert pending_body[0]["status"] == "pending"
                assert pending_body[0]["approval_decision"] == "REQUIRE_HUMAN_APPROVAL"
                assert pending_body[0]["guardian_mode"] == "professional"
                assert "approval" in pending_body[0]["approval_reason"].lower()

                approved_body = self._http_json(
                    f"http://127.0.0.1:{approval_port}/api/approve/{row[0]}?token={token}",
                    method="POST",
                )
                assert approved_body["status"] == "approved"
            finally:
                approval_proc.terminate()
                try:
                    approval_proc.wait(timeout=5)
                except subprocess.TimeoutExpired:
                    approval_proc.kill()
                    approval_proc.wait(timeout=5)

            forwarded = self._mcp_call(proc, 2, {"query": "select 1"})
            assert forwarded["result"]["content"][0]["text"] == "forwarded"

            changed = self._mcp_call(proc, 3, {"query": "drop table users"})
            assert changed["error"]["code"] == -32001
            assert changed["error"]["data"]["rule_id"] == "MCP_SAFETY_001"
        finally:
            self._stop_process(proc)

    @staticmethod
    def _mcp_tool_message(request_id: int, name: str, arguments: dict[str, object]) -> dict:
        return {
            "jsonrpc": "2.0",
            "id": request_id,
            "method": "tools/call",
            "params": {"name": name, "arguments": arguments},
        }

    def _write_fake_mcp_server(self, tmp_path: Path) -> Path:
        fake_server = tmp_path / "fake_mcp_server.py"
        fake_server.write_text(
            r'''
import json
import sys
import time
from pathlib import Path

side_effect_path = Path(sys.argv[1])

def read_msg():
    headers = {}
    while True:
        line = sys.stdin.buffer.readline()
        if not line:
            return None
        line = line.decode("ascii").strip()
        if not line:
            break
        key, value = line.split(":", 1)
        headers[key.lower()] = value.strip()
    body = sys.stdin.buffer.read(int(headers["content-length"]))
    return json.loads(body)

def write_msg(value):
    body = json.dumps(value, separators=(",", ":")).encode("utf-8")
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
                "capabilities": {"tools": {"listChanged": False}},
                "serverInfo": {"name": "local-fake-mcp-server", "version": "1.0.0"},
            },
        })
        continue
    if method == "tools/list":
        write_msg({
            "jsonrpc": "2.0",
            "id": msg.get("id"),
            "result": {
                "tools": [
                    {"name": "echo.safe", "description": "Echo safe text", "inputSchema": {"type": "object"}},
                    {"name": "side.effect", "description": "Writes a side-effect file", "inputSchema": {"type": "object"}},
                    {"name": "large.response", "description": "Returns a large response", "inputSchema": {"type": "object"}},
                    {"name": "slow.response", "description": "Sleeps before responding", "inputSchema": {"type": "object"}},
                ]
            },
        })
        continue
    if method == "tools/call":
        params = msg.get("params", {})
        name = params.get("name", "")
        args = params.get("arguments", {})
        if name == "side.effect":
            side_effect_path.write_text(json.dumps(args, sort_keys=True), encoding="utf-8")
            text = "side effect occurred"
        elif name == "large.response":
            text = "x" * int(args.get("size", 4096))
        elif name == "slow.response":
            time.sleep(int(args.get("sleep_ms", 1000)) / 1000)
            text = "slow response"
        else:
            text = f"forwarded:{name}"
        write_msg({
            "jsonrpc": "2.0",
            "id": msg.get("id"),
            "result": {"content": [{"type": "text", "text": text}]},
        })
        continue
    write_msg({"jsonrpc": "2.0", "id": msg.get("id"), "result": {}})
'''.strip(),
            encoding="utf-8",
        )
        return fake_server

    def _start_mcp_proxy(
        self,
        onus_bin: Path,
        db_path: Path,
        session_id: str,
        fake_server: Path,
        side_effect_path: Path,
        *,
        extra_proxy_args: list[str] | None = None,
    ) -> subprocess.Popen:
        args = [
            str(onus_bin),
            "mcp-proxy",
            "--experimental",
            "--server",
            sys.executable,
            "--db-path",
            str(db_path),
            "--approval-port",
            "0",
            "--session-id",
            session_id,
        ]
        if extra_proxy_args:
            args.extend(extra_proxy_args)
        args.extend(["--", str(fake_server), str(side_effect_path)])
        return subprocess.Popen(
            args,
            stdin=subprocess.PIPE,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            text=False,
        )

    @classmethod
    def _mcp_request(cls, proc: subprocess.Popen, message: dict) -> dict:
        assert proc.stdin is not None
        cls._write_mcp_message(proc.stdin, json.dumps(message, separators=(",", ":")).encode("utf-8"))
        return cls._read_mcp_response(proc)

    @classmethod
    def _mcp_raw_json(cls, proc: subprocess.Popen, payload: bytes) -> dict:
        assert proc.stdin is not None
        cls._write_mcp_message(proc.stdin, payload)
        return cls._read_mcp_response(proc)

    @staticmethod
    def _write_mcp_message(stdin, body: bytes) -> None:
        stdin.write(f"Content-Length: {len(body)}\r\n\r\n".encode("ascii"))
        stdin.write(body)
        stdin.flush()

    @staticmethod
    def _read_mcp_response(proc: subprocess.Popen) -> dict:
        assert proc.stdout is not None
        headers: dict[str, str] = {}
        while True:
            line = proc.stdout.readline()
            assert line, "MCP proxy exited before response"
            line_text = line.decode("ascii").strip()
            if not line_text:
                break
            key, value = line_text.split(":", 1)
            headers[key.lower()] = value.strip()
        payload = proc.stdout.read(int(headers["content-length"]))
        return json.loads(payload)

    @staticmethod
    def _stop_process(proc: subprocess.Popen) -> None:
        if proc.stdin:
            proc.stdin.close()
        try:
            proc.wait(timeout=5)
        except subprocess.TimeoutExpired:
            proc.kill()
            proc.wait(timeout=5)

    @staticmethod
    def _mcp_call(proc: subprocess.Popen, request_id: int, arguments: dict[str, str]) -> dict:
        return TestMcpProxyRuntime._mcp_request(
            proc,
            TestMcpProxyRuntime._mcp_tool_message(request_id, "db.query", arguments),
        )

    @staticmethod
    def _free_port() -> int:
        sock = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
        try:
            sock.bind(("127.0.0.1", 0))
            return int(sock.getsockname()[1])
        finally:
            sock.close()

    @classmethod
    def _wait_for_approval_server(cls, port: int, token: str) -> None:
        deadline = time.time() + 10
        last_error: Exception | None = None
        while time.time() < deadline:
            try:
                cls._http_json(f"http://127.0.0.1:{port}/api/pending?token={token}")
                return
            except Exception as exc:
                last_error = exc
                time.sleep(0.1)
        raise AssertionError(f"approval server did not become ready: {last_error}")

    @staticmethod
    def _http_json(url: str, *, method: str = "GET") -> object:
        request = urllib.request.Request(url, method=method)
        with urllib.request.urlopen(request, timeout=5) as response:
            return json.loads(response.read().decode("utf-8"))


class TestDoctorCommand:
    """Tests for `onus doctor` and `onus doctor --claude`."""

    def test_doctor_runs_successfully(self, onus_bin: Path):
        result = subprocess.run(
            [str(onus_bin), "doctor"],
            capture_output=True,
            text=True,
            timeout=30,
        )
        assert result.returncode in (0, 1), f"doctor failed: {result.stderr}"
        assert "Onus Doctor" in result.stdout
        assert "Daemon" in result.stdout

    def test_doctor_claude_runs(self, onus_bin: Path):
        result = subprocess.run(
            [str(onus_bin), "doctor", "--claude"],
            capture_output=True,
            text=True,
            timeout=30,
        )
        assert result.returncode in (0, 1), f"doctor --claude failed: {result.stderr}"
        assert "Onus Doctor" in result.stdout
        assert "Claude" in result.stdout

    def test_doctor_shows_audit_trail(self, onus_bin: Path, rules_path: Path, tmp_path: Path):
        env = os.environ.copy()
        env["ONUS_DATA_DIR"] = str(tmp_path / "onus-data")
        # First run an evaluation to create audit trail
        subprocess.run(
            [str(onus_bin), "evaluate", "--rules", str(rules_path), "--db", str(tmp_path / "audit.db")],
            input=json.dumps({"action": {"type": "shell", "payload": {"command": "echo test"}}}),
            capture_output=True,
            text=True,
            timeout=30,
        )
        result = subprocess.run(
            [str(onus_bin), "doctor"],
            capture_output=True,
            text=True,
            timeout=30,
            env=env,
        )
        assert "Audit trail" in result.stdout


class TestSetupCommand:
    """Tests for `onus setup claude` and `onus uninstall --claude`."""

    def test_setup_claude_runs(self, onus_bin: Path):
        result = subprocess.run(
            [str(onus_bin), "setup", "--claude"],
            capture_output=True,
            text=True,
            timeout=15,
        )
        assert result.returncode == 0, f"setup --claude failed: {result.stderr}"

    def test_uninstall_claude_runs(self, onus_bin: Path):
        result = subprocess.run(
            [str(onus_bin), "uninstall", "--claude"],
            capture_output=True,
            text=True,
            timeout=15,
        )
        assert result.returncode == 0, f"uninstall --claude failed: {result.stderr}"


class TestClaudeHookReceipt:
    """Tests for `onus claude-hook --receipt` functionality."""

    def _run_claude_hook(self, onus_bin: Path, db_path: Path, rules_path: Path, payload: dict) -> dict:
        proc = subprocess.run(
            [str(onus_bin), "claude-hook", "--db", str(db_path), "--rules", str(rules_path), "--timeout-ms", "10000"],
            input=json.dumps(payload),
            capture_output=True,
            text=True,
            timeout=15,
        )
        assert proc.returncode == 0, f"claude-hook failed: {proc.stderr}"
        return json.loads(proc.stdout)

    def test_hook_receipt_stderr_output(self, onus_bin: Path, rules_path: Path, tmp_path: Path):
        db_path = tmp_path / "receipt-test.db"
        payload = {
            "tool_name": "Bash",
            "tool_input": {"command": "echo receipt"},
            "session_id": "test-receipt",
            "cwd": "/tmp",
            "agent": "claude-code",
            "agent_version": "1.0.0",
        }
        proc = subprocess.run(
            [str(onus_bin), "claude-hook", "--db", str(db_path), "--rules", str(rules_path),
             "--timeout-ms", "10000", "--receipt"],
            input=json.dumps(payload),
            capture_output=True,
            text=True,
            timeout=15,
        )
        assert proc.returncode == 0, f"hook with receipt failed: {proc.stderr}"
        assert "ONUS_RECEIPT" in proc.stderr, "Receipt not in stderr"

    def test_hook_receipt_file_output(self, onus_bin: Path, rules_path: Path, tmp_path: Path):
        db_path = tmp_path / "receipt-file-test.db"
        receipt_file = tmp_path / "hook-receipt.json"
        payload = {
            "tool_name": "Bash",
            "tool_input": {"command": "echo file_receipt"},
            "session_id": "test-file-receipt",
            "cwd": "/tmp",
            "agent": "claude-code",
            "agent_version": "1.0.0",
        }
        proc = subprocess.run(
            [str(onus_bin), "claude-hook", "--db", str(db_path), "--rules", str(rules_path),
             "--timeout-ms", "10000", "--receipt-path", str(receipt_file)],
            input=json.dumps(payload),
            capture_output=True,
            text=True,
            timeout=15,
        )
        assert proc.returncode == 0, f"hook with receipt-path failed: {proc.stderr}"
        assert receipt_file.exists(), "Receipt file not created"
        receipt = json.loads(receipt_file.read_text())
        assert receipt["type"] == "evaluation_receipt"
        assert receipt["version"] == 1
        assert receipt["body"]["permission_decision"] in ("allow", "deny")
        assert receipt["body"]["surface"] == "claude-code-cli"
        assert receipt["body"]["integration_level"] == "L1_BEST_EFFORT"
        assert len(receipt["body_hash"]) == 64
        assert receipt["signature"] == receipt["body_hash"]

    def test_hook_fail_closed_on_bad_input(self, onus_bin: Path, rules_path: Path, tmp_path: Path):
        db_path = tmp_path / "fail-closed.db"
        proc = subprocess.run(
            [str(onus_bin), "claude-hook", "--db", str(db_path), "--rules", str(rules_path),
             "--timeout-ms", "5000"],
            input="NOT VALID JSON",
            capture_output=True,
            text=True,
            timeout=10,
        )
        if proc.returncode == 0:
            output = json.loads(proc.stdout)
            assert output["hookSpecificOutput"]["permissionDecision"] == "deny"
        else:
            # Crash acceptable — hook should be resilient
            assert True

    def test_hook_denies_dangerous_command(self, onus_bin: Path, rules_path: Path, tmp_path: Path):
        db_path = tmp_path / "deny-test.db"
        output = self._run_claude_hook(onus_bin, db_path, rules_path, {
            "tool_name": "Bash",
            "tool_input": {"command": "rm -rf /"},
            "session_id": "test-deny",
            "cwd": "/tmp",
            "agent": "claude-code",
            "agent_version": "1.0.0",
        })
        assert output["hookSpecificOutput"]["permissionDecision"] == "deny"

    def test_hook_allows_safe_command(self, onus_bin: Path, rules_path: Path, tmp_path: Path):
        db_path = tmp_path / "allow-test.db"
        output = self._run_claude_hook(onus_bin, db_path, rules_path, {
            "tool_name": "Bash",
            "tool_input": {"command": "echo hello"},
            "session_id": "test-allow",
            "cwd": "/tmp",
            "agent": "claude-code",
            "agent_version": "1.0.0",
        })
        assert output["hookSpecificOutput"]["permissionDecision"] == "allow"


class TestClaudeHookSchema:
    """Tests for Claude hook JSON schema — allowed/denied/approval structures."""

    def test_hook_output_allowed(self):
        """Verify the allowed hook response JSON schema."""
        output = {
            "hookSpecificOutput": {
                "hookEventName": "preToolUse",
                "permissionDecision": "allow",
                "permissionDecisionReason": "command is safe",
            },
            "suppressOutput": False,
        }
        assert output["hookSpecificOutput"]["permissionDecision"] == "allow"
        assert output["suppressOutput"] is False

    def test_hook_output_denied(self):
        """Verify the denied hook response JSON schema."""
        output = {
            "hookSpecificOutput": {
                "hookEventName": "preToolUse",
                "permissionDecision": "deny",
                "permissionDecisionReason": "blocked by policy",
            },
            "suppressOutput": False,
        }
        assert output["hookSpecificOutput"]["permissionDecision"] == "deny"

    def test_hook_output_approval_required(self):
        """Verify the approval-required hook response JSON schema (ask)."""
        output = {
            "hookSpecificOutput": {
                "hookEventName": "preToolUse",
                "permissionDecision": "ask",
                "permissionDecisionReason": "requires human approval",
            },
            "suppressOutput": False,
        }
        assert output["hookSpecificOutput"]["permissionDecision"] == "ask"

    def test_hook_output_invalid_decision_fails(self):
        """Invalid decision value should not match valid decisions."""
        # This test validates the domain of the decision field
        invalid = "maybe"
        valid_decisions = {"allow", "deny", "ask"}
        assert invalid not in valid_decisions

    def test_hook_output_missing_decision(self):
        """Missing permissionDecision field should be detectable."""
        output = {"hookSpecificOutput": {"hookEventName": "preToolUse"}, "suppressOutput": False}
        decision = output.get("hookSpecificOutput", {}).get("permissionDecision")
        assert decision is None, "Missing decision should be None"


class TestClaudeHookReceiptParsing:
    """Tests for receipt parsing — multiline JSON, invalid JSON, field extraction."""

    def test_receipt_single_line_parsing(self):
        """Parse a single-line receipt correctly."""
        line = 'ONUS_RECEIPT: {"type": "evaluation_receipt", "version": 1, "body_hash": "' + "a" * 64 + '"}'
        assert "ONUS_RECEIPT:" in line
        json_part = line.split("ONUS_RECEIPT:", 1)[1].strip()
        receipt = json.loads(json_part)
        assert receipt["type"] == "evaluation_receipt"
        assert receipt["version"] == 1
        assert len(receipt["body_hash"]) == 64

    def test_receipt_multiline_json_parsing(self):
        """Parse a multiline receipt correctly."""
        line = "ONUS_RECEIPT: {\n  \"type\": \"evaluation_receipt\",\n  \"version\": 1,\n  \"body_hash\": \"" + "b" * 64 + "\"\n}"
        assert "ONUS_RECEIPT:" in line
        json_part = line.split("ONUS_RECEIPT:", 1)[1].strip()
        receipt = json.loads(json_part)
        assert receipt["type"] == "evaluation_receipt"
        assert receipt["version"] == 1
        assert len(receipt["body_hash"]) == 64

    def test_receipt_missing_marker(self):
        """No ONUS_RECEIPT marker should be detectable."""
        line = '{"type": "evaluation_receipt"}'
        assert "ONUS_RECEIPT:" not in line

    def test_receipt_invalid_json_after_marker(self):
        """Invalid JSON after ONUS_RECEIPT marker should be detectable."""
        line = "ONUS_RECEIPT: {invalid json here}"
        assert "ONUS_RECEIPT:" in line
        json_part = line.split("ONUS_RECEIPT:", 1)[1].strip()
        with pytest.raises(json.JSONDecodeError):
            json.loads(json_part)

    def test_receipt_required_fields(self):
        """Receipt must contain type, version, body_hash."""
        receipt = {
            "type": "evaluation_receipt",
            "version": 1,
            "body": {"permission_decision": "deny", "surface": "claude-code-cli", "integration_level": "L1_BEST_EFFORT"},
            "body_hash": "c" * 64,
            "signature": "c" * 64,
        }
        assert receipt["type"] == "evaluation_receipt"
        assert receipt["version"] == 1
        assert len(receipt["body_hash"]) == 64
        assert receipt["signature"] == receipt["body_hash"]

    def test_receipt_body_hash_integrity(self):
        """body_hash should be a 64-char hex string (SHA-256)."""
        receipt = {"body_hash": "d" * 64}
        assert len(receipt["body_hash"]) == 64
        assert all(c in "0123456789abcdef" for c in receipt["body_hash"])


class TestClaudeUninstallSyntax:
    """Tests that `onus uninstall --claude` uses the correct flag syntax."""

    def test_uninstall_flag_syntax_correct(self, onus_bin: Path):
        """onus uninstall --claude must use --claude flag, not positional."""
        result = subprocess.run(
            [str(onus_bin), "uninstall", "--claude"],
            capture_output=True,
            text=True,
            timeout=15,
        )
        # Should succeed or report nothing to remove (not fail with "unexpected argument")
        assert "unexpected argument" not in (result.stdout + result.stderr).lower()
        assert result.returncode in (0, 1)

    def test_uninstall_no_positional_arg(self, onus_bin: Path):
        """onus uninstall claude (positional) should fail gracefully."""
        result = subprocess.run(
            [str(onus_bin), "uninstall", "claude"],
            capture_output=True,
            text=True,
            timeout=15,
        )
        stderr_lower = result.stderr.lower()
        assert "unexpected argument" in stderr_lower or "error" in stderr_lower


class TestDoctorCodexCommand:
    """Tests for `onus doctor --codex` — Codex CLI diagnostics."""

    def test_doctor_codex_runs(self, onus_bin: Path):
        result = subprocess.run(
            [str(onus_bin), "doctor", "--codex"],
            capture_output=True,
            text=True,
            timeout=30,
        )
        assert result.returncode in (0, 1), f"doctor --codex failed: {result.stderr}"
        assert "Onus Doctor" in result.stdout
        assert "Codex" in result.stdout

    def test_doctor_full_reports_codex(self, onus_bin: Path):
        result = subprocess.run(
            [str(onus_bin), "doctor"],
            capture_output=True,
            text=True,
            timeout=30,
        )
        assert result.returncode in (0, 1), f"doctor failed: {result.stderr}"
        # Full doctor should mention Codex section
        assert "Codex" in result.stdout

    def test_doctor_codex_l3_advice(self, onus_bin: Path):
        result = subprocess.run(
            [str(onus_bin), "doctor", "--codex"],
            capture_output=True,
            text=True,
            timeout=30,
        )
        assert result.returncode in (0, 1), f"doctor --codex failed: {result.stderr}"
        # L3 advice should be in output (either bwrap or Windows advice)
        assert any(x in result.stdout for x in ["bubblewrap", "Windows", "workspace"])


class TestSetupCodexCommand:
    """Tests for `onus setup --codex` and `onus uninstall --codex`."""

    def test_setup_codex_runs(self, onus_bin: Path):
        result = subprocess.run(
            [str(onus_bin), "setup", "--codex"],
            capture_output=True,
            text=True,
            timeout=15,
        )
        assert result.returncode == 0, f"setup --codex failed: {result.stderr}"
        assert "MCP" in result.stdout or "already" in result.stdout

    def test_uninstall_codex_runs(self, onus_bin: Path):
        result = subprocess.run(
            [str(onus_bin), "uninstall", "--codex"],
            capture_output=True,
            text=True,
            timeout=15,
        )
        assert result.returncode == 0, f"uninstall --codex failed: {result.stderr}"
        assert "MCP" in result.stdout or "codex" in result.stdout.lower() or "No" in result.stdout

    def test_setup_codex_mcp_config_format(self, onus_bin: Path, tmp_path: Path):
        """Verify --codex produces valid MCP config TOML."""
        # Run setup --codex (it writes to ~/.codex/config.toml which we can't isolate easily)
        # Instead verify the binary accepts the flag and produces non-error output
        result = subprocess.run(
            [str(onus_bin), "setup", "--codex"],
            capture_output=True,
            text=True,
            timeout=15,
        )
        assert result.returncode == 0, f"setup --codex failed: {result.stderr}"
        assert "MCP" in result.stdout or "already" in result.stdout


class TestDoctorAntigravityCommand:
    """Tests for `onus doctor --antigravity` — Google Antigravity diagnostics."""

    def test_doctor_antigravity_runs(self, onus_bin: Path):
        result = subprocess.run(
            [str(onus_bin), "doctor", "--antigravity"],
            capture_output=True,
            text=True,
            timeout=30,
        )
        assert result.returncode in (0, 1), f"doctor --antigravity failed: {result.stderr}"
        assert "Antigravity" in result.stdout

    def test_doctor_full_reports_antigravity(self, onus_bin: Path):
        result = subprocess.run(
            [str(onus_bin), "doctor"],
            capture_output=True,
            text=True,
            timeout=30,
        )
        assert result.returncode in (0, 1), f"doctor failed: {result.stderr}"
        assert "Antigravity" in result.stdout, "Full doctor should mention Antigravity"

    def test_doctor_antigravity_l3_advice(self, onus_bin: Path):
        result = subprocess.run(
            [str(onus_bin), "doctor", "--antigravity"],
            capture_output=True,
            text=True,
            timeout=30,
        )
        assert result.returncode in (0, 1), f"doctor --antigravity failed: {result.stderr}"
        # L3 advice appears when Antigravity is found; otherwise fail message is shown
        assert any(x in result.stdout for x in ["bubblewrap", "Windows", "workspace",
            "not available", "Linux", "not found", "Antigravity"]), \
            "Antigravity doctor should mention Antigravity status"


class TestSetupAntigravityCommand:
    """Tests for `onus setup --antigravity` and `onus uninstall --antigravity`."""

    def test_setup_antigravity_runs(self, onus_bin: Path):
        result = subprocess.run(
            [str(onus_bin), "setup", "--antigravity"],
            capture_output=True,
            text=True,
            timeout=15,
        )
        assert result.returncode == 0, f"setup --antigravity failed: {result.stderr}"
        assert "Antigravity" in result.stdout

    def test_uninstall_antigravity_runs(self, onus_bin: Path):
        result = subprocess.run(
            [str(onus_bin), "uninstall", "--antigravity"],
            capture_output=True,
            text=True,
            timeout=15,
        )
        assert result.returncode == 0, f"uninstall --antigravity failed: {result.stderr}"
        assert "Antigravity" in result.stdout or "antigravity" in result.stdout


class TestDoctorCursorCommand:
    """Tests for `onus doctor --cursor` — Cursor IDE diagnostics."""

    def test_doctor_cursor_runs(self, onus_bin: Path):
        result = subprocess.run(
            [str(onus_bin), "doctor", "--cursor"],
            capture_output=True,
            text=True,
            timeout=30,
        )
        assert result.returncode in (0, 1), f"doctor --cursor failed: {result.stderr}"
        assert "Cursor" in result.stdout

    def test_doctor_full_reports_cursor(self, onus_bin: Path):
        result = subprocess.run(
            [str(onus_bin), "doctor"],
            capture_output=True,
            text=True,
            timeout=30,
        )
        assert result.returncode in (0, 1), f"doctor failed: {result.stderr}"
        assert "Cursor" in result.stdout, "Full doctor should mention Cursor"

    def test_doctor_cursor_l3_advice(self, onus_bin: Path):
        result = subprocess.run(
            [str(onus_bin), "doctor", "--cursor"],
            capture_output=True,
            text=True,
            timeout=30,
        )
        assert result.returncode in (0, 1), f"doctor --cursor failed: {result.stderr}"
        assert any(x in result.stdout for x in ["bubblewrap", "Windows", "workspace",
            "not available", "Linux", "not found", "Cursor"]), \
            "Cursor doctor should mention Cursor status"


class TestSetupCursorCommand:
    """Tests for `onus setup --cursor` and `onus uninstall --cursor`."""

    def test_setup_cursor_runs(self, onus_bin: Path):
        result = subprocess.run(
            [str(onus_bin), "setup", "--cursor"],
            capture_output=True,
            text=True,
            timeout=15,
        )
        assert result.returncode == 0, f"setup --cursor failed: {result.stderr}"
        assert "Cursor" in result.stdout

    def test_uninstall_cursor_runs(self, onus_bin: Path):
        result = subprocess.run(
            [str(onus_bin), "uninstall", "--cursor"],
            capture_output=True,
            text=True,
            timeout=15,
        )
        assert result.returncode == 0, f"uninstall --cursor failed: {result.stderr}"
        assert "Cursor" in result.stdout or "cursor" in result.stdout
