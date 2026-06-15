# Onus L4 Authority Proof Runtime Report

Date: 2026-06-16

## Claim Boundary

This milestone implements the first narrow L4 proof:

```text
ONUS_CONTROLLED_AUTHORITY_DISPOSABLE_SQLITE
```

The controlled privileged operation is:

```text
insert_l4_item
```

against a disposable SQLite database created by Onus. This is not production,
not a cloud credential broker, and not a generalized deployment authority.

Safe wording:

```text
Onus has a runtime-proven narrow L4 proof for one broker-owned disposable
SQLite operation. The agent receives a short-lived scoped capability, never the
broker-held long-lived authority secret, and Onus executes the exact authorized
payload itself.
```

Unsafe wording:

```text
Onus has production credential control.
Onus can safely deploy to production.
Onus supports arbitrary L4 operations.
Onus has a general secret manager.
```

## Implemented Flow

1. `onus authority init-disposable-db`
   - Creates a disposable SQLite DB.
   - Creates the `l4_items` table.
   - Generates a broker-held long-lived authority secret.
   - Stores only the secret hash in metadata.
   - Does not return the raw secret to the caller.

2. `onus authority authorize`
   - Requires `--human-approved`.
   - Verifies environment identity.
   - Verifies the payload schema.
   - Computes canonical payload hash.
   - Issues a short-lived capability token.
   - Stores only the capability token hash.
   - Binds the capability to:
     - authority ID;
     - session ID;
     - action ID;
     - canonical payload hash;
     - policy version;
     - environment identity;
     - expiry;
     - approver.

3. `onus authority execute`
   - Requires the exact short-lived capability.
   - Rejects missing, expired, revoked, used, or altered-payload capabilities.
   - Executes the SQLite insert inside the broker.
   - Marks the capability used.
   - Records an authority receipt.

4. `onus authority revoke`
   - Revokes an unused capability.

5. `onus authority compensate`
   - Deletes the inserted disposable row by `row_id`.
   - Records compensation verification.

## Independent Verification

Verifier:

```text
tools/l4_authority/verify_l4_authority.py
```

Command:

```text
python tools\l4_authority\verify_l4_authority.py --onus-bin D:\Onus\onus\target\debug\onus.exe --json
```

Result:

```text
status: passed
operation: disposable_sqlite_insert_l4_item
```

Verified tests:

- disposable authority initialized;
- raw credential not returned on init;
- verified environment identity;
- human approval required;
- short-lived scoped capability issued;
- raw credential not returned on authorize;
- altered payload denied;
- exact authorized action broker-executed;
- audit receipt recorded without raw credential;
- reuse denied;
- revocation denied execution;
- expiry denied execution;
- compensation deleted disposable row;
- stored receipts, metadata, and capability records did not contain the raw credential;
- authority receipt hash chain verified.

## Limitations

- This proves one disposable SQLite operation only.
- It is not production.
- It is not a general deployment authority.
- It is not a general database authority.
- The broker-held secret is stored in Onus local data for the proof; production L4 needs OS-backed key storage or an external KMS/HSM and must be paired with L3/L4 access boundaries.
- The capability token is returned to the caller once; it is short-lived, exact-payload-bound, revocable, and single-use, but should still be handled as sensitive.

## Runtime Validation Summary

```text
cargo fmt: passed
cargo build: passed
cargo test: 75 passed
cargo clippy: passed
python -m pytest -q -rs onus\bindings\python\tests tests: 63 passed, 2 skipped
spec lock verifier: passed
spec lock unit tests: 6 passed
reality demo: passed, DEMO_ONLY
receipt verification: Hash chain integrity verified: ALL PASS
L4 verifier: passed, including receipt hash-chain verification
```
