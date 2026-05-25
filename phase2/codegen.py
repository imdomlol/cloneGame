"""Phase 2 codegen: assemble the 4-layer prompt and call a Claude backend.

Three dispatch modes, picked with ``--llm-mode``:

- ``claude`` (default): shells out to ``claude -p`` like Phase 1. Uses the
  user's existing Claude Code auth; no API key required.
- ``codex``: shells out to ``codex exec``. Same trade-offs, different vendor.
- ``sdk``: calls ``anthropic.Anthropic().messages.create`` with
  ``cache_control: ephemeral`` on the engine baseline. Requires
  ``ANTHROPIC_API_KEY``; only path that benefits from the prompt cache
  cost model documented in DEPLOYMENT_GUIDE §4.

CLI modes combine the engine baseline and the user message into one stdin
prompt because ``claude -p`` and ``codex exec`` take a single prompt blob.
The 4-section order from §3.5 is preserved::

    [ENGINE BASELINE]                      <- Layer 1 (engine_baseline.md)
    [CURRENT RECREATION PROGRESS]          <- Layer 4 (system_map.yaml)
    [SANITIZED OBSIDIAN VAULT SPECIFICATION] <- Layer 3 (retrieval bundle)
    [TRANSLATION CONSTRAINTS]
    [DEVELOPMENT GOAL]                     <- the task itself

SDK mode keeps the baseline in the ``system`` block so the cache hits.

Generated output is post-checked for a ``// Sources: ...`` header listing
only vault paths from the retrieval bundle, in all three modes.

CLI::

    python phase2/codegen.py "implement the ranger unit"
    python phase2/codegen.py --llm-mode codex "..."
    python phase2/codegen.py --llm-mode sdk --model claude-opus-4-7 "..."
    python phase2/codegen.py --dry-run "..."     # assemble prompt, skip call
"""

from __future__ import annotations

import argparse
import re
import sys
from pathlib import Path
from typing import Any

_REPO_ROOT = Path(__file__).resolve().parent.parent
if str(_REPO_ROOT / "phase2") not in sys.path:
    sys.path.insert(0, str(_REPO_ROOT / "phase2"))
if str(_REPO_ROOT / "scripts") not in sys.path:
    sys.path.insert(0, str(_REPO_ROOT / "scripts"))

from compile_cache import run_llm  # noqa: E402
from model_config import default_llm_mode, model_defaults  # noqa: E402
from retrieval import count_tokens, retrieve  # noqa: E402

LLM_MODE_CLAUDE = "claude"
LLM_MODE_CODEX = "codex"
LLM_MODE_SDK = "sdk"
LLM_MODES = (LLM_MODE_CLAUDE, LLM_MODE_CODEX, LLM_MODE_SDK)
DEFAULT_LLM_MODE = default_llm_mode("phase2_codegen")
_MODEL_DEFAULTS = model_defaults("phase2_codegen")

DEFAULT_MAX_TOKENS = 4096
DEFAULT_BASELINE_PATH = _REPO_ROOT / "build" / "engine_baseline.md"
DEFAULT_SYSTEM_MAP_PATH = _REPO_ROOT / "build" / "system_map.yaml"
CACHE_HIT_RATE_WARN_THRESHOLD = 0.80

_TRANSLATION_CONSTRAINTS = (
    "- YAML frontmatter is absolute truth for numbers. Do not round.\n"
    "- No placeholders, no shorthand, no external deps not in the baseline.\n"
    "- Every generated source file must begin with `// Sources: <vault paths>` "
    "matching the [SANITIZED OBSIDIAN VAULT SPECIFICATION] block."
)

_SOURCES_HEADER_RE = re.compile(r"^(?://|#|--)\s*Sources:\s*(?P<paths>.+?)\s*$", re.MULTILINE)
_FILE_PATH_RE = re.compile(r'<file path="([^"]+)">')


def default_model(llm_mode: str) -> str:
    """Return the documented default model id for ``llm_mode``."""
    if llm_mode not in _MODEL_DEFAULTS:
        raise ValueError(f"unsupported llm_mode: {llm_mode}")
    return _MODEL_DEFAULTS[llm_mode]


def build_user_message(system_map: str, vault_chunks: str, task: str) -> str:
    """Assemble the user-side prompt in the four documented sections."""
    return (
        f"[CURRENT RECREATION PROGRESS]\n{system_map.strip() or '(no prior turns)'}\n\n"
        f"[SANITIZED OBSIDIAN VAULT SPECIFICATION]\n{vault_chunks}\n\n"
        f"[TRANSLATION CONSTRAINTS]\n{_TRANSLATION_CONSTRAINTS}\n\n"
        f"[DEVELOPMENT GOAL]\n{task.strip()}"
    )


