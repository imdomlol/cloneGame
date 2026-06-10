"""Snapshot the current game into ``releases/<slug>/`` so it survives wipes.

The pipeline treats ``game/``, ``vault/``, ``build/``, and ``game-config.json``
as one in-progress working set. Switching target wikis (or just iterating)
wipes those — the previous game's source, notes, and config are then gone.
This script snapshots all of it into a per-game subdirectory under
``releases/`` so a finished game stays preserved.

What gets snapshotted (per release):

- ``releases/<slug>/source/`` — the Cargo project: ``Cargo.toml``,
  ``Cargo.lock``, ``src/`` (all generated leaves + foundation), ``tests/``.
  Anyone with the Rust toolchain can ``cargo build --release`` this and run
  the game, no pipeline involvement.
- ``releases/<slug>/vault/`` — Phase 1 output. The notes that produced this
  game. Lets you replay Phase 2 if you ever want to regen with a newer
  prompt or engine.
- ``releases/<slug>/game-config.json`` — Phase 0 output. Taxonomy, schemas,
  engine choice, codegen flags, system list. Required for any replay.
- ``releases/<slug>/system_map.yaml`` — the implementation ledger snapshot.
  Lets ``--from-vault`` know what was implemented (idempotent re-runs).
- ``releases/<slug>/binary/<platform>/`` — compiled release binary, plus
  the wgpu / winit DLLs Cargo bakes alongside it. Gitignored (too large for
  git); use GitHub Releases or a separate storage tier for distribution.
- ``releases/<slug>/README.md`` — generated description of what this is and
  how to run it.

Slug derivation: ``game-config.json -> game.name``, lowercased, spaces and
non-``[a-z0-9_-]`` characters dropped, collapsed to single ``-``. Engine- and
game-agnostic by construction: the script never assumes Bevy / Rust / any
specific paths beyond what ``chosen_engine`` declares.

Usage:

    python scripts/ship.py                   # build + snapshot, refuse if exists
    python scripts/ship.py --force           # overwrite an existing release
    python scripts/ship.py --skip-binary     # snapshot source only, no cargo build
    python scripts/ship.py --slug my-name    # override the auto-derived slug
"""

from __future__ import annotations

import argparse
import datetime as dt
import json
import platform
import re
import shutil
import subprocess
import sys
from pathlib import Path

_REPO_ROOT = Path(__file__).resolve().parent.parent
DEFAULT_GAME_CONFIG = _REPO_ROOT / "game-config.json"
DEFAULT_GAME_DIR = _REPO_ROOT / "game"
DEFAULT_VAULT_DIR = _REPO_ROOT / "vault"
DEFAULT_SYSTEM_MAP = _REPO_ROOT / "build" / "system_map.yaml"
DEFAULT_RELEASES_ROOT = _REPO_ROOT / "releases"

# Files / dirs the snapshot intentionally skips inside game/.
# - target/ is the cargo build cache (gigabytes; regenerable)
# - .git artifacts would only appear if game/ has a sub-repo (it doesn't, but defensive)
_GAME_SKIP_NAMES = {"target", ".git"}


def derive_slug(name: str) -> str:
    """Derive a filesystem-safe slug from a free-form game name.

    Lowercases, drops apostrophes and other punctuation, replaces spaces with
    hyphens, collapses runs of hyphens, strips leading / trailing hyphens.
    Engine- and game-agnostic; the only assumption is that release directories
    live on a filesystem that accepts ``[a-z0-9_-]+``.
    """
    s = name.lower()
    s = re.sub(r"[^a-z0-9]+", "-", s)
    s = re.sub(r"-+", "-", s)
    return s.strip("-") or "game"


