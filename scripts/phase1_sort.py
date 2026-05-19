"""Phase 1 post-compile sorter.

Reads `game-config.json` to learn which kinds are approved for this project,
then walks the compiler's flat output at `wiki/concepts/*.md` and copies each
file into `vault/<kind>/<slug>.md` based on its frontmatter `kind` (or `type`)
field. Files whose kind is missing, unknown, or not in the approved set are
copied to `vault/_quarantine/<slug>.md` with a `validation_errors:` block.

The deployment guide's Phase 1 §4.1 checklist requires "zero quarantined
files" before Phase 2 may consume the vault. Running this sorter after
`llmwiki compile` produces the quarantine bucket that check inspects.

Usage:
    python scripts/phase1_sort.py [--source wiki/concepts] [--dest vault] \\
                                  [--config game-config.json] [--dry-run]

Requirements:
    pip install pyyaml
"""

from __future__ import annotations

import argparse
import json
import re
import shutil
import sys
from dataclasses import dataclass
from pathlib import Path

try:
    import yaml
except ImportError:
    sys.stderr.write(
        "phase1_sort: missing dependency 'pyyaml'. Install with: pip install pyyaml\n"
    )
    sys.exit(2)

FRONTMATTER_RE = re.compile(r"^---\s*\n(.*?)\n---\s*\n", re.DOTALL)


@dataclass(frozen=True)
class SortConfig:
    source_dir: Path
    dest_dir: Path
    config_path: Path
    dry_run: bool


@dataclass(frozen=True)
class SortReport:
    sorted_count: int
    quarantined_count: int
    skipped_count: int


def load_approved_kinds(config_path: Path) -> set[str]:
    if not config_path.exists():
        raise SystemExit(f"phase1_sort: game-config.json not found at {config_path}")
    raw = json.loads(config_path.read_text(encoding="utf-8"))
    kinds = raw.get("kinds")
    if not isinstance(kinds, dict) or not kinds:
        raise SystemExit("phase1_sort: game-config.json has no `kinds` object")
    if raw.get("human_approved") is not True:
        sys.stderr.write(
            "phase1_sort: WARNING — game-config.json has human_approved != true. "
            "Proceeding, but Phase 0 sign-off is required before production runs.\n"
        )
    return set(kinds.keys())


def read_frontmatter(file_path: Path) -> tuple[dict | None, str]:
    body = file_path.read_text(encoding="utf-8")
    match = FRONTMATTER_RE.match(body)
    if not match:
        return None, body
    try:
        data = yaml.safe_load(match.group(1)) or {}
    except yaml.YAMLError:
        return None, body
    if not isinstance(data, dict):
        return None, body
    return data, body


def resolve_kind(frontmatter: dict | None) -> str | None:
    """Read `kind` first (llmwiki's field), then `type` (guide's field)."""
    if frontmatter is None:
        return None
    for field in ("kind", "type"):
        value = frontmatter.get(field)
        if isinstance(value, str) and value.strip():
            return value.strip()
    return None


def quarantine_body(original_body: str, reason: str) -> str:
    error_block = (
        "---\n"
        f"validation_errors:\n"
        f"  - {reason}\n"
        "---\n\n"
    )
    return error_block + original_body


def route_file(
    source: Path,
    dest_root: Path,
    approved: set[str],
    dry_run: bool,
) -> str:
    frontmatter, body = read_frontmatter(source)
    kind = resolve_kind(frontmatter)
    slug = source.stem

    if kind is None:
        target = dest_root / "_quarantine" / f"{slug}.md"
        payload = quarantine_body(body, "missing `kind`/`type` in frontmatter")
        status = "quarantine"
    elif kind not in approved:
        target = dest_root / "_quarantine" / f"{slug}.md"
        payload = quarantine_body(body, f"unknown kind '{kind}' — not in approved set")
        status = "quarantine"
    else:
        target = dest_root / kind / f"{slug}.md"
        payload = body
        status = "sorted"

    if dry_run:
        return f"{status}\t{source} -> {target}"

    target.parent.mkdir(parents=True, exist_ok=True)
    target.write_text(payload, encoding="utf-8")
    return f"{status}\t{source.name} -> {target.relative_to(dest_root.parent)}"


def reset_dest(dest_root: Path, approved: set[str], dry_run: bool) -> None:
    """Remove only the directories the sorter manages so stale files don't linger.

    Leaves user-authored content in `vault/` (anything outside the approved
    kind directories and `_quarantine/`) untouched.
    """
    managed = {*approved, "_quarantine"}
    if not dest_root.exists():
        return
    for child in dest_root.iterdir():
        if child.is_dir() and child.name in managed:
            if dry_run:
                continue
            shutil.rmtree(child)


def parse_args(argv: list[str]) -> SortConfig:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--source", default="wiki/concepts", help="Compiler output directory")
    parser.add_argument("--dest", default="vault", help="Vault root to write into")
    parser.add_argument("--config", default="game-config.json", help="Phase 0 taxonomy file")
    parser.add_argument("--dry-run", action="store_true", help="Report intended moves only")
    ns = parser.parse_args(argv)
    return SortConfig(
        source_dir=Path(ns.source),
        dest_dir=Path(ns.dest),
        config_path=Path(ns.config),
        dry_run=ns.dry_run,
    )


def run(cfg: SortConfig) -> SortReport:
    approved = load_approved_kinds(cfg.config_path)
    if not cfg.source_dir.exists():
        raise SystemExit(f"phase1_sort: source dir {cfg.source_dir} does not exist")

    reset_dest(cfg.dest_dir, approved, cfg.dry_run)

    sorted_count = 0
    quarantined_count = 0
    skipped_count = 0

    for md_path in sorted(cfg.source_dir.glob("*.md")):
        if md_path.name == "index.md":
            skipped_count += 1
            continue
        line = route_file(md_path, cfg.dest_dir, approved, cfg.dry_run)
        print(line)
        if line.startswith("quarantine"):
            quarantined_count += 1
        else:
            sorted_count += 1

    return SortReport(sorted_count, quarantined_count, skipped_count)


def main(argv: list[str]) -> int:
    cfg = parse_args(argv)
    report = run(cfg)
    print(
        f"\nphase1_sort: sorted={report.sorted_count} "
        f"quarantined={report.quarantined_count} skipped={report.skipped_count}"
    )
    return 1 if report.quarantined_count > 0 else 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
