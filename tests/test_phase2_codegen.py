"""Unit tests for phase2.codegen helpers.

Covers prompt assembly, source-header validation, cache-hit math, and the
three-way dispatch (claude / codex CLI vs sdk SDK). All backends are
mocked end-to-end so tests never spend tokens.
"""

from __future__ import annotations

import io
import unittest
from types import SimpleNamespace
from unittest.mock import MagicMock

from phase2.codegen import (
    CACHE_HIT_RATE_WARN_THRESHOLD,
    DEFAULT_LLM_MODE,
    LLM_MODE_CLAUDE,
    LLM_MODE_CODEX,
    LLM_MODE_SDK,
    LLM_MODES,
    build_cli_prompt,
    build_user_message,
    cache_hit_rate,
    call_anthropic,
    call_cli,
    default_model,
    extract_allowed_paths,
    extract_source_paths,
    log_usage,
    validate_source_header,
)


class BuildUserMessageTests(unittest.TestCase):
    def test_includes_all_four_sections(self) -> None:
        msg = build_user_message("map yaml", "chunks", "build the ranger")
        for section in (
            "[CURRENT RECREATION PROGRESS]",
            "[SANITIZED OBSIDIAN VAULT SPECIFICATION]",
            "[TRANSLATION CONSTRAINTS]",
            "[DEVELOPMENT GOAL]",
        ):
            self.assertIn(section, msg)

    def test_empty_system_map_renders_stub(self) -> None:
        msg = build_user_message("", "chunks", "task")
        self.assertIn("(no prior turns)", msg)

    def test_task_lands_in_goal_section(self) -> None:
        msg = build_user_message("m", "c", "build the ranger")
        goal_idx = msg.index("[DEVELOPMENT GOAL]")
        self.assertIn("build the ranger", msg[goal_idx:])


class BuildCliPromptTests(unittest.TestCase):
    def test_prepends_engine_baseline_section(self) -> None:
        out = build_cli_prompt("baseline body", "user body")
        self.assertTrue(out.startswith("[ENGINE BASELINE]\n"))
        self.assertIn("baseline body", out)
        self.assertIn("user body", out)

    def test_baseline_and_user_separated(self) -> None:
        out = build_cli_prompt("ZZZBASELINEZZZ", "QQQUSERQQQ")
        baseline_end = out.index("ZZZBASELINEZZZ") + len("ZZZBASELINEZZZ")
        user_start = out.index("QQQUSERQQQ")
        self.assertLess(baseline_end, user_start)


class DefaultModelTests(unittest.TestCase):
    def test_each_mode_has_a_default(self) -> None:
        for mode in LLM_MODES:
            self.assertTrue(default_model(mode))

    def test_unknown_mode_raises(self) -> None:
        with self.assertRaises(ValueError):
            default_model("hypothetical")

    def test_default_mode_tracks_pipeline_config(self) -> None:
        # The codegen default is sourced from pipeline.config.toml (a CLI mode,
        # no API key). It must be one of the CLI backends, never sdk (which would
        # require ANTHROPIC_API_KEY).
        from model_config import default_llm_mode

        self.assertEqual(DEFAULT_LLM_MODE, default_llm_mode("phase2_codegen"))
        self.assertIn(DEFAULT_LLM_MODE, (LLM_MODE_CLAUDE, LLM_MODE_CODEX))


class ExtractAllowedPathsTests(unittest.TestCase):
    def test_pulls_path_attribute_from_repomix_envelope(self) -> None:
        chunks = (
            '<file path="vault/unit/ranger.md">\nbody\n</file>\n'
            '<file path="vault/item/bow.md">\nmore\n</file>'
        )
        self.assertEqual(
            extract_allowed_paths(chunks),
            {"vault/unit/ranger.md", "vault/item/bow.md"},
        )

    def test_empty_input_returns_empty_set(self) -> None:
        self.assertEqual(extract_allowed_paths(""), set())


class ExtractSourcePathsTests(unittest.TestCase):
    def test_handles_slash_slash_header(self) -> None:
        text = "// Sources: vault/unit/ranger.md, vault/item/bow.md\nfn main() {}"
        self.assertEqual(
            extract_source_paths(text),
            ["vault/unit/ranger.md", "vault/item/bow.md"],
        )

    def test_handles_hash_header(self) -> None:
        text = "# Sources: vault/x.md\nprint('hi')\n"
        self.assertEqual(extract_source_paths(text), ["vault/x.md"])

    def test_strips_backticks(self) -> None:
        text = "// Sources: `vault/a.md`, `vault/b.md`\n"
        self.assertEqual(extract_source_paths(text), ["vault/a.md", "vault/b.md"])

    def test_returns_empty_when_no_header(self) -> None:
        self.assertEqual(extract_source_paths("fn main(){}"), [])