def platform_dir_name() -> str:
    """Return a short, stable platform tag for binary subdirectories.

    Just enough resolution to distinguish Windows / macOS / Linux x86_64;
    finer detail (e.g. macOS arm64 vs x86_64) can come later if it ever
    matters. Used only for the ``binary/`` subpath; everything else in a
    release is portable across platforms.
    """
    sys_name = {"Windows": "windows", "Darwin": "macos", "Linux": "linux"}.get(
        platform.system(), platform.system().lower()
    )
    return f"{sys_name}-{platform.machine().lower() or 'unknown'}"


def copy_tree(src: Path, dst: Path, skip: set[str] | None = None) -> int:
    """Copy ``src`` into ``dst`` recursively; return the number of files written.

    Used in lieu of ``shutil.copytree`` because the destination may already
    exist (operator passed ``--force``) and we want to overwrite individual
    files without nuking the whole directory tree. Skips any directory entry
    whose name is in ``skip`` — the caller uses this to avoid copying
    ``target/`` or other regenerable build output.
    """
    skip_names = skip or set()
    if not src.exists():
        return 0
    dst.mkdir(parents=True, exist_ok=True)
    written = 0
    for entry in src.iterdir():
        if entry.name in skip_names:
            continue
        target = dst / entry.name
        if entry.is_dir():
            written += copy_tree(entry, target, skip_names)
        else:
            target.parent.mkdir(parents=True, exist_ok=True)
            shutil.copy2(entry, target)
            written += 1
    return written


def build_release_binary(game_dir: Path) -> Path | None:
    """Run ``cargo build --release`` and return the binary's path, or None on failure.

    The release binary may be named anything ``Cargo.toml`` declares as
    ``[[bin]] name``; we discover it by scanning ``target/release/`` for any
    file with the executable bit / ``.exe`` suffix that is NOT a build
    artifact (``.d``, ``.pdb``, ``deps/``). If we find more than one
    candidate, we pick the one whose name matches the package name.
    """
    cargo = shutil.which("cargo")
    if cargo is None:
        sys.stderr.write("ship: cargo not found on PATH; pass --skip-binary\n")
        return None
    proc = subprocess.run(
        [cargo, "build", "--release", "--manifest-path", str(game_dir / "Cargo.toml")],
        cwd=str(game_dir),
        capture_output=True,
        text=True,
        check=False,
    )
    if proc.returncode != 0:
        sys.stderr.write("ship: cargo build --release failed:\n")
        sys.stderr.write(proc.stderr[-2000:])
        return None

    release_dir = game_dir / "target" / "release"
    if not release_dir.is_dir():
        sys.stderr.write(f"ship: {release_dir} missing after build\n")
        return None

    # Find the bin: skip .d / .pdb / deps subdirs / hashed intermediates.
    package_name = _read_package_name(game_dir / "Cargo.toml")
    candidates: list[Path] = []
    for entry in release_dir.iterdir():
        if entry.is_file() and _looks_like_executable(entry):
            candidates.append(entry)
    if not candidates:
        sys.stderr.write(f"ship: no release binary found under {release_dir}\n")
        return None
    if package_name:
        for c in candidates:
            if c.stem == package_name:
                return c
    return candidates[0]


def _looks_like_executable(path: Path) -> bool:
    name = path.name.lower()
    if name.endswith((".d", ".pdb", ".rlib", ".rmeta", ".o", ".obj", ".lib", ".so", ".dylib")):
        return False
    if name.endswith(".exe"):
        return True
    # On POSIX, executables have no extension but are mode +x.
    return "." not in name


def _read_package_name(cargo_toml: Path) -> str | None:
    """Read ``[package].name`` from a Cargo.toml without dragging in tomllib's
    behavior differences across Python versions.

    This is intentionally lenient: any ``name = "..."`` line inside the file
    counts; only ``[package]`` blocks should have it in well-formed Cargo
    manifests, so the first match is good enough for picking the binary.
    """
    if not cargo_toml.exists():
        return None
    in_package = False
    for raw in cargo_toml.read_text(encoding="utf-8").splitlines():
        line = raw.strip()
        if line.startswith("["):
            in_package = line == "[package]"
            continue
        if in_package and line.startswith("name"):
            m = re.match(r'name\s*=\s*"([^"]+)"', line)
            if m:
                return m.group(1)
    return None


