"""Tests for scripts/ship.py — snapshot the working tree into releases/<slug>/.

Covers the pure helpers (slug derivation, executable detection, package-name
parsing) and the end-to-end ship function against a synthetic working tree.
``--skip-binary`` short-circuits the cargo build so the test suite stays
hermetic — operators verify the binary path interactively.
"""

from __future__ import annotations

import json
import sys
import unittest
from pathlib import Path
from tempfile import TemporaryDirectory

_REPO_ROOT = Path(__file__).resolve().parent.parent
if str(_REPO_ROOT / "scripts") not in sys.path:
    sys.path.insert(0, str(_REPO_ROOT / "scripts"))

import ship  # noqa: E402


def _write(path: Path, body: str) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(body, encoding="utf-8")


class DeriveSlugTests(unittest.TestCase):
    def test_basic_name_is_lowercased_and_hyphenated(self) -> None:
        self.assertEqual(ship.derive_slug("Lethal Company"), "lethal-company")
        self.assertEqual(ship.derive_slug("They Are Billions"), "they-are-billions")

    def test_punctuation_collapses(self) -> None:
        self.assertEqual(ship.derive_slug("Pikmin 2: The Adventure"), "pikmin-2-the-adventure")
        self.assertEqual(ship.derive_slug("Don't Starve"), "don-t-starve")

    def test_empty_falls_back(self) -> None:
        self.assertEqual(ship.derive_slug(""), "game")
        self.assertEqual(ship.derive_slug("!!!"), "game")


class ExecutableDetectionTests(unittest.TestCase):
    def setUp(self) -> None:
        self._tmp = TemporaryDirectory()
        self.root = Path(self._tmp.name)

    def tearDown(self) -> None:
        self._tmp.cleanup()

    def test_exe_is_executable(self) -> None:
        p = self.root / "clone-game.exe"
        p.touch()
        self.assertTrue(ship._looks_like_executable(p))

    def test_d_files_ignored(self) -> None:
        p = self.root / "clone-game.d"
        p.touch()
        self.assertFalse(ship._looks_like_executable(p))

    def test_pdb_files_ignored(self) -> None:
        p = self.root / "clone-game.pdb"
        p.touch()
        self.assertFalse(ship._looks_like_executable(p))

    def test_posix_extensionless_is_executable(self) -> None:
        p = self.root / "clone-game"
        p.touch()
        self.assertTrue(ship._looks_like_executable(p))


class ReadPackageNameTests(unittest.TestCase):
    def setUp(self) -> None:
        self._tmp = TemporaryDirectory()
        self.root = Path(self._tmp.name)

    def tearDown(self) -> None:
        self._tmp.cleanup()

    def test_reads_name_from_package_block(self) -> None:
        cargo = self.root / "Cargo.toml"
        _write(
            cargo,
            '[package]\nname = "clone-game"\nversion = "0.1.0"\n',
        )
        self.assertEqual(ship._read_package_name(cargo), "clone-game")

    def test_ignores_dependencies_name(self) -> None:
        cargo = self.root / "Cargo.toml"
        _write(
            cargo,
            (
                '[package]\nname = "clone-game"\n'
                '[dependencies]\nbevy = { name = "bevy_dep_name", version = "0.15" }\n'
            ),
        )
        self.assertEqual(ship._read_package_name(cargo), "clone-game")

    def test_missing_file_returns_none(self) -> None:
        self.assertIsNone(ship._read_package_name(self.root / "no-such.toml"))


class CountImplementationsTests(unittest.TestCase):
    def setUp(self) -> None:
        self._tmp = TemporaryDirectory()
        self.path = Path(self._tmp.name) / "system_map.yaml"

    def tearDown(self) -> None:
        self._tmp.cleanup()

    def test_entity_and_system_counts(self) -> None:
        self.path.write_text(
            """
implemented:
- id: soldier
  file: src/unit/soldier.rs
- id: game_state_machine
  file: src/system/game_state_machine.rs
- id: input_handler
  file: src/system/input_handler.rs
- id: farm
  file: src/building/farm.rs
""",
            encoding="utf-8",
        )
        entity, system = ship._count_implementations(self.path)
        self.assertEqual((entity, system), (2, 2))

    def test_missing_file_returns_zeros(self) -> None:
        entity, system = ship._count_implementations(self.path)
        self.assertEqual((entity, system), (0, 0))