def build_cli_prompt(engine_baseline: str, user_message: str) -> str:
    """Combine baseline + user message for ``claude -p`` / ``codex exec``.

    The SDK path keeps these separate so ``cache_control: ephemeral`` can
    hit the baseline; the CLI tools take a single stdin blob, so we prepend
    the baseline as an ``[ENGINE BASELINE]`` section.
    """
    return f"[ENGINE BASELINE]\n{engine_baseline.strip()}\n\n{user_message}"


def extract_allowed_paths(vault_chunks: str) -> set[str]:
    """Pull every ``<file path="...">`` value out of the retrieval bundle."""
    return set(_FILE_PATH_RE.findall(vault_chunks))


def extract_source_paths(generated_text: str) -> list[str]:
    """Return every comma-separated path listed in ``// Sources: ...`` headers."""
    paths: list[str] = []
    for match in _SOURCES_HEADER_RE.finditer(generated_text):
        for raw in match.group("paths").split(","):
            cleaned = raw.strip().strip("`")
            if cleaned:
                paths.append(cleaned)
    return paths


def validate_source_header(
    generated_text: str,
    allowed_paths: set[str],
) -> tuple[bool, list[str]]:
    """``(ok, offending_paths)``: ok requires at least one header + zero extras.

    A "Sources" header is mandatory; the model is told so in the engine
    baseline output rules. ``offending_paths`` lists every path the model
    cited that was NOT in the retrieval bundle (hallucinated reference).
    """
    listed = extract_source_paths(generated_text)
    if not listed:
        return False, []
    offending = [p for p in listed if p not in allowed_paths]
    return (not offending), offending


def cache_hit_rate(usage: Any) -> float:
    """Cache-read tokens divided by total input tokens this turn, in ``[0, 1]``.

    Only meaningful for SDK mode; CLI mode usage objects are ``None`` so the
    caller should skip this entirely.
    """
    read = getattr(usage, "cache_read_input_tokens", 0) or 0
    create = getattr(usage, "cache_creation_input_tokens", 0) or 0
    fresh = getattr(usage, "input_tokens", 0) or 0
    total = read + create + fresh
    if total == 0:
        return 0.0
    return read / total


def log_usage(usage: Any, stream: Any = sys.stderr) -> None:
    """Print per-turn token + cache stats; warn if hit rate < 80% (SDK only)."""
    if usage is None:
        print("usage: <unavailable in CLI mode>", file=stream)
        return
    rate = cache_hit_rate(usage)
    read = getattr(usage, "cache_read_input_tokens", 0) or 0
    create = getattr(usage, "cache_creation_input_tokens", 0) or 0
    fresh = getattr(usage, "input_tokens", 0) or 0
    out = getattr(usage, "output_tokens", 0) or 0
    print(
        f"usage: input={fresh} cache_create={create} cache_read={read} "
        f"output={out} hit_rate={rate:.2%}",
        file=stream,
    )
    if rate < CACHE_HIT_RATE_WARN_THRESHOLD and (read + create + fresh) > 0:
        print(
            f"warn: cache hit rate {rate:.2%} < "
            f"{CACHE_HIT_RATE_WARN_THRESHOLD:.0%} threshold "
            "(first turn after engine-baseline edit / TTL expiry expected; "
            "sustained low rate erodes the cost model in DEPLOYMENT_GUIDE §4)",
            file=stream,
        )


def call_cli(
    prompt: str,
    llm_mode: str,
    model: str,
    runner: Any = run_llm,
) -> tuple[str, None]:
    """Subprocess to ``claude -p`` or ``codex exec``; returns ``(text, None)``.

    The trailing ``None`` matches ``call_anthropic``'s return shape so the
    dispatcher can treat both modes uniformly.
    """
    text = runner(prompt, llm_mode, model)
    return text, None


def call_anthropic(
    client: Any,
    engine_baseline: str,
    user_message: str,
    model: str,
    max_tokens: int = DEFAULT_MAX_TOKENS,
) -> tuple[str, Any]:
    """SDK call with ``cache_control: ephemeral`` on the baseline.

    Returns ``(generated_text, usage_object)`` so the dispatcher can treat
    both modes uniformly.
    """
    response = client.messages.create(
        model=model,
        max_tokens=max_tokens,
        system=[
            {
                "type": "text",
                "text": engine_baseline,
                "cache_control": {"type": "ephemeral"},
            }
        ],
        messages=[{"role": "user", "content": user_message}],
    )
    text = "".join(getattr(b, "text", "") for b in response.content)
    return text, response.usage


def _load_text(path: Path, default: str = "") -> str:
    return path.read_text(encoding="utf-8") if path.exists() else default