def write_release_readme(
    target: Path,
    game_name: str,
    engine_name: str,
    binary_path: Path | None,
    plugin_count: int,
    system_count: int,
    shipped_at: str,
) -> None:
    """Generate a human-readable description for the release directory."""
    if binary_path:
        run_section = (
            f"1. **Run the bundled binary** — go to "
            f"`binary/{platform_dir_name()}/{binary_path.name}` and execute it. "
            "The OS may show a security warning the first time (it's unsigned).\n"
            "2. **Build from source**"
        )
    else:
        run_section = (
            "1. **Build from source**"
        )
    body = f"""# {game_name}

Shipped: {shipped_at}.
Generated by the cloneGame pipeline. Engine: **{engine_name}**.
Plugin count at ship time: {plugin_count} entity + {system_count} system plugins.

## What this is

A snapshot of the game crate at a known-good moment, plus the source data
that produced it. Ways to use it:

{run_section} — `cargo build --release --manifest-path source/Cargo.toml`,
   then run `source/target/release/<binary>`.
2. **Replay the pipeline** — copy `game-config.json` and `vault/` back to the
   repo root, then run Phase 2 to regenerate. Useful if a newer codegen
   prompt or engine version is worth picking up.

## Files

- `source/` — full Cargo project. `cargo build --release` here produces the binary.
- `vault/` — Phase 1 output: the structured notes that fed Phase 2 codegen.
- `game-config.json` — Phase 0 output: taxonomy + schemas + engine + system list.
- `system_map.yaml` — Phase 2 ledger: what was implemented at ship time.
- `binary/<platform>/` — compiled binary for the OS / arch the ship script ran on.
"""
    target.write_text(body, encoding="utf-8")


def update_releases_gitignore(releases_root: Path) -> None:
    """Ensure `releases/*/binary/` is gitignored.

    Source + vault + config snapshots are tiny and belong in git; compiled
    binaries are megabytes and don't. A single ``releases/.gitignore`` keeps
    git clean without polluting the user's repo root .gitignore.
    """
    releases_root.mkdir(parents=True, exist_ok=True)
    ignore = releases_root / ".gitignore"
    desired = (
        "# Compiled binaries are too large for git. "
        "Source + vault + config still commit.\n"
        "*/binary/\n"
    )
    if ignore.exists() and ignore.read_text(encoding="utf-8") == desired:
        return
    ignore.write_text(desired, encoding="utf-8")


