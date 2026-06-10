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
_BEVY_REGISTRATION_CR = {
    "aggregator": "{dir}/mod.rs",
    "declaration": "pub mod {stem};",
    "crate_root": "src/lib.rs",
}

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

    def _seed_lib(self, game: Path) -> Path:
        (game / "src").mkdir(parents=True)
        lib = game / "src" / "lib.rs"
        lib.write_text("pub mod sim;\npub mod units;\n", encoding="utf-8")
        return lib

    def test_crate_root_wires_new_kind_into_lib(self) -> None:
        # A brand-new kind dir must be declared in the crate root, else cargo
        # silently leaves it out of the crate (false-green gate).
        with TemporaryDirectory() as tmp:
            game = Path(tmp) / "game"
            lib = self._seed_lib(game)
            leaf = game / "src" / "buildings" / "wood_gate.rs"
            leaf.parent.mkdir(parents=True)
            leaf.write_text("pub struct WoodGate;\n", encoding="utf-8")

            trail = loop_driver.register_modules([leaf], game, _BEVY_REGISTRATION_CR)

            mod_txt = (game / "src" / "buildings" / "mod.rs").read_text(encoding="utf-8")
            self.assertIn("pub mod wood_gate;", mod_txt)
            lib_txt = lib.read_text(encoding="utf-8")
            self.assertIn("pub mod buildings;", lib_txt)
            self.assertIn("pub mod units;", lib_txt)  # untouched shared decl preserved
            touched = {p.path.name for p in trail}
            self.assertEqual(touched, {"mod.rs", "lib.rs"})

    def test_crate_root_leaf_directly_under_src_goes_to_lib(self) -> None:
        # The historical "known limit" (src/sim.rs) is now handled: a leaf whose
        # parent IS the crate-root dir registers in lib.rs, not a bogus src/mod.rs.
        with TemporaryDirectory() as tmp:
            game = Path(tmp) / "game"
            lib = self._seed_lib(game)
            leaf = game / "src" / "render.rs"
            leaf.write_text("pub fn draw() {}\n", encoding="utf-8")

            loop_driver.register_modules([leaf], game, _BEVY_REGISTRATION_CR)

            self.assertIn("pub mod render;", lib.read_text(encoding="utf-8"))
            self.assertFalse((game / "src" / "mod.rs").exists())

    def test_crate_root_existing_kind_is_idempotent(self) -> None:
        with TemporaryDirectory() as tmp:
            game = Path(tmp) / "game"
            lib = self._seed_lib(game)
            (game / "src" / "units").mkdir()
            leaf = game / "src" / "units" / "ranger.rs"
            leaf.write_text("pub struct Ranger;\n", encoding="utf-8")

            loop_driver.register_modules([leaf], game, _BEVY_REGISTRATION_CR)

            self.assertEqual(lib.read_text(encoding="utf-8").count("pub mod units;"), 1)


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

    def test_emitted_crate_root_is_skipped(self) -> None:
        with TemporaryDirectory() as tmp:
            game = Path(tmp) / "game"
            game.mkdir()
            parsed = [
                ("src/buildings/wood_gate.rs", "pub struct WoodGate;"),
                ("src/lib.rs", "pub mod buildings;"),  # driver-owned crate root, must skip
            ]
            written, skipped = loop_driver.merge_into_game(
                parsed, game, aggregator_name="mod.rs", crate_root_name="lib.rs"
            )
        written_rels = [str(w.path.relative_to(game)).replace("\\", "/") for w in written]
        self.assertEqual(written_rels, ["src/buildings/wood_gate.rs"])
        self.assertIn("src/lib.rs", skipped)


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

    def test_valid_kinds_filter_drops_stale_dirs(self) -> None:
        # 'unit'/'building' are stale singular dirs; only the plural kinds are
        # real, so a valid_kinds filter must drop the stale ones entirely.
        with TemporaryDirectory() as tmp:
            vault = self._seed_vault(tmp)
            (vault / "units").mkdir(parents=True)
            (vault / "units" / "soldier.md").write_text("x", encoding="utf-8")
            goals = loop_driver.derive_goals_from_vault(vault, valid_kinds={"units", "buildings"})
        self.assertEqual(goals, [("soldier", "implement the soldier units")])

    def test_load_valid_kinds_reads_config(self) -> None:
        with TemporaryDirectory() as tmp:
            cfg = Path(tmp) / "game-config.json"
            cfg.write_text('{"kinds": {"units": {}, "buildings": {}}}', encoding="utf-8")
            self.assertEqual(loop_driver.load_valid_kinds(cfg), {"units", "buildings"})

    def test_load_valid_kinds_absent_returns_none(self) -> None:
        self.assertIsNone(loop_driver.load_valid_kinds(Path("/no/such/config.json")))

    def test_derive_goals_from_systems_basic(self) -> None:
        systems = [
            {"name": "wave_spawner", "description": "spawns waves", "depends_on": ["infected"]},
            {"name": "hud", "description": "ui", "depends_on": []},
        ]
        goals = loop_driver.derive_goals_from_systems(systems)
        self.assertEqual(
            goals,
            [
                ("wave_spawner", "implement the wave spawner system: spawns waves"),
                ("hud", "implement the hud system: ui"),
            ],
        )

    def test_derive_goals_from_systems_filter(self) -> None:
        systems = [
            {"name": "wave_spawner", "description": "x", "depends_on": []},
            {"name": "hud", "description": "y", "depends_on": []},
        ]
        goals = loop_driver.derive_goals_from_systems(systems, names=["hud"])
        self.assertEqual([g[0] for g in goals], ["hud"])

    def test_load_systems_reads_chosen_engine_block(self) -> None:
        with TemporaryDirectory() as tmp:
            cfg = Path(tmp) / "game-config.json"
            cfg.write_text(
                '{"chosen_engine": {"systems": [{"name": "hud", "description": "ui"}]}}',
                encoding="utf-8",
            )
            systems = loop_driver.load_systems(cfg)
            self.assertEqual([s["name"] for s in systems], ["hud"])

    def test_load_systems_falls_back_to_top_level(self) -> None:
        with TemporaryDirectory() as tmp:
            cfg = Path(tmp) / "game-config.json"
            cfg.write_text(
                '{"systems": [{"name": "hud", "description": "ui"}]}',
                encoding="utf-8",
            )
            systems = loop_driver.load_systems(cfg)
            self.assertEqual([s["name"] for s in systems], ["hud"])

    def test_load_systems_returns_empty_when_absent(self) -> None:
        with TemporaryDirectory() as tmp:
            cfg = Path(tmp) / "game-config.json"
            cfg.write_text("{}", encoding="utf-8")
            self.assertEqual(loop_driver.load_systems(cfg), [])

    def test_load_valid_kinds_excludes_codegen_false(self) -> None:
        with TemporaryDirectory() as tmp:
            cfg = Path(tmp) / "game-config.json"
            cfg.write_text(
                '{"kinds": {"units": {}, "buildings": {"codegen": true}, '
                '"campaign_maps": {"codegen": false}, '
                '"updates": {"codegen": false, "description": "release notes"}}}',
                encoding="utf-8",
            )
            self.assertEqual(loop_driver.load_valid_kinds(cfg), {"units", "buildings"})


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


