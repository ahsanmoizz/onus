"""Onus Python SDK.

The SDK is intentionally thin over the Rust ``onus`` binary for policy,
hash-chained audit storage, and CLI compatibility. ``Guardian`` adds a real
pre-execution wrapper for Python agents and tools.
"""

from __future__ import annotations

import json
import hashlib
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
    canonical_payload_hash: Optional[str] = None
    reversibility: Optional[str] = None
    approval_decision: Optional[str] = None
    guardian_mode: Optional[str] = None
    obligations: list[str] = field(default_factory=list)
    approval_reason: Optional[str] = None
    raw: dict[str, Any] = field(default_factory=dict)

    @property
    def allowed(self) -> bool:
        return self.decision in ("allow", "warn")

    @property
    def blocked(self) -> bool:
        return self.decision in ("block", "escalate")

    @classmethod
    def from_json(cls, data: dict[str, Any]) -> "OnusResult":
        decision = data.get("decision")
        if decision not in {"allow", "warn", "block", "escalate"}:
            raise ValueError(f"Invalid or missing Onus decision: {decision!r}")
        return cls(
            decision=decision,
            correction=data.get("correction"),
            rule_id=data.get("rule_id"),
            rule_name=data.get("rule_name"),
            latency_us=data.get("latency_us"),
            action_id=data.get("action_id"),
            canonical_payload_hash=data.get("canonical_payload_hash"),
            reversibility=data.get("reversibility"),
            approval_decision=data.get("approval_decision"),
            guardian_mode=data.get("guardian_mode"),
            obligations=list(data.get("obligations", [])),
            approval_reason=data.get("approval_reason"),
            raw=data,
        )


class OnusBlockError(RuntimeError):
    """Raised when Guardian blocks or escalates an action before execution."""

    def __init__(self, result: OnusResult) -> None:
        self.result = result
        message = result.correction or f"Onus blocked action: {result.decision}"
        super().__init__(message)


class OnusEvaluationError(RuntimeError):
    """Raised when Onus Core cannot return a trustworthy verdict."""


class OnusRollbackUnsupportedError(RuntimeError):
    """Raised when a resource has no implemented rollback adapter."""


class ReversibilityClass:
    R0 = "R0_READ_ONLY"
    R1 = "R1_AUTOMATICALLY_REVERSIBLE"
    R2 = "R2_SNAPSHOT_REVERSIBLE"
    R3 = "R3_COMPENSATABLE"
    R4 = "R4_IRREVERSIBLE_OR_MITIGATION_ONLY"

    LABELS = {
        R0: "read-only",
        R1: "automatically reversible",
        R2: "snapshot reversible",
        R3: "compensatable external action",
        R4: "irreversible or mitigation-only",
    }


@dataclass
class RollbackRecord:
    action_id: str
    action_type: str
    target: str
    reversibility_class: str = ReversibilityClass.R4
    group_id: str = "default"
    before_exists: bool = False
    before_content: Optional[str] = None
    backup_path: Optional[str] = None
    pre_state: dict[str, Any] = field(default_factory=dict)
    proposed_mutation: dict[str, Any] = field(default_factory=dict)
    decision: str = ""
    executed_payload: dict[str, Any] = field(default_factory=dict)
    post_state: dict[str, Any] = field(default_factory=dict)
    inverse_operation: dict[str, Any] = field(default_factory=dict)
    compensation_metadata: dict[str, Any] = field(default_factory=dict)
    verification_result: dict[str, Any] = field(default_factory=dict)
    rollback_status: str = "pending"

    def to_dict(self) -> dict[str, Any]:
        return {
            "action_id": self.action_id,
            "action_type": self.action_type,
            "target": self.target,
            "reversibility_class": self.reversibility_class,
            "reversibility_label": ReversibilityClass.LABELS.get(
                self.reversibility_class, "unknown"
            ),
            "group_id": self.group_id,
            "before_exists": self.before_exists,
            "backup_path": self.backup_path,
            "pre_state": self.pre_state,
            "proposed_mutation": self.proposed_mutation,
            "decision": self.decision,
            "executed_payload": self.executed_payload,
            "post_state": self.post_state,
            "inverse_operation": self.inverse_operation,
            "compensation_metadata": self.compensation_metadata,
            "verification_result": self.verification_result,
            "rollback_status": self.rollback_status,
        }


@dataclass
class RollbackResult:
    status: str
    action_ids: list[str] = field(default_factory=list)
    restored: bool = False
    unsupported: list[dict[str, Any]] = field(default_factory=list)
    verification_result: dict[str, Any] = field(default_factory=dict)
    message: str = ""

    def to_dict(self) -> dict[str, Any]:
        return {
            "status": self.status,
            "action_ids": self.action_ids,
            "restored": self.restored,
            "unsupported": self.unsupported,
            "verification_result": self.verification_result,
            "message": self.message,
        }


