"""Onus Python SDK.

The SDK is intentionally thin over the Rust ``onus`` binary for policy,
hash-chained audit storage, and CLI compatibility. ``Guardian`` adds a real
pre-execution wrapper for Python agents and tools.
"""

from __future__ import annotations

import json
import os
import platform
import shutil
import sqlite3
import subprocess
import uuid
from contextlib import contextmanager
from dataclasses import dataclass, field
from pathlib import Path
from typing import Any, Callable, Iterator, Optional, Union
from urllib import request as urllib_request


@dataclass
class OnusResult:
    """Result of an Onus evaluation."""

    decision: str
    correction: Optional[str] = None
    rule_id: Optional[str] = None
    rule_name: Optional[str] = None
    latency_us: Optional[int] = None
    action_id: Optional[str] = None
    reversibility: Optional[str] = None
    raw: dict[str, Any] = field(default_factory=dict)

    @property
    def allowed(self) -> bool:
        return self.decision in ("allow", "warn")

    @property
    def blocked(self) -> bool:
        return self.decision in ("block", "escalate")

    @classmethod
    def from_json(cls, data: dict[str, Any]) -> "OnusResult":
        return cls(
            decision=data.get("decision", "allow"),
            correction=data.get("correction"),
            rule_id=data.get("rule_id"),
            rule_name=data.get("rule_name"),
            latency_us=data.get("latency_us"),
            action_id=data.get("action_id"),
            reversibility=data.get("reversibility"),
            raw=data,
        )


class OnusBlockError(RuntimeError):
    """Raised when Guardian blocks or escalates an action before execution."""

    def __init__(self, result: OnusResult) -> None:
        self.result = result
        message = result.correction or f"Onus blocked action: {result.decision}"
        super().__init__(message)


@dataclass
class RollbackRecord:
    action_id: str
    action_type: str
    target: str
    before_exists: bool = False
    before_content: Optional[str] = None
    backup_path: Optional[str] = None


class OnusClient:
    """Python client for the Onus Rust core."""

    def __init__(
        self,
        bin_path: Optional[str] = None,
        *,
        rules_path: Optional[Union[str, os.PathLike[str]]] = None,
        db_path: Optional[Union[str, os.PathLike[str]]] = None,
    ) -> None:
        self._bin = bin_path or self._find_binary()
        self._rules_path = str(rules_path) if rules_path else None
        self._db_path = str(db_path) if db_path else None
        self._session_id: Optional[str] = None
        self._sequence = 0

    @property
    def bin_path(self) -> str:
        return self._bin

    @property
    def db_path(self) -> Optional[str]:
        return self._db_path

    def evaluate(
        self,
        action_type: str,
        payload: Any,
        *,
        session_id: Optional[str] = None,
        tool: Optional[str] = None,
        sequence: Optional[int] = None,
    ) -> OnusResult:
        """Evaluate an action through Onus Core before executing it."""

        sid = session_id or self._session_id or f"py-{uuid.uuid4()}"
        if sequence is None:
            self._sequence += 1
            sequence = self._sequence

        if isinstance(payload, str):
            payload = {"command": payload}

        request = {
            "version": 1,
            "session_id": sid,
            "sequence": sequence,
            "action": {
                "type": action_type,
                "tool": tool or action_type,
                "payload": payload,
            },
        }

        args = [self._bin, "evaluate"]
        if self._rules_path:
            args += ["--rules", self._rules_path]
        if self._db_path:
            args += ["--db", self._db_path]

        proc = subprocess.run(
            args,
            input=json.dumps(request),
            capture_output=True,
            text=True,
            timeout=10,
        )

        try:
            data = json.loads(proc.stdout.strip())
        except json.JSONDecodeError:
            data = {
                "decision": "allow",
                "stderr": proc.stderr,
                "returncode": proc.returncode,
            }

        return OnusResult.from_json(data)

    def check_command(
        self,
        command: str,
        *,
        session_id: Optional[str] = None,
    ) -> OnusResult:
        return self.evaluate("shell", {"command": command}, session_id=session_id, tool="Bash")

    @contextmanager
    def session(self, task_description: str = "") -> Iterator["OnusClient"]:
        session_id = f"py-{uuid.uuid4()}"
        old_sid = self._session_id
        self._session_id = session_id
        self.evaluate(
            "shell",
            {"command": f"# onus session start: {task_description}"},
            session_id=session_id,
            tool="session_start",
        )
        try:
            yield self
        finally:
            self._session_id = old_sid

    def install_shell_wrapper(self, path: Optional[str] = None) -> str:
        args = [self._bin, "shell", "install"]
        if path:
            args += ["--path", path]
        result = subprocess.run(args, capture_output=True, text=True, timeout=10)
        if result.returncode != 0:
            raise RuntimeError(f"Failed to install shell wrapper: {result.stderr}")
        return result.stdout.strip()

    def remove_shell_wrapper(self) -> None:
        subprocess.run([self._bin, "shell", "remove"], capture_output=True, timeout=10)

    def load_rules(self) -> list[dict[str, Any]]:
        result = subprocess.run(
            [self._bin, "rules", "list"],
            capture_output=True,
            text=True,
            timeout=10,
        )
        if result.returncode != 0:
            raise RuntimeError(f"Failed to load rules: {result.stderr}")
        rules: list[dict[str, Any]] = []
        for line in result.stdout.strip().splitlines():
            try:
                rules.append(json.loads(line))
            except json.JSONDecodeError:
                continue
        return rules

    @staticmethod
    def _find_binary() -> str:
        exe = shutil.which("onus")
        if exe:
            return exe

        cargo_bin = Path.home() / ".cargo" / "bin"
        onus_in_cargo = cargo_bin / ("onus.exe" if platform.system() == "Windows" else "onus")
        if onus_in_cargo.is_file():
            return str(onus_in_cargo)

        if platform.system() == "Windows":
            candidates = [
                Path(os.environ.get("LOCALAPPDATA", "")) / "onus" / "onus.exe",
                Path(os.environ.get("PROGRAMFILES", "")) / "onus" / "onus.exe",
            ]
        else:
            candidates = [
                Path("/usr/local/bin/onus"),
                Path.home() / ".local" / "bin" / "onus",
                Path.home() / ".onus" / "onus",
            ]

        for path in candidates:
            if path.is_file():
                return str(path)

        raise FileNotFoundError(
            "Onus binary not found. Pass bin_path=... or install the onus CLI."
        )


