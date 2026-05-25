"""Unit tests for phase2.baseline rendering helpers.

Covers template substitution, engine profile selection, schema rendering,
and the cap assertion. Does not exercise the CLI or read game-config.json
from disk so tests stay self-contained.
"""

from __future__ import annotations

import unittest

from phase2.baseline import (
    TOKEN_CAP,
    _format_kinds,
    _format_universal_fields,
    assert_under_cap,
    render_baseline,
)

_TEMPLATE = """\
Game: {{game_name}}
Engine: {{engine_name}} ({{engine_language}})
Net: {{networking_model}}
Arch: {{architecture_summary}}

[rules]
{{determinism_rules}}
[universal]
{{universal_fields}}
[kinds]
{{kinds_section}}
"""

_UNIVERSAL = {
    "required": ["id", "name"],
    "properties": {
        "id": {"type": "string", "description": "snake_case"},
        "name": {"type": "string"},
        "tags": {"type": "array"},
    },
}


def _bevy_config() -> dict:
    return {
        "game": {"name": "Test Game"},
        "chosen_engine": {
            "name": "Bevy",
            "language": "Rust",
            "architecture": "fixed-tick sim",
            "networking_model": "lockstep",
        },
        "kinds": {
            "unit": {
                "frontmatter_schema": {
                    "properties": {
                        "hp": {"type": "string"},
                        "damage": {"type": "string"},
                    }
                }
            },
            "concept": {},
        },
    }


class RenderBaselineTests(unittest.TestCase):
    def test_substitutes_all_placeholders(self) -> None:
        rendered = render_baseline(_bevy_config(), _TEMPLATE, _UNIVERSAL)
        self.assertIn("Game: Test Game", rendered)
        self.assertIn("Engine: Bevy (Rust)", rendered)
        self.assertIn("Net: lockstep", rendered)
        self.assertIn("Arch: fixed-tick sim", rendered)

    def test_bevy_profile_pulls_in_lockstep_rules(self) -> None:
        rendered = render_baseline(_bevy_config(), _TEMPLATE, _UNIVERSAL)
        self.assertIn("Fixed-point math", rendered)
        self.assertIn("ChaCha8Rng", rendered)
        self.assertIn("HashMap", rendered)
        self.assertIn("Desync detection", rendered)

    def test_unknown_engine_falls_back_to_generic_note(self) -> None:
        cfg = _bevy_config()
        cfg["chosen_engine"]["name"] = "QuantumLeap"
        rendered = render_baseline(cfg, _TEMPLATE, _UNIVERSAL)
        self.assertIn("does not match a known engine profile", rendered)

    def test_missing_chosen_engine_raises(self) -> None:
        cfg = _bevy_config()
        cfg.pop("chosen_engine")
        with self.assertRaises(KeyError):
            render_baseline(cfg, _TEMPLATE, _UNIVERSAL)

    def test_unknown_placeholder_raises(self) -> None:
        template_with_bad = _TEMPLATE + "\n{{nope}}\n"
        with self.assertRaises(KeyError):
            render_baseline(_bevy_config(), template_with_bad, _UNIVERSAL)


class UniversalFieldsTests(unittest.TestCase):
    def test_marks_required_vs_optional(self) -> None:
        out = _format_universal_fields(_UNIVERSAL)
        self.assertIn("`id` (string, required)", out)
        self.assertIn("`tags` (array, optional)", out)

    def test_includes_description_when_present(self) -> None:
        out = _format_universal_fields(_UNIVERSAL)
        self.assertIn("snake_case", out)


class KindsSectionTests(unittest.TestCase):
    def test_renders_each_kind_with_fields(self) -> None:
        cfg = _bevy_config()
        out = _format_kinds(cfg["kinds"])
        self.assertIn("### `unit`", out)
        self.assertIn("`hp` (string)", out)

    def test_marks_kinds_without_schema(self) -> None:
        cfg = _bevy_config()
        out = _format_kinds(cfg["kinds"])
        self.assertIn("### `concept`", out)
        self.assertIn("No per-kind fields", out)


class CapAssertionTests(unittest.TestCase):
    def test_under_cap_returns_count(self) -> None:
        tokens = assert_under_cap("short text", cap_tokens=100)
        self.assertLess(tokens, 100)

    def test_over_cap_raises(self) -> None:
        with self.assertRaises(AssertionError):
            assert_under_cap("word " * 50, cap_tokens=10)

    def test_default_cap_matches_documented_budget(self) -> None:
        # Bumped 2500 -> 2800 (2026-05-25) for the lockstep RNG + desync-checksum
        # determinism rules. See baseline.py TOKEN_CAP comment.
        self.assertEqual(TOKEN_CAP, 2800)


if __name__ == "__main__":
    unittest.main()
