"""Render the per-game engine baseline prompt (Layer 1 sticky cache).

Reads ``game-config.json`` and ``prompts/engine_baseline.template.md``, picks
the determinism rules block matching ``chosen_engine.name``, formats the
universal + per-kind frontmatter sections, and writes ``build/engine_baseline.md``.

The output is the system-block payload that ``phase2/codegen.py`` sends with
``cache_control: ephemeral``. Hit-rate on this block is the main cost mechanism
documented in DEPLOYMENT_GUIDE §4. A hard ``<= 2,500 token`` assert prevents
silent baseline bloat from eroding cache value.

CLI::

    python phase2/baseline.py                            # render to build/
    python phase2/baseline.py --dry-run                  # stdout only
    python phase2/baseline.py --output path/to/out.md    # custom destination
"""

from __future__ import annotations

import argparse
import json
import re
import sys
from pathlib import Path
from typing import Any

_REPO_ROOT = Path(__file__).resolve().parent.parent
if str(_REPO_ROOT / "phase2") not in sys.path:
    sys.path.insert(0, str(_REPO_ROOT / "phase2"))

from retrieval import count_tokens  # noqa: E402

DEFAULT_GAME_CONFIG = _REPO_ROOT / "game-config.json"
DEFAULT_TEMPLATE = _REPO_ROOT / "prompts" / "engine_baseline.template.md"
DEFAULT_UNIVERSAL_SCHEMA = _REPO_ROOT / "schemas" / "_universal.schema.json"
DEFAULT_OUTPUT = _REPO_ROOT / "build" / "engine_baseline.md"
DETERMINISM_DIR = _REPO_ROOT / "prompts" / "engine_determinism"
TOKEN_CAP = 2500

_PLACEHOLDER_RE = re.compile(r"\{\{(?P<key>[a-z_]+)\}\}")


def _load_determinism_rules(engine_name: str, rules_dir: Path = DETERMINISM_DIR) -> str:
    """Pick a per-engine rules file by lowercased name; fall back to ``_generic.md``."""
    key = engine_name.strip().lower().replace(" ", "_").replace("-", "_")
    candidate = rules_dir / f"{key}.md"
    if candidate.exists():
        return candidate.read_text(encoding="utf-8").strip()
    fallback = rules_dir / "_generic.md"
    return fallback.read_text(encoding="utf-8").strip()


def _format_universal_fields(schema: dict[str, Any]) -> str:
    """Markdown list of universal frontmatter fields with types + notes."""
    required = set(schema.get("required", []))
    out: list[str] = []
    for name, spec in schema.get("properties", {}).items():
        flag = "required" if name in required else "optional"
        type_repr = spec.get("type", "any")
        if isinstance(type_repr, list):
            type_repr = " | ".join(type_repr)
        desc = spec.get("description", "")
        line = f"- `{name}` ({type_repr}, {flag})"
        if desc:
            line += f" — {desc}"
        out.append(line)
    return "\n".join(out)


def _format_kinds(kinds: dict[str, Any]) -> str:
    """Render each kind's frontmatter_schema.properties as a markdown subsection."""
    sections: list[str] = []
    for kind, spec in kinds.items():
        props = (spec.get("frontmatter_schema") or {}).get("properties") or {}
        if not props:
            sections.append(
                f"### `{kind}`\n\n_No per-kind fields. Validation falls back to universal-only._"
            )
            continue
        field_lines = [f"- `{name}` ({_describe_property(p)})" for name, p in props.items()]
        sections.append(f"### `{kind}`\n\n" + "\n".join(field_lines))
    return "\n\n".join(sections)


def _describe_property(spec: dict[str, Any]) -> str:
    type_repr = spec.get("type", "any")
    if isinstance(type_repr, list):
        type_repr = " | ".join(type_repr)
    enum = spec.get("enum")
    if enum:
        return f"{type_repr}, one of: {', '.join(repr(e) for e in enum)}"
    return str(type_repr)


def _fill_template(template: str, values: dict[str, str]) -> str:
    """Replace ``{{key}}`` markers; unknown keys raise so silent drift is impossible."""

    def repl(match: re.Match[str]) -> str:
        key = match.group("key")
        if key not in values:
            raise KeyError(f"template placeholder {{{{{key}}}}} has no value")
        return values[key]

    return _PLACEHOLDER_RE.sub(repl, template)


def render_baseline(
    game_config: dict[str, Any],
    template: str,
    universal_schema: dict[str, Any],
) -> str:
    """Build the rendered baseline string from config + template + schema.

    Raises ``KeyError`` if ``chosen_engine`` is missing — Phase 2 cannot run
    without an approved engine selection.
    """
    chosen = game_config.get("chosen_engine") or {}
    if not chosen.get("name"):
        raise KeyError(
            "game-config.json has no chosen_engine.name; "
            "run Phase 0 v2 and pick from engine_candidates first"
        )
    rules = _load_determinism_rules(str(chosen["name"]))
    values = {
        "game_name": game_config.get("game", {}).get("name", "(unnamed game)"),
        "engine_name": chosen.get("name", ""),
        "engine_language": chosen.get("language", ""),
        "architecture_summary": chosen.get("architecture", ""),
        "networking_model": chosen.get("networking_model", ""),
        "determinism_rules": rules,
        "universal_fields": _format_universal_fields(universal_schema),
        "kinds_section": _format_kinds(game_config.get("kinds", {})),
    }
    return _fill_template(template, values)


def assert_under_cap(rendered: str, cap_tokens: int = TOKEN_CAP) -> int:
    """Verify ``rendered`` fits the sticky-cache token budget. Returns token count."""
    tokens = count_tokens(rendered)
    assert tokens <= cap_tokens, (
        f"engine baseline is {tokens} tokens; cap is {cap_tokens}. "
        "Trim the template or move detail into per-kind retrieval."
    )
    return tokens


def _parse_args(argv: list[str] | None = None) -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.add_argument("--game-config", type=Path, default=DEFAULT_GAME_CONFIG)
    parser.add_argument("--template", type=Path, default=DEFAULT_TEMPLATE)
    parser.add_argument("--universal-schema", type=Path, default=DEFAULT_UNIVERSAL_SCHEMA)
    parser.add_argument("--output", type=Path, default=DEFAULT_OUTPUT)
    parser.add_argument("--dry-run", action="store_true", help="print to stdout, skip write")
    return parser.parse_args(argv)


def main(argv: list[str] | None = None) -> int:
    args = _parse_args(argv)
    game_config = json.loads(args.game_config.read_text(encoding="utf-8"))
    template = args.template.read_text(encoding="utf-8")
    universal_schema = json.loads(args.universal_schema.read_text(encoding="utf-8"))
    rendered = render_baseline(game_config, template, universal_schema)
    tokens = assert_under_cap(rendered)
    if args.dry_run:
        print(rendered)
    else:
        args.output.parent.mkdir(parents=True, exist_ok=True)
        args.output.write_text(rendered, encoding="utf-8")
    print(
        f"engine baseline: {tokens}/{TOKEN_CAP} tokens "
        f"({'stdout' if args.dry_run else args.output})",
        file=sys.stderr,
    )
    return 0


if __name__ == "__main__":
    sys.exit(main())