def _dispatch(
    engine_baseline: str,
    user_message: str,
    llm_mode: str,
    model: str,
    max_tokens: int,
    client: Any | None,
    cli_runner: Any,
) -> tuple[str, Any]:
    """Route to the right backend; returns ``(text, usage_or_None)``."""
    if llm_mode == LLM_MODE_SDK:
        if client is None:
            import anthropic

            client = anthropic.Anthropic()
        return call_anthropic(
            client, engine_baseline, user_message, model=model, max_tokens=max_tokens
        )
    if llm_mode in (LLM_MODE_CLAUDE, LLM_MODE_CODEX):
        prompt = build_cli_prompt(engine_baseline, user_message)
        return call_cli(prompt, llm_mode, model, runner=cli_runner)
    raise ValueError(f"unsupported llm_mode: {llm_mode}")


def generate(
    task: str,
    *,
    llm_mode: str = DEFAULT_LLM_MODE,
    model: str | None = None,
    max_tokens: int = DEFAULT_MAX_TOKENS,
    client: Any | None = None,
    cli_runner: Any = run_llm,
    baseline_path: Path = DEFAULT_BASELINE_PATH,
    system_map_path: Path = DEFAULT_SYSTEM_MAP_PATH,
    dry_run: bool = False,
) -> dict[str, Any]:
    """End-to-end codegen turn. Returns ``{prompt, response?, usage?, ...}``.

    With ``dry_run=True`` no backend call is made; useful for inspecting
    the assembled prompt or estimating token cost before spending.
    """
    engine_baseline = _load_text(baseline_path)
    if not engine_baseline:
        raise FileNotFoundError(f"{baseline_path} missing; run phase2/baseline.py first")
    system_map = _load_text(system_map_path)
    vault_chunks, included_ids = retrieve(task)
    allowed_paths = extract_allowed_paths(vault_chunks)
    user_message = build_user_message(system_map, vault_chunks, task)
    resolved_model = model or default_model(llm_mode)

    summary: dict[str, Any] = {
        "llm_mode": llm_mode,
        "model": resolved_model,
        "engine_baseline_tokens": count_tokens(engine_baseline),
        "user_message_tokens": count_tokens(user_message),
        "included_vault_ids": included_ids,
        "allowed_paths": sorted(allowed_paths),
    }

    if dry_run:
        summary["prompt"] = (
            user_message
            if llm_mode == LLM_MODE_SDK
            else build_cli_prompt(engine_baseline, user_message)
        )
        return summary

    text, usage = _dispatch(
        engine_baseline,
        user_message,
        llm_mode,
        resolved_model,
        max_tokens,
        client,
        cli_runner,
    )
    log_usage(usage)
    ok, offending = validate_source_header(text, allowed_paths)
    summary["response_text"] = text
    summary["usage"] = usage
    summary["sources_header_ok"] = ok
    summary["sources_header_offending"] = offending
    if not ok:
        print(
            "warn: generated output missing or invalid `// Sources:` header "
            f"(offending paths: {offending or '<none listed>'})",
            file=sys.stderr,
        )
    return summary


def _parse_args(argv: list[str] | None = None) -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.add_argument("task", help="task string fed to the codegen LLM")
    parser.add_argument(
        "--llm-mode",
        default=DEFAULT_LLM_MODE,
        choices=LLM_MODES,
        help="claude / codex (CLI, no API key) or sdk (Anthropic SDK + cache)",
    )
    parser.add_argument(
        "--model",
        default=None,
        help="override per-mode default model (see default_model)",
    )
    parser.add_argument("--max-tokens", type=int, default=DEFAULT_MAX_TOKENS)
    parser.add_argument("--baseline", type=Path, default=DEFAULT_BASELINE_PATH)
    parser.add_argument("--system-map", type=Path, default=DEFAULT_SYSTEM_MAP_PATH)
    parser.add_argument(
        "--output",
        type=Path,
        default=None,
        help="write response to this file (utf-8); otherwise print to stdout",
    )
    parser.add_argument(
        "--dry-run",
        action="store_true",
        help="assemble + print the prompt, skip the backend call",
    )
    return parser.parse_args(argv)


def main(argv: list[str] | None = None) -> int:
    args = _parse_args(argv)
    if hasattr(sys.stdout, "reconfigure"):
        sys.stdout.reconfigure(encoding="utf-8", errors="replace")
    result = generate(
        args.task,
        llm_mode=args.llm_mode,
        model=args.model,
        max_tokens=args.max_tokens,
        baseline_path=args.baseline,
        system_map_path=args.system_map,
        dry_run=args.dry_run,
    )
    if args.dry_run:
        print(result["prompt"])
        print(
            f"\n# dry-run: mode={result['llm_mode']} model={result['model']} "
            f"baseline={result['engine_baseline_tokens']} tok, "
            f"user={result['user_message_tokens']} tok, "
            f"included={len(result['included_vault_ids'])} files",
            file=sys.stderr,
        )
        return 0

    if args.output is not None:
        args.output.parent.mkdir(parents=True, exist_ok=True)
        args.output.write_text(result["response_text"], encoding="utf-8")
        print(f"wrote {args.output}", file=sys.stderr)
    else:
        print(result["response_text"])
    return 0


if __name__ == "__main__":
    sys.exit(main())
