from __future__ import annotations

import difflib
import json
import os
import sys
from typing import Any


def _supports_ansi_color() -> bool:
    try:
        return os.isatty(sys.stdout.fileno())
    except Exception:
        return False


def _colorize_diff_line(line: str) -> str:
    # Avoid colorizing diff headers.
    if line.startswith("---") or line.startswith("+++"):
        return line
    if line.startswith("-"):
        return f"\x1b[31m{line}\x1b[0m"
    if line.startswith("+"):
        return f"\x1b[32m{line}\x1b[0m"
    return line


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


def write_proposal(
    analysis: dict,
    config_path: str = "game-config.json",
    proposed_path: str = "game-config.proposed.json",
) -> bool:
    """
    Writes a taxonomy proposal to `proposed_path`, prints a unified diff, and
    optionally promotes it to the active config. Returns True if the user
    approved the proposal, False if aborted.
    """
    current = _read_json(config_path)

    merged = dict(current)
    merged["kinds"] = analysis["kinds"]
    merged["categories"] = analysis["categories"]
    merged["seedPages"] = []
    merged["human_approved"] = False
    if "engine_candidates" in analysis:
        merged["engine_candidates"] = analysis["engine_candidates"]
    if "chosen_engine" not in merged and "chosen_engine" in current:
        merged["chosen_engine"] = current["chosen_engine"]

    _write_json(proposed_path, merged)

    old_text = json.dumps(current, indent=2, ensure_ascii=False).splitlines(keepends=True)
    new_text = json.dumps(merged, indent=2, ensure_ascii=False).splitlines(keepends=True)

    diff_lines = difflib.unified_diff(
        old_text,
        new_text,
        fromfile=config_path,
        tofile=proposed_path,
        lineterm="",
    )

    use_color = _supports_ansi_color()
    for line in diff_lines:
        if use_color:
            print(_colorize_diff_line(line))
        else:
            print(line)

    try:
        accept = input("Review the diff above.\nAccept this taxonomy proposal? [y/N]: ").strip()
    except EOFError:
        accept = ""

    if accept.lower() != "y":
        print("Aborted. game-config.proposed.json written for manual review.")
        return False

    try:
        promote = input("Set human_approved: true and make this the active config? [y/N]: ").strip()
    except EOFError:
        promote = ""

    if promote.lower() == "y":
        promoted = dict(merged)
        promoted["human_approved"] = True
        _write_json(config_path, promoted)
        print("game-config.json updated. Phase 1 is unblocked.")

    print(
        "\n--- Phase 0 Complete ---\n\n"
        "Next steps:\n"
        "  1. Review the proposed engine_candidates block in game-config.json and set\n"
        "     `chosen_engine` to one of them (or to a custom entry) before Phase 2.\n"
        "  2. Run Phase 1 ingest. It will use kinds.<kind>.frontmatter_schema for\n"
        "     per-kind validation; categories drive what gets fetched via the MediaWiki API.\n"
    )

    return True
