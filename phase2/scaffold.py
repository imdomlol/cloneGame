"""Phase 2 prelude: materialize per-engine foundation files into ``game/``.

The codegen loop (see ``loop_driver.py``) writes only per-kind leaf modules.
Every game also needs engine scaffolding that the LLM should not author per-game:

- ``Cargo.toml`` / `pyproject.toml` / `Project.godot`: crate or project manifest.
- Foundation source (e.g. Bevy ``src/sim.rs``): types and resources the per-kind
  determinism rules name (``Health``, ``UnitStats``, ``SimChecksumState``,
  ``SimEventsPlugin``, the fixed-point RNG ...).
- ``main.rs`` / engine entrypoint: window, camera, sim-tick loop. Engine-specific,
  not game-specific — Bevy's window setup looks the same regardless of game.
- Smoke-test boilerplate: same shape across games on the same engine.
- ``app_plugins.rs`` stub: empty ``add_all`` until ``entrypoint.regenerate_aggregator``
  produces the real one on the first loop turn.

These files live as a per-engine tree under ``prompts/engine_scaffold/<engine>/``
mirroring the target layout (``Cargo.toml``, ``src/lib.rs``, ``src/main.rs``,
``src/sim.rs``, ``src/app_plugins.rs``, ``tests/app_smoke.rs``). The renderer
copies the tree into ``game/`` byte-exact. No string substitution: per-engine
files are fully literal, since they encode engine architecture (the determinism
contract) not per-game data. Per-game variation lives elsewhere — in
``game-config.json`` and the vault.

Safety: by default the renderer **never overwrites a file whose current content
differs from the scaffold** — that file may carry a hand-edit the operator
wants to keep (e.g. a tuned window resolution). ``--force`` overrides. Files
that don't exist are always written. Files that match the scaffold byte-exact
are skipped (no mtime churn, keeps Cargo's incremental cache hot).

Engine dispatch is by ``chosen_engine.name`` lowercased — matches the same key
used by ``prompts/engine_determinism/<engine>.md``. Adding a new engine is a
new directory under ``prompts/engine_scaffold/``, not a code change here.
"""

from __future__ import annotations

import argparse
import json
import sys
from dataclasses import dataclass
from pathlib import Path

_REPO_ROOT = Path(__file__).resolve().parent.parent
DEFAULT_GAME_CONFIG = _REPO_ROOT / "game-config.json"
DEFAULT_GAME_DIR = _REPO_ROOT / "game"
DEFAULT_SCAFFOLD_ROOT = _REPO_ROOT / "prompts" / "engine_scaffold"


@dataclass
class RenderResult:
    """One file's outcome from a scaffold pass."""

    target: Path
    status: str  # "written" | "skipped_identical" | "skipped_hand_edit" | "would_write"


def load_engine_name(game_config_path: Path) -> str | None:
    """Read ``chosen_engine.name`` from game-config.json.

    Returns the lowercased engine name (matching the scaffold-dir naming
    convention), or ``None`` when the config or block is absent. Lowercase
    keeps the dispatch consistent with ``prompts/engine_determinism/<engine>.md``.
    """
    if not game_config_path.exists():
        return None
    config = json.loads(game_config_path.read_text(encoding="utf-8"))
    name = (config.get("chosen_engine") or {}).get("name")
    return name.lower() if isinstance(name, str) else None


def find_scaffold_dir(engine_name: str, scaffold_root: Path = DEFAULT_SCAFFOLD_ROOT) -> Path | None:
    """Locate the scaffold tree for an engine; return None when missing.

    The caller can then warn and continue without scaffolding (which makes the
    operator's mistake — engine name not yet supported — visible without
    crashing). The scaffold root itself is allowed to be absent (e.g. tests).
    """
    if not scaffold_root.is_dir():
        return None
    candidate = scaffold_root / engine_name
    return candidate if candidate.is_dir() else None


def _iter_scaffold_files(scaffold_dir: Path):
    """Yield every regular file under the scaffold tree, in deterministic order."""
    for path in sorted(scaffold_dir.rglob("*")):
        if path.is_file():
            yield path


