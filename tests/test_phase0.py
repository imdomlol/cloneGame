import unittest
from typing import Any, ClassVar

from scripts.phase0 import (
    _kinds_missing_frontmatter_schema,
    _merge_proposals,
)
from scripts.phase0_analyze import (
    IncompleteCoverageError,
    _validate_codegen_flags,
    _validate_output,
    _validate_systems_output,
)


class Phase0CompletenessGateTests(unittest.TestCase):
    INPUT_CATEGORIES: ClassVar[list[dict[str, Any]]] = [
        {"name": "Buildings", "member_count": 42, "members": ["Farm", "Mill"]},
        {"name": "Mechanics", "member_count": 9, "members": ["Swarms", "Time"]},
        {"name": "Browse", "member_count": 6, "members": ["Guides"]},
    ]

    def test_silent_omission_raises_incomplete_coverage(self) -> None:
        # LLM forgets Mechanics and Browse — exactly the regression that started this work.
        result = {
            "kinds": {"building": {"minWikilinks": 1, "description": "structures"}},
            "categories": [{"name": "Buildings", "kind": "building"}],
        }
        with self.assertRaises(IncompleteCoverageError) as ctx:
            _validate_output(result, self.INPUT_CATEGORIES)
        self.assertEqual(ctx.exception.missing, ["Browse", "Mechanics"])

    def test_drop_reason_satisfies_coverage_and_splits_output(self) -> None:
        result = {
            "kinds": {
                "building": {"minWikilinks": 1, "description": "structures"},
                "mechanic": {"minWikilinks": 2, "description": "gameplay rules"},
            },
            "categories": [
                {"name": "Buildings", "kind": "building"},
                {"name": "Mechanics", "kind": "mechanic"},
                {"name": "Browse", "drop_reason": "portal/index category, not an entity type"},
            ],
        }
        validated = _validate_output(result, self.INPUT_CATEGORIES)
        self.assertEqual([c["name"] for c in validated["categories"]], ["Buildings", "Mechanics"])
        self.assertEqual(len(validated["dropped_categories"]), 1)
        dropped = validated["dropped_categories"][0]
        self.assertEqual(dropped["name"], "Browse")
        self.assertEqual(dropped["sample_members"], ["Guides"])
        self.assertEqual(dropped["member_count"], 6)
        self.assertTrue(dropped["drop_reason"])

    def test_duplicate_category_name_rejected(self) -> None:
        result = {
            "kinds": {"building": {"minWikilinks": 1, "description": "x"}},
            "categories": [
                {"name": "Buildings", "kind": "building"},
                {"name": "Buildings", "drop_reason": "duplicate"},
                {"name": "Mechanics", "drop_reason": "n/a"},
                {"name": "Browse", "drop_reason": "n/a"},
            ],
        }
        with self.assertRaisesRegex(ValueError, "appears more than once"):
            _validate_output(result, self.INPUT_CATEGORIES)

    def test_entry_missing_both_kind_and_drop_reason_rejected(self) -> None:
        result = {
            "kinds": {"building": {"minWikilinks": 1, "description": "x"}},
            "categories": [
                {"name": "Buildings", "kind": "building"},
                {"name": "Mechanics"},
                {"name": "Browse", "drop_reason": "n/a"},
            ],
        }
        with self.assertRaisesRegex(ValueError, "must have either 'kind' or 'drop_reason'"):
            _validate_output(result, self.INPUT_CATEGORIES)


class Phase0StabilityTests(unittest.TestCase):
    def test_merge_reuses_existing_schema_for_stable_kind(self) -> None:
        current = {
            "kinds": {
                "infected": {
                    "minWikilinks": 2,
                    "description": "Existing approved enemy kind",
                    "frontmatter_schema": {"properties": {"hp": {"type": "number"}}},
                }
            }
        }
        proposal = _merge_proposals(
            current,
            {"infected": {"minWikilinks": 1, "description": "Changed description"}},
            {"infected": {"properties": {"hp": {"type": "integer"}}}},
            [],
            [{"name": "Infected", "kind": "infected", "member_count": 17}],
        )

        self.assertEqual(
            proposal["kinds"]["infected"]["frontmatter_schema"],
            {"properties": {"hp": {"type": "number"}}},
        )
        self.assertEqual(proposal["kinds"]["infected"]["description"], "Changed description")

    def test_kinds_missing_frontmatter_schema_only_returns_new_or_schema_less_kinds(self) -> None:
        current = {
            "kinds": {
                "building": {"frontmatter_schema": {"properties": {"hp": {"type": "number"}}}},
                "unit": {},
            }
        }
        missing = _kinds_missing_frontmatter_schema(
            current,
            {
                "building": {"description": "stable"},
                "unit": {"description": "needs schema"},
                "technology": {"description": "new kind"},
            },
        )

        self.assertEqual(set(missing), {"unit", "technology"})


