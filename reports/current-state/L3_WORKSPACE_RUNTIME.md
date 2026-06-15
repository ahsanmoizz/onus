# Onus Linux L3 Workspace Runtime Report

Date: 2026-06-16

## Claim Boundary

This milestone adds the first Linux-only Onus workspace boundary surface:

- `onus workspace create`
- `onus run --isolate -- <agent command>`
- `onus workspace inspect`
- `onus workspace export`
- `onus workspace destroy`

L3 is certified only for the runtime-tested Linux configuration below. Windows
itself remains a fail-closed host for this feature; `onus run --isolate` refuses
to execute there.

`workspace create` metadata is intentionally not marked as verified L3. New
workspaces are labeled:

```text
isolation_level=L3_PENDING_RUNTIME_VERIFICATION
enforcement_label=L3_LINUX_WORKSPACE_PENDING_VERIFICATION
boundary_verified=false
```

The label changes to:

```text
isolation_level=L3_LINUX_BUBBLEWRAP
enforcement_label=L3_LINUX_WORKSPACE_RUNTIME_VERIFIED
boundary_verified=true
```

only after `onus run --isolate` starts through the Linux isolation runner.

Safe wording:

```text
Onus now includes Linux L3 workspace infrastructure that fails closed outside a
real Linux bubblewrap boundary. L3 certification requires the adversarial
verifier to pass on a Linux host with bubblewrap installed.
```

Unsafe wording:

```text
Onus L3 isolation is verified.
Onus cannot be bypassed.
Onus controls all filesystem, process, network, and credential access.
```

## Implemented Boundary Design

Workspace creation stores session state under the Onus data directory:

```text
<onus-data>/workspaces/<session>/
  workspace.json
  worktree/
  checkpoints/initial.manifest.json
  artifacts/
```

The original repository is not modified. A writable session worktree is created
outside the repository. The initial checkpoint is a deterministic SHA-256
manifest of copied worktree files.

On Linux, `onus run --isolate` requires `bubblewrap` (`bwrap`) and refuses to
execute if it is absent. The isolated command is launched with:

- controlled process tree via Linux namespaces and `--die-with-parent`;
- read-only original repository mounted at `/original`;
- writable session worktree mounted at `/workspace`;
- host paths omitted by default;
- deny-all network namespace by default;
- optional explicit `--allow-network`;
- `--clearenv` and Onus-only safe environment variables;
- no inherited raw production credentials;
- CPU, memory, process-count, and open-file resource limits;
- child-process inheritance of the same namespace and limits.

## Independent Verifier

Verifier:

```text
tools/l3_workspace/verify_l3_workspace.py
```

Adversarial checks included:

- direct filesystem write to read-only original repository;
- subprocess write attempt to original repository;
- raw socket network egress;
- `urllib` HTTP egress;
- `requests` HTTP egress;
- `httpx` HTTP egress;
- `curl` HTTP egress;
- environment secret reads;
- direct SQLite access to host database;
- host file reads;
- attempts to disable Onus through inherited environment;
- positive control: write inside `/workspace`.

Linux verifier environment:

```text
Host: WSL2
Kernel: Linux 6.18.33.1-microsoft-standard-WSL2
Distro: Ubuntu 24.04.4 LTS
bubblewrap: 0.9.0
Python: 3.12.3
curl: 8.5.0
rustc: 1.96.0
cargo: 1.96.0
```

Linux verifier command:

```bash
cd /mnt/d/Onus
python3 tools/l3_workspace/verify_l3_workspace.py \
  --onus-bin /mnt/d/Onus/onus/target/debug/onus \
  --json
```

Linux verifier result:

```json
{
  "verifier": "l3_workspace",
  "platform": "Linux-6.18.33.1-microsoft-standard-WSL2-x86_64-with-glibc2.39",
  "status": "passed",
  "tests": [
    "filesystem_write_to_original_repo",
    "subprocess_inherits_filesystem_boundary",
    "raw_socket_network_egress",
    "urllib_http_egress",
    "requests_http_egress",
    "httpx_http_egress",
    "curl_http_egress",
    "environment_secret_read",
    "direct_sqlite_host_db_access",
    "host_file_read",
    "attempt_disable_onus_env",
    "writable_session_worktree",
    "runtime_verified_metadata"
  ],
  "limitations": []
}
```

## Runtime Validation

Commands run locally:

```text
cargo fmt
cargo build
cargo test
cargo clippy
python -m pytest -q -rs onus\bindings\python\tests\test_onus.py::TestL3WorkspaceCli
python tools\l3_workspace\verify_l3_workspace.py --onus-bin D:\Onus\onus\target\debug\onus.exe --json
wsl.exe -d Ubuntu-24.04 -u root -- bash -lc "cd /mnt/d/Onus && python3 tools/l3_workspace/verify_l3_workspace.py --onus-bin /mnt/d/Onus/onus/target/debug/onus --json"
```

Observed results:

```text
cargo fmt: passed
cargo build: passed
cargo test: 74 passed
cargo clippy: passed
TestL3WorkspaceCli: 2 passed
L3 verifier on Windows host: UNVERIFIABLE / fail-closed
L3 verifier on Ubuntu 24.04 WSL2 with bubblewrap: PASSED
```

## Required Certification Step

Run this on any additional Linux host with `bwrap`, `python3`, `curl`,
`requests`, and `httpx` available:

```bash
cargo build
python3 tools/l3_workspace/verify_l3_workspace.py \
  --onus-bin ./onus/target/debug/onus \
  --json
```

Only if the verifier reports:

```json
{ "status": "passed" }
```

may this milestone be claimed as runtime-proven L3 for the tested Linux
configuration.
