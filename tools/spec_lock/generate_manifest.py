from __future__ import annotations

import argparse
import sys
from pathlib import Path

from spec_lock import APPROVAL_PHRASE, DEFAULT_MANIFEST, repo_root_from_script, write_manifest


def main() -> int:
    root_default = repo_root_from_script()
    parser = argparse.ArgumentParser(
        description="Regenerate the locked Onus specification SHA-256 manifest."
    )
    parser.add_argument(
        "--approval",
        required=True,
        help=f"Must be exactly {APPROVAL_PHRASE!r}",
    )
    parser.add_argument("--root", default=str(root_default), help="Repository root")
    parser.add_argument(
        "--manifest",
        default=str(root_default / DEFAULT_MANIFEST),
        help="Manifest output path",
    )
    args = parser.parse_args()

    if args.approval != APPROVAL_PHRASE:
        print(
            f"Refusing to regenerate manifest without exact approval phrase: {APPROVAL_PHRASE}",
            file=sys.stderr,
        )
        return 2

    root = Path(args.root).resolve()
    manifest_path = Path(args.manifest).resolve()
    write_manifest(root, manifest_path)
    print(f"Regenerated {manifest_path}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
