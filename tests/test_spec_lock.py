from __future__ import annotations

import json
import shutil
import sys
import tempfile
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / "tools" / "spec_lock"))

from spec_lock import LOCKED_DOCUMENTS, validate_manifest_schema, verify, write_manifest


class SpecLockVerifierTests(unittest.TestCase):
    def setUp(self) -> None:
        self.tmp = Path(tempfile.mkdtemp(prefix="onus-spec-lock-test-"))
        for relative_path in LOCKED_DOCUMENTS:
            target = self.tmp / relative_path
            target.parent.mkdir(parents=True, exist_ok=True)
            target.write_text(f"locked content for {relative_path}\n", encoding="utf-8")
        self.manifest = self.tmp / "tools" / "spec_lock" / "manifest.json"
        write_manifest(self.tmp, self.manifest)

    def tearDown(self) -> None:
        shutil.rmtree(self.tmp)

    def test_valid_manifest_passes(self) -> None:
        self.assertEqual([], verify(self.tmp, self.manifest))

    def test_changed_locked_document_fails(self) -> None:
        (self.tmp / "SPEC.md").write_text("changed\n", encoding="utf-8")
        errors = verify(self.tmp, self.manifest)
        self.assertTrue(any("hash mismatch for SPEC.md" in error for error in errors))

    def test_missing_locked_document_fails(self) -> None:
        (self.tmp / "AGENTS.md").unlink()
        errors = verify(self.tmp, self.manifest)
        self.assertTrue(any("missing locked document: AGENTS.md" in error for error in errors))

    def test_renamed_locked_document_fails(self) -> None:
        (self.tmp / "MANIFESTO.md").rename(self.tmp / "MANIFESTO_RENAMED.md")
        errors = verify(self.tmp, self.manifest)
        self.assertTrue(any("missing locked document: MANIFESTO.md" in error for error in errors))

    def test_malformed_json_fails(self) -> None:
        self.manifest.write_text("{not json", encoding="utf-8")
        errors = verify(self.tmp, self.manifest)
        self.assertTrue(any("manifest is malformed JSON" in error for error in errors))

    def test_manifest_missing_canonical_path_fails_schema(self) -> None:
        manifest = json.loads(self.manifest.read_text(encoding="utf-8"))
        manifest["locked_documents"] = manifest["locked_documents"][1:]
        errors = validate_manifest_schema(manifest)
        self.assertTrue(
            any("locked document path list does not match canonical set" in error for error in errors)
        )


if __name__ == "__main__":
    unittest.main()
