"""Tests for phase2/entrypoint.py — the app-plugin aggregator generator.

Covers the three layers the loop driver depends on: plugin discovery from
on-disk Rust source, deterministic rendering with tuple chunking, and the
exclusion knob that lets a known-broken plugin be held out while a fix is in
flight. Touches the real filesystem via tmp_path so any change to the regex
or the path-to-module-path derivation surfaces in CI rather than the next
loop run.
"""

from __future__ import annotations

import sys
import unittest
from pathlib import Path

_REPO_ROOT = Path(__file__).resolve().parent.parent
if str(_REPO_ROOT / "phase2") not in sys.path:
    sys.path.insert(0, str(_REPO_ROOT / "phase2"))

from entrypoint import find_plugins, regenerate_aggregator, render_aggregator  # noqa: E402


def _write(path: Path, body: str) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(body, encoding="utf-8")


class FindPluginsTests(unittest.TestCase):
    def setUp(self) -> None:
        self.tmp = Path(self.id())  # not used; using a real tmpdir below
        import tempfile

        self._tmpdir = tempfile.TemporaryDirectory()
        self.game = Path(self._tmpdir.name)
        self.src = self.game / "src"

    def tearDown(self) -> None:
        self._tmpdir.cleanup()

    def test_extracts_plugin_struct_and_module_path(self) -> None:
        _write(
            self.src / "sim.rs",
            "pub struct SimChecksumPlugin;\nimpl Plugin for SimChecksumPlugin {}\n",
        )
        _write(
            self.src / "buildings" / "farm.rs",
            "pub struct FarmPlugin;\nimpl Plugin for FarmPlugin {\n    // body\n}\n",
        )
        result = find_plugins(self.game)
        self.assertEqual(
            result,
            [
                ("FarmPlugin", "crate::buildings::farm"),
                ("SimChecksumPlugin", "crate::sim"),
            ],
        )

    def test_skips_main_rs_and_mod_files(self) -> None:
        _write(self.src / "main.rs", "impl Plugin for ShouldBeIgnoredMainPlugin {}\n")
        _write(self.src / "buildings" / "mod.rs", "pub mod farm;\n")
        _write(
            self.src / "buildings" / "farm.rs",
            "impl Plugin for FarmPlugin {}\n",
        )
        names = [n for n, _ in find_plugins(self.game)]
        self.assertEqual(names, ["FarmPlugin"])

    def test_exclusion_drops_named_plugins(self) -> None:
        _write(
            self.src / "buildings" / "broken.rs",
            "impl Plugin for BrokenPlugin {}\n",
        )
        _write(
            self.src / "buildings" / "good.rs",
            "impl Plugin for GoodPlugin {}\n",
        )
        result = find_plugins(self.game, excluded={"BrokenPlugin": "B0001 conflict"})
        self.assertEqual([n for n, _ in result], ["GoodPlugin"])

    def test_dedup_keeps_first_file_alphabetically(self) -> None:
        # Two files declaring the same plugin name — first wins after sort.
        _write(self.src / "a.rs", "impl Plugin for DupPlugin {}\n")
        _write(self.src / "b.rs", "impl Plugin for DupPlugin {}\n")
        result = find_plugins(self.game)
        self.assertEqual(result, [("DupPlugin", "crate::a")])


class RenderAggregatorTests(unittest.TestCase):
    def test_empty_plugin_list_produces_compilable_noop(self) -> None:
        out = render_aggregator([])
        self.assertIn("pub fn add_all(app: &mut App) -> &mut App", out)
        self.assertIn("    app\n}", out)
        # No semicolons in the body when there are no .add_plugins calls.
        self.assertNotIn(";", out.split("pub fn add_all")[1])

    def test_chunking_respects_tuple_size(self) -> None:
        plugins = [(f"P{i}", "crate::x") for i in range(33)]
        out = render_aggregator(plugins, chunk_size=14)
        # 33 plugins / 14 = 3 chunks (14 + 14 + 5).
        self.assertEqual(out.count(".add_plugins"), 3)
        self.assertIn("(P0, P1, P2, P3, P4, P5, P6, P7, P8, P9, P10, P11, P12, P13)", out)
        self.assertIn(".add_plugins((P28, P29, P30, P31, P32))", out)

    def test_single_plugin_renders_without_tuple(self) -> None:
        out = render_aggregator([("OnlyPlugin", "crate::x")])
        self.assertIn(".add_plugins(OnlyPlugin)", out)
        self.assertNotIn(".add_plugins((", out)

    def test_no_trailing_semicolon_so_return_expression_flows(self) -> None:
        # Rust function body's trailing expression must not end in `;` for the
        # `&mut App` to return. Regression guard.
        out = render_aggregator([("P", "crate::x"), ("Q", "crate::y")])
        body = out.split("pub fn add_all(app: &mut App) -> &mut App {")[1].split("}")[0]
        for line in body.splitlines():
            stripped = line.strip()
            if stripped.startswith(".add_plugins"):
                self.assertFalse(
                    stripped.endswith(";"),
                    f".add_plugins line must not end with `;`: {stripped!r}",
                )


class RegenerateAggregatorTests(unittest.TestCase):
    def setUp(self) -> None:
        import tempfile

        self._tmpdir = tempfile.TemporaryDirectory()
        self.game = Path(self._tmpdir.name)
        _write(self.game / "src" / "buildings" / "farm.rs", "impl Plugin for FarmPlugin {}\n")

    def tearDown(self) -> None:
        self._tmpdir.cleanup()

    def test_no_config_is_noop(self) -> None:
        self.assertIsNone(regenerate_aggregator(self.game, None))
        self.assertIsNone(regenerate_aggregator(self.game, {}))

    def test_writes_aggregator_to_configured_path(self) -> None:
        cfg = {"aggregator_file": "src/app_plugins.rs"}
        out_path = regenerate_aggregator(self.game, cfg)
        self.assertIsNotNone(out_path)
        body = out_path.read_text(encoding="utf-8")
        self.assertIn("use crate::buildings::farm::FarmPlugin;", body)
        self.assertIn(".add_plugins(FarmPlugin)", body)

    def test_idempotent_when_content_unchanged(self) -> None:
        cfg = {"aggregator_file": "src/app_plugins.rs"}
        first = regenerate_aggregator(self.game, cfg)
        mtime_first = first.stat().st_mtime_ns
        # Second run must not rewrite the file (preserves Cargo's incremental
        # build cache when nothing has changed).
        second = regenerate_aggregator(self.game, cfg)
        self.assertEqual(first, second)
        self.assertEqual(mtime_first, second.stat().st_mtime_ns)


if __name__ == "__main__":
    unittest.main()
