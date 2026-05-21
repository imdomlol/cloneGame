import unittest

from scripts.phase0 import (
    _kinds_missing_frontmatter_schema,
    _merge_proposals,
)


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
