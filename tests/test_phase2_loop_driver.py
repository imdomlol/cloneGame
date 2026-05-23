"""Unit tests for phase2/loop_driver.py.

Covers the pure helpers (parser, slug derivation, goal-file parsing,
merge / revert) and the orchestration in ``run_loop`` with mocked
``codegen.generate`` + ``cargo`` runners. The real Anthropic + cargo
paths are not exercised here; that happens via ``python
phase2/loop_driver.py`` end-to-end.
"""

from __future__ import annotations

import textwrap
import unittest
from pathlib import Path
from tempfile import TemporaryDirectory
from typing import Any
from unittest.mock import patch

from phase2 import loop_driver, system_map

# Mirror the shape of build/turn1_soldier.md without copying its 18kB.
_FIXTURE_CODEGEN_OUTPUT = textwrap.dedent(
    """
    Permissions are blocking writes. Here is the implementation.

    ---

    **`Cargo.toml`**
    ```toml
    [package]
    name = "clone-game"
    version = "0.1.0"
    ```

    ---

    **`src/units/ranger.rs`** — Empire ranged infantry
    ```rust
    // Sources: vault/unit/ranger.md
    pub struct Ranger;
    ```

    ---

    **`src/units/mod.rs`**
    ```rust
    pub mod ranger;
    pub mod soldier;
    ```

    ---

    Trailing prose the parser must ignore.
    """
).strip()


class ParseCodegenOutputTests(unittest.TestCase):
    def test_extracts_each_file_block(self) -> None:
        parsed = loop_driver.parse_codegen_output(_FIXTURE_CODEGEN_OUTPUT)
        paths = [p for p, _ in parsed]
        self.assertEqual(paths, ["Cargo.toml", "src/units/ranger.rs", "src/units/mod.rs"])

    def test_preserves_block_contents(self) -> None:
        parsed = dict(loop_driver.parse_codegen_output(_FIXTURE_CODEGEN_OUTPUT))
        self.assertIn('name = "clone-game"', parsed["Cargo.toml"])
        self.assertIn("pub struct Ranger;", parsed["src/units/ranger.rs"])
        self.assertIn("// Sources: vault/unit/ranger.md", parsed["src/units/ranger.rs"])

    def test_returns_empty_for_unstructured_output(self) -> None:
        self.assertEqual(loop_driver.parse_codegen_output("just prose, no blocks"), [])


class DeriveNoteIdTests(unittest.TestCase):
    def test_pulls_slug_from_canonical_goal(self) -> None:
        self.assertEqual(loop_driver.derive_note_id("implement the ranger unit"), "ranger")
        self.assertEqual(loop_driver.derive_note_id("Implement THE Harpy enemy"), "harpy")
        self.assertEqual(loop_driver.derive_note_id("implement soldier"), "soldier")

    def test_falls_back_to_slugified_text(self) -> None:
        slug = loop_driver.derive_note_id("Refactor combat to use events")
        self.assertEqual(slug, "refactor_combat_to_use_events")


class ParseGoalLineTests(unittest.TestCase):
    def test_explicit_note_id(self) -> None:
        self.assertEqual(
            loop_driver.parse_goal_line("ranger|implement the ranger unit"),
            ("ranger", "implement the ranger unit"),
        )

    def test_no_separator(self) -> None:
        self.assertEqual(
            loop_driver.parse_goal_line("implement the soldier"),
            (None, "implement the soldier"),
        )

    def test_empty_id_treated_as_absent(self) -> None:
        self.assertEqual(
            loop_driver.parse_goal_line("|implement the bee"),
            (None, "implement the bee"),
        )


class LoadGoalsTests(unittest.TestCase):
    def test_positional_wins_over_file(self) -> None:
        with TemporaryDirectory() as tmp:
            gf = Path(tmp) / "goals.txt"
            gf.write_text("from_file|implement the file unit\n", encoding="utf-8")
            goals = loop_driver.load_goals(gf, ["implement the cli unit"])
        self.assertEqual(goals, [(None, "implement the cli unit")])

    def test_file_format_skips_comments_and_blanks(self) -> None:
        with TemporaryDirectory() as tmp:
            gf = Path(tmp) / "goals.txt"
            gf.write_text(
                "# this is a comment\n"
                "\n"
                "soldier|implement the soldier unit\n"
                "implement the ranger unit\n",
                encoding="utf-8",
            )
            goals = loop_driver.load_goals(gf, [])
        self.assertEqual(
            goals,
            [
                ("soldier", "implement the soldier unit"),
                (None, "implement the ranger unit"),
            ],
        )

    def test_empty_inputs_return_empty(self) -> None:
        self.assertEqual(loop_driver.load_goals(None, []), [])


