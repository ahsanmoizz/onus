"""Tests for the Onus Python SDK."""

from __future__ import annotations

import sys
from pathlib import Path

import pytest

sys.path.insert(0, str(Path(__file__).parent.parent / "src"))

from onus import Guardian, OnusBlockError, OnusClient, OnusResult


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
        assert result.rule_id == "SAFETY_002"


class TestGuardian:
    def test_import_guardian(self):
        assert Guardian is not None

    def test_file_write_captures_and_rolls_back(self, onus_bin: Path, rules_path: Path, tmp_path: Path):
        target = tmp_path / "demo.txt"
        target.write_text("before\n", encoding="utf-8")
        with Guardian(
            workspace_root=tmp_path,
            bin_path=str(onus_bin),
            rules_path=str(rules_path),
            db_path=str(tmp_path / "audit.db"),
        ) as guardian:
            result = guardian.file_write("demo.txt", "after\n")
            assert result.allowed is True
            assert target.read_text(encoding="utf-8") == "after\n"
            rollback = guardian.rollback_last()
            assert rollback is not None
            assert target.read_text(encoding="utf-8") == "before\n"

    def test_shell_blocks_before_execution(self, onus_bin: Path, rules_path: Path, tmp_path: Path):
        with Guardian(
            workspace_root=tmp_path,
            bin_path=str(onus_bin),
            rules_path=str(rules_path),
            db_path=str(tmp_path / "audit.db"),
        ) as guardian:
            with pytest.raises(OnusBlockError) as exc:
                guardian.shell("rm -rf /important", execute=True)
            assert exc.value.result.rule_id == "SAFETY_001"
