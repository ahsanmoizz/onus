from __future__ import annotations

import argparse
import sys
from pathlib import Path

from spec_lock import DEFAULT_MANIFEST, repo_root_from_script, verify


def main() -> int:
    root_default = repo_root_from_script()
    parser = argparse.ArgumentParser(
        description="Verify locked Onus specification documents against the SHA-256 manifest."
    )
    parser.add_argument("--root", default=str(root_default), help="Repository root")
    parser.add_argument(
        "--manifest",
        default=str(root_default / DEFAULT_MANIFEST),
        help="Path to the spec lock manifest",
    )
    args = parser.parse_args()

    root = Path(args.root).resolve()
    manifest_path = Path(args.manifest).resolve()
    errors = verify(root, manifest_path)
    if errors:
        print("SPEC LOCK VERIFICATION FAILED", file=sys.stderr)
        for error in errors:
            print(f"- {error}", file=sys.stderr)
        return 1

    print("SPEC LOCK VERIFICATION PASSED")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