class MergeIntoGameTests(unittest.TestCase):
    def test_writes_new_files_and_tracks_for_revert(self) -> None:
        with TemporaryDirectory() as tmp:
            game = Path(tmp)
            parsed = [
                ("src/units/ranger.rs", "// Sources: vault/unit/ranger.md\npub struct Ranger;"),
                ("Cargo.toml", '[package]\nname="x"'),  # must be skipped
            ]
            written, skipped = loop_driver.merge_into_game(parsed, game)

            self.assertEqual(skipped, ["Cargo.toml"])
            self.assertFalse((game / "Cargo.toml").exists())
            self.assertTrue((game / "src/units/ranger.rs").exists())
            self.assertEqual(len(written), 1)
            self.assertFalse(written[0].existed_before)

    def test_revert_deletes_new_file_and_restores_existing(self) -> None:
        with TemporaryDirectory() as tmp:
            game = Path(tmp)
            existing = game / "src" / "lib.rs"
            existing.parent.mkdir(parents=True, exist_ok=True)
            existing.write_text("original lib\n", encoding="utf-8")

            parsed = [
                ("src/lib.rs", "rewritten lib\n"),
                ("src/units/new.rs", "new file\n"),
            ]
            written, _ = loop_driver.merge_into_game(parsed, game)
            self.assertEqual(existing.read_text(encoding="utf-8"), "rewritten lib\n")
            self.assertTrue((game / "src/units/new.rs").exists())

            loop_driver.revert_merge(written)
            self.assertEqual(existing.read_text(encoding="utf-8"), "original lib\n")
            self.assertFalse((game / "src/units/new.rs").exists())


class AlreadyImplementedTests(unittest.TestCase):
    def test_match_on_id(self) -> None:
        state = {"implemented": [{"id": "soldier"}, {"id": "ranger"}]}
        self.assertTrue(loop_driver.already_implemented(state, "soldier"))
        self.assertFalse(loop_driver.already_implemented(state, "harpy"))

    def test_empty_state(self) -> None:
        self.assertFalse(loop_driver.already_implemented({}, "x"))


def _fake_generate_factory(text: str, ok: bool = True, offending: list[str] | None = None) -> Any:
    """Return a callable that mimics ``codegen.generate``'s return shape."""

    def _generate(_task: str, **_kw: Any) -> dict[str, Any]:
        return {
            "response_text": text,
            "sources_header_ok": ok,
            "sources_header_offending": offending or [],
            "included_vault_ids": [],
            "allowed_paths": [],
            "engine_baseline_tokens": 0,
            "user_message_tokens": 0,
            "llm_mode": "claude",
            "model": "test",
        }

    return _generate


_VALID_TURN_OUTPUT = textwrap.dedent(
    """
    **`src/units/ranger.rs`** — Empire ranged infantry
    ```rust
    // Sources: vault/unit/ranger.md
    pub struct Ranger;
    ```
    """
).strip()


