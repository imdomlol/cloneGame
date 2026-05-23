"""Regenerate build/repomix-output.xml when vault/ has changed.

Hashes vault/**/*.md (excluding vault/_quarantine/**) and compares against
build/.repomix.stamp. If the hash differs (or the output file is missing),
runs repomix with the canonical flags and updates the stamp. The bundle is
a local-only artifact (build/ is gitignored) used for Phase 2 work.
"""

from __future__ import annotations

import argparse
import hashlib
import shutil
import subprocess
import sys
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[1]
VAULT_DIR = REPO_ROOT / "vault"
QUARANTINE_DIR = VAULT_DIR / "_quarantine"
BUILD_DIR = REPO_ROOT / "build"
OUTPUT_FILE = BUILD_DIR / "repomix-output.xml"
STAMP_FILE = BUILD_DIR / ".repomix.stamp"


def vault_files() -> list[Path]:
    if not VAULT_DIR.is_dir():
        return []
    files: list[Path] = []
    for path in VAULT_DIR.rglob("*.md"):
        if QUARANTINE_DIR in path.parents:
            continue
        files.append(path)
    files.sort()
    return files


def compute_hash(files: list[Path]) -> str:
    h = hashlib.sha256()
    for path in files:
        rel = path.relative_to(REPO_ROOT).as_posix().encode("utf-8")
        h.update(rel)
        h.update(b"\0")
        h.update(path.read_bytes())
        h.update(b"\0")
    return h.hexdigest()


def run_repomix() -> None:
    BUILD_DIR.mkdir(parents=True, exist_ok=True)
    exe = shutil.which("repomix")
    if exe is None:
        sys.stderr.write(
            "regenerate_repomix: 'repomix' not found on PATH. "
            "Install with `npm install -g repomix`.\n"
        )
        sys.exit(127)
    cmd = [
        exe,
        "--include",
        "vault/**/*.md",
        "--ignore",
        "vault/_quarantine/**",
        "--style",
        "xml",
        "--output",
        OUTPUT_FILE.relative_to(REPO_ROOT).as_posix(),
        "--no-file-summary",
        "--remove-comments",
        # vault/ is gitignored locally; tell repomix to scan it anyway.
        "--no-gitignore",
    ]
    subprocess.run(cmd, check=True, cwd=REPO_ROOT)


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--force",
        action="store_true",
        help="regenerate even when the vault hash is unchanged",
    )
    args = parser.parse_args()

    files = vault_files()
    current_hash = compute_hash(files)
    stored_hash = STAMP_FILE.read_text(encoding="utf-8").strip() if STAMP_FILE.exists() else ""

    up_to_date = not args.force and current_hash == stored_hash and OUTPUT_FILE.exists()
    if up_to_date:
        return 0

    run_repomix()
    STAMP_FILE.parent.mkdir(parents=True, exist_ok=True)
    STAMP_FILE.write_text(current_hash, encoding="utf-8")

    return 0


if __name__ == "__main__":
    sys.exit(main())
