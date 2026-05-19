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
    with open(path, "r", encoding="utf-8") as f:
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
    merged["seedPages"] = analysis["seedPages"]
    merged["human_approved"] = False

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
        accept = input(
            "Review the diff above.\nAccept this taxonomy proposal? [y/N]: "
        ).strip()
    except EOFError:
        accept = ""

    if accept.lower() != "y":
        print("Aborted. game-config.proposed.json written for manual review.")
        return False

    try:
        promote = input(
            "Set human_approved: true and make this the active config? [y/N]: "
        ).strip()
    except EOFError:
        promote = ""

    if promote.lower() == "y":
        promoted = dict(merged)
        promoted["human_approved"] = True
        _write_json(config_path, promoted)
        print("game-config.json updated. Phase 1 is unblocked.")

    print(
        "\n--- Phase 0 Complete ---\n\n"
        "Next step (semi-automated): obtain the They Are Billions XML page dump.\n\n"
        "Option A - Fandom Special:Export (small wikis / targeted pages):\n"
        "  https://they-are-billions.fandom.com/wiki/Special:Export\n"
        "  Select pages by category, download XML, save to: sources/tab-wiki-dump.xml\n\n"
        "Option B - Wikia/Fandom database dump mirror (full wiki):\n"
        "  Check: https://s3.amazonaws.com/wikia_xml_dumps/  (if available for this wiki)\n"
        "  Or use the MediaWiki API with action=query&export=1&exportnowrap=1\n\n"
        "Place the dump at sources/tab-wiki-dump.xml before running Phase 1.\n"
        "Run: python scripts/phase1_validate_dump.py  (not yet written) to verify.\n"
    )

    return True
