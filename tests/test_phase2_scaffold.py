"""Tests for phase2/scaffold.py — engine-foundation rendering.

Covers the dispatch path (engine name → scaffold dir), tree-mirror behaviour,
and the safety rules around hand-edits + idempotency. The fresh-run pipeline
relies on this step running before Phase 2 codegen, so a regression in the
dispatch or the hand-edit check would silently break new-game setup.
"""

from __future__ import annotations

import json
import sys
import tempfile
import unittest
from pathlib import Path

_REPO_ROOT = Path(__file__).resolve().parent.parent
if str(_REPO_ROOT / "phase2") not in sys.path:
    sys.path.insert(0, str(_REPO_ROOT / "phase2"))

from scaffold import (  # noqa: E402
    find_scaffold_dir,
    load_engine_name,
    render,
    render_scaffold,
)


def _write(path: Path, body: str) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(body, encoding="utf-8")


class LoadEngineNameTests(unittest.TestCase):
    def test_returns_lowercased_engine_name(self) -> None:
        with tempfile.TemporaryDirectory() as td:
            cfg = Path(td) / "game-config.json"
            cfg.write_text(json.dumps({"chosen_engine": {"name": "Bevy"}}))
            self.assertEqual(load_engine_name(cfg), "bevy")

    def test_missing_file_returns_none(self) -> None:
        self.assertIsNone(load_engine_name(Path("/no/such/path.json")))

    def test_missing_block_returns_none(self) -> None:
        with tempfile.TemporaryDirectory() as td:
            cfg = Path(td) / "game-config.json"
            cfg.write_text(json.dumps({"other": 1}))
            self.assertIsNone(load_engine_name(cfg))


class FindScaffoldDirTests(unittest.TestCase):
    def test_returns_matching_subdirectory(self) -> None:
        with tempfile.TemporaryDirectory() as td:
            root = Path(td)
            (root / "bevy").mkdir()
            self.assertEqual(find_scaffold_dir("bevy", root), root / "bevy")

    def test_returns_none_for_unknown_engine(self) -> None:
        with tempfile.TemporaryDirectory() as td:
            root = Path(td)
            self.assertIsNone(find_scaffold_dir("godot", root))

    def test_returns_none_when_scaffold_root_absent(self) -> None:
        self.assertIsNone(find_scaffold_dir("bevy", Path("/no/such/root")))


class RenderScaffoldTests(unittest.TestCase):
    def setUp(self) -> None:
        self._tmp = tempfile.TemporaryDirectory()
        self.root = Path(self._tmp.name)
        self.scaffold = self.root / "scaffold" / "bevy"
        self.game = self.root / "game"
        _write(self.scaffold / "Cargo.toml", "[package]\nname = 'x'\n")
        _write(self.scaffold / "src" / "sim.rs", "// sim\n")
        _write(self.scaffold / "src" / "main.rs", "fn main() {}\n")

    def tearDown(self) -> None:
        self._tmp.cleanup()

    def test_fresh_run_writes_every_file(self) -> None:
        results = render_scaffold(self.scaffold, self.game)
        statuses = {r.target.name: r.status for r in results}
        self.assertEqual(
            statuses,
            {"Cargo.toml": "written", "sim.rs": "written", "main.rs": "written"},
        )
        self.assertEqual((self.game / "src" / "sim.rs").read_text(), "// sim\n")

    def test_second_run_is_a_noop(self) -> None:
        render_scaffold(self.scaffold, self.game)
        results = render_scaffold(self.scaffold, self.game)
        self.assertTrue(all(r.status == "skipped_identical" for r in results))

    def test_hand_edited_file_is_preserved_by_default(self) -> None:
        render_scaffold(self.scaffold, self.game)
        (self.game / "src" / "main.rs").write_text("// HAND EDIT\n")
        results = render_scaffold(self.scaffold, self.game)
        statuses = {r.target.name: r.status for r in results}
        self.assertEqual(statuses["main.rs"], "skipped_hand_edit")
        self.assertEqual((self.game / "src" / "main.rs").read_text(), "// HAND EDIT\n")

    def test_force_overwrites_hand_edited_file(self) -> None:
        render_scaffold(self.scaffold, self.game)
        (self.game / "src" / "main.rs").write_text("// HAND EDIT\n")
        render_scaffold(self.scaffold, self.game, force=True)
        self.assertEqual((self.game / "src" / "main.rs").read_text(), "fn main() {}\n")

    def test_dry_run_writes_nothing(self) -> None:
        results = render_scaffold(self.scaffold, self.game, dry_run=True)
        self.assertTrue(all(r.status == "would_write" for r in results))
        self.assertFalse((self.game / "src" / "sim.rs").exists())


class RenderHighLevelTests(unittest.TestCase):
    def test_returns_engine_and_results(self) -> None:
        with tempfile.TemporaryDirectory() as td:
            root = Path(td)
            cfg = root / "game-config.json"
            cfg.write_text(json.dumps({"chosen_engine": {"name": "Bevy"}}))
            scaffold_root = root / "scaffold"
            _write(scaffold_root / "bevy" / "Cargo.toml", "[package]\nname = 'x'\n")
            engine, results = render(
                game_config_path=cfg,
                game_dir=root / "game",
                scaffold_root=scaffold_root,
            )
            self.assertEqual(engine, "bevy")
            self.assertEqual([r.status for r in results], ["written"])

    def test_returns_none_engine_when_unset(self) -> None:
        with tempfile.TemporaryDirectory() as td:
            root = Path(td)
            cfg = root / "game-config.json"
            cfg.write_text(json.dumps({}))
            engine, results = render(
                game_config_path=cfg,
                game_dir=root / "game",
                scaffold_root=root / "scaffold",
            )
            self.assertIsNone(engine)
            self.assertEqual(results, [])


if __name__ == "__main__":
    unittest.main()