def _per_goal_generate(calls: list[str]) -> Any:
    """Generate stub that emits a unique file per goal and records call order."""

    def _g(task: str, **_kw: Any) -> dict[str, Any]:
        calls.append(task)
        slug = task.replace("implement the ", "").replace(" unit", "").strip().replace(" ", "_")
        text = (
            f"=== FILE: src/units/{slug}.rs ===\n"
            f"// Sources: vault/units/{slug}.md\n"
            f"pub struct {slug.title()};\n"
            f"=== END FILE ==="
        )
        return {
            "response_text": text,
            "sources_header_ok": True,
            "sources_header_offending": [],
            "included_vault_ids": [],
            "allowed_paths": [],
            "engine_baseline_tokens": 0,
            "user_message_tokens": 0,
            "llm_mode": "claude",
            "model": "test",
        }

    return _g


_CONCURRENCY_GOALS = [
    (None, "implement the a unit"),
    (None, "implement the b unit"),
    (None, "implement the c unit"),
]


class ConcurrencyTests(unittest.TestCase):
    def _run(self, tmp: str, concurrency: int) -> list[Any]:
        game = Path(tmp) / "game"
        game.mkdir()
        return loop_driver.run_loop(
            _CONCURRENCY_GOALS,
            game_dir=game,
            state_path=Path(tmp) / "system_map.yaml",
            baseline_path=Path(tmp) / "b.md",
            generate=_per_goal_generate([]),
            cargo_runner=lambda _gd: (True, ""),
            concurrency=concurrency,
        )

    def test_parallel_matches_serial(self) -> None:
        with TemporaryDirectory() as t1, TemporaryDirectory() as t2:
            serial = self._run(t1, 1)
            parallel = self._run(t2, 3)

        def shape(rs: list[Any]) -> list[tuple[str, str, tuple[str, ...]]]:
            return [(r.note_id, r.status, tuple(r.files_written)) for r in rs]

        self.assertEqual(shape(serial), shape(parallel))
        self.assertEqual([r.status for r in parallel], ["implemented"] * 3)

    def test_parallel_preserves_order_with_skips(self) -> None:
        with TemporaryDirectory() as tmp:
            game = Path(tmp) / "game"
            game.mkdir()
            state_path = Path(tmp) / "system_map.yaml"
            preexisting = system_map.empty_state()
            preexisting["implemented"].append(
                {"id": "b", "file": "", "hash": "", "verified_against": ""}
            )
            system_map.save_state(state_path, preexisting)

            results = loop_driver.run_loop(
                _CONCURRENCY_GOALS,
                game_dir=game,
                state_path=state_path,
                baseline_path=Path(tmp) / "b.md",
                generate=_per_goal_generate([]),
                cargo_runner=lambda _gd: (True, ""),
                concurrency=3,
            )
        self.assertEqual(
            [(r.note_id, r.status) for r in results],
            [("a", "implemented"), ("b", "skipped"), ("c", "implemented")],
        )

    def test_error_budget_bounds_wasted_calls(self) -> None:
        calls: list[str] = []
        with TemporaryDirectory() as tmp:
            game = Path(tmp) / "game"
            game.mkdir()
            results = loop_driver.run_loop(
                [(None, f"implement the {c} unit") for c in "abcd"],
                game_dir=game,
                state_path=Path(tmp) / "system_map.yaml",
                baseline_path=Path(tmp) / "b.md",
                generate=_per_goal_generate(calls),
                cargo_runner=lambda _gd: (False, "fail"),
                error_budget=2,
                concurrency=3,
            )
        # All four share kind "unit" with no exemplar yet, so the chunk cap forces
        # one-at-a-time prep until the first success seeds the exemplar. Here every
        # turn fails, so each is prepared solo; the budget trips after 2 pendings
        # and no further (3rd/4th) turn is ever prepared.
        self.assertEqual([r.status for r in results], ["pending", "pending"])
        self.assertEqual(len(calls), 2)