class RecoveryAdapter:
    """Extension point for resource-specific recovery adapters."""

    resource_type = "unsupported"

    def classify(self, _payload: dict[str, Any]) -> str:
        return ReversibilityClass.R4

    def compensation_metadata(self, payload: dict[str, Any]) -> dict[str, Any]:
        return {
            "resource_type": self.resource_type,
            "supported": False,
            "reason": f"{self.resource_type} rollback adapter is not implemented.",
            "payload_hash": _stable_hash(payload),
        }


class RepositoryFileRecoveryAdapter(RecoveryAdapter):
    resource_type = "repository_file"

    def classify(self, payload: dict[str, Any]) -> str:
        if payload.get("operation") == "file_read":
            return ReversibilityClass.R0
        if payload.get("before_exists"):
            return ReversibilityClass.R2
        return ReversibilityClass.R1


class SQLiteRecoveryAdapter(RecoveryAdapter):
    resource_type = "sqlite"

    def classify(self, _payload: dict[str, Any]) -> str:
        return ReversibilityClass.R2


class UnsupportedExternalRecoveryAdapter(RecoveryAdapter):
    def __init__(self, resource_type: str) -> None:
        self.resource_type = resource_type


@dataclass
class ChangeBudget:
    max_files_changed: int = 25
    max_actions: int = 500


@dataclass
class RequiredEvidence:
    id: str
    description: str
    kind: str = "manual"


@dataclass
class CompletionEvidence:
    id: str
    passed: bool
    value: str = ""
    kind: str = "manual"


@dataclass
class TaskContract:
    session_id: str
    original_prompt: str
    normalized_objective: str
    allowed_paths: list[str] = field(default_factory=list)
    allowed_resources: list[str] = field(default_factory=list)
    protected_paths: list[str] = field(default_factory=list)
    protected_resources: list[str] = field(default_factory=list)
    required_evidence: list[RequiredEvidence] = field(default_factory=list)
    forbidden_actions: list[str] = field(default_factory=list)
    approval_required_actions: list[str] = field(default_factory=list)
    change_budget: ChangeBudget = field(default_factory=ChangeBudget)
    environment_identity: str = ""
    policy_version: str = ""
    canonical_hash: str = ""
    schema_version: int = 1

    def to_dict(self) -> dict[str, Any]:
        return {
            "schema_version": self.schema_version,
            "session_id": self.session_id,
            "original_prompt": self.original_prompt,
            "normalized_objective": self.normalized_objective,
            "allowed_paths": self.allowed_paths,
            "allowed_resources": self.allowed_resources,
            "protected_paths": self.protected_paths,
            "protected_resources": self.protected_resources,
            "required_evidence": [e.__dict__ for e in self.required_evidence],
            "forbidden_actions": self.forbidden_actions,
            "approval_required_actions": self.approval_required_actions,
            "change_budget": self.change_budget.__dict__,
            "environment_identity": self.environment_identity,
            "policy_version": self.policy_version,
            "canonical_hash": self.canonical_hash,
        }

    @classmethod
    def from_dict(cls, data: dict[str, Any]) -> "TaskContract":
        return cls(
            schema_version=int(data.get("schema_version", 1)),
            session_id=data["session_id"],
            original_prompt=data["original_prompt"],
            normalized_objective=data["normalized_objective"],
            allowed_paths=list(data.get("allowed_paths", [])),
            allowed_resources=list(data.get("allowed_resources", [])),
            protected_paths=list(data.get("protected_paths", [])),
            protected_resources=list(data.get("protected_resources", [])),
            required_evidence=[
                RequiredEvidence(**item) for item in data.get("required_evidence", [])
            ],
            forbidden_actions=list(data.get("forbidden_actions", [])),
            approval_required_actions=list(data.get("approval_required_actions", [])),
            change_budget=ChangeBudget(**data.get("change_budget", {})),
            environment_identity=data.get("environment_identity", ""),
            policy_version=data.get("policy_version", ""),
            canonical_hash=data.get("canonical_hash", ""),
        )


@dataclass
class PromptIntakeResult:
    status: str
    provider_mode: str
    semantic_review: str
    semantic_roles: list[dict[str, Any]]
    reasons: list[str]
    questions: list[str]
    session_started: bool = False
    session_id: Optional[str] = None
    contract_hash: Optional[str] = None
    proposed_contract: Optional[TaskContract] = None
    raw: dict[str, Any] = field(default_factory=dict)

    @classmethod
    def from_json(cls, data: dict[str, Any]) -> "PromptIntakeResult":
        contract_data = data.get("proposed_contract")
        return cls(
            status=data["status"],
            provider_mode=data.get("provider_mode", "disabled"),
            semantic_review=data.get("semantic_review", ""),
            semantic_roles=list(data.get("semantic_roles", [])),
            reasons=list(data.get("reasons", [])),
            questions=list(data.get("questions", [])),
            session_started=bool(data.get("session_started", False)),
            session_id=data.get("session_id"),
            contract_hash=data.get("contract_hash"),
            proposed_contract=TaskContract.from_dict(contract_data) if contract_data else None,
            raw=data,
        )