class CodegenFlagClassifierTests(unittest.TestCase):
    KINDS: ClassVar[dict[str, Any]] = {
        "unit": {"description": "controllable units"},
        "campaign_map": {"description": "scenario maps"},
        "update_log": {"description": "patch notes"},
    }

    def test_valid_flags_pass_through(self) -> None:
        result = {
            "codegen_flags": {"unit": True, "campaign_map": False, "update_log": False}
        }
        flags = _validate_codegen_flags(result, self.KINDS)
        self.assertEqual(flags, {"unit": True, "campaign_map": False, "update_log": False})

    def test_missing_kind_raises(self) -> None:
        result = {"codegen_flags": {"unit": True, "campaign_map": False}}
        with self.assertRaisesRegex(ValueError, "update_log"):
            _validate_codegen_flags(result, self.KINDS)

    def test_extra_kind_raises(self) -> None:
        result = {
            "codegen_flags": {
                "unit": True,
                "campaign_map": False,
                "update_log": False,
                "phantom": True,
            }
        }
        with self.assertRaisesRegex(ValueError, "phantom"):
            _validate_codegen_flags(result, self.KINDS)

    def test_non_bool_value_raises(self) -> None:
        result = {
            "codegen_flags": {"unit": "yes", "campaign_map": False, "update_log": False}
        }
        with self.assertRaisesRegex(ValueError, "must be bool"):
            _validate_codegen_flags(result, self.KINDS)

    def test_merge_proposal_inherits_hand_set_codegen_flag(self) -> None:
        """Operator hand-set `codegen: false` survives a Phase 0 re-run that
        proposes `codegen: true` (no surprise flips)."""
        current = {
            "kinds": {
                "unit": {"codegen": False, "description": "demoted by hand"},
            }
        }
        proposal = _merge_proposals(
            current,
            {"unit": {"description": "controllable units"}},
            {},
            [],
            [],
            codegen_flags={"unit": True},
        )
        self.assertEqual(proposal["kinds"]["unit"]["codegen"], False)

    def test_merge_proposal_applies_classifier_when_no_prior(self) -> None:
        proposal = _merge_proposals(
            {},
            {"campaign_map": {"description": "scenario maps"}},
            {},
            [],
            [],
            codegen_flags={"campaign_map": False},
        )
        self.assertEqual(proposal["kinds"]["campaign_map"]["codegen"], False)


class GameplaySystemsValidatorTests(unittest.TestCase):
    CODE_KINDS: ClassVar[set[str]] = {"unit", "infected", "building"}

    def test_valid_systems_pass_through(self) -> None:
        result = {
            "systems": [
                {
                    "name": "wave_spawner",
                    "description": "spawns waves on a timer",
                    "depends_on": ["infected"],
                    "produces": ["WaveTimer"],
                },
                {
                    "name": "game_state_machine",
                    "description": "tracks menu/playing/paused/win/lose",
                    "depends_on": [],
                },
            ]
        }
        systems = _validate_systems_output(result, self.CODE_KINDS)
        self.assertEqual(len(systems), 2)
        self.assertEqual(systems[0]["name"], "wave_spawner")
        self.assertEqual(systems[1]["produces"], [])

    def test_empty_systems_list_raises(self) -> None:
        with self.assertRaisesRegex(ValueError, "at least one system"):
            _validate_systems_output({"systems": []}, self.CODE_KINDS)

    def test_duplicate_name_raises(self) -> None:
        result = {
            "systems": [
                {"name": "hud", "description": "x", "depends_on": []},
                {"name": "hud", "description": "y", "depends_on": []},
            ]
        }
        with self.assertRaisesRegex(ValueError, "Duplicate system name: hud"):
            _validate_systems_output(result, self.CODE_KINDS)

    def test_depends_on_unknown_kind_raises(self) -> None:
        result = {
            "systems": [
                {
                    "name": "wave_spawner",
                    "description": "x",
                    "depends_on": ["phantom_kind"],
                }
            ]
        }
        with self.assertRaisesRegex(ValueError, "phantom_kind"):
            _validate_systems_output(result, self.CODE_KINDS)

    def test_missing_description_raises(self) -> None:
        result = {"systems": [{"name": "hud", "depends_on": []}]}
        with self.assertRaisesRegex(ValueError, "description"):
            _validate_systems_output(result, self.CODE_KINDS)


if __name__ == "__main__":
    unittest.main()