def _recording_generate(calls: list[tuple[str, str | None]]) -> Any:
    """Generate stub recording ``(note_slug, exemplar)`` and emitting one leaf."""

    def _g(task: str, **kw: Any) -> dict[str, Any]:
        slug = loop_driver.derive_note_id(task)
        calls.append((slug, kw.get("exemplar")))
        text = (
            f"=== FILE: src/units/{slug}.rs ===\n"
            f"// Sources: vault/units/{slug}.md\n"
            f"pub struct {slug.title()};\n"
            f"=== END FILE ==="
        )
        return {
            "response_text": text,
            "sources_header_ok": True,
            "sources_header_offending": [],
            "included_vault_ids": [],
            "allowed_paths": [],
            "engine_baseline_tokens": 0,
            "user_message_tokens": 0,
            "llm_mode": "claude",
            "model": "test",
        }

    return _g


class DeriveKindTests(unittest.TestCase):
    def test_trailing_token_is_kind(self) -> None:
        self.assertEqual(loop_driver.derive_kind("implement the soldier units"), "units")
        self.assertEqual(
            loop_driver.derive_kind("implement the advanced farm buildings"), "buildings"
        )

    def test_too_short_returns_none(self) -> None:
        self.assertIsNone(loop_driver.derive_kind("soldier"))
        self.assertIsNone(loop_driver.derive_kind(""))