def _stable_hash(value: Any) -> str:
    return hashlib.sha256(
        json.dumps(value, sort_keys=True, default=str).encode("utf-8")
    ).hexdigest()


def _content_descriptor(content: Optional[Union[str, bytes]]) -> dict[str, Any]:
    if content is None:
        return {"exists": False, "sha256": None, "size": 0}
    data = content.encode("utf-8") if isinstance(content, str) else content
    return {
        "exists": True,
        "sha256": hashlib.sha256(data).hexdigest(),
        "size": len(data),
        "content_stored": "snapshot_or_operation_target",
    }


def _redact_sensitive_value(value: Any) -> Any:
    if isinstance(value, dict):
        return {key: _redact_sensitive_value(item) for key, item in value.items()}
    if isinstance(value, list):
        return [_redact_sensitive_value(item) for item in value]
    if isinstance(value, str):
        lower = value.lower()
        if any(token in lower for token in ["secret", "token", "password", "api_key", "apikey"]):
            return "[REDACTED]"
    return value


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
        except json.JSONDecodeError as exc:
            raise OnusEvaluationError(
                "Onus Core did not return a valid JSON verdict; action was not executed."
            ) from exc

        try:
            return OnusResult.from_json(data)
        except ValueError as exc:
            raise OnusEvaluationError(
                "Onus Core returned an invalid verdict; action was not executed."
            ) from exc

    def check_command(
        self,
        command: str,
        *,
        session_id: Optional[str] = None,
    ) -> OnusResult:
        return self.evaluate("shell", {"command": command}, session_id=session_id, tool="Bash")

    def start_contract(
        self,
        contract: TaskContract,
        *,
        workspace_root: Union[str, os.PathLike[str]],
        agent_name: str = "python-agent",
    ) -> dict[str, Any]:
        args = [self._bin, "contract", "start"]
        if self._db_path:
            args += ["--db", self._db_path]
        args += ["--workspace-root", str(Path(workspace_root).resolve()), "--agent-name", agent_name]
        proc = subprocess.run(
            args,
            input=json.dumps(contract.to_dict()),
            capture_output=True,
            text=True,
            timeout=10,
        )
        if proc.returncode != 0:
            raise OnusEvaluationError(
                f"Onus failed to persist task contract; action was not executed: {proc.stderr}"
            )
        return json.loads(proc.stdout.strip())

    def complete_contract(
        self,
        session_id: str,
        evidence: list[CompletionEvidence],
    ) -> dict[str, Any]:
        args = [self._bin, "contract", "complete", "--session-id", session_id]
        if self._db_path:
            args += ["--db", self._db_path]
        proc = subprocess.run(
            args,
            input=json.dumps([item.__dict__ for item in evidence]),
            capture_output=True,
            text=True,
            timeout=10,
        )
        data = json.loads(proc.stdout.strip())
        if proc.returncode not in (0, 4, 5, 6, 7):
            raise OnusEvaluationError(
                f"Onus failed to verify task completion: {proc.stderr}"
            )
        return data

    def intake_prompt(
        self,
        prompt: str,
        *,
        workspace_root: Union[str, os.PathLike[str]] = ".",
        session_id: Optional[str] = None,
        agent_name: str = "python-agent",
        start_session: bool = False,
        provider: str = "disabled",
        semantic_model: Optional[str] = None,
        semantic_endpoint: Optional[str] = None,
        semantic_api_key_env: Optional[str] = None,
        semantic_local_command: Optional[str] = None,
        semantic_timeout_ms: Optional[int] = None,
        semantic_privacy: str = "strict",
        semantic_redaction: bool = True,
        semantic_token_budget: Optional[int] = None,
        semantic_cost_budget_micro_usd: Optional[int] = None,
        semantic_cost_per_1k_tokens_micro_usd: Optional[int] = None,
        semantic_fail_closed: bool = False,
    ) -> PromptIntakeResult:
        args = [
            self._bin,
            "intake",
            "--workspace-root",
            str(Path(workspace_root).resolve()),
            "--provider",
            provider,
            "--agent-name",
            agent_name,
            "--semantic-privacy",
            semantic_privacy,
        ]
        if semantic_model:
            args += ["--semantic-model", semantic_model]
        if semantic_endpoint:
            args += ["--semantic-endpoint", semantic_endpoint]
        if semantic_api_key_env:
            args += ["--semantic-api-key-env", semantic_api_key_env]
        if semantic_local_command:
            args += ["--semantic-local-command", semantic_local_command]
        if semantic_timeout_ms is not None:
            args += ["--semantic-timeout-ms", str(semantic_timeout_ms)]
        if not semantic_redaction:
            args.append("--no-semantic-redaction")
        if semantic_token_budget is not None:
            args += ["--semantic-token-budget", str(semantic_token_budget)]
        if semantic_cost_budget_micro_usd is not None:
            args += [
                "--semantic-cost-budget-micro-usd",
                str(semantic_cost_budget_micro_usd),
            ]
        if semantic_cost_per_1k_tokens_micro_usd is not None:
            args += [
                "--semantic-cost-per-1k-tokens-micro-usd",
                str(semantic_cost_per_1k_tokens_micro_usd),
            ]
        if semantic_fail_closed:
            args.append("--semantic-fail-closed")
        if session_id:
            args += ["--session-id", session_id]
        if start_session:
            args.append("--start-session")
        if self._db_path:
            args += ["--db", self._db_path]
        proc = subprocess.run(
            args,
            input=prompt,
            capture_output=True,
            text=True,
            timeout=10,
        )
        if proc.returncode != 0:
            raise OnusEvaluationError(f"Prompt intake failed: {proc.stderr}")
        return PromptIntakeResult.from_json(json.loads(proc.stdout.strip()))

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
        contract: Optional[Union[TaskContract, dict[str, Any]]] = None,
        missing_contract_behavior: Optional[str] = None,
        bin_path: Optional[str] = None,
        rules_path: Optional[Union[str, os.PathLike[str]]] = None,
        db_path: Optional[Union[str, os.PathLike[str]]] = None,
    ) -> None:
        self.workspace_root = Path(workspace_root).resolve()
        self.session_id = f"guardian-{uuid.uuid4()}"
        self.task = task
        self.agent_name = agent_name
        if contract is None and missing_contract_behavior is None:
            raise ValueError(
                "Guardian requires a task contract. Pass contract=TaskContract(...) "
                "or explicitly set missing_contract_behavior='allow_legacy'."
            )
        if isinstance(contract, dict):
            contract = TaskContract.from_dict(contract)
        if contract is not None:
            contract.session_id = self.session_id
        self.contract = contract
        self.missing_contract_behavior = missing_contract_behavior
        self.client = OnusClient(bin_path, rules_path=rules_path, db_path=db_path)
        self._rollbacks: list[RollbackRecord] = []
        self._journal_dir = self.workspace_root / ".onus"
        self._journal_dir.mkdir(parents=True, exist_ok=True)
        self._backup_dir = self._journal_dir / "backups"
        self._backup_dir.mkdir(parents=True, exist_ok=True)
        self._checkpoint_dir = self._journal_dir / "checkpoints"
        self._checkpoint_dir.mkdir(parents=True, exist_ok=True)
        self._checkpoint_root = self._checkpoint_dir / self.session_id
        self._checkpoint_root.mkdir(parents=True, exist_ok=True)
        self._checkpoint_path = self._checkpoint_root / "checkpoint.json"
        self._file_recovery = RepositoryFileRecoveryAdapter()
        self._sqlite_recovery = SQLiteRecoveryAdapter()
        self._external_recovery_adapters = {
            "postgresql": UnsupportedExternalRecoveryAdapter("postgresql"),
            "api": UnsupportedExternalRecoveryAdapter("api"),
            "deployment": UnsupportedExternalRecoveryAdapter("deployment"),
            "infrastructure": UnsupportedExternalRecoveryAdapter("infrastructure"),
        }
        self._corrections: list[str] = []
        self._old_missing_contract_behavior: Optional[str] = None

    @classmethod
    def from_prompt(
        cls,
        prompt: str,
        *,
        workspace_root: Union[str, os.PathLike[str]] = ".",
        agent_name: str = "python-agent",
        bin_path: Optional[str] = None,
        rules_path: Optional[Union[str, os.PathLike[str]]] = None,
        db_path: Optional[Union[str, os.PathLike[str]]] = None,
        provider: str = "disabled",
    ) -> "Guardian":
        client = OnusClient(bin_path, rules_path=rules_path, db_path=db_path)
        intake = client.intake_prompt(
            prompt,
            workspace_root=workspace_root,
            agent_name=agent_name,
            start_session=False,
            provider=provider,
        )
        if intake.status not in {"READY", "READY_WITH_SAFE_CONTRACT"}:
            raise OnusEvaluationError(
                f"Prompt intake returned {intake.status}; questions={intake.questions}"
            )
        if intake.proposed_contract is None:
            raise OnusEvaluationError("Prompt intake did not produce a task contract.")
        return cls(
            task=intake.proposed_contract.normalized_objective,
            workspace_root=workspace_root,
            agent_name=agent_name,
            contract=intake.proposed_contract,
            bin_path=bin_path,
            rules_path=rules_path,
            db_path=db_path,
        )

    def __enter__(self) -> "Guardian":
        self.client._session_id = self.session_id
        if self.contract is not None:
            self.client.start_contract(
                self.contract,
                workspace_root=self.workspace_root,
                agent_name=self.agent_name,
            )
            self._create_checkpoint()
        elif self.missing_contract_behavior:
            self._old_missing_contract_behavior = os.environ.get("ONUS_MISSING_CONTRACT")
            os.environ["ONUS_MISSING_CONTRACT"] = self.missing_contract_behavior
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
        if self._old_missing_contract_behavior is not None:
            os.environ["ONUS_MISSING_CONTRACT"] = self._old_missing_contract_behavior
        elif self.missing_contract_behavior:
            os.environ.pop("ONUS_MISSING_CONTRACT", None)

    def complete(self, evidence: list[CompletionEvidence]) -> dict[str, Any]:
        if self.contract is None:
            raise OnusEvaluationError("Cannot complete a Guardian session without a task contract.")
        return self.client.complete_contract(self.session_id, evidence)

    @property
    def corrections(self) -> list[str]:
        return list(self._corrections)

    @property
    def checkpoint_path(self) -> Path:
        return self._checkpoint_path

    @property
    def recovery_records(self) -> list[RollbackRecord]:
        return list(self._rollbacks)

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

    def file_read(
        self,
        path: Union[str, os.PathLike[str]],
        *,
        tool: str = "Read",
    ) -> tuple[OnusResult, str]:
        target = self._resolve(path)
        state = self._file_state(target)
        payload = {
            "file_path": str(target),
            "path": str(target),
            "sha256": state["sha256"],
            "size": state["size"],
        }
        result = self.evaluate("file_read", payload, tool=tool)
        return result, target.read_text(encoding="utf-8")

    def file_write(
        self,
        path: Union[str, os.PathLike[str]],
        content: str,
        *,
        tool: str = "Write",
        action_group: str = "default",
    ) -> OnusResult:
        target = self._resolve(path)
        before_state = self._file_state(target)
        before_exists = before_state["exists"]
        before_content = target.read_text(encoding="utf-8") if before_exists else None
        backup_path = self._snapshot_file(target, "before") if before_exists else None
        payload = {
            "file_path": str(target),
            "path": str(target),
            "before_exists": before_exists,
            "before_sha256": before_state["sha256"],
            "before_content": before_content,
            "after_content": content,
            "content": content,
        }
        result = self.evaluate("file_write", payload, tool=tool)
        self._assert_file_unchanged(target, before_state)
        target.parent.mkdir(parents=True, exist_ok=True)
        target.write_text(content, encoding="utf-8", newline="")
        post_state = self._file_state(target)
        self._record_rollback(
            RollbackRecord(
                action_id=result.action_id or str(uuid.uuid4()),
                action_type="file_write",
                target=str(target),
                reversibility_class=self._file_recovery.classify(
                    {"operation": "file_write", "before_exists": before_exists}
                ),
                group_id=action_group,
                before_exists=before_exists,
                before_content=None,
                backup_path=backup_path,
                pre_state={**before_state, "backup_path": backup_path},
                proposed_mutation={
                    "operation": "file_write",
                    "path": str(target),
                    "content_sha256": _content_descriptor(content)["sha256"],
                    "size": _content_descriptor(content)["size"],
                },
                decision=result.decision,
                executed_payload={
                    "operation": "file_write",
                    "path": str(target),
                    "content_sha256": _content_descriptor(content)["sha256"],
                    "size": _content_descriptor(content)["size"],
                    "raw_content_in_journal": False,
                },
                post_state=post_state,
                inverse_operation={
                    "type": "restore_file_snapshot" if before_exists else "delete_created_file",
                    "backup_path": backup_path,
                    "target": str(target),
                },
                compensation_metadata={},
                verification_result={"status": "recorded", "post_state": post_state},
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

    def file_delete(
        self,
        path: Union[str, os.PathLike[str]],
        *,
        tool: str = "Delete",
        action_group: str = "default",
    ) -> OnusResult:
        target = self._resolve(path)
        before_state = self._file_state(target)
        before_exists = before_state["exists"]
        before_content = target.read_text(encoding="utf-8") if before_exists else None
        backup_path = self._snapshot_file(target, "before") if before_exists else None
        payload = {
            "file_path": str(target),
            "path": str(target),
            "before_exists": before_exists,
            "before_sha256": before_state["sha256"],
            "before_content": before_content,
            "after_content": None,
        }
        result = self.evaluate("file_delete", payload, tool=tool)
        self._assert_file_unchanged(target, before_state)
        if target.exists():
            target.unlink()
        post_state = self._file_state(target)
        self._record_rollback(
            RollbackRecord(
                action_id=result.action_id or str(uuid.uuid4()),
                action_type="file_delete",
                target=str(target),
                reversibility_class=ReversibilityClass.R2 if before_exists else ReversibilityClass.R1,
                group_id=action_group,
                before_exists=before_exists,
                before_content=None,
                backup_path=backup_path,
                pre_state={**before_state, "backup_path": backup_path},
                proposed_mutation={"operation": "file_delete", "path": str(target)},
                decision=result.decision,
                executed_payload={
                    "operation": "file_delete",
                    "path": str(target),
                    "raw_content_in_journal": False,
                },
                post_state=post_state,
                inverse_operation={
                    "type": "restore_file_snapshot" if before_exists else "noop_missing_file",
                    "backup_path": backup_path,
                    "target": str(target),
                },
                compensation_metadata={},
                verification_result={"status": "recorded", "post_state": post_state},
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
        action_group: str = "default",
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
            response_data = response.read()
        adapter = self._external_recovery_adapters["api"]
        self._record_rollback(
            RollbackRecord(
                action_id=result.action_id or str(uuid.uuid4()),
                action_type="api_call",
                target=url,
                reversibility_class=ReversibilityClass.R3,
                group_id=action_group,
                before_exists=False,
                pre_state={"external_resource": url, "captured": False},
                proposed_mutation={
                    "operation": "api_call",
                    "method": method.upper(),
                    "url": url,
                    "body_hash": _stable_hash(payload.get("body_preview")),
                },
                decision=result.decision,
                executed_payload={
                    "operation": "api_call",
                    "method": method.upper(),
                    "url": url,
                    "body_hash": _stable_hash(payload.get("body_preview")),
                    "raw_body_in_journal": False,
                },
                post_state={
                    "response_size": len(response_data),
                    "response_sha256": hashlib.sha256(response_data).hexdigest(),
                },
                inverse_operation={"type": "unsupported_external_rollback"},
                compensation_metadata=adapter.compensation_metadata(payload),
                verification_result={
                    "status": "unsupported",
                    "reason": "API compensation requires a provider-specific adapter.",
                },
                rollback_status="unsupported",
            )
        )
        return result, response_data

    def record_external_compensation_required(
        self,
        resource_type: str,
        target: str,
        payload: dict[str, Any],
        *,
        action_group: str = "default",
    ) -> RollbackRecord:
        adapter = self._external_recovery_adapters.get(
            resource_type, UnsupportedExternalRecoveryAdapter(resource_type)
        )
        record = RollbackRecord(
            action_id=str(uuid.uuid4()),
            action_type=resource_type,
            target=target,
            reversibility_class=ReversibilityClass.R3,
            group_id=action_group,
            pre_state={"external_resource": target, "captured": False},
            proposed_mutation={
                "operation": resource_type,
                "target": target,
                "payload_hash": _stable_hash(payload),
            },
            decision="external_action_recorded",
            executed_payload={
                "operation": resource_type,
                "target": target,
                "payload_hash": _stable_hash(payload),
                "raw_payload_in_journal": False,
            },
            inverse_operation={"type": "unsupported_external_rollback"},
            compensation_metadata=adapter.compensation_metadata(payload),
            verification_result={
                "status": "unsupported",
                "reason": f"{resource_type} compensation requires a provider-specific adapter.",
            },
            rollback_status="unsupported",
        )
        self._record_rollback(record)
        return record

    def db_execute(
        self,
        db_path: Union[str, os.PathLike[str]],
        sql: str,
        params: Union[tuple[Any, ...], list[Any]] = (),
        *,
        tool: str = "SQLite",
        action_group: str = "default",
    ) -> OnusResult:
        target = self._resolve(db_path)
        before_state = self._file_state(target)
        payload = {
            "db_path": str(target),
            "before_sha256": before_state["sha256"],
            "sql": sql,
            "params": list(params),
        }
        result = self.evaluate("db_mutation", payload, tool=tool)
        self._assert_file_unchanged(target, before_state)

        backup_path = self._snapshot_file(target, "before") if target.exists() else None

        target.parent.mkdir(parents=True, exist_ok=True)
        con = sqlite3.connect(target)
        try:
            con.execute(sql, tuple(params))
            con.commit()
        finally:
            con.close()
        post_state = self._file_state(target)

        self._record_rollback(
            RollbackRecord(
                action_id=result.action_id or str(uuid.uuid4()),
                action_type="db_mutation",
                target=str(target),
                reversibility_class=self._sqlite_recovery.classify(payload),
                group_id=action_group,
                before_exists=backup_path is not None,
                backup_path=backup_path,
                pre_state={**before_state, "backup_path": backup_path},
                proposed_mutation={
                    "operation": "sqlite_execute",
                    "db_path": str(target),
                    "sql_sha256": hashlib.sha256(sql.encode("utf-8")).hexdigest(),
                    "params_hash": _stable_hash(list(params)),
                },
                decision=result.decision,
                executed_payload={
                    "operation": "sqlite_execute",
                    "db_path": str(target),
                    "sql_sha256": hashlib.sha256(sql.encode("utf-8")).hexdigest(),
                    "params": _redact_sensitive_value(list(params)),
                    "raw_sql_in_journal": False,
                },
                post_state=post_state,
                inverse_operation={
                    "type": "restore_sqlite_snapshot" if backup_path else "delete_created_sqlite",
                    "backup_path": backup_path,
                    "target": str(target),
                },
                compensation_metadata={},
                verification_result={"status": "recorded", "post_state": post_state},
            )
        )
        return result

    def rollback_last(self) -> Optional[RollbackRecord]:
        if not self._rollbacks:
            return None
        record = self._rollbacks[-1]
        self._revert_record(record)
        if record.rollback_status == "reverted":
            self._rollbacks.pop()
        return record

    def revert_action(self, action_id: str) -> RollbackResult:
        record = self._find_rollback(action_id)
        if record is None:
            return RollbackResult(
                status="NOT_FOUND",
                action_ids=[action_id],
                restored=False,
                message="No recovery record exists for this action.",
            )
        if record.rollback_status == "unsupported":
            return RollbackResult(
                status="ROLLBACK_UNSUPPORTED",
                action_ids=[action_id],
                restored=False,
                unsupported=[record.compensation_metadata],
                verification_result=record.verification_result,
                message="This action requires compensation or mitigation; rollback is unsupported.",
            )
        self._revert_record(record)
        return RollbackResult(
            status="REVERTED" if record.rollback_status == "reverted" else "VERIFY_FAILED",
            action_ids=[record.action_id],
            restored=record.rollback_status == "reverted",
            verification_result=record.verification_result,
            message=record.verification_result.get("message", ""),
        )

    def revert_action_group(self, group_id: str) -> RollbackResult:
        records = [
            record
            for record in self._rollbacks
            if record.group_id == group_id and record.rollback_status in {"pending", "unsupported"}
        ]
        return self._revert_records(records, f"No pending actions found for group {group_id}.")

    def revert_session(self) -> RollbackResult:
        records = [
            record
            for record in self._rollbacks
            if record.rollback_status in {"pending", "unsupported"}
        ]
        return self._revert_records(records, "No pending actions found for this session.")

    def restore_checkpoint(self) -> RollbackResult:
        if not self._checkpoint_path.exists():
            return RollbackResult(
                status="CHECKPOINT_MISSING",
                restored=False,
                message="No session checkpoint exists to restore.",
            )
        session_result = self.revert_session()
        checkpoint = json.loads(self._checkpoint_path.read_text(encoding="utf-8"))
        verification: dict[str, Any] = {"files": []}
        restored = True
        for item in checkpoint.get("files", []):
            target = self._resolve(item["path"])
            backup_path = item.get("backup_path")
            if item.get("exists") and backup_path:
                target.parent.mkdir(parents=True, exist_ok=True)
                shutil.copy2(backup_path, target)
            elif target.exists():
                target.unlink()
            current = self._file_state(target)
            ok = (
                current["exists"] == item.get("exists")
                and current["sha256"] == item.get("sha256")
                and current["size"] == item.get("size")
            )
            restored = restored and ok
            verification["files"].append(
                {
                    "path": item["path"],
                    "verified": ok,
                    "expected_sha256": item.get("sha256"),
                    "actual_sha256": current["sha256"],
                }
            )
        if session_result.unsupported:
            restored = False
        return RollbackResult(
            status="CHECKPOINT_RESTORED" if restored else "CHECKPOINT_PARTIAL",
            action_ids=session_result.action_ids,
            restored=restored,
            unsupported=session_result.unsupported,
            verification_result=verification,
            message=(
                "Restored repository files and SQLite state to the safe checkpoint."
                if restored
                else "Checkpoint restore completed with unsupported or failed reversions."
            ),
        )

    def _revert_records(self, records: list[RollbackRecord], empty_message: str) -> RollbackResult:
        if not records:
            return RollbackResult(status="NOOP", restored=True, message=empty_message)
        unsupported: list[dict[str, Any]] = []
        reverted: list[str] = []
        verification: dict[str, Any] = {"actions": []}
        for record in reversed(records):
            if record.rollback_status == "unsupported":
                unsupported.append(record.compensation_metadata)
                verification["actions"].append(
                    {
                        "action_id": record.action_id,
                        "status": "unsupported",
                        "reason": record.compensation_metadata.get("reason", "unsupported"),
                    }
                )
                continue
            self._revert_record(record)
            reverted.append(record.action_id)
            verification["actions"].append(
                {
                    "action_id": record.action_id,
                    "status": record.rollback_status,
                    "verification": record.verification_result,
                }
            )
        restored = not unsupported and all(
            item["status"] == "reverted" for item in verification["actions"]
        )
        return RollbackResult(
            status="REVERTED" if restored else "PARTIAL_OR_UNSUPPORTED",
            action_ids=reverted,
            restored=restored,
            unsupported=unsupported,
            verification_result=verification,
            message=(
                "All reversible actions restored."
                if restored
                else "Some actions could not be rolled back and require mitigation."
            ),
        )

    def _revert_record(self, record: RollbackRecord) -> None:
        target = Path(record.target)
        if record.rollback_status == "reverted":
            return
        if record.rollback_status == "unsupported":
            return
        if record.action_type in ("file_write", "file_delete", "db_mutation"):
            if record.backup_path:
                target.parent.mkdir(parents=True, exist_ok=True)
                shutil.copy2(record.backup_path, target)
            elif target.exists():
                target.unlink()
        else:
            record.rollback_status = "unsupported"
            record.verification_result = {
                "status": "unsupported",
                "message": "No rollback adapter exists for this action type.",
            }
            self._write_recovery_record(record)
            return

        current = self._file_state(target)
        expected = {
            "exists": record.pre_state.get("exists", False),
            "sha256": record.pre_state.get("sha256"),
            "size": record.pre_state.get("size"),
        }
        verified = current == expected
        record.rollback_status = "reverted" if verified else "verification_failed"
        record.verification_result = {
            "status": record.rollback_status,
            "verified": verified,
            "expected": expected,
            "actual": current,
            "message": "State restored." if verified else "Restored state did not match pre-state.",
        }
        self._write_recovery_record(record)

    def _find_rollback(self, action_id: str) -> Optional[RollbackRecord]:
        for record in reversed(self._rollbacks):
            if record.action_id == action_id:
                return record
        return None

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
        self._write_recovery_record(record)

    def _write_recovery_record(self, record: RollbackRecord) -> None:
        journal = self._journal_dir / "rollback_journal.jsonl"
        with journal.open("a", encoding="utf-8") as fh:
            fh.write(json.dumps(record.to_dict(), sort_keys=True) + "\n")

    def _snapshot_file(self, path: Path, label: str) -> str:
        snapshot_path = self._backup_dir / f"{uuid.uuid4()}.{label}.snapshot"
        snapshot_path.parent.mkdir(parents=True, exist_ok=True)
        shutil.copy2(path, snapshot_path)
        return str(snapshot_path)

    def _create_checkpoint(self) -> None:
        files: list[dict[str, Any]] = []
        if self.contract is not None:
            for raw_path in self.contract.allowed_paths:
                for target in self._checkpoint_targets(raw_path):
                    state = self._file_state(target)
                    try:
                        checkpoint_path = str(target.relative_to(self.workspace_root))
                    except ValueError:
                        checkpoint_path = str(target)
                    backup_path = self._checkpoint_snapshot(target) if state["exists"] else None
                    files.append(
                        {
                            "path": checkpoint_path,
                            "exists": state["exists"],
                            "sha256": state["sha256"],
                            "size": state["size"],
                            "backup_path": backup_path,
                        }
                    )

        payload = {
            "schema_version": 1,
            "checkpoint_type": "SAFE_SESSION_START",
            "session_id": self.session_id,
            "agent_name": self.agent_name,
            "workspace_root": str(self.workspace_root),
            "task": self.task,
            "contract_hash": self.contract.canonical_hash if self.contract else "",
            "files": files,
        }
        self._checkpoint_path.write_text(
            json.dumps(payload, sort_keys=True, indent=2),
            encoding="utf-8",
        )

    def _checkpoint_targets(self, raw_path: str) -> list[Path]:
        normalized = raw_path.replace("\\", "/")
        if normalized.endswith("/**"):
            base = self._resolve(normalized[:-3])
            candidates = [path for path in base.rglob("*") if path.is_file()] if base.exists() else []
        elif any(marker in raw_path for marker in "*?[]"):
            candidates = [path for path in self.workspace_root.glob(raw_path) if path.is_file()]
        else:
            target = self._resolve(raw_path)
            candidates = [target] if target.exists() and target.is_file() else []
        safe_candidates = []
        for candidate in candidates:
            resolved = candidate.resolve()
            if self._within_workspace(resolved) and ".onus" not in resolved.parts:
                safe_candidates.append(resolved)
        return safe_candidates[:1000]

    def _checkpoint_snapshot(self, path: Path) -> str:
        digest = self._file_state(path)["sha256"] or str(uuid.uuid4())
        snapshot_path = self._checkpoint_root / "files" / digest[:2] / digest
        snapshot_path.parent.mkdir(parents=True, exist_ok=True)
        if not snapshot_path.exists():
            shutil.copy2(path, snapshot_path)
        return str(snapshot_path)

    def _resolve(self, path: Union[str, os.PathLike[str]]) -> Path:
        candidate = Path(path)
        if not candidate.is_absolute():
            candidate = self.workspace_root / candidate
        return candidate.resolve()

    def _within_workspace(self, path: Path) -> bool:
        try:
            path.resolve().relative_to(self.workspace_root)
            return True
        except ValueError:
            return False

    def _file_state(self, path: Path) -> dict[str, Any]:
        if not path.exists():
            return {"exists": False, "sha256": None, "size": None}
        digest = hashlib.sha256()
        with path.open("rb") as fh:
            for chunk in iter(lambda: fh.read(1024 * 1024), b""):
                digest.update(chunk)
        stat = path.stat()
        return {"exists": True, "sha256": digest.hexdigest(), "size": stat.st_size}

    def _assert_file_unchanged(self, path: Path, expected: dict[str, Any]) -> None:
        current = self._file_state(path)
        if current != expected:
            raise OnusEvaluationError(
                f"Refusing to execute because {path} changed after Onus evaluated the proposed action."
            )


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
    "OnusEvaluationError",
    "OnusRollbackUnsupportedError",
    "OnusClient",
    "OnusResult",
    "RollbackRecord",
    "RollbackResult",
    "ReversibilityClass",
    "RecoveryAdapter",
    "RepositoryFileRecoveryAdapter",
    "SQLiteRecoveryAdapter",
    "UnsupportedExternalRecoveryAdapter",
    "TaskContract",
    "ChangeBudget",
    "RequiredEvidence",
    "CompletionEvidence",
    "PromptIntakeResult",
    "evaluate",
    "check_command",
    "get_client",
]
