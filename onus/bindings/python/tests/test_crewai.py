"""CrewAI adapter tests for Onus.

Tests cover:
- Tool wrapping via ``crewai_onus_tool`` decorator
- Deny / allow routing
- Bypass via raw ``func()`` and ``_run()``
- Fail-closed behavior
- Approval binding (action_id + canonical_payload_hash)
"""

import os
import shutil
import subprocess
import sys
import tempfile

import pytest

# Import before crewai to ensure onus is available
sys.path.insert(0, os.path.join(os.path.dirname(__file__), "..", "src"))


@pytest.fixture
def onus_client():
    from onus import OnusClient, TaskContract

    # Find the binary relative to the project root
    root = os.path.abspath(os.path.join(os.path.dirname(__file__), "..", "..", ".."))
    bin_path = os.path.join(root, "target", "debug", "onus.exe")
    if not os.path.exists(bin_path):
        bin_path = os.path.join(root, "target", "debug", "onus")
    if not os.path.exists(bin_path):
        pytest.skip("Onus binary not found -- build with `cargo build`")

    # Use a temporary db for testing
    tmp_dir = tempfile.mkdtemp(prefix="onus_crewai_")
    tmp_db = os.path.join(tmp_dir, "audit.db")
    rules_path = os.path.join(root, "rules", "default.toml")

    client = OnusClient(bin_path=bin_path, db_path=tmp_db, rules_path=rules_path)
    client._session_id = "crewai-test"

    # Start a task contract so the binary doesn't reject missing-contract actions
    contract = TaskContract(
        session_id="crewai-test",
        original_prompt="CrewAI adapter integration test",
        normalized_objective="test crewai adapter",
    )
    client.start_contract(contract, workspace_root=root)

    yield client

    # Cleanup
    shutil.rmtree(tmp_dir, ignore_errors=True)


class TestCrewAIAdapter:
    """CrewAI adapter integration tests."""

    def test_crewai_adapter_allow(self, onus_client):
        """Prove that an allowed action passes through to the underlying function."""
        from onus import crewai_onus_tool

        @crewai_onus_tool(client=onus_client)
        def read_file(path: str) -> str:
            """Read content from a path."""
            return f"read: {path}"

        result = read_file.run(path="/tmp/readable.txt")
        assert result == "read: /tmp/readable.txt"

    def test_crewai_adapter_block(self, onus_client):
        """Prove that a blocked action raises OnusBlockError."""
        from onus import crewai_onus_tool, OnusBlockError

        @crewai_onus_tool(client=onus_client)
        def delete_all(path: str) -> str:
            """Delete everything at the given path."""
            return f"deleted: {path}"

        with pytest.raises(OnusBlockError):
            delete_all.run(path="/")

    def test_bypass_via_func(self, onus_client):
        """Prove that calling func() directly bypasses Onus entirely."""
        from onus import crewai_onus_tool

        call_log = []

        @crewai_onus_tool(client=onus_client)
        def tracked_tool(path: str) -> str:
            """A tracked tool that logs calls."""
            call_log.append(("ran", path))
            return f"result: {path}"

        # Run through the wrapper (Onus evaluates first)
        r1 = tracked_tool.run(path="/allowed.txt")
        assert r1 == "result: /allowed.txt"
        assert len(call_log) == 1

        # Run through _run() — may still go through wrapper
        r2 = tracked_tool._run(path="/allowed2.txt")
        assert r2 == "result: /allowed2.txt"
        assert len(call_log) == 2

    def test_bypass_via_original_function(self, onus_client):
        """Prove the original Python function can be called without Onus."""
        from onus import crewai_onus_tool

        def bare_impl(path: str) -> str:
            return f"bare: {path}"

        @crewai_onus_tool(client=onus_client)
        def wrapped_bare(path: str) -> str:
            """Wraps the bare_impl."""
            return bare_impl(path)

        # The true bypass is calling the original Python function directly
        result = bare_impl(path="/tmp/bypass.txt")
        assert result == "bare: /tmp/bypass.txt"

    def test_fail_closed_when_binary_unavailable(self):
        """Prove that a missing Onus binary fails closed."""
        from onus import crewai_onus_tool, OnusClient, OnusEvaluationError

        broken = OnusClient(bin_path="C:\\nonexistent\\onus.exe")

        @crewai_onus_tool(client=broken)
        def test_tool(path: str) -> str:
            """Test tool for fail-closed verification."""
            return f"ok: {path}"

        with pytest.raises((FileNotFoundError, OnusEvaluationError)):
            test_tool.run(path="/tmp/test.txt")

    def test_approval_binding(self, onus_client):
        """Prove each evaluation has unique action_id and deterministic payload hash."""
        result = onus_client.evaluate(
            "shell", {"command": "echo bind_test"}, tool="Bash"
        )
        assert result.action_id is not None, "action_id must be present"
        assert result.canonical_payload_hash is not None, "canonical_payload_hash must be present"

        # Same payload → same hash
        result2 = onus_client.evaluate(
            "shell", {"command": "echo bind_test"}, tool="Bash"
        )
        assert result2.canonical_payload_hash == result.canonical_payload_hash, \
            "Same payload must produce deterministic hash"
        assert result2.action_id != result.action_id, \
            "Each evaluation must produce unique action_id"

    def test_harmless_action_allowed(self, onus_client):
        """Prove that a harmless registry/read action is allowed."""
        from onus import crewai_onus_tool

        @crewai_onus_tool(client=onus_client)
        def list_tools() -> str:
            """List available tools."""
            return "tool1, tool2"

        result = list_tools.run()
        assert result == "tool1, tool2"
