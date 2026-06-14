# Onus Specification Lock

This directory stores integrity tooling for locked Onus specification documents.
It intentionally lives outside `docs/`.

## Verify

Run from the repository root:

```bash
python tools/spec_lock/verify_spec_lock.py
```

The verifier fails when:

- a locked document changes;
- a locked document is missing;
- a locked document is renamed;
- the manifest JSON is malformed;
- the manifest path list differs from the canonical locked path list.

## Regenerate

Only regenerate the manifest after the user explicitly writes:

```text
SPEC CHANGE APPROVED
```

Then run:

```bash
python tools/spec_lock/generate_manifest.py --approval "SPEC CHANGE APPROVED"
python tools/spec_lock/verify_spec_lock.py
python -m unittest tests.test_spec_lock -v
```

Do not regenerate the manifest to hide accidental edits, shortened documents,
renames, deletions, or weakened requirements.
