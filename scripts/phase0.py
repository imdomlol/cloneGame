#!/usr/bin/env python3
"""Phase 0 orchestrator: taxonomy discovery CLI entry point.

Parses args, then dispatches the three-stage pipeline: fetch the wiki's
category graph (`phase0_fetch`), ask an LLM to propose kinds, per-kind
frontmatter schemas, and engine candidates (`phase0_analyze`), and write the
proposal plus two-stage human confirmation that flips `human_approved`
(`phase0_write`). `--wiki-url` overrides the configured base URL; `--dry-run`
prints the proposed JSON and skips the write/confirm step.
"""

from __future__ import annotations

import argparse
import json
import os
import sys
from typing import Any


def _add_scripts_to_sys_path() -> None:
    scripts_dir = os.path.dirname(os.path.abspath(__file__))
    if scripts_dir not in sys.path:
        sys.path.insert(0, scripts_dir)


def _read_json(path: str) -> dict[str, Any]:
    with open(path, encoding="utf-8") as f:
        data = json.load(f)
    if not isinstance(data, dict):
        raise ValueError(f"{path} must contain a JSON object at the top level.")
    return data


def _parse_args(argv: list[str]) -> argparse.Namespace:
    _add_scripts_to_sys_path()
    from model_config import default_llm_mode

    parser = argparse.ArgumentParser(description="Phase 0: Taxonomy discovery")
    parser.add_argument(
        "--wiki-url",
        dest="wiki_url",
        help="Override the wiki base URL (default: game-config.json -> game.wiki_base_url)",
    )
    parser.add_argument(
        "--min-members",
        dest="min_members",
        type=int,
        default=3,
        help="Minimum category member count (default: 3)",
    )
    parser.add_argument(
        "--dry-run",
        action="store_true",
        help="Run fetch + analyze, print proposed JSON to stdout, skip write/confirmation",
    )
    parser.add_argument(
        "--model",
        default=None,
        help=("Model for taxonomy analysis. Defaults come from pipeline.config.toml."),
    )
    parser.add_argument(
        "--llm-mode",
        choices=("claude", "codex"),
        default=default_llm_mode("phase0"),
        help="LLM CLI to use for taxonomy analysis (default: pipeline.config.toml)",
    )
    return parser.parse_args(argv)


def _sample_pages_by_category(
    fetch_module: Any,
    wiki_url: str,
    categories: list[dict[str, Any]],
) -> dict[str, list[str]]:
    # Pull real wikitext for the first 2 pages per category so the schema
    # proposer sees actual frontmatter-shaped content, not just page titles.
    # Falls back to titles-only if every wikitext fetch fails (offline / API down).
    samples = fetch_module.fetch_sample_pages_by_category(wiki_url, categories)
    if any(snippets for snippets in samples.values()):
        return samples
    fallback: dict[str, list[str]] = {}
    for cat in categories:
        name = cat.get("name")
        members = cat.get("members") or []
        if isinstance(name, str) and isinstance(members, list):
            fallback[name] = [str(m) for m in members[:2] if isinstance(m, str)]
    return fallback


