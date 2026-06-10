"""Tests for scripts/load_game.py — restore a shipped game into the working tree."""

from __future__ import annotations

import json
import sys
import unittest
from pathlib import Path
from tempfile import TemporaryDirectory

_REPO_ROOT = Path(__file__).resolve().parent.parent
if str(_REPO_ROOT / "scripts") not in sys.path:
    sys.path.insert(0, str(_REPO_ROOT / "scripts"))

import load_game as lg  # noqa: E402


def _write(path: Path, body: str) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(body, encoding="utf-8")


def _make_release(releases: Path, slug: str) -> Path:
    """Build a minimal valid release on disk for load_game tests."""
    rel = releases / slug
    _write(rel / "source" / "Cargo.toml", '[package]\nname = "clone-game"\n')
    _write(rel / "source" / "src" / "lib.rs", "// foundation\n")
    _write(rel / "source" / "tests" / "smoke.rs", "// smoke\n")
    _write(rel / "vault" / "unit" / "soldier.md", "---\nid: soldier\n---\n")
    _write(rel / "game-config.json", json.dumps({"game": {"name": "X"}}))
    _write(rel / "system_map.yaml", "implemented: []\n")
    return rel


class ListReleasesTests(unittest.TestCase):
    def setUp(self) -> None:
        self._tmp = TemporaryDirectory()
        self.root = Path(self._tmp.name)

    def tearDown(self) -> None:
        self._tmp.cleanup()

    def test_lists_complete_snapshots_only(self) -> None:
        _make_release(self.root, "lethal-company")
        # incomplete release: no source/Cargo.toml
        _write(self.root / "broken" / "vault" / "x.md", "")
        self.assertEqual(lg.list_releases(self.root), ["lethal-company"])

    def test_empty_root_returns_empty(self) -> None:
        self.assertEqual(lg.list_releases(self.root), [])

    def test_missing_root_returns_empty(self) -> None:
        self.assertEqual(lg.list_releases(self.root / "no-such"), [])


class HasContentTests(unittest.TestCase):
    def setUp(self) -> None:
        self._tmp = TemporaryDirectory()
        self.root = Path(self._tmp.name)

    def tearDown(self) -> None:
        self._tmp.cleanup()

    def test_nonexistent_is_empty(self) -> None:
        self.assertFalse(lg._has_content(self.root / "no-such"))

    def test_empty_dir_is_empty(self) -> None:
        (self.root / "empty").mkdir()
        self.assertFalse(lg._has_content(self.root / "empty"))

    def test_dir_with_file_is_not_empty(self) -> None:
        _write(self.root / "x" / "a.txt", "data")
        self.assertTrue(lg._has_content(self.root / "x"))

    def test_empty_file_is_empty(self) -> None:
        (self.root / "f").write_text("")
        self.assertFalse(lg._has_content(self.root / "f"))


class LoadGameEndToEndTests(unittest.TestCase):
    def setUp(self) -> None:
        self._tmp = TemporaryDirectory()
        self.root = Path(self._tmp.name)
        self.releases = self.root / "releases"
        _make_release(self.releases, "lethal-company")
        self.game_dir = self.root / "game"
        self.vault_dir = self.root / "vault"
        self.config = self.root / "game-config.json"
        self.system_map = self.root / "build" / "system_map.yaml"

    def tearDown(self) -> None:
        self._tmp.cleanup()

    def test_restores_full_release(self) -> None:
        rc = lg.load_game(
            "lethal-company",
            releases_root=self.releases,
            game_config=self.config,
            game_dir=self.game_dir,
            vault_dir=self.vault_dir,
            system_map_path=self.system_map,
        )
        self.assertEqual(rc, 0)
        self.assertTrue((self.game_dir / "src" / "lib.rs").is_file())
        self.assertTrue((self.game_dir / "tests" / "smoke.rs").is_file())
        self.assertTrue((self.game_dir / "Cargo.toml").is_file())
        self.assertTrue((self.vault_dir / "unit" / "soldier.md").is_file())
        self.assertTrue(self.config.is_file())
        self.assertTrue(self.system_map.is_file())

    def test_unknown_slug_fails(self) -> None:
        rc = lg.load_game(
            "no-such",
            releases_root=self.releases,
            game_config=self.config,
            game_dir=self.game_dir,
            vault_dir=self.vault_dir,
            system_map_path=self.system_map,
        )
        self.assertEqual(rc, 1)

    def test_refuses_dirty_working_tree(self) -> None:
        _write(self.game_dir / "src" / "main.rs", "existing\n")
        rc = lg.load_game(
            "lethal-company",
            releases_root=self.releases,
            game_config=self.config,
            game_dir=self.game_dir,
            vault_dir=self.vault_dir,
            system_map_path=self.system_map,
        )
        self.assertEqual(rc, 1)
        # Existing file untouched
        self.assertEqual((self.game_dir / "src" / "main.rs").read_text(), "existing\n")

    def test_force_overwrites_dirty_working_tree(self) -> None:
        _write(self.game_dir / "src" / "main.rs", "existing\n")
        rc = lg.load_game(
            "lethal-company",
            releases_root=self.releases,
            game_config=self.config,
            game_dir=self.game_dir,
            vault_dir=self.vault_dir,
            system_map_path=self.system_map,
            force=True,
        )
        self.assertEqual(rc, 0)
        # Old file gone (the entire src/ dir gets replaced)
        self.assertFalse((self.game_dir / "src" / "main.rs").exists())
        self.assertTrue((self.game_dir / "src" / "lib.rs").is_file())


if __name__ == "__main__":
    unittest.main()
