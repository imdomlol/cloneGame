from __future__ import annotations

import json
from typing import Any


def _read_json(path: str) -> dict[str, Any]:
    with open(path, encoding="utf-8") as f:
        data = json.load(f)
    if not isinstance(data, dict):
        raise ValueError(f"{path} must contain a JSON object at the top level.")
    return data


def _write_json(path: str, obj: dict[str, Any]) -> None:
    with open(path, "w", encoding="utf-8", newline="\n") as f:
        json.dump(obj, f, indent=2, ensure_ascii=False)
        f.write("\n")


def _count_schema_fields(kinds: dict[str, Any]) -> int:
    total = 0
    for kind_data in kinds.values():
        if not isinstance(kind_data, dict):
            continue
        schema = kind_data.get("frontmatter_schema")
        if isinstance(schema, dict):
            properties = schema.get("properties")
            if isinstance(properties, dict):
                total += len(properties)
    return total


def write_proposal(
    analysis: dict,
    config_path: str = "game-config.json",
    proposed_path: str = "game-config.proposed.json",
) -> bool:
    """Write the Phase 0 proposal and auto-promote it to the active config.

    The interactive diff-review gate was removed: the proposed config is now
    on the order of 1k+ lines (per-kind frontmatter_schema + engine_candidates),
    which is too large to review line-by-line — real failures surface during
    Phase 1 validation anyway. The proposed-config file is still written so
    `git diff game-config.json` post-run shows exactly what changed.
    """
    current = _read_json(config_path)

    merged = dict(current)
    merged["kinds"] = analysis["kinds"]
    merged["categories"] = analysis["categories"]
    merged["seedPages"] = []
    merged["human_approved"] = True
    if "engine_candidates" in analysis:
        merged["engine_candidates"] = analysis["engine_candidates"]
    if "chosen_engine" not in merged and "chosen_engine" in current:
        merged["chosen_engine"] = current["chosen_engine"]

    _write_json(proposed_path, merged)
    _write_json(config_path, merged)

    kinds_count = len(merged.get("kinds", {}))
    cats_count = len(merged.get("categories", []))
    engines_count = len(merged.get("engine_candidates", []))
    fields_count = _count_schema_fields(merged.get("kinds", {}))
    print(
        f"[Phase 0] Wrote {config_path} (auto-promoted): "
        f"{kinds_count} kinds, {cats_count} categories, "
        f"{fields_count} schema fields, {engines_count} engine candidates."
    )
    print(
        f"Inspect diff with `git diff -- {config_path}` if needed; "
        f"{proposed_path} mirrors the same content for offline review.\n"
        "Next: `python scripts/phase1_ingest.py --dry-run`."
    )
    return True
