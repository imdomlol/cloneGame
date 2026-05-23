import unittest
from typing import Any, ClassVar

from scripts.phase0 import (
    _kinds_missing_frontmatter_schema,
    _merge_proposals,
)
from scripts.phase0_analyze import IncompleteCoverageError, _validate_output


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


if __name__ == "__main__":
    unittest.main()
