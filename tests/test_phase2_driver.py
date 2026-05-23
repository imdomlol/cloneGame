"""Unit tests for phase2.driver. The codegen.generate seam is monkeypatched
so no API call is made; system_map round-trips through a tempfile.
"""

from __future__ import annotations

import unittest
from pathlib import Path
from tempfile import TemporaryDirectory
from typing import Any

from phase2 import driver, system_map


def _fake_summary(
    *,
    response_text: str,
    ok: bool,
    offending: list[str] | None = None,
) -> dict[str, Any]:
    return {
        "engine_baseline_tokens": 100,
        "user_message_tokens": 200,
        "included_vault_ids": ["ranger"],
        "allowed_paths": ["vault/unit/ranger.md"],
        "response_text": response_text,
        "usage": None,
        "sources_header_ok": ok,
        "sources_header_offending": offending or [],
    }


class RunTurnTests(unittest.TestCase):
    def setUp(self) -> None:
        self._tmp = TemporaryDirectory()
        self.addCleanup(self._tmp.cleanup)
        self.tmp = Path(self._tmp.name)
        self.state_path = self.tmp / "system_map.yaml"
        self.baseline_path = self.tmp / "baseline.md"
        self.baseline_path.write_text("baseline content", encoding="utf-8")
        self.output = self.tmp / "out" / "ranger.rs"

    def _generate_stub(self, summary: dict) -> Any:
        def stub(task, *, llm_mode=None, model=None, baseline_path=None, dry_run=False):
            return summary

        return stub

    def test_writes_file_and_records_implementation_on_success(self) -> None:
        text = "// Sources: vault/unit/ranger.md\nfn main() {}\n"
        summary = _fake_summary(response_text=text, ok=True)
        result = driver.run_turn(
            "ranger",
            self.output,
            "build it",
            generate=self._generate_stub(summary),
            state_path=self.state_path,
            baseline_path=self.baseline_path,
        )
        self.assertEqual(result["written_to"], str(self.output))
        self.assertEqual(self.output.read_text(encoding="utf-8"), text)
        state = system_map.load_state(self.state_path)
        self.assertEqual(state["implemented"][0]["id"], "ranger")
        self.assertEqual(state["implemented"][0]["file"], str(self.output))
        self.assertEqual(state["implemented"][0]["verified_against"], "vault/unit/ranger.md")
        self.assertNotEqual(state["last_engine_baseline_hash"], "")

    def test_skips_write_and_records_pending_on_offending_paths(self) -> None:
        text = "// Sources: vault/unit/wizard.md\nfn main(){}\n"
        summary = _fake_summary(response_text=text, ok=False, offending=["vault/unit/wizard.md"])
        result = driver.run_turn(
            "ranger",
            self.output,
            "build it",
            generate=self._generate_stub(summary),
            state_path=self.state_path,
            baseline_path=self.baseline_path,
        )
        self.assertIsNone(result["written_to"])
        self.assertFalse(self.output.exists())
        state = system_map.load_state(self.state_path)
        self.assertEqual(state["pending"][0]["id"], "ranger")
        self.assertEqual(state["pending"][0]["blocked_by"], ["vault/unit/wizard.md"])
        self.assertEqual(state["implemented"], [])

    def test_missing_header_records_sentinel_blocker(self) -> None:
        summary = _fake_summary(response_text="fn main(){}", ok=False, offending=[])
        driver.run_turn(
            "ranger",
            self.output,
            "build it",
            generate=self._generate_stub(summary),
            state_path=self.state_path,
            baseline_path=self.baseline_path,
        )
        state = system_map.load_state(self.state_path)
        self.assertEqual(state["pending"][0]["blocked_by"], ["missing_sources_header"])

    def test_dry_run_short_circuits(self) -> None:
        summary = {
            "engine_baseline_tokens": 100,
            "user_message_tokens": 200,
            "included_vault_ids": ["ranger"],
            "allowed_paths": ["vault/unit/ranger.md"],
            "prompt": "[CURRENT RECREATION PROGRESS]...",
        }
        result = driver.run_turn(
            "ranger",
            self.output,
            "build it",
            generate=self._generate_stub(summary),
            state_path=self.state_path,
            baseline_path=self.baseline_path,
            dry_run=True,
        )
        self.assertEqual(result, summary)
        self.assertFalse(self.output.exists())
        self.assertFalse(self.state_path.exists())


if __name__ == "__main__":
    unittest.main()
