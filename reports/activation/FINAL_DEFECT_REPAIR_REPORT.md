# FINAL DEFECT REPAIR REPORT

Generated: 2026-06-18
Branch: final/activation-readiness
Commit: [to be added at commit time]

---

## Defect #1 — Receipt hash chain across sessions

### Reproduced original defect

`verify_all_actions()` treated ALL actions as one continuous hash chain (ordered
by `id ASC`), but `record_action()` chains per-session (prev_hash comes from
the previous action in the same session via `WHERE session_id = ?`).

When two independent sessions existed, `verify_all_actions()` would fail because
it expected the first action of Session B to chain from the last action of Session A.

### Root cause

The `verify_all_actions` function at `src/audit/db.rs` had a single sequential
loop over all actions regardless of session_id.  It assumed one global chain,
while the recording code correctly implements per-session chains.

### Production files changed

- `src/audit/db.rs`:
  - Added `session_anchors` table to schema (tracks global session ordering)
  - Replaced `verify_all_actions()` with per-session verification + anchor verification
  - Added `verify_session_anchors()` for cross-session ordering
  - Added `create_session_anchor()` called by `end_session()`
  - Modified `end_session()` to call `create_session_anchor()`

### Regression tests added (12 tests)

All in `src/audit/db.rs` tests module:

| Test | What it verifies |
|------|-----------------|
| `chain_multiple_actions_in_one_session` | Basic single-session chain integrity |
| `chain_two_independent_sessions` | Per-session chains do not cross-contaminate |
| `chain_handoff_preserves_continuity` | Same-session handoff preserves chain |
| `chain_detects_removed_receipt` | Tampering by deletion detected |
| `chain_detects_modified_receipt` | Tampering by modification detected |
| `chain_detects_reordered_receipts` | Reordered sequences detected |
| `chain_detects_corrupted_prev_hash` | Corrupted prev_hash detected |
| `chain_verified_after_process_restart` | Chain survives process restart |
| `chain_concurrent_writes_do_not_corrupt` | Interleaved sessions are safe |
| `chain_secrets_remain_redacted_in_receipts` | Secrets redacted in stored payload |
| `chain_old_receipts_never_rewritten` | Adding actions doesn't change old hashes |
| `chain_session_anchor_verification` | Session anchors are created and verifiable |

### Runtime proof

```
cargo test --lib audit::db::tests::chain_  → 12 passed, 0 failed
```

---

## Defect #2 — Checkpoint restore

### Reproduced original defect

`restore_checkpoint()` only **reported** differences between the current workspace
and the checkpoint manifest.  It never:

1. Copied file content from checkpoint storage back to the workspace
2. Removed files created after the checkpoint
3. Verified final manifest equality after restore

Also, `create_checkpoint()` did not store file copies — only hashes.

### Root cause

The checkpoint system stored only a `manifest.json` with file hashes, never the
actual file contents.  `restore_checkpoint` was designed as a verification-only
operation rather than an actual restore operation.

### Production files changed

- `src/rollback.rs`:
  - `create_checkpoint()`: Now copies every tracked file to `<cp_dir>/files/<rel_path>`
  - `restore_checkpoint()`: Now actually restores files from checkpoint storage,
    removes newly created files, and verifies final manifest equality after restore.
  - Added proper `RollbackStatus::Partial` reporting when full restoration is impossible.

### Regression tests added (10 tests)

All in `src/rollback.rs` tests module:

| Test | What it verifies |
|------|-----------------|
| `test_restore_modified_file` | Restores modified file content |
| `test_restore_deleted_file` | Restores deleted file |
| `test_restore_removes_new_file` | Removes files created after checkpoint |
| `test_restore_preserves_unchanged_files` | Unchanged files are not touched |
| `test_restore_nested_directories` | Nested directory structure is restored |
| `test_restore_wrong_repository_fails` | Wrong workspace is rejected |
| `test_restore_corrupted_manifest_fails` | Corrupted data is rejected |
| `test_restore_interrupted_then_repeated` | Repeated restore is idempotent |
| `test_restore_creates_receipt` | Restore receipt is created |
| `test_restore_final_manifest_matches_checkpoint` | Post-restore manifest matches exactly |

### Runtime proof

```
cargo test --lib rollback::tests::test_restore_  → 10 passed, 0 failed
```

---

## Overall test results

```
cargo test --lib                → 176 passed, 0 failed
cargo build --all-targets       → builds clean
cargo clippy --all-targets      → no new warnings
cargo build --release           → release binary built
```

## Remaining limitations

1. **No web console** — The `console/` directory does not exist yet.  Use `onus dashboard`.
2. **VS Code extension** — Registry exists but extension is not fully implemented.
3. **Python SDK** — Referenced in site but not published.
4. **Continuity end-to-end** — The Rust API supports handoff/lease, but the CLI `handoff`/
   `lease` commands are not exposed (`clap` subcommands not registered).  The
   `onus setup` command registers both Claude and Codex hooks, enabling manual
   handoff, but the automated cross-agent continuity flow requires configuring
   the MCP proxy and shared workspace lease.
5. **Antigravity integration** — Setup and MCP proxy are present but not tested.
