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
    RequiredEvidence,
    TaskContract,
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


class TestMcpProxyRuntime:
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
            assert changed["error"]["code"] == -32000
            assert "Pending human approval" in changed["error"]["message"]
        finally:
            if proc.stdin:
                proc.stdin.close()
            try:
                proc.wait(timeout=5)
            except subprocess.TimeoutExpired:
                proc.kill()
                proc.wait(timeout=5)

    @staticmethod
    def _mcp_call(proc: subprocess.Popen, request_id: int, arguments: dict[str, str]) -> dict:
        assert proc.stdin is not None
        assert proc.stdout is not None
        message = {
            "jsonrpc": "2.0",
            "id": request_id,
            "method": "tools/call",
            "params": {"name": "db.query", "arguments": arguments},
        }
        body = json.dumps(message, separators=(",", ":")).encode("utf-8")
        proc.stdin.write(f"Content-Length: {len(body)}\r\n\r\n".encode("ascii"))
        proc.stdin.write(body)
        proc.stdin.flush()

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