class SiblingExemplarTests(unittest.TestCase):
    def test_first_of_kind_gets_no_exemplar_then_siblings_get_it(self) -> None:
        calls: list[tuple[str, str | None]] = []
        with TemporaryDirectory() as tmp:
            game = Path(tmp) / "game"
            game.mkdir()
            loop_driver.run_loop(
                [
                    (None, "implement the soldier units"),
                    (None, "implement the ranger units"),
                    (None, "implement the sniper units"),
                ],
                game_dir=game,
                state_path=Path(tmp) / "system_map.yaml",
                baseline_path=Path(tmp) / "b.md",
                generate=_recording_generate(calls),
                cargo_runner=lambda _gd: (True, ""),
            )
        by_slug = dict(calls)
        self.assertIsNone(by_slug["soldier"], "first of kind has no sibling exemplar")
        # ranger + sniper must receive soldier's generated source as the pattern.
        self.assertIsNotNone(by_slug["ranger"])
        self.assertIn("pub struct Soldier;", by_slug["ranger"])
        self.assertIn("pub struct Soldier;", by_slug["sniper"])

    def test_exemplar_seeded_from_already_implemented_module(self) -> None:
        calls: list[tuple[str, str | None]] = []
        with TemporaryDirectory() as tmp:
            game = Path(tmp) / "game"
            (game / "src" / "units").mkdir(parents=True)
            (game / "src" / "units" / "soldier.rs").write_text(
                "// Sources: vault/units/soldier.md\npub struct Soldier;\n", encoding="utf-8"
            )
            state_path = Path(tmp) / "system_map.yaml"
            preexisting = system_map.empty_state()
            preexisting["implemented"].append(
                {
                    "id": "soldier",
                    "file": "src/units/soldier.rs",
                    "hash": "",
                    "verified_against": "",
                }
            )
            system_map.save_state(state_path, preexisting)

            loop_driver.run_loop(
                [
                    (None, "implement the soldier units"),  # already implemented -> skipped
                    (None, "implement the ranger units"),
                ],
                game_dir=game,
                state_path=state_path,
                baseline_path=Path(tmp) / "b.md",
                generate=_recording_generate(calls),
                cargo_runner=lambda _gd: (True, ""),
            )
        by_slug = dict(calls)
        self.assertNotIn("soldier", by_slug, "skipped module is not regenerated")
        self.assertIn("pub struct Soldier;", by_slug["ranger"] or "")


def _repairable_generate(calls: list[str]) -> Any:
    """Generate stub recording 'fresh'/'repair' per call; emits one leaf each time."""

    def _g(task: str, **kw: Any) -> dict[str, Any]:
        calls.append("repair" if kw.get("repair") else "fresh")
        slug = loop_driver.derive_note_id(task)
        text = (
            f"=== FILE: src/units/{slug}.rs ===\n"
            f"// Sources: vault/units/{slug}.md\n"
            f"pub struct {slug.title()};\n"
            f"=== END FILE ==="
        )
        return {
            "response_text": text,
            "sources_header_ok": True,
            "sources_header_offending": [],
            "included_vault_ids": [],
            "allowed_paths": [],
            "engine_baseline_tokens": 0,
            "user_message_tokens": 0,
            "llm_mode": "claude",
            "model": "test",
        }

    return _g


