from __future__ import annotations

import hashlib
import json
import re
from pathlib import Path
from typing import Any


LOCKED_DOCUMENTS: tuple[str, ...] = (
    "AGENTS.md",
    "MANIFESTO.md",
    "SPEC.md",
    "docs/ONUS_ACCEPTANCE_TESTS.md",
    "docs/Onus_current_state.md",
    "docs/ONUS_IMPLEMENTATION_ROADMAP.md",
    "docs/ONUS_PRODUCT_VISION.md",
    "docs/ONUS_SECURITY_REQUIREMENTS.md",
    "docs/ONUS_TARGET_ARCHITECTURE.md",
    "docs/Onus_Whitepaper.txt",
)

MANIFEST_VERSION = 1
ALGORITHM = "sha256"
APPROVAL_PHRASE = "SPEC CHANGE APPROVED"
DEFAULT_MANIFEST = Path("tools/spec_lock/manifest.json")
HEX_64 = re.compile(r"^[0-9a-f]{64}$")


class SpecLockError(Exception):
    pass


def repo_root_from_script() -> Path:
    return Path(__file__).resolve().parents[2]


def normalize_manifest_path(path: str) -> str:
    normalized = path.replace("\\", "/")
    if normalized.startswith("/") or normalized.startswith("../") or "/../" in normalized:
        raise SpecLockError(f"unsafe manifest path: {path!r}")
    return normalized


def sha256_file(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        for chunk in iter(lambda: handle.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def build_manifest(root: Path) -> dict[str, Any]:
    documents: list[dict[str, str]] = []
    for relative_path in LOCKED_DOCUMENTS:
        file_path = root / Path(relative_path)
        if not file_path.is_file():
            raise SpecLockError(f"locked document is missing: {relative_path}")
        documents.append(
            {
                "path": relative_path,
                "sha256": sha256_file(file_path),
            }
        )

    return {
        "manifest_version": MANIFEST_VERSION,
        "algorithm": ALGORITHM,
        "regeneration_policy": (
            f"Regenerate only after the user writes exactly {APPROVAL_PHRASE!r}."
        ),
        "locked_documents": documents,
    }


def write_manifest(root: Path, manifest_path: Path) -> None:
    manifest = build_manifest(root)
    manifest_path.parent.mkdir(parents=True, exist_ok=True)
    manifest_path.write_text(
        json.dumps(manifest, indent=2, sort_keys=False) + "\n",
        encoding="utf-8",
        newline="\n",
    )


def load_manifest(path: Path) -> dict[str, Any]:
    try:
        loaded = json.loads(path.read_text(encoding="utf-8"))
    except FileNotFoundError as exc:
        raise SpecLockError(f"manifest is missing: {path}") from exc
    except json.JSONDecodeError as exc:
        raise SpecLockError(f"manifest is malformed JSON: {exc}") from exc

    if not isinstance(loaded, dict):
        raise SpecLockError("manifest must be a JSON object")
    return loaded


def validate_manifest_schema(manifest: dict[str, Any]) -> list[str]:
    errors: list[str] = []

    if manifest.get("manifest_version") != MANIFEST_VERSION:
        errors.append(f"manifest_version must be {MANIFEST_VERSION}")

    if manifest.get("algorithm") != ALGORITHM:
        errors.append(f"algorithm must be {ALGORITHM!r}")

    entries = manifest.get("locked_documents")
    if not isinstance(entries, list) or not entries:
        errors.append("locked_documents must be a non-empty list")
        return errors

    seen: set[str] = set()
    manifest_paths: list[str] = []
    for index, entry in enumerate(entries):
        if not isinstance(entry, dict):
            errors.append(f"locked_documents[{index}] must be an object")
            continue

        raw_path = entry.get("path")
        digest = entry.get("sha256")
        if not isinstance(raw_path, str):
            errors.append(f"locked_documents[{index}].path must be a string")
            continue

        try:
            relative_path = normalize_manifest_path(raw_path)
        except SpecLockError as exc:
            errors.append(str(exc))
            continue

        if relative_path in seen:
            errors.append(f"duplicate locked document path: {relative_path}")
        seen.add(relative_path)
        manifest_paths.append(relative_path)

        if not isinstance(digest, str) or not HEX_64.match(digest):
            errors.append(f"invalid sha256 for {relative_path}")

    expected = list(LOCKED_DOCUMENTS)
    if manifest_paths != expected:
        errors.append(
            "locked document path list does not match canonical set; "
            f"expected {expected}, got {manifest_paths}"
        )

    return errors


def verify(root: Path, manifest_path: Path) -> list[str]:
    errors: list[str] = []
    try:
        manifest = load_manifest(manifest_path)
    except SpecLockError as exc:
        return [str(exc)]

    schema_errors = validate_manifest_schema(manifest)
    if schema_errors:
        return schema_errors

    manifest_by_path = {
        normalize_manifest_path(entry["path"]): entry["sha256"]
        for entry in manifest["locked_documents"]
    }

    for relative_path in LOCKED_DOCUMENTS:
        file_path = root / Path(relative_path)
        if not file_path.is_file():
            errors.append(f"missing locked document: {relative_path}")
            continue

        actual = sha256_file(file_path)
        expected = manifest_by_path[relative_path]
        if actual != expected:
            errors.append(
                f"hash mismatch for {relative_path}: expected {expected}, got {actual}"
            )

    return errors
