"""Global AI model defaults for pipeline stages.

The repo-level ``pipeline.config.toml`` is the user-editable source of truth.
Callers may still pass an explicit CLI/model override; these helpers only
resolve defaults when no override is supplied.
"""

from __future__ import annotations

import tomllib
from copy import deepcopy
from pathlib import Path
from typing import Any

REPO_ROOT = Path(__file__).resolve().parents[1]
CONFIG_PATH = REPO_ROOT / "pipeline.config.toml"

FALLBACK_CONFIG: dict[str, Any] = {
    "phase0": {
        "llm_mode": "claude",
        "models": {
            "claude": "claude-haiku-4-5-20251001",
            "codex": "gpt-5.4-mini",
        },
    },
    "phase1": {
        "llm_mode": "codex",
        "models": {
            "claude": "claude-haiku-4-5-20251001",
            "codex": "gpt-5.4-mini",
        },
    },
    "phase2_codegen": {
        "llm_mode": "claude",
        "models": {
            "claude": "claude-sonnet-4-6",
            "codex": "gpt-5.4-mini",
            "sdk": "claude-sonnet-4-6",
        },
    },
    "phase2_retrieval": {"embedding_model": "BAAI/bge-small-en-v1.5"},
    "phase2_system_map": {"compactor_model": "claude-haiku-4-5-20251001"},
}


def _deep_merge(base: dict[str, Any], override: dict[str, Any]) -> dict[str, Any]:
    merged = deepcopy(base)
    for key, value in override.items():
        if isinstance(value, dict) and isinstance(merged.get(key), dict):
            merged[key] = _deep_merge(merged[key], value)
        else:
            merged[key] = value
    return merged


def load_model_config(path: Path = CONFIG_PATH) -> dict[str, Any]:
    """Load global model defaults, falling back to built-in safe values."""
    if not path.exists():
        return deepcopy(FALLBACK_CONFIG)
    loaded = tomllib.loads(path.read_text(encoding="utf-8"))
    return _deep_merge(FALLBACK_CONFIG, loaded)


def default_llm_mode(stage: str, *, config: dict[str, Any] | None = None) -> str:
    cfg = config or load_model_config()
    stage_cfg = cfg.get(stage)
    if not isinstance(stage_cfg, dict):
        raise ValueError(f"unknown model config stage: {stage}")
    mode = stage_cfg.get("llm_mode")
    if not isinstance(mode, str) or not mode:
        raise ValueError(f"missing llm_mode for stage: {stage}")
    return mode


def model_defaults(stage: str, *, config: dict[str, Any] | None = None) -> dict[str, str]:
    cfg = config or load_model_config()
    stage_cfg = cfg.get(stage)
    if not isinstance(stage_cfg, dict):
        raise ValueError(f"unknown model config stage: {stage}")
    models = stage_cfg.get("models")
    if not isinstance(models, dict):
        raise ValueError(f"missing models table for stage: {stage}")
    return {str(k): str(v) for k, v in models.items() if isinstance(v, str) and v}


def default_model(stage: str, mode: str, *, config: dict[str, Any] | None = None) -> str:
    models = model_defaults(stage, config=config)
    if mode not in models:
        raise ValueError(f"no default model configured for {stage}.{mode}")
    return models[mode]


def resolve_llm(
    stage: str,
    *,
    mode: str | None = None,
    model: str | None = None,
    config: dict[str, Any] | None = None,
) -> tuple[str, str]:
    """Return ``(mode, model)`` with explicit args winning over global defaults."""
    resolved_mode = mode or default_llm_mode(stage, config=config)
    resolved_model = model or default_model(stage, resolved_mode, config=config)
    return resolved_mode, resolved_model


def default_embedding_model(stage: str = "phase2_retrieval") -> str:
    cfg = load_model_config()
    stage_cfg = cfg.get(stage)
    if not isinstance(stage_cfg, dict):
        raise ValueError(f"unknown model config stage: {stage}")
    model = stage_cfg.get("embedding_model")
    if not isinstance(model, str) or not model:
        raise ValueError(f"missing embedding_model for stage: {stage}")
    return model


def default_compactor_model(stage: str = "phase2_system_map") -> str:
    cfg = load_model_config()
    stage_cfg = cfg.get(stage)
    if not isinstance(stage_cfg, dict):
        raise ValueError(f"unknown model config stage: {stage}")
    model = stage_cfg.get("compactor_model")
    if not isinstance(model, str) or not model:
        raise ValueError(f"missing compactor_model for stage: {stage}")
    return model