def _propose_schemas_and_engines(
    analyze_module: Any,
    kinds: dict[str, Any],
    mapped_categories: list[dict[str, Any]],
    sample_pages: dict[str, list[str]],
    kwargs: dict[str, Any],
    schema_kinds: dict[str, Any] | None = None,
    schema_categories: list[dict[str, Any]] | None = None,
) -> tuple[dict[str, dict[str, Any]], list[dict[str, Any]]]:
    kinds_for_schema = kinds if schema_kinds is None else schema_kinds
    categories_for_schema = mapped_categories if schema_categories is None else schema_categories
    if kinds_for_schema:
        print("[Phase 0] Proposing per-kind frontmatter schemas...")
        schemas = analyze_module.propose_frontmatter_schemas(
            kinds_for_schema, categories_for_schema, sample_pages, **kwargs
        )
    else:
        print("[Phase 0] Reusing existing per-kind frontmatter schemas.")
        schemas = {}
    print("[Phase 0] Proposing target-engine candidates...")
    engines = analyze_module.propose_engine_candidates(kinds, mapped_categories, **kwargs)
    print("[Phase 0] Classifying kinds as code-vs-data (codegen flags)...")
    codegen_flags = analyze_module.propose_codegen_flags(
        kinds, mapped_categories, sample_pages, **kwargs
    )
    # Phase 3 systems: per-game gameplay-loop plugins (state machine, HUD,
    # input, combat, etc.) generated AFTER per-entity leaves. Marks the
    # `chosen_engine.systems` block consumed by
    # `phase2/loop_driver.py --from-systems`.
    print("[Phase 0] Proposing gameplay systems (Phase 3 goals)...")
    # Apply codegen flags to kinds before proposing systems so the proposer
    # only depends_on code-kinds (no `depends_on: ["version_pages"]`).
    flagged_kinds = {}
    for name, data in kinds.items():
        d = dict(data) if isinstance(data, dict) else {}
        if name in codegen_flags:
            d["codegen"] = codegen_flags[name]
        flagged_kinds[name] = d
    systems = analyze_module.propose_gameplay_systems(
        flagged_kinds, sample_pages, **kwargs
    )
    return schemas, engines, codegen_flags, systems


def _kinds_missing_frontmatter_schema(
    current_config: dict[str, Any], kinds: dict[str, Any]
) -> dict[str, Any]:
    current_kinds = current_config.get("kinds", {})
    if not isinstance(current_kinds, dict):
        current_kinds = {}
    missing: dict[str, Any] = {}
    for kind, kind_data in kinds.items():
        current_kind_data = current_kinds.get(kind)
        current_schema = (
            current_kind_data.get("frontmatter_schema")
            if isinstance(current_kind_data, dict)
            else None
        )
        if not isinstance(current_schema, dict) or not current_schema:
            missing[kind] = kind_data
    return missing


def _categories_for_kinds(
    categories: list[dict[str, Any]], kinds: dict[str, Any]
) -> list[dict[str, Any]]:
    return [cat for cat in categories if cat.get("kind") in kinds]


def _merge_proposals(
    current_config: dict[str, Any],
    kinds: dict[str, Any],
    schemas: dict[str, dict[str, Any]],
    engines: list[dict[str, Any]],
    mapped_categories: list[dict[str, Any]],
    dropped_categories: list[dict[str, Any]] | None = None,
    codegen_flags: dict[str, bool] | None = None,
    systems: list[dict[str, Any]] | None = None,
) -> dict[str, Any]:
    enriched_kinds: dict[str, Any] = {}
    current_kinds = current_config.get("kinds", {})
    if not isinstance(current_kinds, dict):
        current_kinds = {}
    for kind_key, kind_data in kinds.items():
        previous_kind_data = current_kinds.get(kind_key)
        previous_schema = (
            previous_kind_data.get("frontmatter_schema")
            if isinstance(previous_kind_data, dict)
            else None
        )
        previous_codegen = (
            previous_kind_data.get("codegen")
            if isinstance(previous_kind_data, dict)
            else None
        )
        entry = dict(kind_data)
        if isinstance(previous_schema, dict) and previous_schema:
            entry["frontmatter_schema"] = previous_schema
        elif kind_key in schemas:
            entry["frontmatter_schema"] = schemas[kind_key]
        # codegen flag precedence: existing hand-set value wins over the
        # auto-classification (operators may toggle a kind by hand and not
        # want Phase 0 to flip it back); fall back to the classifier output.
        if isinstance(previous_codegen, bool):
            entry["codegen"] = previous_codegen
        elif codegen_flags and kind_key in codegen_flags:
            entry["codegen"] = codegen_flags[kind_key]
        enriched_kinds[kind_key] = entry
    out: dict[str, Any] = {
        "kinds": enriched_kinds,
        "categories": mapped_categories,
        "dropped_categories": dropped_categories or [],
        "engine_candidates": engines,
    }
    if systems:
        # Top-level so it survives even when chosen_engine is null at first
        # write; `derive_goals_from_systems` reads from
        # ``chosen_engine.systems`` first and falls back here.
        out["systems"] = systems
    return out


