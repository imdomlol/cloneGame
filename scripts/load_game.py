"""Restore a shipped game from ``releases/<slug>/`` back to the working tree.

The inverse of ``scripts/ship.py``. Replaces the current ``game/`` crate,
``vault/``, and ``game-config.json`` with the snapshot from a prior release,
so the operator can ``cargo run`` the shipped game OR resume pipeline work
against it (regenerate with a newer prompt, add a new kind, etc.).

This is destructive to the current working tree. By default it refuses to
run if ``game/src/``, ``vault/``, or ``game-config.json`` exist with
non-trivial content; pass ``--force`` to overwrite.

Usage:

    python scripts/load_game.py                  # list available releases
    python scripts/load_game.py lethal-company   # restore that slug
    python scripts/load_game.py lethal-company --force  # overwrite working tree
"""

from __future__ import annotations

import argparse
import shutil
import sys
from pathlib import Path

_REPO_ROOT = Path(__file__).resolve().parent.parent
DEFAULT_GAME_CONFIG = _REPO_ROOT / "game-config.json"
DEFAULT_GAME_DIR = _REPO_ROOT / "game"
DEFAULT_VAULT_DIR = _REPO_ROOT / "vault"
DEFAULT_SYSTEM_MAP = _REPO_ROOT / "build" / "system_map.yaml"
DEFAULT_RELEASES_ROOT = _REPO_ROOT / "releases"


def list_releases(releases_root: Path) -> list[str]:
    """Enumerate slugs that have a ``source/Cargo.toml`` (a complete snapshot)."""
    if not releases_root.is_dir():
        return []
    out: list[str] = []
    for entry in sorted(releases_root.iterdir()):
        if entry.is_dir() and (entry / "source" / "Cargo.toml").is_file():
            out.append(entry.name)
    return out


def _has_content(path: Path) -> bool:
    """True if ``path`` exists and has any file (or non-empty subdir).

    Used as the "is the working tree dirty" check before refusing to load.
    A bare empty directory does not count as content; an actual file does.
    """
    if not path.exists():
        return False
    if path.is_file():
        return path.stat().st_size > 0
    for _ in path.rglob("*"):
        return True
    return False


def _replace_dir(src: Path, dst: Path) -> int:
    """Replace ``dst`` with a copy of ``src``; return number of files written.

    Removes the existing ``dst`` directory first (the caller has already
    confirmed that's OK via ``--force`` or an empty working tree). Avoids
    ``shutil.copytree``'s "destination must not exist" constraint while
    keeping the operation atomic enough that a partial failure leaves a
    visible bad state instead of silent drift.
    """
    if dst.exists():
        shutil.rmtree(dst)
    if not src.exists():
        return 0
    written = 0
    for entry in src.rglob("*"):
        if entry.is_file():
            rel = entry.relative_to(src)
            target = dst / rel
            target.parent.mkdir(parents=True, exist_ok=True)
            shutil.copy2(entry, target)
            written += 1
    return written


def _check_dirty_or_force(
    game_dir: Path,
    vault_dir: Path,
    game_config: Path,
    force: bool,
) -> bool:
    """Return True if it's safe to proceed; print + return False if not."""
    if force:
        return True
    dirty: list[str] = []
    if _has_content(game_dir / "src"):
        dirty.append(str(game_dir / "src"))
    if _has_content(vault_dir):
        dirty.append(str(vault_dir))
    if _has_content(game_config):
        dirty.append(str(game_config))
    if dirty:
        sys.stderr.write(
            "load_game: working tree has content:\n  "
            + "\n  ".join(dirty)
            + "\nRe-run with --force to overwrite.\n"
        )
        return False
    return True


def _restore_game_dir(release: Path, game_dir: Path) -> None:
    """Replace game/src/ + tests/ + Cargo files from the release snapshot."""
    src_count = _replace_dir(release / "source" / "src", game_dir / "src")
    print(f"  game/src/: {src_count} files")
    tests_src = release / "source" / "tests"
    if tests_src.is_dir():
        tests_count = _replace_dir(tests_src, game_dir / "tests")
        print(f"  game/tests/: {tests_count} files")
    for filename in ("Cargo.toml", "Cargo.lock"):
        src_file = release / "source" / filename
        if src_file.is_file():
            (game_dir / filename).parent.mkdir(parents=True, exist_ok=True)
            shutil.copy2(src_file, game_dir / filename)
            print(f"  game/{filename}: 1 file")


def load_game(
    slug: str,
    *,
    releases_root: Path = DEFAULT_RELEASES_ROOT,
    game_config: Path = DEFAULT_GAME_CONFIG,
    game_dir: Path = DEFAULT_GAME_DIR,
    vault_dir: Path = DEFAULT_VAULT_DIR,
    system_map_path: Path = DEFAULT_SYSTEM_MAP,
    force: bool = False,
) -> int:
    """Restore a release snapshot into the working tree."""
    release = releases_root / slug
    if not release.is_dir():
        sys.stderr.write(
            f"load_game: {release} not found. Available: {list_releases(releases_root)}\n"
        )
        return 1
    if not (release / "source" / "Cargo.toml").is_file():
        sys.stderr.write(f"load_game: {release} is missing source/Cargo.toml\n")
        return 1
    if not _check_dirty_or_force(game_dir, vault_dir, game_config, force):
        return 1

    print(f"load_game: restoring {slug!r} into {_REPO_ROOT}")
    _restore_game_dir(release, game_dir)

    vault_count = _replace_dir(release / "vault", vault_dir)
    print(f"  vault/: {vault_count} files")

    cfg_src = release / "game-config.json"
    if cfg_src.is_file():
        shutil.copy2(cfg_src, game_config)
        print("  game-config.json: 1 file")

    sm_src = release / "system_map.yaml"
    if sm_src.is_file():
        system_map_path.parent.mkdir(parents=True, exist_ok=True)
        shutil.copy2(sm_src, system_map_path)
        print("  build/system_map.yaml: 1 file")

    print(
        "load_game: done. Next: `cargo build --manifest-path game/Cargo.toml` "
        "to rebuild against the loaded source."
    )
    return 0


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.add_argument("slug", nargs="?", default=None, help="release slug to restore")
    parser.add_argument("--releases-root", type=Path, default=DEFAULT_RELEASES_ROOT)
    parser.add_argument("--game-config", type=Path, default=DEFAULT_GAME_CONFIG)
    parser.add_argument("--game-dir", type=Path, default=DEFAULT_GAME_DIR)
    parser.add_argument("--vault-dir", type=Path, default=DEFAULT_VAULT_DIR)
    parser.add_argument("--system-map", type=Path, default=DEFAULT_SYSTEM_MAP)
    parser.add_argument(
        "--force",
        action="store_true",
        help="overwrite an existing game/, vault/, or game-config.json",
    )
    args = parser.parse_args(argv)

    if not args.slug:
        slugs = list_releases(args.releases_root)
        if not slugs:
            print(f"No releases under {args.releases_root}. Run `scripts/ship.py` first.")
            return 0
        print("Available releases:")
        for slug in slugs:
            print(f"  {slug}")
        print("Pick one: python scripts/load_game.py <slug>")
        return 0

    return load_game(
        args.slug,
        releases_root=args.releases_root,
        game_config=args.game_config,
        game_dir=args.game_dir,
        vault_dir=args.vault_dir,
        system_map_path=args.system_map,
        force=args.force,
    )


if __name__ == "__main__":
    sys.exit(main())