class ValidateSourceHeaderTests(unittest.TestCase):
    def setUp(self) -> None:
        self.allowed = {"vault/unit/ranger.md", "vault/item/bow.md"}

    def test_ok_when_all_paths_in_allowed(self) -> None:
        text = "// Sources: vault/unit/ranger.md, vault/item/bow.md\n"
        self.assertEqual(validate_source_header(text, self.allowed), (True, []))

    def test_not_ok_when_header_missing(self) -> None:
        ok, offending = validate_source_header("fn main(){}", self.allowed)
        self.assertFalse(ok)
        self.assertEqual(offending, [])

    def test_offending_paths_listed(self) -> None:
        text = "// Sources: vault/unit/ranger.md, vault/unit/wizard.md\n"
        ok, offending = validate_source_header(text, self.allowed)
        self.assertFalse(ok)
        self.assertEqual(offending, ["vault/unit/wizard.md"])


class CacheHitRateTests(unittest.TestCase):
    def test_pure_cache_read_is_one(self) -> None:
        usage = SimpleNamespace(
            cache_read_input_tokens=2000,
            cache_creation_input_tokens=0,
            input_tokens=0,
        )
        self.assertAlmostEqual(cache_hit_rate(usage), 1.0)

    def test_first_turn_is_zero(self) -> None:
        usage = SimpleNamespace(
            cache_read_input_tokens=0,
            cache_creation_input_tokens=2000,
            input_tokens=500,
        )
        self.assertEqual(cache_hit_rate(usage), 0.0)

    def test_mixed_turn(self) -> None:
        usage = SimpleNamespace(
            cache_read_input_tokens=1800,
            cache_creation_input_tokens=0,
            input_tokens=200,
        )
        self.assertAlmostEqual(cache_hit_rate(usage), 0.9)

    def test_empty_usage_is_zero(self) -> None:
        self.assertEqual(cache_hit_rate(SimpleNamespace()), 0.0)


class LogUsageTests(unittest.TestCase):
    def test_warns_below_threshold(self) -> None:
        usage = SimpleNamespace(
            cache_read_input_tokens=0,
            cache_creation_input_tokens=2000,
            input_tokens=500,
            output_tokens=100,
        )
        buf = io.StringIO()
        log_usage(usage, stream=buf)
        text = buf.getvalue()
        self.assertIn("hit_rate=0.00%", text)
        self.assertIn("warn:", text)

    def test_quiet_when_above_threshold(self) -> None:
        usage = SimpleNamespace(
            cache_read_input_tokens=2000,
            cache_creation_input_tokens=0,
            input_tokens=100,
            output_tokens=80,
        )
        buf = io.StringIO()
        log_usage(usage, stream=buf)
        text = buf.getvalue()
        self.assertIn("hit_rate=95.24%", text)
        self.assertNotIn("warn:", text)

    def test_cli_mode_usage_none_logs_unavailable(self) -> None:
        buf = io.StringIO()
        log_usage(None, stream=buf)
        self.assertIn("unavailable in CLI mode", buf.getvalue())

    def test_threshold_is_eighty_percent(self) -> None:
        self.assertEqual(CACHE_HIT_RATE_WARN_THRESHOLD, 0.80)


class CallAnthropicTests(unittest.TestCase):
    def test_passes_cache_control_on_system_block(self) -> None:
        client = MagicMock()
        client.messages.create.return_value = SimpleNamespace(
            content=[SimpleNamespace(text="hi")],
            usage=SimpleNamespace(input_tokens=1),
        )
        text, usage = call_anthropic(client, "baseline txt", "user msg", model="m", max_tokens=42)
        self.assertEqual(text, "hi")
        self.assertEqual(usage.input_tokens, 1)
        kwargs = client.messages.create.call_args.kwargs
        self.assertEqual(kwargs["model"], "m")
        self.assertEqual(kwargs["max_tokens"], 42)
        self.assertEqual(kwargs["system"][0]["text"], "baseline txt")
        self.assertEqual(kwargs["system"][0]["cache_control"], {"type": "ephemeral"})
        self.assertEqual(kwargs["messages"][0]["role"], "user")
        self.assertEqual(kwargs["messages"][0]["content"], "user msg")


class CallCliTests(unittest.TestCase):
    def test_passes_prompt_mode_and_model_to_runner(self) -> None:
        captured: dict = {}

        def fake_runner(prompt, mode, model):
            captured["prompt"] = prompt
            captured["mode"] = mode
            captured["model"] = model
            return "generated text"

        text, usage = call_cli("the prompt", LLM_MODE_CODEX, "gpt-x", runner=fake_runner)
        self.assertEqual(text, "generated text")
        self.assertIsNone(usage)
        self.assertEqual(captured["prompt"], "the prompt")
        self.assertEqual(captured["mode"], LLM_MODE_CODEX)
        self.assertEqual(captured["model"], "gpt-x")


class LlmModeConstantsTests(unittest.TestCase):
    def test_constants_have_expected_strings(self) -> None:
        self.assertEqual(LLM_MODE_CLAUDE, "claude")
        self.assertEqual(LLM_MODE_CODEX, "codex")
        self.assertEqual(LLM_MODE_SDK, "sdk")

    def test_modes_tuple_covers_all_three(self) -> None:
        self.assertEqual(set(LLM_MODES), {LLM_MODE_CLAUDE, LLM_MODE_CODEX, LLM_MODE_SDK})


if __name__ == "__main__":
    unittest.main()
