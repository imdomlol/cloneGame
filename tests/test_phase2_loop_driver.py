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

# The codegen contract: each file delimited by === FILE: ... === / === END FILE ===.
_FIXTURE_CODEGEN_OUTPUT = textwrap.dedent(
    """
    === FILE: Cargo.toml ===
    [package]
    name = "clone-game"
    version = "0.1.0"
    === END FILE ===

    === FILE: src/units/ranger.rs ===
    // Sources: vault/unit/ranger.md
    pub struct Ranger;
    === END FILE ===

    === FILE: src/units/mod.rs ===
    pub mod ranger;
    pub mod soldier;
    === END FILE ===
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

    def test_ignores_preamble_and_trailing_prose(self) -> None:
        wrapped = (
            "Here is the implementation:\n\n"
            + _FIXTURE_CODEGEN_OUTPUT
            + "\n\nLet me know if you need changes."
        )
        paths = [p for p, _ in loop_driver.parse_codegen_output(wrapped)]
        self.assertEqual(paths, ["Cargo.toml", "src/units/ranger.rs", "src/units/mod.rs"])


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

    def test_strips_redundant_crate_dir_prefix(self) -> None:
        with TemporaryDirectory() as tmp:
            game = Path(tmp) / "game"
            game.mkdir()
            # Model sometimes prefixes the crate dir; must not nest game/game/.
            parsed = [("game/src/units/ranger.rs", "pub struct Ranger;")]
            written, _ = loop_driver.merge_into_game(parsed, game)
            self.assertTrue((game / "src/units/ranger.rs").exists())
            self.assertFalse((game / "game").exists())
            self.assertEqual(len(written), 1)


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
    === FILE: src/units/ranger.rs ===
    // Sources: vault/unit/ranger.md
    pub struct Ranger;
    === END FILE ===
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

    def test_backend_exception_becomes_pending_not_crash(self) -> None:
        with TemporaryDirectory() as tmp:
            game = Path(tmp) / "game"
            game.mkdir()
            state_path = self._stub_state_path(tmp)

            def _boom(_task: str, **_kw: Any) -> dict[str, Any]:
                raise RuntimeError("claude exited 1")

            results = loop_driver.run_loop(
                [
                    (None, "implement the a unit"),
                    (None, "implement the b unit"),
                ],
                game_dir=game,
                state_path=state_path,
                baseline_path=Path(tmp) / "b.md",
                generate=_boom,
                cargo_runner=lambda _gd: (True, ""),
                error_budget=5,
            )

            self.assertEqual([r.status for r in results], ["pending", "pending"])
            self.assertIn("codegen_error", results[0].error or "")
            state = system_map.load_state(state_path)
            self.assertEqual({p["id"] for p in state["pending"]}, {"a", "b"})

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
        # Patch the resolver so neither PATH nor ~/.cargo/bin is consulted.
        with patch("phase2.loop_driver._resolve_cargo", return_value=None):
            ok, msg = loop_driver.run_cargo_build(Path("."))
        self.assertFalse(ok)
        self.assertIn("cargo not found", msg)

    def test_explicit_cargo_bin_skips_resolution(self) -> None:
        captured: dict[str, Any] = {}

        def _fake_run(cmd: list[str], **kwargs: Any) -> Any:
            captured["cmd"] = cmd
            captured["cwd"] = kwargs.get("cwd")

            class _Result:
                returncode = 0
                stdout = ""
                stderr = ""

            return _Result()

        custom = Path("/custom/cargo")
        with patch("phase2.loop_driver.subprocess.run", _fake_run):
            ok, _ = loop_driver.run_cargo_build(Path("game"), cargo_bin=custom)
        self.assertTrue(ok)
        self.assertEqual(captured["cmd"][0], str(custom))


_BEVY_REGISTRATION = {"aggregator": "{dir}/mod.rs", "declaration": "pub mod {stem};"}

_LOSSY_MULTI_FILE = textwrap.dedent(
    """
    === FILE: src/units/ranger.rs ===
    // Sources: vault/unit/ranger.md
    pub struct Ranger;
    === END FILE ===

    === FILE: src/units/mod.rs ===
    pub mod ranger;
    === END FILE ===
    """
).strip()


class LoadModuleRegistrationTests(unittest.TestCase):
    def test_reads_block_from_config(self) -> None:
        with TemporaryDirectory() as tmp:
            cfg = Path(tmp) / "game-config.json"
            cfg.write_text(
                '{"chosen_engine": {"module_registration": '
                '{"aggregator": "{dir}/mod.rs", "declaration": "pub mod {stem};"}}}',
                encoding="utf-8",
            )
            reg = loop_driver.load_module_registration(cfg)
        self.assertEqual(reg, _BEVY_REGISTRATION)

    def test_absent_file_or_block_returns_none(self) -> None:
        self.assertIsNone(loop_driver.load_module_registration(Path("/no/such/config.json")))
        with TemporaryDirectory() as tmp:
            cfg = Path(tmp) / "game-config.json"
            cfg.write_text('{"chosen_engine": {"name": "Godot"}}', encoding="utf-8")
            self.assertIsNone(loop_driver.load_module_registration(cfg))


class AggregatorBasenameTests(unittest.TestCase):
    def test_derives_filename(self) -> None:
        self.assertEqual(loop_driver.aggregator_basename(_BEVY_REGISTRATION), "mod.rs")

    def test_none_registration(self) -> None:
        self.assertIsNone(loop_driver.aggregator_basename(None))


class RegisterModulesTests(unittest.TestCase):
    def _seed_units(self, game: Path) -> Path:
        (game / "src" / "units").mkdir(parents=True)
        mod = game / "src" / "units" / "mod.rs"
        mod.write_text(
            "#[derive(Component)]\npub struct Infected;\n\npub mod soldier;\n",
            encoding="utf-8",
        )
        ranger = game / "src" / "units" / "ranger.rs"
        ranger.write_text("pub struct Ranger;\n", encoding="utf-8")
        return mod

    def test_appends_declaration_preserving_shared_content(self) -> None:
        with TemporaryDirectory() as tmp:
            game = Path(tmp) / "game"
            mod = self._seed_units(game)
            trail = loop_driver.register_modules(
                [game / "src" / "units" / "ranger.rs"], game, _BEVY_REGISTRATION
            )
            txt = mod.read_text(encoding="utf-8")
            self.assertIn("pub struct Infected;", txt)
            self.assertIn("pub mod soldier;", txt)
            self.assertIn("pub mod ranger;", txt)
            self.assertEqual(len(trail), 1)
            self.assertTrue(trail[0].existed_before)

    def test_idempotent(self) -> None:
        with TemporaryDirectory() as tmp:
            game = Path(tmp) / "game"
            mod = self._seed_units(game)
            leaf = [game / "src" / "units" / "ranger.rs"]
            loop_driver.register_modules(leaf, game, _BEVY_REGISTRATION)
            loop_driver.register_modules(leaf, game, _BEVY_REGISTRATION)
            self.assertEqual(mod.read_text(encoding="utf-8").count("pub mod ranger;"), 1)

    def test_revert_restores_aggregator(self) -> None:
        with TemporaryDirectory() as tmp:
            game = Path(tmp) / "game"
            mod = self._seed_units(game)
            original = mod.read_bytes()
            trail = loop_driver.register_modules(
                [game / "src" / "units" / "ranger.rs"], game, _BEVY_REGISTRATION
            )
            self.assertNotEqual(mod.read_bytes(), original)
            loop_driver.revert_merge(trail)
            self.assertEqual(mod.read_bytes(), original)

    def test_none_registration_is_noop(self) -> None:
        with TemporaryDirectory() as tmp:
            game = Path(tmp) / "game"
            self._seed_units(game)
            trail = loop_driver.register_modules([game / "src" / "units" / "ranger.rs"], game, None)
        self.assertEqual(trail, [])


class MergeSkipsAggregatorTests(unittest.TestCase):
    def test_emitted_aggregator_is_skipped(self) -> None:
        with TemporaryDirectory() as tmp:
            game = Path(tmp) / "game"
            game.mkdir()
            parsed = [
                ("src/units/ranger.rs", "pub struct Ranger;"),
                ("src/units/mod.rs", "pub mod ranger;"),  # driver-owned, must skip
            ]
            written, skipped = loop_driver.merge_into_game(parsed, game, aggregator_name="mod.rs")
        written_rels = [str(w.path.relative_to(game)).replace("\\", "/") for w in written]
        self.assertEqual(written_rels, ["src/units/ranger.rs"])
        self.assertIn("src/units/mod.rs", skipped)


class CohesionTests(unittest.TestCase):
    def _seed(self, tmp: str) -> tuple[Path, Path, Path]:
        game = Path(tmp) / "game"
        (game / "src" / "units").mkdir(parents=True)
        mod = game / "src" / "units" / "mod.rs"
        mod.write_text("pub struct Infected;\n\npub mod soldier;\n", encoding="utf-8")
        return game, mod, Path(tmp) / "system_map.yaml"

    def test_aggregator_not_clobbered_shared_decl_preserved(self) -> None:
        with TemporaryDirectory() as tmp:
            game, mod, state_path = self._seed(tmp)
            results = loop_driver.run_loop(
                [(None, "implement the ranger unit")],
                game_dir=game,
                state_path=state_path,
                baseline_path=Path(tmp) / "b.md",
                generate=_fake_generate_factory(_LOSSY_MULTI_FILE),
                cargo_runner=lambda _gd: (True, ""),
                registration=_BEVY_REGISTRATION,
            )
            txt = mod.read_text(encoding="utf-8")
            self.assertEqual([r.status for r in results], ["implemented"])
            self.assertIn("pub struct Infected;", txt)  # shared marker survived
            self.assertIn("pub mod soldier;", txt)  # sibling survived
            self.assertIn("pub mod ranger;", txt)  # new module registered
            self.assertEqual(results[0].files_written, ["src/units/ranger.rs"])
            self.assertIn("src/units/mod.rs", results[0].files_skipped)

    def test_cargo_failure_restores_aggregator(self) -> None:
        with TemporaryDirectory() as tmp:
            game, mod, state_path = self._seed(tmp)
            original = mod.read_bytes()
            results = loop_driver.run_loop(
                [(None, "implement the ranger unit")],
                game_dir=game,
                state_path=state_path,
                baseline_path=Path(tmp) / "b.md",
                generate=_fake_generate_factory(_LOSSY_MULTI_FILE),
                cargo_runner=lambda _gd: (False, "error"),
                registration=_BEVY_REGISTRATION,
            )
            self.assertEqual([r.status for r in results], ["pending"])
            self.assertEqual(mod.read_bytes(), original)  # aggregator restored
            self.assertFalse((game / "src" / "units" / "ranger.rs").exists())


class DeriveGoalsFromVaultTests(unittest.TestCase):
    def _seed_vault(self, tmp: str) -> Path:
        vault = Path(tmp) / "vault"
        (vault / "unit").mkdir(parents=True)
        (vault / "building").mkdir(parents=True)
        (vault / "_quarantine").mkdir(parents=True)
        (vault / "unit" / "ranger.md").write_text("x", encoding="utf-8")
        (vault / "unit" / "the_titan.md").write_text("x", encoding="utf-8")
        (vault / "building" / "bank.md").write_text("x", encoding="utf-8")
        (vault / "_quarantine" / "broken.md").write_text("x", encoding="utf-8")
        return vault

    def test_derives_sorted_pairs_skipping_quarantine(self) -> None:
        with TemporaryDirectory() as tmp:
            vault = self._seed_vault(tmp)
            goals = loop_driver.derive_goals_from_vault(vault)
        self.assertEqual(
            goals,
            [
                ("bank", "implement the bank building"),
                ("ranger", "implement the ranger unit"),
                ("the_titan", "implement the the titan unit"),
            ],
        )

    def test_kinds_filter(self) -> None:
        with TemporaryDirectory() as tmp:
            vault = self._seed_vault(tmp)
            goals = loop_driver.derive_goals_from_vault(vault, kinds=["unit"])
        self.assertEqual([g[0] for g in goals], ["ranger", "the_titan"])

    def test_missing_vault_returns_empty(self) -> None:
        self.assertEqual(loop_driver.derive_goals_from_vault(Path("/no/vault")), [])


class MaxTurnsTests(unittest.TestCase):
    def test_limit_counts_only_attempted_not_skipped(self) -> None:
        with TemporaryDirectory() as tmp:
            game = Path(tmp) / "game"
            game.mkdir()
            state_path = Path(tmp) / "system_map.yaml"
            preexisting = system_map.empty_state()
            preexisting["implemented"].append(
                {"id": "soldier", "file": "", "hash": "", "verified_against": ""}
            )
            system_map.save_state(state_path, preexisting)

            results = loop_driver.run_loop(
                [
                    ("soldier", "implement the soldier unit"),  # already done -> skipped
                    ("ranger", "implement the ranger unit"),
                    ("sniper", "implement the sniper unit"),
                ],
                game_dir=game,
                state_path=state_path,
                baseline_path=Path(tmp) / "b.md",
                generate=_fake_generate_factory(_VALID_TURN_OUTPUT),
                cargo_runner=lambda _gd: (True, ""),
                max_turns=1,
            )

        statuses = [(r.note_id, r.status) for r in results]
        # soldier skipped (free), ranger is the one attempted turn, sniper never reached.
        self.assertEqual(statuses, [("soldier", "skipped"), ("ranger", "implemented")])


if __name__ == "__main__":
    unittest.main()