def main(argv: list[str]) -> int:
    args = _parse_args(argv)

    config = _read_json("game-config.json")
    default_wiki_url = (config.get("game") or {}).get("wiki_base_url")
    wiki_url = args.wiki_url or default_wiki_url
    if not wiki_url:
        raise SystemExit(
            "No wiki URL provided. Pass --wiki-url, or set game.wiki_base_url in game-config.json."
        )

    _add_scripts_to_sys_path()

    print("[Phase 0] Fetching category taxonomy from MediaWiki API...")
    try:
        import phase0_fetch  # type: ignore
    except Exception as e:  # pragma: no cover
        raise SystemExit(f"Failed to import scripts/phase0_fetch.py ({e}).") from e

    categories = phase0_fetch.fetch_taxonomy(wiki_url, args.min_members)
    print(f"Found {len(categories)} primary categories.")

    print("[Phase 0] Analysing taxonomy with LLM...")
    try:
        import phase0_analyze  # type: ignore
    except Exception as e:  # pragma: no cover
        raise SystemExit(f"Failed to import scripts/phase0_analyze.py ({e}).") from e

    analyze_kwargs: dict[str, Any] = {"mode": args.llm_mode}
    if args.model:
        analyze_kwargs["model"] = args.model
    analysis = phase0_analyze.analyze_taxonomy(categories, **analyze_kwargs)
    kinds = analysis.get("kinds") if isinstance(analysis, dict) else None
    mapped_categories = analysis.get("categories") if isinstance(analysis, dict) else None
    dropped_categories = (
        analysis.get("dropped_categories") if isinstance(analysis, dict) else None
    ) or []
    if not isinstance(kinds, dict) or not isinstance(mapped_categories, list):
        raise SystemExit(
            "phase0_analyze.analyze_taxonomy() must return a dict with "
            "'kinds' (object) and 'categories' (array)."
        )
    print(
        f"Discovered {len(kinds)} kinds, {len(mapped_categories)} mapped categories, "
        f"{len(dropped_categories)} dropped categories."
    )
    if dropped_categories:
        print(
            "  Dropped (review in game-config.json -> dropped_categories): "
            + ", ".join(c.get("name", "?") for c in dropped_categories)
        )

    print("[Phase 0] Fetching wikitext samples for schema proposal...")
    sample_pages = _sample_pages_by_category(phase0_fetch, wiki_url, categories)
    schema_kinds = _kinds_missing_frontmatter_schema(config, kinds)
    schema_categories = _categories_for_kinds(mapped_categories, schema_kinds)
    schemas, engines, codegen_flags, systems = _propose_schemas_and_engines(
        phase0_analyze,
        kinds,
        mapped_categories,
        sample_pages,
        analyze_kwargs,
        schema_kinds,
        schema_categories,
    )
    code_kinds_n = sum(1 for v in codegen_flags.values() if v)
    print(
        f"Proposed schemas for {len(schemas)} kinds, {len(engines)} engine candidates, "
        f"{code_kinds_n} code-kinds / {len(codegen_flags) - code_kinds_n} data-kinds, "
        f"{len(systems)} gameplay systems."
    )
    proposal = _merge_proposals(
        config,
        kinds,
        schemas,
        engines,
        mapped_categories,
        dropped_categories,
        codegen_flags=codegen_flags,
        systems=systems,
    )

    if args.dry_run:
        print(json.dumps(proposal, indent=2, ensure_ascii=False))
        return 0

    try:
        import phase0_write  # type: ignore
    except Exception as e:  # pragma: no cover
        raise SystemExit(f"Failed to import scripts/phase0_write.py ({e}).") from e

    approved = phase0_write.write_proposal(proposal)
    return 0 if approved else 1


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
