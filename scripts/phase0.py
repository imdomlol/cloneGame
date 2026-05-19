#!/usr/bin/env python3

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
    with open(path, "r", encoding="utf-8") as f:
        data = json.load(f)
    if not isinstance(data, dict):
        raise ValueError(f"{path} must contain a JSON object at the top level.")
    return data


def _parse_args(argv: list[str]) -> argparse.Namespace:
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
    return parser.parse_args(argv)


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
        raise SystemExit(
            f"Failed to import scripts/phase0_fetch.py ({e}). This file is out of scope for this task."
        )

    categories = phase0_fetch.fetch_taxonomy(wiki_url, args.min_members)
    print(f"Found {len(categories)} primary categories.")

    print("[Phase 0] Analysing taxonomy with LLM...")
    try:
        import phase0_analyze  # type: ignore
    except Exception as e:  # pragma: no cover
        raise SystemExit(
            f"Failed to import scripts/phase0_analyze.py ({e}). This file is out of scope for this task."
        )

    analysis = phase0_analyze.analyze_taxonomy(categories)
    kinds = analysis.get("kinds") if isinstance(analysis, dict) else None
    seed_pages = analysis.get("seedPages") if isinstance(analysis, dict) else None
    if not isinstance(kinds, dict) or not isinstance(seed_pages, list):
        raise SystemExit(
            "phase0_analyze.analyze_taxonomy() must return a dict with 'kinds' (object) and 'seedPages' (array)."
        )
    print(f"Discovered {len(kinds)} kinds, {len(seed_pages)} seed pages.")

    if args.dry_run:
        print(json.dumps({"kinds": kinds, "seedPages": seed_pages}, indent=2, ensure_ascii=False))
        return 0

    try:
        import phase0_write  # type: ignore
    except Exception as e:  # pragma: no cover
        raise SystemExit(f"Failed to import scripts/phase0_write.py ({e}).")

    approved = phase0_write.write_proposal(analysis)
    return 0 if approved else 1


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
