# Onus Antigravity Live Verification Fixtures

This directory contains test fixtures used by the Antigravity live
verification scripts.

## Structure

- `allowed/` — Files that should be accessible (read allowed)
- `protected/` — Files that should be blocked (policy violation)
- `secrets/` — Files that should be redacted (secrets detection)
- `tests/` — Test fixtures for contract evaluation
