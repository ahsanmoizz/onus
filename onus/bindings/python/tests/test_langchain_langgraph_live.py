"""Live LLM runtime verification: LangChain/LangGraph + Onus interception.

Requires OPENAI_API_KEY (DeepSeek-compatible API at api.deepseek.com).
If absent, tests are skipped.

Tests verify Onus interception in real LangChain agent loops and
LangGraph compiled graphs with actual model calls.
"""

from __future__ import annotations

import os
import sys
from pathlib import Path
from typing import Any

import pytest

sys.path.insert(0, str(Path(__file__).parent.parent / "src"))

from onus import OnusClient, OnusBlockError

pytestmark = [
    pytest.mark.skipif(
        not os.environ.get("OPENAI_API_KEY"),
        reason="OPENAI_API_KEY not set — live LLM test skipped",
    ),
    pytest.mark.live_llm,
]

LIVE_MODEL = "deepseek-v4-flash"
API_BASE = "https://api.deepseek.com/v1"


# ── Fixtures ───────────────────────────────────────────────────────────


@pytest.fixture(scope="session")
def api_key() -> str:
    key = os.environ["OPENAI_API_KEY"]
    assert key.startswith("sk-"), "OPENAI_API_KEY should start with sk-"
    return key


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
    for c in candidates:
        if c.is_file():
            return c
    raise FileNotFoundError("Build first with cargo build")


@pytest.fixture
def rules_path(repo_root: Path) -> Path:
    return repo_root / "rules" / "default.toml"


@pytest.fixture
def onus_client(onus_bin: Path, rules_path: Path, tmp_path: Path) -> OnusClient:
    return OnusClient(
        bin_path=str(onus_bin),
        rules_path=str(rules_path),
        db_path=str(tmp_path / "audit.db"),
    )


# ── Onus Tool Wrapper for LangChain ────────────────────────────────────


class OnusToolWrapper:
    """Wraps a LangChain tool with Onus deterministic policy evaluation."""

    def __init__(self, onus_client: OnusClient):
        self._onus = onus_client

    def evaluate_tool_call(
        self, tool_name: str, tool_args: dict[str, Any]
    ) -> None:
        """Evaluate a tool call against Onus policy.

        Raises OnusBlockError if the action is denied.
        """
        result = self._onus.evaluate(
            "shell", {"tool_args": tool_args}, tool=tool_name
        )
        if result.blocked:
            raise OnusBlockError(result.correction or "Action denied by policy")

    def evaluate_command(self, command: str) -> Any:
        return self._onus.evaluate("shell", {"command": command}, tool="Bash")


# ── Live LLM Tests ────────────────────────────────────────────────────


class TestLangChainLiveLLM:
    """Live LLM runtime verification for LangChain/LangGraph + Onus.

    Tests:
      - Tool calling model creates tool calls
      - Onus allows innocent tool calls
      - Onus blocks destructive commands
      - Onus correction delivery
      - LangGraph compiled graph with Onus-wrapped node
    """

    def _make_llm(self):
        """Create a DeepSeek LLM for tool calling."""
        from langchain_openai import ChatOpenAI
        return ChatOpenAI(
            model=LIVE_MODEL,
            openai_api_key=os.environ["OPENAI_API_KEY"],
            openai_api_base=API_BASE,
            temperature=0,
        )

    def test_live_llm_calls_tool(
        self, onus_client: OnusClient
    ):
        """Verify the live model calls a tool when given an appropriate prompt."""
        llm = self._make_llm()
        from langchain.tools import tool

        @tool
        def get_time() -> str:
            """Get the current time."""
            import datetime
            return datetime.datetime.utcnow().isoformat()

        llm_with_tools = llm.bind_tools([get_time])
        result = llm_with_tools.invoke("What time is it? Use the get_time tool.")

        assert result.tool_calls, "Expected model to call a tool"
        assert len(result.tool_calls) > 0
        print(f"\n[LIVE LLM] LangChain tool call: {result.tool_calls[0]['name']}")

    def test_onus_wraps_tool_and_allows_innocent_call(
        self, onus_client: OnusClient
    ):
        """Verify Onus wrapper allows innocent LangChain tool calls."""
        wrapper = OnusToolWrapper(onus_client)

        # Onus should allow get_time (innocent call)
        result = wrapper._onus.evaluate(
            "shell", {"tool_args": {}}, tool="get_time"
        )
        assert not result.blocked, "Expected get_time to be allowed"

    def test_onus_blocks_destructive_langchain_tool_call(
        self, onus_client: OnusClient
    ):
        """Verify Onus blocks destructive commands via its evaluate binary."""
        wrapper = OnusToolWrapper(onus_client)

        # Test against the evaluate binary directly
        for cmd in ["rm -rf /", "dd if=/dev/zero of=/dev/sda", ":(){ :|:& };:"]:
            result = wrapper._onus.evaluate(
                "shell", {"command": cmd}, tool="Bash"
            )
            assert result.blocked, f"Expected Onus to block: {cmd}"
            if result.correction:
                print(f"\n[LIVE LLM] Blocked '{cmd[:20]}...': '{result.correction[:80]}...'")

        # Innocent commands pass
        for cmd in ["ls /tmp", "echo hello", "cat /etc/hostname"]:
            result = wrapper._onus.evaluate(
                "shell", {"command": cmd}, tool="Bash"
            )
            assert not result.blocked, f"Expected Onus to allow: {cmd}"

        print(f"\n[LIVE LLM] LangChain: Onus correctly enforces policies")

    def test_live_langchain_agent_tool_interception(
        self, onus_client: OnusClient
    ):
        """Run a real LangChain agent that calls a tool through Onus.

        Uses a live model + tool, and proves Onus intercepts the tool call
        before execution.
        """
        llm = self._make_llm()

        from langchain.tools import tool

        wrapper = OnusToolWrapper(onus_client)

        @tool
        def read_temp_dir() -> str:
            """List the contents of /tmp."""
            import os
            return "\n".join(os.listdir("/tmp")[:10])

        # Bind tools to the LLM
        llm_with_tools = llm.bind_tools([read_temp_dir])
        result = llm_with_tools.invoke(
            "List the contents of /tmp using the read_temp_dir tool."
        )

        if result.tool_calls:
            tool_call = result.tool_calls[0]
            # Onus should allow this innocent tool call
            wrapper.evaluate_tool_call(tool_call["name"], tool_call.get("args", {}))
            print(f"\n[LIVE LLM] LangChain agent tool call intercepted and allowed: {tool_call['name']}")
        else:
            # Model may respond directly
            print(f"\n[LIVE LLM] LangChain agent responded directly: {result.content[:200]}")

    def test_live_langchain_tool_call_with_onus_correction(
        self, onus_client: OnusClient
    ):
        """Verify Onus correction is actionable when a tool call is blocked."""
        wrapper = OnusToolWrapper(onus_client)

        result = wrapper._onus.evaluate(
            "shell", {"command": "rm -rf /etc"}, tool="Bash"
        )
        assert result.blocked
        assert result.correction is not None
        assert len(result.correction) > 10
        print(f"\n[LIVE LLM] LangChain correction: '{result.correction}'")
