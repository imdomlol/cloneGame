import json
import os
import re
import subprocess
import sys
import tempfile
from typing import Any


REQUIRED_FIELDS_BY_KIND = {
    "item": ["stats", "requirements", "tags"],
    "skill": ["cost", "cooldown", "effects", "scaling"],
    "enemy": ["stats", "loot_table", "ai_profile", "resistances"],
    "mechanic": ["formula", "inputs", "outputs", "edge_cases"],
    "quest": ["prerequisites", "objectives", "rewards", "flags_set"],
    "system": ["controlled_entities", "state_transitions"],
}


def _snake_case(text: str) -> str:
    text = text.strip().lower()
    text = re.sub(r"[^a-z0-9]+", "_", text)
    text = re.sub(r"_+", "_", text).strip("_")
    return text or "kind"


DEFAULT_MODE = "claude"
DEFAULT_CLAUDE_MODEL = "claude-haiku-4-5-20251001"
SUPPORTED_MODES = {"claude", "codex"}


def _run_llm_command(cmd: list[str], prompt: str, label: str) -> subprocess.CompletedProcess[str]:
    try:
        return subprocess.run(
            cmd,
            input=prompt,
            capture_output=True,
            text=True,
            encoding="utf-8",
        )
    except FileNotFoundError:
        print(
            f"Error: '{cmd[0]}' CLI not found on PATH for {label} mode.",
            file=sys.stderr,
        )
        sys.exit(1)


def _claude_call(prompt: str, model: str | None) -> str:
    selected_model = model or DEFAULT_CLAUDE_MODEL
    cmd = ["claude", "-p", "--model", selected_model]
    result = _run_llm_command(cmd, prompt, "claude")
    if result.returncode != 0:
        raise ValueError(
            f"claude -p exited {result.returncode}:\n"
            f"  stderr: {result.stderr.strip()}\n"
            f"  stdout: {result.stdout.strip()}"
        )

    out = result.stdout.strip()
    if not out:
        raise ValueError(
            f"claude -p returned empty output.\n"
            f"  stderr: {result.stderr.strip()}"
        )

    return out


def _codex_call(prompt: str, model: str | None) -> str:
    codex_cmd = "codex.cmd" if os.name == "nt" else "codex"
    output_path = ""
    try:
        with tempfile.NamedTemporaryFile("w", suffix=".txt", delete=False, encoding="utf-8") as f:
            output_path = f.name

        cmd = [
            codex_cmd,
            "-s",
            "read-only",
            "exec",
            "--output-last-message",
            output_path,
        ]
        if model:
            cmd.extend(["--model", model])
        cmd.append("-")

        result = _run_llm_command(cmd, prompt, "codex")
        if result.returncode != 0:
            raise ValueError(
                f"codex exec exited {result.returncode}:\n"
                f"  stderr: {result.stderr.strip()}\n"
                f"  stdout: {result.stdout.strip()}"
            )

        out = ""
        if output_path and os.path.exists(output_path):
            with open(output_path, "r", encoding="utf-8") as f:
                out = f.read().strip()
        if not out:
            out = result.stdout.strip()
        if not out:
            raise ValueError(
                f"codex exec returned empty output.\n"
                f"  stderr: {result.stderr.strip()}"
            )
        return out
    finally:
        if output_path:
            try:
                os.unlink(output_path)
            except OSError:
                pass


def _llm_call(prompt: str, mode: str = DEFAULT_MODE, model: str | None = None) -> str:
    normalized_mode = mode.strip().lower()
    if normalized_mode == "claude":
        return _claude_call(prompt, model)
    if normalized_mode == "codex":
        return _codex_call(prompt, model)
    expected = ", ".join(sorted(SUPPORTED_MODES))
    raise ValueError(f"Unsupported LLM mode: {mode!r}. Expected one of: {expected}.")


BASELINE_KINDS = ["item", "skill", "enemy", "mechanic", "location", "npc", "quest", "system"]


def _build_prompt(categories: list[dict[str, Any]]) -> str:
    trimmed: list[dict[str, Any]] = []
    for cat in categories:
        members = cat.get("members") or []
        trimmed.append(
            {
                "name": cat.get("name"),
                "member_count": cat.get("member_count"),
                "sample_members": members,
            }
        )

    required_lines = []
    for kind, fields in REQUIRED_FIELDS_BY_KIND.items():
        required_lines.append(f"- {kind}: {', '.join(fields)}")
    required_block = "\n".join(required_lines)

    baseline_block = ", ".join(BASELINE_KINDS)

    return (
        "You are given a MediaWiki category list for the game 'They Are Billions'.\n"
        "Each category has a name, member_count, and sample member page titles for context.\n"
        "\n"
        "Task:\n"
        "1) Discard any remaining meta/community/maintenance categories you recognise semantically.\n"
        "2) Map surviving categories into a `kinds` object. Use fine-grained kinds - do NOT collapse\n"
        "   distinct entity types (e.g. buildings, heroes, enemies, locations) into a single catch-all.\n"
        f"   Baseline kinds to start from (keep all that have evidence in the categories): {baseline_block}\n"
        "   You may add new kinds if the wiki has entity types not covered by the baseline.\n"
        "   Each kind must have:\n"
        "   - a snake_case key\n"
        "   - `minWikilinks` as an int 0..3 (higher for more interconnected entity types)\n"
        "   - a one-sentence `description` that explicitly names the required frontmatter sub-fields for that kind.\n"
        "     Required fields by kind:\n"
        f"{required_block}\n"
        "3) For `categories`: emit one entry per surviving input category, classifying it to a kind.\n"
        "   - Category `name` must be VERBATIM from the input (case-sensitive, no paraphrasing).\n"
        "   - `kind` must be one of the kinds you defined above.\n"
        "   - This list will drive Phase 1 ingest: every member page of these categories will be\n"
        "     fetched from the MediaWiki API and routed to vault/<kind>/.\n"
        "\n"
        "Return ONLY raw JSON (no markdown, no prose) in this exact shape:\n"
        "{\n"
        '  "kinds": {\n'
        '    "<snake_case_kind>": {"minWikilinks": 1, "description": "..."}\n'
        "  },\n"
        '  "categories": [\n'
        '    {"name": "Exact Category Name", "kind": "<snake_case_kind>"}\n'
        "  ]\n"
        "}\n"
        "\n"
        "Input categories JSON:\n"
        f"{json.dumps(trimmed, ensure_ascii=False)}"
    )


