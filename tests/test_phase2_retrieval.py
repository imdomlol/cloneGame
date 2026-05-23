"""Unit tests for phase2.retrieval helpers.

Covers the pure-Python helpers: vector merge, graph expansion, and
token-capped packing. Chroma + sentence-transformers paths are out of
scope (they require the persistent store + a downloaded model).
"""

from __future__ import annotations

import unittest

from phase2.retrieval import (
    DEFAULT_CAP_TOKENS,
    graph_expand,
    merge_vector_results,
    pack_files,
)


def _fake_token_counter(text: str) -> int:
    """Whitespace-split token counter; deterministic and dep-free for tests."""
    return len(text.split())


class MergeVectorResultsTests(unittest.TestCase):
    def test_orders_by_distance_across_collections(self) -> None:
        merged = merge_vector_results(
            prose_ids=["a", "b"],
            prose_distances=[0.5, 0.9],
            mechanics_ids=["c", "d"],
            mechanics_distances=[0.1, 0.7],
        )
        self.assertEqual(merged, ["c", "a", "d", "b"])

    def test_dedup_keeps_better_distance(self) -> None:
        merged = merge_vector_results(
            prose_ids=["a", "b"],
            prose_distances=[0.4, 0.9],
            mechanics_ids=["a", "c"],
            mechanics_distances=[0.1, 0.6],
        )
        self.assertEqual(merged, ["a", "c", "b"])

    def test_tie_prefers_prose_first(self) -> None:
        merged = merge_vector_results(
            prose_ids=["a"],
            prose_distances=[0.3],
            mechanics_ids=["b"],
            mechanics_distances=[0.3],
        )
        self.assertEqual(merged, ["a", "b"])

    def test_handles_empty_inputs(self) -> None:
        self.assertEqual(merge_vector_results([], [], [], []), [])


class GraphExpandTests(unittest.TestCase):
    def test_seeds_first_then_unique_neighbours(self) -> None:
        graph = {
            "ranger": ["bow", "soldier"],
            "bow": ["wood"],
            "soldier": ["bow"],
        }
        result = graph_expand(["ranger"], graph)
        self.assertEqual(result, ["ranger", "bow", "soldier"])

    def test_seeds_preserved_when_multi(self) -> None:
        graph = {"a": ["x"], "b": ["x", "y"]}
        result = graph_expand(["a", "b"], graph)
        self.assertEqual(result, ["a", "b", "x", "y"])

    def test_missing_seed_in_graph_is_noop(self) -> None:
        self.assertEqual(graph_expand(["ghost"], {}), ["ghost"])

    def test_does_not_recurse(self) -> None:
        graph = {"a": ["b"], "b": ["c"]}
        self.assertEqual(graph_expand(["a"], graph), ["a", "b"])


class PackFilesTests(unittest.TestCase):
    def setUp(self) -> None:
        self.id_to_path = {
            "ranger": "vault/unit/ranger.md",
            "bow": "vault/item/bow.md",
            "soldier": "vault/unit/soldier.md",
        }
        self.path_to_body = {
            "vault/unit/ranger.md": "ranger body words go here",
            "vault/item/bow.md": "bow body words",
            "vault/unit/soldier.md": "soldier soldier soldier soldier soldier soldier",
        }

    def test_packs_in_order_until_cap(self) -> None:
        packed, included = pack_files(
            ["ranger", "bow", "soldier"],
            self.id_to_path,
            self.path_to_body,
            cap_tokens=12,
            token_counter=_fake_token_counter,
        )
        self.assertEqual(included, ["ranger"])
        self.assertIn("vault/unit/ranger.md", packed)

    def test_skips_blocks_that_would_overshoot_but_continues(self) -> None:
        packed, included = pack_files(
            ["soldier", "bow"],
            self.id_to_path,
            self.path_to_body,
            cap_tokens=8,
            token_counter=_fake_token_counter,
        )
        self.assertEqual(included, ["bow"])
        self.assertIn("bow body", packed)
        self.assertNotIn("soldier soldier", packed)

    def test_assert_fires_when_post_pack_count_exceeds_cap(self) -> None:
        """Defends against per-block + final-count drift in the tokenizer.

        The greedy loop trusts the per-block count; the post-pack assert is
        the safety net. Simulate a tokenizer that under-counts each block
        but over-counts the joined string, and prove the net catches it.
        """

        def drifting_counter(text: str) -> int:
            return 999 if text.count("</file>") >= 2 else 3

        with self.assertRaises(AssertionError):
            pack_files(
                ["ranger", "bow"],
                self.id_to_path,
                self.path_to_body,
                cap_tokens=10,
                token_counter=drifting_counter,
            )

    def test_skips_unknown_id_silently(self) -> None:
        _packed, included = pack_files(
            ["ranger", "ghost"],
            self.id_to_path,
            self.path_to_body,
            cap_tokens=DEFAULT_CAP_TOKENS,
            token_counter=_fake_token_counter,
        )
        self.assertEqual(included, ["ranger"])

    def test_emits_repomix_envelope(self) -> None:
        packed, included = pack_files(
            ["bow"],
            self.id_to_path,
            self.path_to_body,
            cap_tokens=DEFAULT_CAP_TOKENS,
            token_counter=_fake_token_counter,
        )
        self.assertEqual(included, ["bow"])
        self.assertTrue(packed.startswith('<file path="vault/item/bow.md">'))
        self.assertTrue(packed.endswith("</file>"))

    def test_empty_input_returns_empty_pack(self) -> None:
        packed, included = pack_files(
            [],
            self.id_to_path,
            self.path_to_body,
            cap_tokens=DEFAULT_CAP_TOKENS,
            token_counter=_fake_token_counter,
        )
        self.assertEqual((packed, included), ("", []))


if __name__ == "__main__":
    unittest.main()