class RunLoopTests(unittest.TestCase):
    def _stub_state_path(self, tmp: str) -> Path:
        return Path(tmp) / "system_map.yaml"

    def test_cargo_success_records_implementation(self) -> None:
        with TemporaryDirectory() as tmp:
            game = Path(tmp) / "game"
            game.mkdir()
            state_path = self._stub_state_path(tmp)

            results = loop_driver.run_loop(
                [(None, "implement the ranger unit")],
                game_dir=game,
                state_path=state_path,
                baseline_path=Path(tmp) / "baseline_unused.md",
                generate=_fake_generate_factory(_VALID_TURN_OUTPUT),
                cargo_runner=lambda _gd: (True, ""),
            )

            self.assertEqual([r.status for r in results], ["implemented"])
            self.assertEqual(results[0].note_id, "ranger")
            self.assertEqual(results[0].files_written, ["src/units/ranger.rs"])

            state = system_map.load_state(state_path)
            ids = [impl["id"] for impl in state["implemented"]]
            self.assertEqual(ids, ["ranger"])

    def test_cargo_failure_reverts_and_records_pending(self) -> None:
        with TemporaryDirectory() as tmp:
            game = Path(tmp) / "game"
            game.mkdir()
            state_path = self._stub_state_path(tmp)

            results = loop_driver.run_loop(
                [(None, "implement the ranger unit")],
                game_dir=game,
                state_path=state_path,
                baseline_path=Path(tmp) / "baseline_unused.md",
                generate=_fake_generate_factory(_VALID_TURN_OUTPUT),
                cargo_runner=lambda _gd: (False, "error: cannot find type"),
            )

            self.assertEqual([r.status for r in results], ["pending"])
            self.assertIn("cargo_build_failed", results[0].error or "")
            self.assertFalse((game / "src/units/ranger.rs").exists())

            state = system_map.load_state(state_path)
            pending_ids = [p["id"] for p in state["pending"]]
            self.assertEqual(pending_ids, ["ranger"])

    def test_already_implemented_is_skipped(self) -> None:
        with TemporaryDirectory() as tmp:
            game = Path(tmp) / "game"
            game.mkdir()
            state_path = self._stub_state_path(tmp)
            preexisting = system_map.empty_state()
            preexisting["implemented"].append(
                {"id": "ranger", "file": "", "hash": "", "verified_against": ""}
            )
            system_map.save_state(state_path, preexisting)

            sentinel_calls: list[str] = []

            def _generate_should_not_be_called(task: str, **_kw: Any) -> dict[str, Any]:
                sentinel_calls.append(task)
                raise AssertionError("generate must not be called for already-implemented ids")

            results = loop_driver.run_loop(
                [(None, "implement the ranger unit")],
                game_dir=game,
                state_path=state_path,
                baseline_path=Path(tmp) / "baseline_unused.md",
                generate=_generate_should_not_be_called,
                cargo_runner=lambda _gd: (True, ""),
            )

        self.assertEqual([r.status for r in results], ["skipped"])
        self.assertEqual(sentinel_calls, [])

    def test_error_budget_stops_loop(self) -> None:
        with TemporaryDirectory() as tmp:
            game = Path(tmp) / "game"
            game.mkdir()
            state_path = self._stub_state_path(tmp)

            results = loop_driver.run_loop(
                [
                    (None, "implement the a unit"),
                    (None, "implement the b unit"),
                    (None, "implement the c unit"),
                    (None, "implement the d unit"),
                ],
                game_dir=game,
                state_path=state_path,
                baseline_path=Path(tmp) / "baseline_unused.md",
                generate=_fake_generate_factory(_VALID_TURN_OUTPUT),
                cargo_runner=lambda _gd: (False, "fail"),
                error_budget=2,
            )

        self.assertEqual(len(results), 2)
        self.assertEqual([r.status for r in results], ["pending", "pending"])

    def test_invalid_source_header_records_pending_without_writing(self) -> None:
        with TemporaryDirectory() as tmp:
            game = Path(tmp) / "game"
            game.mkdir()
            state_path = self._stub_state_path(tmp)

            cargo_calls: list[Path] = []

            def _cargo(gd: Path) -> tuple[bool, str]:
                cargo_calls.append(gd)
                return True, ""

            results = loop_driver.run_loop(
                [(None, "implement the ranger unit")],
                game_dir=game,
                state_path=state_path,
                baseline_path=Path(tmp) / "baseline_unused.md",
                generate=_fake_generate_factory(
                    _VALID_TURN_OUTPUT, ok=False, offending=["vault/bogus.md"]
                ),
                cargo_runner=_cargo,
            )

        self.assertEqual([r.status for r in results], ["pending"])
        self.assertEqual(cargo_calls, [], "cargo must not run when sources_header_ok is False")
        self.assertFalse((game / "src/units/ranger.rs").exists())


class RunCargoBuildTests(unittest.TestCase):
    def test_missing_cargo_returns_failure_not_exception(self) -> None:
        with patch("phase2.loop_driver.shutil.which", return_value=None):
            ok, msg = loop_driver.run_cargo_build(Path("."))
        self.assertFalse(ok)
        self.assertIn("cargo not on PATH", msg)


if __name__ == "__main__":
    unittest.main()