class RepairLoopTests(unittest.TestCase):
    def _run(self, tmp: str, calls: list[str], build_seq: list[tuple[bool, str]], attempts: int):
        game = Path(tmp) / "game"
        game.mkdir()
        results_iter = iter(build_seq)
        return loop_driver.run_loop(
            [(None, "implement the lucifer units")],
            game_dir=game,
            state_path=Path(tmp) / "system_map.yaml",
            baseline_path=Path(tmp) / "b.md",
            generate=_repairable_generate(calls),
            cargo_runner=lambda _gd: next(results_iter),
            repair_attempts=attempts,
        )

    def test_build_failure_repaired_then_recorded(self) -> None:
        calls: list[str] = []
        with TemporaryDirectory() as tmp:
            results = self._run(tmp, calls, [(False, "error[E0277]"), (True, "")], attempts=2)
        self.assertEqual([r.status for r in results], ["implemented"])
        self.assertEqual(calls, ["fresh", "repair"])  # one repair sufficed

    def test_repair_exhausted_records_pending(self) -> None:
        calls: list[str] = []
        with TemporaryDirectory() as tmp:
            results = self._run(tmp, calls, [(False, "e1"), (False, "e2")], attempts=1)
        self.assertEqual([r.status for r in results], ["pending"])
        self.assertEqual(calls, ["fresh", "repair"])  # exhausted after 1 repair

    def test_repair_disabled_fails_immediately(self) -> None:
        calls: list[str] = []
        with TemporaryDirectory() as tmp:
            results = self._run(tmp, calls, [(False, "e1")], attempts=0)
        self.assertEqual([r.status for r in results], ["pending"])
        self.assertEqual(calls, ["fresh"])  # no repair attempted

    def test_build_error_is_fed_to_repair_turn(self) -> None:
        seen: list[tuple[str, str] | None] = []

        def _gen(task: str, **kw: Any) -> dict[str, Any]:
            seen.append(kw.get("repair"))
            slug = loop_driver.derive_note_id(task)
            text = (
                f"=== FILE: src/units/{slug}.rs ===\n"
                f"// Sources: vault/units/{slug}.md\nx\n=== END FILE ==="
            )
            return {
                "response_text": text,
                "sources_header_ok": True,
                "sources_header_offending": [],
                "included_vault_ids": [],
                "allowed_paths": [],
                "engine_baseline_tokens": 0,
                "user_message_tokens": 0,
                "llm_mode": "claude",
                "model": "test",
            }

        with TemporaryDirectory() as tmp:
            game = Path(tmp) / "game"
            game.mkdir()
            builds = iter([(False, "error[E0277]: 17-tuple"), (True, "")])
            loop_driver.run_loop(
                [(None, "implement the lucifer units")],
                game_dir=game,
                state_path=Path(tmp) / "system_map.yaml",
                baseline_path=Path(tmp) / "b.md",
                generate=_gen,
                cargo_runner=lambda _gd: next(builds),
                repair_attempts=1,
            )
        self.assertIsNone(seen[0])  # fresh turn: no repair context
        self.assertIsNotNone(seen[1])  # repair turn carries (build_error, prior_text)
        self.assertIn("17-tuple", seen[1][0])


class BuildPlanDedupTests(unittest.TestCase):
    """A slug repeated across kinds in one run is attempted only once."""

    def test_duplicate_note_id_skipped_within_run(self) -> None:
        goals = [
            ("technology_tree", "implement the technology tree game_mechanics"),
            ("technology_tree", "implement the technology tree research"),
            ("noise", "implement the noise game_mechanics"),
        ]
        goal_kinds = {"technology_tree": "research", "noise": "game_mechanics"}
        plan, attempts = loop_driver._build_plan(goals, system_map.empty_state(), goal_kinds, None)
        attempt_ids = [nid for tag, nid, *_ in plan if tag == "attempt"]
        skip_ids = [tr.note_id for tag, tr, *_ in plan if tag == "skip"]
        self.assertEqual(attempt_ids, ["technology_tree", "noise"])
        self.assertEqual(skip_ids, ["technology_tree"])  # the 2nd occurrence
        self.assertEqual(len(attempts), 2)

    def test_already_implemented_still_skips(self) -> None:
        state = system_map.empty_state()
        system_map.record_implementation(state, "soldier", "src/units/soldier.rs", "h", "v")
        goals = [
            ("soldier", "implement the soldier units"),
            ("ranger", "implement the ranger units"),
        ]
        plan, _attempts = loop_driver._build_plan(goals, state, {}, None)
        self.assertEqual([nid for tag, nid, *_ in plan if tag == "attempt"], ["ranger"])


if __name__ == "__main__":
    unittest.main()
