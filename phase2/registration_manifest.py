"""Parse registered resources and events from Rust/Bevy source files.

Scans `game/src/` before each codegen turn and returns a small manifest block
that lists every resource and event already registered by committed plugins.
The codegen prompt includes this block so the LLM knows what it must NOT
re-register (a duplicate `add_event` or `init_resource` causes a Bevy B0001
panic at runtime) and what it may consume without registering itself.

Engine-agnostic: the regexes target Bevy's registration call patterns
(`add_event::<T>`, `init_resource::<T>`, `insert_resource(T::...)`), but the
manifest format is plain text and the caller (codegen) treats it as an opaque
string injected into the prompt. A different engine's registration API would
need its own regex set; the injection path stays the same.
"""

from __future__ import annotations

import re
from pathlib import Path

# Matches: app.add_event::<SomeType>()
_ADD_EVENT_RE = re.compile(r"\.add_event::<([^>]+)>")

# Matches: app.init_resource::<SomeType>()
_INIT_RES_RE = re.compile(r"\.init_resource::<([^>]+)>")

# Matches: app.insert_resource::<SomeType>() — the typed form
_INSERT_RES_TYPED_RE = re.compile(r"\.insert_resource::<([^>]+)>")

# Matches: app.insert_resource(SomeType::default()) or insert_resource(SomeType { .. })
# Captures only the leading type name (the token before :: or whitespace/brace).
_INSERT_RES_VAL_RE = re.compile(r"\.insert_resource\(\s*([A-Z][A-Za-z0-9_]*)(?:::|[\s{(])")


def _extract_from_source(src: str) -> tuple[list[str], list[str]]:
    """Return (events, resources) found in one Rust source string.

    Each list contains raw type tokens as they appear in the source (e.g.
    ``"IncomingDamageEvent"``, ``"SimChecksumState"``). Callers deduplicate
    across files.
    """
    events = _ADD_EVENT_RE.findall(src)
    resources = (
        _INIT_RES_RE.findall(src)
        + _INSERT_RES_TYPED_RE.findall(src)
        + _INSERT_RES_VAL_RE.findall(src)
    )
    return events, resources


def build_manifest(game_dir: Path) -> str:
    """Scan ``game_dir/src/`` and return a formatted registration manifest.

    Walks every ``*.rs`` file under ``src/``, collects all ``add_event`` and
    ``init_resource`` / ``insert_resource`` calls, deduplicates, and renders a
    ``[REGISTERED RESOURCES AND EVENTS]`` block for inclusion in the codegen
    prompt. Returns an empty string when nothing is found (e.g. cold start) so
    the prompt is not padded with an empty section.
    """
    src_root = game_dir / "src"
    if not src_root.is_dir():
        return ""

    all_events: set[str] = set()
    all_resources: set[str] = set()

    for rs_file in sorted(src_root.rglob("*.rs")):
        try:
            src = rs_file.read_text(encoding="utf-8")
        except OSError:
            continue
        events, resources = _extract_from_source(src)
        all_events.update(events)
        all_resources.update(resources)

    if not all_events and not all_resources:
        return ""

    lines = [
        "[REGISTERED RESOURCES AND EVENTS]",
        "The following are already registered by existing plugins. Do NOT register",
        "them again — a duplicate add_event or init_resource causes a Bevy B0001",
        "panic at runtime that cargo build cannot detect.",
        "",
    ]
    if all_events:
        lines.append("Events (add_event called — read/write freely via EventReader/EventWriter):")
        for e in sorted(all_events):
            lines.append(f"  Events<{e}>")
    if all_resources:
        lines.append("Resources (init_resource/insert_resource called — use Res/ResMut freely):")
        for r in sorted(all_resources):
            lines.append(f"  {r}")

    return "\n".join(lines) + "\n\n"