def ship(
    *,
    game_config: Path = DEFAULT_GAME_CONFIG,
    game_dir: Path = DEFAULT_GAME_DIR,
    vault_dir: Path = DEFAULT_VAULT_DIR,
    system_map_path: Path = DEFAULT_SYSTEM_MAP,
    releases_root: Path = DEFAULT_RELEASES_ROOT,
    slug_override: str | None = None,
    force: bool = False,
    skip_binary: bool = False,
) -> int:
    """End-to-end ship: snapshot + binary build + README. Returns CLI exit code."""
    if not game_config.exists():
        sys.stderr.write(f"ship: {game_config} missing\n")
        return 1
    config = json.loads(game_config.read_text(encoding="utf-8"))
    game_name = (config.get("game") or {}).get("name") or "Unnamed Game"
    engine_name = (config.get("chosen_engine") or {}).get("name") or "(unset)"
    slug = slug_override or derive_slug(game_name)

    target = releases_root / slug
    if target.exists() and not force:
        sys.stderr.write(
            f"ship: {target} already exists. Re-run with --force to overwrite "
            "or pass --slug <other-name>.\n"
        )
        return 1

    print(f"ship: snapshotting {game_name!r} -> {target}")
    update_releases_gitignore(releases_root)
    target.mkdir(parents=True, exist_ok=True)

    # source/ — the full Cargo project sans target/
    src_count = copy_tree(game_dir, target / "source", skip=_GAME_SKIP_NAMES)
    print(f"  source: {src_count} files")

    # vault/ — Phase 1 output
    vault_count = copy_tree(vault_dir, target / "vault")
    print(f"  vault: {vault_count} files")

    # game-config.json + system_map.yaml
    shutil.copy2(game_config, target / "game-config.json")
    print("  game-config.json: 1 file")

    if system_map_path.exists():
        shutil.copy2(system_map_path, target / "system_map.yaml")
        print("  system_map.yaml: 1 file")

    plugin_count, system_count = _count_implementations(system_map_path)

    # binary/<platform>/<name>
    binary_path: Path | None = None
    if not skip_binary:
        print("  binary: cargo build --release ... (this takes a minute on a cold build)")
        binary_path = build_release_binary(game_dir)
        if binary_path is not None:
            platform_subdir = target / "binary" / platform_dir_name()
            platform_subdir.mkdir(parents=True, exist_ok=True)
            shutil.copy2(binary_path, platform_subdir / binary_path.name)
            print(f"  binary: {binary_path.name} -> {platform_subdir.relative_to(target)}/")
        else:
            print("  binary: SKIPPED (build failed; see stderr)")

    shipped_at = dt.datetime.now(dt.UTC).strftime("%Y-%m-%d %H:%M UTC")
    write_release_readme(
        target / "README.md",
        game_name=game_name,
        engine_name=engine_name,
        binary_path=binary_path,
        plugin_count=plugin_count,
        system_count=system_count,
        shipped_at=shipped_at,
    )
    print("  README.md: 1 file")
    try:
        rel = target.relative_to(_REPO_ROOT)
    except ValueError:
        rel = target
    print(f"ship: done -> {rel}")
    return 0


def _count_implementations(system_map_path: Path) -> tuple[int, int]:
    """Count entity-leaf and system-leaf plugins in the system_map ledger.

    Heuristic: any `file:` entry whose path starts with `src/system/` is a
    Phase 3 system; everything else with a `file:` is a Phase 2 entity leaf.
    Returns ``(entity_count, system_count)``. Used for the release README;
    a small inaccuracy here is fine — the ledger remains the source of truth.
    """
    if not system_map_path.exists():
        return 0, 0
    entity = 0
    system = 0
    for raw in system_map_path.read_text(encoding="utf-8").splitlines():
        line = raw.strip()
        if not line.startswith("file:"):
            continue
        if "/system/" in line or "src/system/" in line:
            system += 1
        else:
            entity += 1
    return entity, system


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.add_argument("--game-config", type=Path, default=DEFAULT_GAME_CONFIG)
    parser.add_argument("--game-dir", type=Path, default=DEFAULT_GAME_DIR)
    parser.add_argument("--vault-dir", type=Path, default=DEFAULT_VAULT_DIR)
    parser.add_argument("--system-map", type=Path, default=DEFAULT_SYSTEM_MAP)
    parser.add_argument("--releases-root", type=Path, default=DEFAULT_RELEASES_ROOT)
    parser.add_argument("--slug", default=None, help="override the auto-derived release slug")
    parser.add_argument(
        "--force", action="store_true", help="overwrite an existing release directory"
    )
    parser.add_argument(
        "--skip-binary",
        action="store_true",
        help="snapshot source + vault + config only; skip cargo build --release",
    )
    args = parser.parse_args(argv)
    return ship(
        game_config=args.game_config,
        game_dir=args.game_dir,
        vault_dir=args.vault_dir,
        system_map_path=args.system_map,
        releases_root=args.releases_root,
        slug_override=args.slug,
        force=args.force,
        skip_binary=args.skip_binary,
    )


if __name__ == "__main__":
    sys.exit(main())
