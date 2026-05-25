import contextlib
import json
import os
import re
import subprocess
import sys
import tempfile
from typing import Any

try:
    from .model_config import default_llm_mode, default_model
except ImportError:  # pragma: no cover - direct script execution path
    from model_config import default_llm_mode, default_model


def _snake_case(text: str) -> str:
    text = re.sub(r"[^a-z0-9]+", "_", text.strip().lower())
    return re.sub(r"_+", "_", text).strip("_") or "kind"


DEFAULT_MODE = default_llm_mode("phase0")
DEFAULT_CLAUDE_MODEL = default_model("phase0", "claude")
SUPPORTED_MODES = {"claude", "codex"}
MAX_SCHEMA_SAMPLES_PER_CATEGORY, MAX_SCHEMA_SAMPLE_CHARS = 3, 1600
MIN_ENGINE_CANDIDATES, MAX_ENGINE_CANDIDATES = 2, 4


def _run_llm_command(cmd: list[str], prompt: str, label: str) -> subprocess.CompletedProcess[str]:
    try:
        return subprocess.run(cmd, input=prompt, capture_output=True, text=True, encoding="utf-8")
    except FileNotFoundError:
        print(
            f"Error: '{cmd[0]}' CLI not found on PATH for {label} mode.",
            file=sys.stderr,
        )
        sys.exit(1)


def _claude_call(prompt: str, model: str | None) -> str:
    cmd = ["claude", "-p", "--model", model or DEFAULT_CLAUDE_MODEL]
    result = _run_llm_command(cmd, prompt, "claude")
    if result.returncode != 0:
        raise ValueError(
            f"claude -p exited {result.returncode}:\n"
            f"  stderr: {result.stderr.strip()}\n"
            f"  stdout: {result.stdout.strip()}"
        )

    out = result.stdout.strip()
    if not out:
        raise ValueError(f"claude -p returned empty output.\n  stderr: {result.stderr.strip()}")

    return out


def _build_codex_cmd(output_path: str, model: str | None) -> list[str]:
    cmd = ["codex.cmd" if os.name == "nt" else "codex"]
    cmd.extend(["-s", "read-only", "exec", "--output-last-message", output_path])
    if model:
        cmd.extend(["--model", model])
    cmd.append("-")
    return cmd


def _read_codex_output(output_path: str, result: subprocess.CompletedProcess[str]) -> str:
    out = ""
    if output_path and os.path.exists(output_path):
        with open(output_path, encoding="utf-8") as f:
            out = f.read().strip()
    out = out or result.stdout.strip()
    if not out:
        raise ValueError(f"codex exec returned empty output.\n  stderr: {result.stderr.strip()}")
    return out


def _codex_call(prompt: str, model: str | None) -> str:
    output_path = ""
    try:
        with tempfile.NamedTemporaryFile("w", suffix=".txt", delete=False, encoding="utf-8") as f:
            output_path = f.name

        result = _run_llm_command(_build_codex_cmd(output_path, model), prompt, "codex")
        if result.returncode != 0:
            raise ValueError(
                f"codex exec exited {result.returncode}:\n"
                f"  stderr: {result.stderr.strip()}\n"
                f"  stdout: {result.stdout.strip()}"
            )
        return _read_codex_output(output_path, result)
    finally:
        if output_path:
            with contextlib.suppress(OSError):
                os.unlink(output_path)


def _llm_call(prompt: str, mode: str = DEFAULT_MODE, model: str | None = None) -> str:
    normalized_mode = mode.strip().lower()
    resolved_model = model or default_model("phase0", normalized_mode)
    if normalized_mode == "claude":
        return _claude_call(prompt, resolved_model)
    if normalized_mode == "codex":
        return _codex_call(prompt, resolved_model)
    expected = ", ".join(sorted(SUPPORTED_MODES))
    raise ValueError(f"Unsupported LLM mode: {mode!r}. Expected one of: {expected}.")