class Guardian:
    """Pre-execution guard for Python agents and tools.

    Guardian turns proposed actions into Onus evaluations, blocks before side
    effects when the verdict is block/escalate, executes allowed actions, and
    keeps enough local state to roll back simple file writes and SQLite changes.
    """

    def __init__(
        self,
        *,
        task: str = "",
        workspace_root: Union[str, os.PathLike[str]] = ".",
        agent_name: str = "python-agent",
        bin_path: Optional[str] = None,
        rules_path: Optional[Union[str, os.PathLike[str]]] = None,
        db_path: Optional[Union[str, os.PathLike[str]]] = None,
    ) -> None:
        self.workspace_root = Path(workspace_root).resolve()
        self.session_id = f"guardian-{uuid.uuid4()}"
        self.task = task
        self.agent_name = agent_name
        self.client = OnusClient(bin_path, rules_path=rules_path, db_path=db_path)
        self._rollbacks: list[RollbackRecord] = []
        self._journal_dir = self.workspace_root / ".onus"
        self._journal_dir.mkdir(parents=True, exist_ok=True)
        self._backup_dir = self._journal_dir / "backups"
        self._backup_dir.mkdir(parents=True, exist_ok=True)
        self._corrections: list[str] = []

    def __enter__(self) -> "Guardian":
        self.client._session_id = self.session_id
        self.client.evaluate(
            "shell",
            {
                "command": f"# guardian session start: {self.task}",
                "agent": self.agent_name,
                "workspace_root": str(self.workspace_root),
            },
            session_id=self.session_id,
            tool="guardian_session_start",
        )
        return self

    def __exit__(self, *_exc: object) -> None:
        self.client._session_id = None

    @property
    def corrections(self) -> list[str]:
        return list(self._corrections)

    def evaluate(self, action_type: str, payload: dict[str, Any], *, tool: str) -> OnusResult:
        result = self.client.evaluate(
            action_type,
            payload,
            session_id=self.session_id,
            tool=tool,
        )
        if result.blocked:
            if result.correction:
                self._corrections.append(result.correction)
            raise OnusBlockError(result)
        if result.correction:
            self._corrections.append(result.correction)
        return result

    def file_write(self, path: Union[str, os.PathLike[str]], content: str, *, tool: str = "Write") -> OnusResult:
        target = self._resolve(path)
        before_exists = target.exists()
        before_content = target.read_text(encoding="utf-8") if before_exists else None
        payload = {
            "file_path": str(target),
            "path": str(target),
            "before_exists": before_exists,
            "before_content": before_content,
            "after_content": content,
            "content": content,
        }
        result = self.evaluate("file_write", payload, tool=tool)
        target.parent.mkdir(parents=True, exist_ok=True)
        target.write_text(content, encoding="utf-8")
        self._record_rollback(
            RollbackRecord(
                action_id=result.action_id or str(uuid.uuid4()),
                action_type="file_write",
                target=str(target),
                before_exists=before_exists,
                before_content=before_content,
            )
        )
        return result

    def shell(
        self,
        command: str,
        *,
        execute: bool = False,
        cwd: Optional[Union[str, os.PathLike[str]]] = None,
        tool: str = "Bash",
        timeout: float = 30.0,
    ) -> Union[OnusResult, tuple[OnusResult, subprocess.CompletedProcess[str]]]:
        run_cwd = Path(cwd).resolve() if cwd else self.workspace_root
        payload = {
            "command": command,
            "cwd": str(run_cwd),
        }
        result = self.evaluate("shell", payload, tool=tool)
        if not execute:
            return result
        completed = subprocess.run(
            command,
            cwd=run_cwd,
            shell=True,
            capture_output=True,
            text=True,
            timeout=timeout,
        )
        return result, completed

    def file_delete(self, path: Union[str, os.PathLike[str]], *, tool: str = "Delete") -> OnusResult:
        target = self._resolve(path)
        before_exists = target.exists()
        before_content = target.read_text(encoding="utf-8") if before_exists else None
        payload = {
            "file_path": str(target),
            "path": str(target),
            "before_exists": before_exists,
            "before_content": before_content,
            "after_content": None,
        }
        result = self.evaluate("file_delete", payload, tool=tool)
        if target.exists():
            target.unlink()
        self._record_rollback(
            RollbackRecord(
                action_id=result.action_id or str(uuid.uuid4()),
                action_type="file_delete",
                target=str(target),
                before_exists=before_exists,
                before_content=before_content,
            )
        )
        return result

    def api_call(
        self,
        url: str,
        *,
        method: str = "GET",
        headers: Optional[dict[str, str]] = None,
        body: Optional[Union[bytes, str]] = None,
        timeout: float = 10.0,
        tool: str = "ApiCall",
    ) -> tuple[OnusResult, bytes]:
        payload = {
            "method": method.upper(),
            "url": url,
            "headers": headers or {},
            "body_preview": body.decode("utf-8", "replace") if isinstance(body, bytes) else body,
        }
        result = self.evaluate("api_call", payload, tool=tool)
        data = body.encode("utf-8") if isinstance(body, str) else body
        req = urllib_request.Request(url, data=data, method=method.upper(), headers=headers or {})
        with urllib_request.urlopen(req, timeout=timeout) as response:
            return result, response.read()

    def db_execute(
        self,
        db_path: Union[str, os.PathLike[str]],
        sql: str,
        params: Union[tuple[Any, ...], list[Any]] = (),
        *,
        tool: str = "SQLite",
    ) -> OnusResult:
        target = self._resolve(db_path)
        payload = {
            "db_path": str(target),
            "sql": sql,
            "params": list(params),
        }
        result = self.evaluate("db_mutation", payload, tool=tool)

        backup_path = None
        if target.exists():
            backup_path = str(self._backup_dir / f"{uuid.uuid4()}.sqlite")
            shutil.copy2(target, backup_path)

        target.parent.mkdir(parents=True, exist_ok=True)
        con = sqlite3.connect(target)
        try:
            con.execute(sql, tuple(params))
            con.commit()
        finally:
            con.close()

        self._record_rollback(
            RollbackRecord(
                action_id=result.action_id or str(uuid.uuid4()),
                action_type="db_mutation",
                target=str(target),
                before_exists=backup_path is not None,
                backup_path=backup_path,
            )
        )
        return result

    def rollback_last(self) -> Optional[RollbackRecord]:
        if not self._rollbacks:
            return None
        record = self._rollbacks.pop()
        target = Path(record.target)
        if record.action_type in ("file_write", "file_delete"):
            if record.before_exists:
                target.parent.mkdir(parents=True, exist_ok=True)
                target.write_text(record.before_content or "", encoding="utf-8")
            elif target.exists():
                target.unlink()
        elif record.action_type == "db_mutation":
            if record.backup_path:
                shutil.copy2(record.backup_path, target)
            elif target.exists():
                target.unlink()
        return record

    def run_agent(self, agent: Any, *args: Any, **kwargs: Any) -> Any:
        """Run an agent object with this Guardian injected.

        The agent may expose ``run(guardian, *args, **kwargs)``. If an action is
        blocked and the agent has ``receive_correction(text)``, Guardian sends
        the correction back before re-raising.
        """

        try:
            return agent.run(self, *args, **kwargs)
        except OnusBlockError as exc:
            if hasattr(agent, "receive_correction"):
                agent.receive_correction(str(exc))
            raise

    def _record_rollback(self, record: RollbackRecord) -> None:
        self._rollbacks.append(record)
        journal = self._journal_dir / "rollback_journal.jsonl"
        with journal.open("a", encoding="utf-8") as fh:
            fh.write(json.dumps(record.__dict__, sort_keys=True) + "\n")

    def _resolve(self, path: Union[str, os.PathLike[str]]) -> Path:
        candidate = Path(path)
        if not candidate.is_absolute():
            candidate = self.workspace_root / candidate
        return candidate.resolve()


_default_client: Optional[OnusClient] = None


def get_client() -> OnusClient:
    global _default_client
    if _default_client is None:
        _default_client = OnusClient()
    return _default_client


def evaluate(action_type: str, payload: Any, **kwargs: Any) -> OnusResult:
    return get_client().evaluate(action_type, payload, **kwargs)


def check_command(command: str, **kwargs: Any) -> OnusResult:
    return get_client().check_command(command, **kwargs)


__all__ = [
    "Guardian",
    "OnusBlockError",
    "OnusClient",
    "OnusResult",
    "RollbackRecord",
    "evaluate",
    "check_command",
    "get_client",
]