class CopyTreeTests(unittest.TestCase):
    def setUp(self) -> None:
        self._tmp = TemporaryDirectory()
        self.root = Path(self._tmp.name)

    def tearDown(self) -> None:
        self._tmp.cleanup()

    def test_copies_files_and_recurses(self) -> None:
        src = self.root / "src"
        dst = self.root / "dst"
        _write(src / "a.rs", "a\n")
        _write(src / "sub" / "b.rs", "b\n")
        n = ship.copy_tree(src, dst)
        self.assertEqual(n, 2)
        self.assertEqual((dst / "a.rs").read_text(), "a\n")
        self.assertEqual((dst / "sub" / "b.rs").read_text(), "b\n")

    def test_skip_names_excludes_dir(self) -> None:
        src = self.root / "src"
        dst = self.root / "dst"
        _write(src / "a.rs", "a\n")
        _write(src / "target" / "build.txt", "skip me\n")
        ship.copy_tree(src, dst, skip={"target"})
        self.assertFalse((dst / "target").exists())
        self.assertTrue((dst / "a.rs").exists())

    def test_missing_source_returns_zero(self) -> None:
        self.assertEqual(ship.copy_tree(self.root / "no-such", self.root / "dst"), 0)


class ShipEndToEndTests(unittest.TestCase):
    """Run ``ship.ship(...)`` against a synthetic working tree (no cargo).

    --skip-binary lets the test exercise every other code path: source/
    snapshot, vault/ snapshot, game-config copy, README generation,
    .gitignore convention.
    """

    def setUp(self) -> None:
        self._tmp = TemporaryDirectory()
        self.root = Path(self._tmp.name)
        # Working tree
        self.game_dir = self.root / "game"
        _write(self.game_dir / "Cargo.toml", '[package]\nname = "clone-game"\n')
        _write(self.game_dir / "src" / "lib.rs", "pub mod sim;\n")
        _write(self.game_dir / "src" / "sim.rs", "// sim\n")
        _write(self.game_dir / "target" / "release" / "ignored", "")
        self.vault_dir = self.root / "vault"
        _write(self.vault_dir / "unit" / "soldier.md", "---\nid: soldier\n---\n")
        self.config = self.root / "game-config.json"
        self.config.write_text(
            json.dumps({"game": {"name": "Lethal Company"}, "chosen_engine": {"name": "Bevy"}}),
            encoding="utf-8",
        )
        self.system_map = self.root / "build" / "system_map.yaml"
        _write(self.system_map, "implemented:\n- id: soldier\n  file: src/unit/soldier.rs\n")
        self.releases = self.root / "releases"

    def tearDown(self) -> None:
        self._tmp.cleanup()

    def test_ships_to_derived_slug(self) -> None:
        rc = ship.ship(
            game_config=self.config,
            game_dir=self.game_dir,
            vault_dir=self.vault_dir,
            system_map_path=self.system_map,
            releases_root=self.releases,
            skip_binary=True,
        )
        self.assertEqual(rc, 0)
        target = self.releases / "lethal-company"
        self.assertTrue((target / "source" / "src" / "lib.rs").is_file())
        self.assertTrue((target / "source" / "Cargo.toml").is_file())
        self.assertTrue((target / "vault" / "unit" / "soldier.md").is_file())
        self.assertTrue((target / "game-config.json").is_file())
        self.assertTrue((target / "system_map.yaml").is_file())
        self.assertTrue((target / "README.md").is_file())
        # target/ inside game/ must be excluded
        self.assertFalse((target / "source" / "target").exists())
        # .gitignore at releases root keeps binaries out of git
        self.assertTrue((self.releases / ".gitignore").is_file())

    def test_refuses_to_overwrite_without_force(self) -> None:
        ship.ship(
            game_config=self.config,
            game_dir=self.game_dir,
            vault_dir=self.vault_dir,
            system_map_path=self.system_map,
            releases_root=self.releases,
            skip_binary=True,
        )
        rc = ship.ship(
            game_config=self.config,
            game_dir=self.game_dir,
            vault_dir=self.vault_dir,
            system_map_path=self.system_map,
            releases_root=self.releases,
            skip_binary=True,
        )
        self.assertEqual(rc, 1)

    def test_force_overwrites(self) -> None:
        ship.ship(
            game_config=self.config,
            game_dir=self.game_dir,
            vault_dir=self.vault_dir,
            system_map_path=self.system_map,
            releases_root=self.releases,
            skip_binary=True,
        )
        _write(self.game_dir / "src" / "newfile.rs", "// new\n")
        rc = ship.ship(
            game_config=self.config,
            game_dir=self.game_dir,
            vault_dir=self.vault_dir,
            system_map_path=self.system_map,
            releases_root=self.releases,
            force=True,
            skip_binary=True,
        )
        self.assertEqual(rc, 0)
        self.assertTrue(
            (self.releases / "lethal-company" / "source" / "src" / "newfile.rs").is_file()
        )

    def test_slug_override(self) -> None:
        rc = ship.ship(
            game_config=self.config,
            game_dir=self.game_dir,
            vault_dir=self.vault_dir,
            system_map_path=self.system_map,
            releases_root=self.releases,
            slug_override="custom-name",
            skip_binary=True,
        )
        self.assertEqual(rc, 0)
        self.assertTrue((self.releases / "custom-name" / "source" / "Cargo.toml").is_file())


if __name__ == "__main__":
    unittest.main()