def _build_prompt(
    categories: list[dict[str, Any]],
    *,
    missing_names: list[str] | None = None,
) -> str:
    trimmed = [
        dict(
            name=cat.get("name"),
            member_count=cat.get("member_count"),
            sample_members=cat.get("members") or [],
        )
        for cat in categories
    ]
    coverage_clause = ""
    if missing_names:
        coverage_clause = (
            "Previous attempt omitted these category names — every one MUST appear"
            " in this attempt's output, either with a kind or with a drop_reason:\n"
            f"{json.dumps(missing_names, ensure_ascii=False)}\n"
        )
    return (
        "You are given a MediaWiki category list for the game 'They Are Billions'.\n"
        "Derive fine-grained kinds from the category structure alone; do not collapse"
        " distinct entity types into a catch-all.\n"
        "Each kind needs a snake_case key, minWikilinks int 0..3, and a one-sentence"
        " gameplay-role description.\n"
        "EVERY input category MUST appear exactly once in the output. Default to"
        " mapping each category to a kind — even small categories or ones whose only"
        " content page shares the category's name. A drop_reason is allowed ONLY for:\n"
        "  (a) wiki-administrative portal/index categories whose members are project"
        " navigation pages (e.g. 'Browse', 'Wiki', 'Main Page'), OR\n"
        "  (b) a category whose entire member list is fully contained in another"
        " mapped category (strict subset, not partial overlap).\n"
        "Heterogeneity, small size, single-page overlap with another category, and"
        " 'the category only contains its own index page' are NOT valid drop reasons —"
        " map them to a kind (creating a new kind if needed) instead.\n"
        + coverage_clause
        + "Return ONLY raw JSON shaped as: "
        '{"kinds":{"<kind>":{"minWikilinks":1,"description":"..."}},'
        '"categories":[{"name":"Exact Category Name","kind":"<kind>"},'
        '{"name":"Exact Category Name","drop_reason":"..."}]}.\n'
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


def _parse_json_with_retry(prompt: str, mode: str = DEFAULT_MODE, model: str | None = None) -> dict:
    last_err: Exception | None = None
    for attempt in range(2):
        p = prompt if attempt == 0 else (prompt + "\n\nOutput raw JSON only, no markdown.")
        content = _strip_fences(_llm_call(p, mode, model))
        try:
            return json.loads(content)
        except Exception as e:
            last_err = e
    assert last_err is not None
    raise ValueError(f"LLM returned invalid JSON after retry: {last_err}")


def _assert_output_shape(result: dict[str, Any]) -> None:
    if not isinstance(result, dict):
        raise ValueError("Expected result to be a JSON object.")
    kinds = result.get("kinds")
    categories = result.get("categories")
    if kinds is None or categories is None:
        raise ValueError('Expected top-level keys: "kinds" and "categories".')
    if not isinstance(kinds, dict):
        raise ValueError('"kinds" must be an object.')
    if not isinstance(categories, list):
        raise ValueError('"categories" must be an array.')


def _coerce_member_count(raw: Any) -> int:
    try:
        return int(raw or 0)
    except (TypeError, ValueError):
        return 0


def _build_name_to_count(categories: list[dict[str, Any]]) -> dict[str, int]:
    return {
        name: _coerce_member_count(cat.get("member_count"))
        for cat in categories
        if isinstance(name := cat.get("name"), str)
    }


def _normalize_kinds(raw_kinds: dict[str, Any]) -> tuple[dict[str, dict[str, Any]], dict[str, str]]:
    normalized: dict[str, dict[str, Any]] = {}
    key_map: dict[str, str] = {}
    for key, val in raw_kinds.items():
        norm_key = _snake_case(str(key))
        key_map[str(key)] = norm_key
        if not isinstance(val, dict):
            raise ValueError(f'Kind "{key}" must be an object.')
        min_wikilinks = max(0, min(3, int(val.get("minWikilinks", 0))))
        description = str(val.get("description", "")).strip()
        normalized[norm_key] = {"minWikilinks": min_wikilinks, "description": description}
    return normalized, key_map


def _normalize_category_entry(
    entry: dict[str, Any],
    name_to_count: dict[str, int],
    name_to_members: dict[str, list[str]],
    normalized_kinds: dict[str, dict[str, Any]],
    kind_key_map: dict[str, str],
) -> dict[str, Any]:
    """Returns a mapped entry (has 'kind') or a dropped entry (has 'drop_reason')."""
    name = entry.get("name")
    if not isinstance(name, str) or name not in name_to_count:
        raise ValueError(f"Category name must be verbatim from input. Bad name: {name!r}")

    drop_reason = entry.get("drop_reason")
    if isinstance(drop_reason, str) and drop_reason.strip():
        return {
            "name": name,
            "member_count": name_to_count[name],
            "sample_members": list(name_to_members.get(name, [])),
            "drop_reason": drop_reason.strip(),
        }

    kind = entry.get("kind")
    if not isinstance(kind, str) or not kind.strip():
        raise ValueError(f"Category {name!r} must have either 'kind' or 'drop_reason'.")
    kind_norm = kind_key_map.get(kind, _snake_case(kind))
    if kind_norm not in normalized_kinds:
        raise ValueError(f"Category {name!r} maps to unknown kind: {kind_norm!r}")
    return {"name": name, "kind": kind_norm, "member_count": name_to_count[name]}


def _build_name_to_members(categories: list[dict[str, Any]]) -> dict[str, list[str]]:
    out: dict[str, list[str]] = {}
    for cat in categories:
        name = cat.get("name")
        members = cat.get("members") or []
        if isinstance(name, str) and isinstance(members, list):
            out[name] = [str(m) for m in members if isinstance(m, str)]
    return out


class IncompleteCoverageError(ValueError):
    """LLM omitted some input categories from its output — caller should re-prompt."""

    def __init__(self, missing: list[str]) -> None:
        super().__init__(f"LLM omitted {len(missing)} input categories: {missing}")
        self.missing = missing


def _validate_output(result: dict[str, Any], categories: list[dict[str, Any]]) -> dict[str, Any]:
    _assert_output_shape(result)
    name_to_count = _build_name_to_count(categories)
    name_to_members = _build_name_to_members(categories)
    normalized_kinds, kind_key_map = _normalize_kinds(result["kinds"])

    mapped: list[dict[str, Any]] = []
    dropped: list[dict[str, Any]] = []
    seen: set[str] = set()
    for entry in result["categories"]:
        if not isinstance(entry, dict):
            continue
        normalized = _normalize_category_entry(
            entry, name_to_count, name_to_members, normalized_kinds, kind_key_map
        )
        name = normalized["name"]
        if name in seen:
            raise ValueError(f"Category {name!r} appears more than once in LLM output.")
        seen.add(name)
        (dropped if "drop_reason" in normalized else mapped).append(normalized)

    missing = sorted(set(name_to_count) - seen)
    if missing:
        raise IncompleteCoverageError(missing)

    return {
        "kinds": normalized_kinds,
        "categories": mapped,
        "dropped_categories": dropped,
    }


def _trim_categories_for_proposals(categories: list[dict[str, Any]]) -> list[dict[str, Any]]:
    return [
        dict(
            name=cat.get("name"),
            kind=cat.get("kind"),
            member_count=_coerce_member_count(cat.get("member_count")),
        )
        for cat in categories
    ]


def _trim_sample_pages(sample_pages_by_category: dict[str, list[str]]) -> dict[str, list[str]]:
    return {
        category_name: [
            str(sample)[:MAX_SCHEMA_SAMPLE_CHARS]
            for sample in samples[:MAX_SCHEMA_SAMPLES_PER_CATEGORY]
        ]
        for category_name, samples in sample_pages_by_category.items()
        if isinstance(samples, list)
    }


def _build_schema_prompt(
    kinds: dict[str, Any],
    categories: list[dict[str, Any]],
    sample_pages_by_category: dict[str, list[str]],
) -> str:
    category_json = json.dumps(_trim_categories_for_proposals(categories), ensure_ascii=False)
    sample_json = json.dumps(_trim_sample_pages(sample_pages_by_category), ensure_ascii=False)
    return (
        "You are proposing per-kind frontmatter schemas for a wiki-to-code pipeline.\n"
        "Use the taxonomy, category assignments, and sample page text. Create only"
        " game-specific sub-fields. Universal fields are handled elsewhere. Every input kind"
        " key must appear exactly once. Return ONLY raw JSON shaped as "
        '{"<kind>":{"properties":{"field":{"type":"string"}}}}\n\n'
        f"Kinds JSON:\n{json.dumps(kinds, ensure_ascii=False)}\n\n"
        f"Categories JSON:\n{category_json}\n\n"
        f"Sample pages by category JSON:\n{sample_json}"
    )


def _assert_string_list(value: Any, label: str) -> list[str]:
    if not isinstance(value, list) or not all(isinstance(item, str) for item in value):
        raise ValueError(f"{label} must be a list of strings.")
    return value


def _validate_schema_entry(kind: str, schema: Any) -> dict[str, Any]:
    if not isinstance(schema, dict):
        raise ValueError(f'Schema for kind "{kind}" must be an object.')
    properties = schema.get("properties")
    if not isinstance(properties, dict):
        raise ValueError(f"{kind}.properties must be an object.")
    return {"properties": properties}


def _validate_schema_output(result: dict[str, Any], kinds: dict[str, Any]) -> dict[str, dict]:
    if not isinstance(result, dict):
        raise ValueError("Expected schema proposal to be a JSON object.")
    schemas: dict[str, dict] = {}
    for kind in kinds:
        if kind not in result:
            raise ValueError(f'Missing schema for kind "{kind}".')
        schemas[kind] = _validate_schema_entry(kind, result[kind])
    extras = sorted(set(result) - set(kinds))
    if extras:
        raise ValueError(f"Schema proposal included unknown kinds: {extras}.")
    return schemas


def propose_frontmatter_schemas(
    kinds: dict,
    categories: list[dict],
    sample_pages_by_category: dict[str, list[str]],
    mode: str = DEFAULT_MODE,
    model: str | None = None,
) -> dict[str, dict]:
    prompt = _build_schema_prompt(kinds, categories, sample_pages_by_category)
    raw = _parse_json_with_retry(prompt, mode, model)
    return _validate_schema_output(raw, kinds)


def _summarize_kind_counts(
    kinds: dict[str, Any],
    categories: list[dict[str, Any]],
) -> dict[str, dict[str, Any]]:
    summary = {
        kind: {"description": data.get("description", ""), "page_count": 0}
        for kind, data in kinds.items()
    }
    for cat in categories:
        kind = cat.get("kind")
        if kind in summary:
            summary[kind]["page_count"] += _coerce_member_count(cat.get("member_count"))
    return summary


def _build_engine_prompt(kinds: dict[str, Any], categories: list[dict[str, Any]]) -> str:
    kind_summary = _summarize_kind_counts(kinds, categories)
    category_json = json.dumps(_trim_categories_for_proposals(categories), ensure_ascii=False)
    return (
        "You are proposing target game-engine candidates for code generation from wiki data.\n"
        "Use only the supplied taxonomy/category evidence, not fixed presets. Prefer engines"
        " whose language, data model, determinism, tooling, and networking fit the inferred"
        " game. Return ONLY raw JSON shaped as "
        '{"engine_candidates": [{"name": "Engine", "language": "Language", "pros": [], '
        '"cons": [], "fit_score": 0.75, "links": []}]}. '
        "Return 2 to 4 candidates, ranked by fit_score descending.\n"
        f"Kind summary JSON:\n{json.dumps(kind_summary, ensure_ascii=False)}\n\n"
        f"Categories JSON:\n{category_json}"
    )


def _validate_engine_candidate(entry: Any, index: int) -> dict[str, Any]:
    if not isinstance(entry, dict):
        raise ValueError(f"Engine candidate {index} must be an object.")
    name = entry.get("name")
    language = entry.get("language")
    if not isinstance(name, str) or not isinstance(language, str):
        raise ValueError(f"Engine candidate {index} needs string name and language.")
    fit_score = entry.get("fit_score")
    if not isinstance(fit_score, int | float) or fit_score < 0 or fit_score > 1:
        raise ValueError(f"Engine candidate {index} fit_score must be in [0, 1].")
    return {
        "name": name,
        "language": language,
        "pros": _assert_string_list(entry.get("pros"), f"candidate {index}.pros"),
        "cons": _assert_string_list(entry.get("cons"), f"candidate {index}.cons"),
        "fit_score": float(fit_score),
        "links": _assert_string_list(entry.get("links"), f"candidate {index}.links"),
    }


def _validate_engine_output(result: dict[str, Any]) -> list[dict]:
    if not isinstance(result, dict) or not isinstance(result.get("engine_candidates"), list):
        raise ValueError('Expected top-level "engine_candidates" array.')
    candidates = result["engine_candidates"]
    if not MIN_ENGINE_CANDIDATES <= len(candidates) <= MAX_ENGINE_CANDIDATES:
        raise ValueError("Expected 2 to 4 engine candidates.")
    validated = [_validate_engine_candidate(entry, i) for i, entry in enumerate(candidates)]
    return sorted(validated, key=lambda entry: entry["fit_score"], reverse=True)


def propose_engine_candidates(
    kinds: dict,
    categories: list[dict],
    mode: str = DEFAULT_MODE,
    model: str | None = None,
) -> list[dict]:
    prompt = _build_engine_prompt(kinds, categories)
    raw = _parse_json_with_retry(prompt, mode, model)
    return _validate_engine_output(raw)


def analyze_taxonomy(
    categories: list[dict],
    mode: str = DEFAULT_MODE,
    model: str | None = None,
) -> dict:
    prompt = _build_prompt(categories)
    raw = _parse_json_with_retry(prompt, mode, model)
    try:
        return _validate_output(raw, categories)
    except IncompleteCoverageError as exc:
        prompt = _build_prompt(categories, missing_names=exc.missing)
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
    except Exception as e:
        print(f"Error: {e}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    raise SystemExit(_main())
