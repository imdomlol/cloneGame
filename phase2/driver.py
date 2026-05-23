"""Single-shot Phase 2 codegen driver: one task in, one file + system_map update out.

Glues ``phase2/codegen.py`` and ``phase2/system_map.py`` so a single CLI call
turns a task string into:

1. An Anthropic codegen call (via ``codegen.generate``).
2. A post-check of the generated text's ``// Sources:`` header against the
   retrieval bundle's allowed paths.
3. A file written to ``--output`` *only if* the header validates.
4. A ``system_map.yaml`` mutation: ``record_implementation`` on success,
   ``record_pending`` (blocked on offending paths or a missing header) on
   failure.
5. A persisted ``build/system_map.yaml`` with the latest baseline hash.

One-shot per invocation. Loop it from a shell script or task runner; this
module is deliberately not a daemon so each turn is auditable.

CLI::

    python phase2/driver.py ranger src/units/ranger.rs "implement the ranger unit"
    python phase2/driver.py --dry-run x src/x.rs "..."     # assemble prompt only
"""

from __future__ import annotations

import argparse
import hashlib
import sys
from pathlib import Path
from typing import Any

_REPO_ROOT = Path(__file__).resolve().parent.parent
if str(_REPO_ROOT / "phase2") not in sys.path:
    sys.path.insert(0, str(_REPO_ROOT / "phase2"))

import codegen  # noqa: E402
import system_map  # noqa: E402


def _sha256(text: str) -> str:
    return hashlib.sha256(text.encode("utf-8")).hexdigest()


def _baseline_hash(baseline_path: Path) -> str:
    if not baseline_path.exists():
        return ""
    return _sha256(baseline_path.read_text(encoding="utf-8"))


def run_turn(
    note_id: str,
    output_file: Path,
    task: str,
    *,
    llm_mode: str = codegen.DEFAULT_LLM_MODE,
    model: str | None = None,
    generate: Any = codegen.generate,
    state_path: Path = system_map.DEFAULT_PATH,
    baseline_path: Path = codegen.DEFAULT_BASELINE_PATH,
    dry_run: bool = False,
) -> dict[str, Any]:
    """Execute one codegen turn end-to-end. Returns the codegen summary dict.

    On a dry run, no backend call is made, no file is written, and the
    system map is left untouched. The returned dict carries the assembled
    prompt so the caller can inspect token counts before spending.
    """
    summary = generate(
        task,
        llm_mode=llm_mode,
        model=model,
        baseline_path=baseline_path,
        dry_run=dry_run,
    )
    if dry_run:
        return summary

    state = system_map.load_state(state_path)
    system_map.set_baseline_hash(state, _baseline_hash(baseline_path))

    text: str = summary["response_text"]
    ok: bool = summary["sources_header_ok"]
    offending: list[str] = summary["sources_header_offending"]

    if ok:
        output_file.parent.mkdir(parents=True, exist_ok=True)
        output_file.write_text(text, encoding="utf-8")
        sources = codegen.extract_source_paths(text)
        system_map.record_implementation(
            state,
            note_id=note_id,
            file=str(output_file),
            sha=_sha256(text),
            verified_against=",".join(sources),
        )
        summary["written_to"] = str(output_file)
    else:
        blocked_by = offending or ["missing_sources_header"]
        system_map.record_pending(state, note_id, blocked_by)
        summary["written_to"] = None

    state = system_map.cap_tokens(state)
    system_map.save_state(state_path, state)
    return summary


def _parse_args(argv: list[str] | None = None) -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.add_argument("note_id", help="vault id this generated file derives from")
    parser.add_argument("output", type=Path, help="destination path for generated source")
    parser.add_argument("task", help="task string fed to the codegen LLM")
    parser.add_argument(
        "--llm-mode",
        default=codegen.DEFAULT_LLM_MODE,
        choices=codegen.LLM_MODES,
    )
    parser.add_argument("--model", default=None)
    parser.add_argument("--state-path", type=Path, default=system_map.DEFAULT_PATH)
    parser.add_argument("--baseline", type=Path, default=codegen.DEFAULT_BASELINE_PATH)
    parser.add_argument(
        "--dry-run",
        action="store_true",
        help="assemble prompt + print summary, skip backend call and system_map writes",
    )
    return parser.parse_args(argv)


def main(argv: list[str] | None = None) -> int:
    args = _parse_args(argv)
    summary = run_turn(
        args.note_id,
        args.output,
        args.task,
        llm_mode=args.llm_mode,
        model=args.model,
        state_path=args.state_path,
        baseline_path=args.baseline,
        dry_run=args.dry_run,
    )
    if args.dry_run:
        print(summary["prompt"])
        print(
            f"\n# dry-run: baseline={summary['engine_baseline_tokens']} tok, "
            f"user={summary['user_message_tokens']} tok, "
            f"included={len(summary['included_vault_ids'])} files",
            file=sys.stderr,
        )
        return 0

    if summary.get("written_to"):
        print(f"wrote {summary['written_to']}")
        return 0
    print(
        f"warn: codegen output failed source-header validation "
        f"(offending: {summary['sources_header_offending'] or '<missing>'}); "
        f"recorded {args.note_id} as pending",
        file=sys.stderr,
    )
    return 1


if __name__ == "__main__":
    sys.exit(main())