def render_scaffold(
    scaffold_dir: Path,
    game_dir: Path,
    force: bool = False,
    dry_run: bool = False,
) -> list[RenderResult]:
    """Mirror every file under ``scaffold_dir`` into ``game_dir``.

    Returns one ``RenderResult`` per scaffold file describing what happened:

    - ``written``: file was created or overwritten with the scaffold bytes.
    - ``skipped_identical``: target already matched the scaffold byte-exact.
    - ``skipped_hand_edit``: target existed with different content and
      ``force`` is False; left untouched. Surface these as a warning so the
      operator notices a possibly intentional divergence.
    - ``would_write``: dry-run; nothing was written.

    Idempotent under repeat runs; safe under partial scaffolds; preserves
    hand-edited files when ``force`` is False.
    """
    results: list[RenderResult] = []
    for src in _iter_scaffold_files(scaffold_dir):
        rel = src.relative_to(scaffold_dir)
        dst = game_dir / rel
        scaffold_bytes = src.read_bytes()

        if dst.exists():
            current = dst.read_bytes()
            if current == scaffold_bytes:
                results.append(RenderResult(dst, "skipped_identical"))
                continue
            if not force:
                results.append(RenderResult(dst, "skipped_hand_edit"))
                continue

        if dry_run:
            results.append(RenderResult(dst, "would_write"))
            continue

        dst.parent.mkdir(parents=True, exist_ok=True)
        dst.write_bytes(scaffold_bytes)
        results.append(RenderResult(dst, "written"))
    return results


def render(
    game_config_path: Path = DEFAULT_GAME_CONFIG,
    game_dir: Path = DEFAULT_GAME_DIR,
    scaffold_root: Path = DEFAULT_SCAFFOLD_ROOT,
    force: bool = False,
    dry_run: bool = False,
) -> tuple[str | None, list[RenderResult]]:
    """High-level entrypoint: resolve engine, render, return ``(engine, results)``.

    ``engine`` is ``None`` when ``chosen_engine`` is unset or has no matching
    scaffold dir; ``results`` is then ``[]``. Used both by the CLI and by other
    pipeline drivers that want to scaffold programmatically.
    """
    engine = load_engine_name(game_config_path)
    if engine is None:
        return None, []
    scaffold_dir = find_scaffold_dir(engine, scaffold_root)
    if scaffold_dir is None:
        return engine, []
    return engine, render_scaffold(scaffold_dir, game_dir, force=force, dry_run=dry_run)


def main(argv: list[str] | None = None) -> int:
    """CLI: ``python phase2/scaffold.py [--force] [--dry-run]``.

    Exit status: 0 on success, 1 when the config doesn't name a supported
    engine, 2 when hand-edits were preserved (informational — the run isn't
    blocked, but the operator should review).
    """
    parser = argparse.ArgumentParser(
        description="Render per-engine foundation files into game/ before the codegen loop."
    )
    parser.add_argument("--game-config", type=Path, default=DEFAULT_GAME_CONFIG)
    parser.add_argument("--game-dir", type=Path, default=DEFAULT_GAME_DIR)
    parser.add_argument("--scaffold-root", type=Path, default=DEFAULT_SCAFFOLD_ROOT)
    parser.add_argument(
        "--force",
        action="store_true",
        help="overwrite files even if they have been hand-edited",
    )
    parser.add_argument(
        "--dry-run",
        action="store_true",
        help="print what would change without writing",
    )
    args = parser.parse_args(argv)

    engine, results = render(
        game_config_path=args.game_config,
        game_dir=args.game_dir,
        scaffold_root=args.scaffold_root,
        force=args.force,
        dry_run=args.dry_run,
    )

    if engine is None:
        print(
            "error: chosen_engine.name missing from game-config.json — "
            "run phase0 first or pick an engine manually.",
            file=sys.stderr,
        )
        return 1
    if not results:
        print(
            f"error: no scaffold for engine '{engine}' "
            f"(expected dir under {args.scaffold_root}).",
            file=sys.stderr,
        )
        return 1

    hand_edited = 0
    for r in results:
        if r.target.is_relative_to(args.game_dir):
            rel = r.target.relative_to(args.game_dir)
        else:
            rel = r.target
        print(f"{r.status:>20}  {rel}")
        if r.status == "skipped_hand_edit":
            hand_edited += 1

    print(
        f"\nengine: {engine}  "
        f"written: {sum(1 for r in results if r.status == 'written')}  "
        f"identical: {sum(1 for r in results if r.status == 'skipped_identical')}  "
        f"hand-edited (skipped): {hand_edited}  "
        f"would-write: {sum(1 for r in results if r.status == 'would_write')}"
    )
    if hand_edited and not args.force:
        print(
            "\nnote: some files differ from the scaffold and were left alone. "
            "Re-run with --force to overwrite.",
            file=sys.stderr,
        )
        return 2
    return 0


if __name__ == "__main__":
    sys.exit(main())
