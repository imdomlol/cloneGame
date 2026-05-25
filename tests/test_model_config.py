import unittest
from pathlib import Path
from tempfile import TemporaryDirectory

from scripts.model_config import (
    default_llm_mode,
    default_model,
    load_model_config,
    resolve_llm,
)


class ModelConfigTests(unittest.TestCase):
    def test_load_merges_user_config_over_fallbacks(self) -> None:
        with TemporaryDirectory() as tmp:
            path = Path(tmp) / "pipeline.config.toml"
            path.write_text(
                '[phase1]\nllm_mode = "claude"\n\n[phase1.models]\nclaude = "custom-claude"\n',
                encoding="utf-8",
            )

            config = load_model_config(path)

        self.assertEqual(default_llm_mode("phase1", config=config), "claude")
        self.assertEqual(default_model("phase1", "claude", config=config), "custom-claude")
        self.assertEqual(default_model("phase1", "codex", config=config), "gpt-5.4-mini")

    def test_resolve_llm_keeps_explicit_overrides(self) -> None:
        config = {
            "phase2_codegen": {
                "llm_mode": "claude",
                "models": {"claude": "configured-claude", "codex": "configured-codex"},
            }
        }

        mode, model = resolve_llm(
            "phase2_codegen",
            mode="codex",
            model="one-off-model",
            config=config,
        )

        self.assertEqual((mode, model), ("codex", "one-off-model"))


if __name__ == "__main__":
    unittest.main()