def _strip_fences(text: str) -> str:
    """Remove leading ```json / ``` fences that models add despite instructions."""
    text = text.strip()
    if text.startswith("```"):
        text = re.sub(r"^```[a-z]*\n?", "", text)
        text = re.sub(r"\n?```$", "", text.rstrip())
    return text.strip()


def _parse_json_with_retry(
    prompt: str,
    mode: str = DEFAULT_MODE,
    model: str | None = None,
) -> dict[str, Any]:
    last_err: Exception | None = None
    for attempt in range(2):
        p = prompt if attempt == 0 else (prompt + "\n\nOutput raw JSON only, no markdown.")
        content = _strip_fences(_llm_call(p, mode, model))
        try:
            return json.loads(content)
        except Exception as e:  # noqa: BLE001
            last_err = e
    assert last_err is not None
    raise ValueError(f"LLM returned invalid JSON after retry: {last_err}")


def _validate_output(result: dict[str, Any], categories: list[dict[str, Any]]) -> dict[str, Any]:
    if not isinstance(result, dict):
        raise ValueError("Expected result to be a JSON object.")
    if "kinds" not in result or "categories" not in result:
        raise ValueError('Expected top-level keys: "kinds" and "categories".')
    if not isinstance(result["kinds"], dict):
        raise ValueError('"kinds" must be an object.')
    if not isinstance(result["categories"], list):
        raise ValueError('"categories" must be an array.')

    name_to_count: dict[str, int] = {}
    for cat in categories:
        name = cat.get("name")
        if isinstance(name, str):
            try:
                name_to_count[name] = int(cat.get("member_count") or 0)
            except (TypeError, ValueError):
                name_to_count[name] = 0

    normalized_kinds: dict[str, dict[str, Any]] = {}
    kind_key_map: dict[str, str] = {}
    for key, val in result["kinds"].items():
        norm_key = _snake_case(str(key))
        kind_key_map[str(key)] = norm_key
        if not isinstance(val, dict):
            raise ValueError(f'Kind "{key}" must be an object.')
        min_wikilinks = int(val.get("minWikilinks", 0))
        min_wikilinks = max(0, min(3, min_wikilinks))
        description = str(val.get("description", "")).strip()
        normalized_kinds[norm_key] = {"minWikilinks": min_wikilinks, "description": description}

    normalized_categories: list[dict[str, Any]] = []
    for entry in result["categories"]:
        if not isinstance(entry, dict):
            continue
        name = entry.get("name")
        if not isinstance(name, str) or name not in name_to_count:
            raise ValueError(f'Category name must be verbatim from input. Bad name: {name!r}')
        kind = entry.get("kind")
        kind_str = str(kind) if kind is not None else ""
        kind_norm = kind_key_map.get(kind_str, _snake_case(kind_str))
        if kind_norm not in normalized_kinds:
            raise ValueError(f'Category {name!r} maps to unknown kind: {kind_norm!r}')
        normalized_categories.append(
            {"name": name, "kind": kind_norm, "member_count": name_to_count[name]}
        )

    return {"kinds": normalized_kinds, "categories": normalized_categories}


def analyze_taxonomy(
    categories: list[dict],
    mode: str = DEFAULT_MODE,
    model: str | None = None,
) -> dict:
    prompt = _build_prompt(categories)
    raw = _parse_json_with_retry(prompt, mode, model)
    return _validate_output(raw, categories)


def _main() -> int:
    try:
        categories = json.loads(sys.stdin.read() or "[]")
        if not isinstance(categories, list):
            raise ValueError("stdin JSON must be a list (output of phase0_fetch.fetch_taxonomy()).")
        mode = os.getenv("PHASE0_LLM_MODE", DEFAULT_MODE)
        model = os.getenv("PHASE0_LLM_MODEL")
        result = analyze_taxonomy(categories, mode=mode, model=model)
        sys.stdout.write(json.dumps(result, ensure_ascii=False, indent=2))
        sys.stdout.write("\n")
        return 0
    except SystemExit:
        raise
    except Exception as e:  # noqa: BLE001
        print(f"Error: {e}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    raise SystemExit(_main())
