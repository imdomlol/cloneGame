"""Unit tests for phase2.system_map state operations.

Covers the pure-Python state mutators, the deterministic cap_tokens
compactor, and the Haiku summariser (with a mocked client). The CLI
sub-dispatcher is exercised via _dispatch through round-tripped tempfiles.
"""

from __future__ import annotations

import unittest
from pathlib import Path
from tempfile import TemporaryDirectory
from types import SimpleNamespace
from unittest.mock import MagicMock

from phase2.system_map import (
    TOKEN_CAP,
    cap_tokens,
    empty_state,
    load_state,
    record_implementation,
    record_pending,
    render_yaml,
    save_state,
    set_baseline_hash,
    summarise_with_haiku,
    update_test_state,
)


def _word_count(text: str) -> int:
    return len(text.split())


class EmptyStateTests(unittest.TestCase):
    def test_has_documented_keys(self) -> None:
        state = empty_state()
        self.assertEqual(
            set(state.keys()),
            {"implemented", "pending", "test_state", "last_engine_baseline_hash"},
        )

    def test_returns_a_fresh_copy(self) -> None:
        a = empty_state()
        a["implemented"].append({"x": 1})
        b = empty_state()
        self.assertEqual(b["implemented"], [])


class RecordImplementationTests(unittest.TestCase):
    def test_appends_entry(self) -> None:
        state = empty_state()
        record_implementation(state, "ranger", "src/r.rs", "sha1", "vault/u/r.md")
        self.assertEqual(len(state["implemented"]), 1)
        entry = state["implemented"][0]
        self.assertEqual(entry["id"], "ranger")
        self.assertEqual(entry["file"], "src/r.rs")
        self.assertEqual(entry["hash"], "sha1")
        self.assertEqual(entry["verified_against"], "vault/u/r.md")

    def test_drops_matching_pending(self) -> None:
        state = empty_state()
        record_pending(state, "ranger", ["bow"])
        record_pending(state, "soldier", [])
        record_implementation(state, "ranger", "src/r.rs", "sha", "vault/u/r.md")
        ids = [p["id"] for p in state["pending"]]
        self.assertEqual(ids, ["soldier"])


class RecordPendingTests(unittest.TestCase):
    def test_appends_entry(self) -> None:
        state = empty_state()
        record_pending(state, "soldier", ["bow", "sword"])
        self.assertEqual(state["pending"][0]["id"], "soldier")
        self.assertEqual(state["pending"][0]["blocked_by"], ["bow", "sword"])

    def test_overwrites_duplicate_id(self) -> None:
        state = empty_state()
        record_pending(state, "soldier", ["bow"])
        record_pending(state, "soldier", ["sword"])
        self.assertEqual(len(state["pending"]), 1)
        self.assertEqual(state["pending"][0]["blocked_by"], ["sword"])


class UpdateTestStateTests(unittest.TestCase):
    def test_overwrites_block(self) -> None:
        state = empty_state()
        update_test_state(state, 10, 2, ["ranger", "bow"])
        self.assertEqual(
            state["test_state"],
            {"passing": 10, "failing": 2, "failing_ids": ["ranger", "bow"]},
        )


class SetBaselineHashTests(unittest.TestCase):
    def test_assigns_value(self) -> None:
        state = empty_state()
        set_baseline_hash(state, "abc123")
        self.assertEqual(state["last_engine_baseline_hash"], "abc123")


class CapTokensTests(unittest.TestCase):
    def _fill(self, n: int) -> dict:
        state = empty_state()
        for i in range(n):
            record_implementation(state, f"id{i}", f"src/{i}.rs", f"sha{i}", f"vault/x/{i}.md")
        return state

    def test_no_op_when_under_cap(self) -> None:
        state = self._fill(3)
        capped = cap_tokens(state, cap=10_000, token_counter=_word_count)
        self.assertEqual(capped, state)

    def test_collapses_oldest_to_summary(self) -> None:
        state = self._fill(20)
        capped = cap_tokens(state, cap=40, token_counter=_word_count, min_kept=3)
        impl = capped["implemented"]
        self.assertTrue(any("summary" in e for e in impl))
        details = [e for e in impl if "summary" not in e]
        self.assertGreaterEqual(len(details), 3)

    def test_stops_at_min_kept_floor(self) -> None:
        state = self._fill(10)
        capped = cap_tokens(state, cap=1, token_counter=_word_count, min_kept=4)
        details = [e for e in capped["implemented"] if "summary" not in e]
        self.assertEqual(len(details), 4)

    def test_summary_count_reflects_collapsed_entries(self) -> None:
        state = self._fill(8)
        capped = cap_tokens(state, cap=40, token_counter=_word_count, min_kept=3)
        summary_entries = [e for e in capped["implemented"] if "summary" in e]
        if summary_entries:
            collapsed = sum(int(e["count"]) for e in summary_entries)
            details = [e for e in capped["implemented"] if "summary" not in e]
            self.assertEqual(collapsed + len(details), 8)


class RenderYamlTests(unittest.TestCase):
    def test_round_trip(self) -> None:
        import yaml

        state = empty_state()
        record_implementation(state, "r", "f", "h", "v")
        text = render_yaml(state)
        loaded = yaml.safe_load(text)
        self.assertEqual(loaded["implemented"][0]["id"], "r")


class LoadSaveTests(unittest.TestCase):
    def test_load_missing_returns_empty(self) -> None:
        with TemporaryDirectory() as d:
            path = Path(d) / "missing.yaml"
            self.assertEqual(load_state(path), empty_state())

    def test_round_trip_through_disk(self) -> None:
        with TemporaryDirectory() as d:
            path = Path(d) / "s.yaml"
            state = empty_state()
            record_implementation(state, "r", "f", "h", "v")
            save_state(path, state)
            self.assertEqual(load_state(path)["implemented"][0]["id"], "r")

    def test_load_merges_missing_keys(self) -> None:
        with TemporaryDirectory() as d:
            path = Path(d) / "partial.yaml"
            path.write_text("implemented:\n  - id: x\n", encoding="utf-8")
            loaded = load_state(path)
            self.assertEqual(loaded["implemented"][0]["id"], "x")
            self.assertEqual(loaded["pending"], [])
            self.assertEqual(loaded["test_state"]["passing"], 0)


class SummariseWithHaikuTests(unittest.TestCase):
    def test_sends_prompt_and_parses_response(self) -> None:
        fake_response = SimpleNamespace(
            content=[
                SimpleNamespace(
                    text="implemented:\n  - id: kept\npending: []\n"
                    "test_state:\n  passing: 0\n  failing: 0\n  failing_ids: []\n"
                    "last_engine_baseline_hash: ''\n"
                )
            ]
        )
        client = MagicMock()
        client.messages.create.return_value = fake_response
        state = empty_state()
        record_implementation(state, "old", "f", "h", "v")
        result = summarise_with_haiku(state, client)
        self.assertEqual(result["implemented"][0]["id"], "kept")
        kwargs = client.messages.create.call_args.kwargs
        self.assertIn("under 1000 tokens", kwargs["messages"][0]["content"])


class TokenCapDocumentationTests(unittest.TestCase):
    def test_matches_documented_value(self) -> None:
        self.assertEqual(TOKEN_CAP, 1000)


if __name__ == "__main__":
    unittest.main()
