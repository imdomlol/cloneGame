import json
import os
import re
import sys
from typing import Any


MODEL_BY_PROVIDER = {
    "openai": "gpt-4o-mini",
    "anthropic": "claude-haiku-4-5-20251001",
    "gemini": "gemini-2.0-flash",
}

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


def _detect_provider() -> tuple[str, str]:
    if os.getenv("OPENAI_API_KEY"):
        return ("openai", "OPENAI_API_KEY")
    if os.getenv("ANTHROPIC_API_KEY"):
        return ("anthropic", "ANTHROPIC_API_KEY")
    if os.getenv("GEMINI_API_KEY"):
        return ("gemini", "GEMINI_API_KEY")
    print(
        "Error: no LLM provider key found. Set one of: OPENAI_API_KEY, ANTHROPIC_API_KEY, GEMINI_API_KEY.",
        file=sys.stderr,
    )
    sys.exit(1)


def _llm_call(provider: str, prompt: str) -> str:
    model = MODEL_BY_PROVIDER[provider]

    if provider == "openai":
        from openai import OpenAI  # type: ignore

        client = OpenAI()
        resp = client.chat.completions.create(
            model=model,
            messages=[
                {"role": "system", "content": "You are a careful JSON generator."},
                {"role": "user", "content": prompt},
            ],
            temperature=0.2,
        )
        return (resp.choices[0].message.content or "").strip()

    if provider == "anthropic":
        from anthropic import Anthropic  # type: ignore

        client = Anthropic(api_key=os.environ["ANTHROPIC_API_KEY"])
        resp = client.messages.create(
            model=model,
            max_tokens=2000,
            temperature=0.2,
            messages=[{"role": "user", "content": prompt}],
        )
        chunks = getattr(resp, "content", [])
        if chunks and getattr(chunks[0], "text", None):
            return chunks[0].text.strip()
        return str(resp).strip()

    if provider == "gemini":
        import google.generativeai as genai  # type: ignore

        genai.configure(api_key=os.environ["GEMINI_API_KEY"])
        gen_model = genai.GenerativeModel(model)
        resp = gen_model.generate_content(
            prompt,
            generation_config={
                "temperature": 0.2,
                "response_mime_type": "application/json",
            },
        )
        return (getattr(resp, "text", "") or "").strip()

    raise ValueError(f"Unknown provider: {provider}")


def _build_prompt(categories: list[dict[str, Any]]) -> str:
    trimmed: list[dict[str, Any]] = []
    for cat in categories:
        members = cat.get("members") or []
        trimmed.append(
            {
                "name": cat.get("name"),
                "member_count": cat.get("member_count"),
                "sample_members": members[:2],
            }
        )

    required_lines = []
    for kind, fields in REQUIRED_FIELDS_BY_KIND.items():
        required_lines.append(f"- {kind}: {', '.join(fields)}")
    required_block = "\n".join(required_lines)

    return (
        "You are given a MediaWiki category list. Each category has a name, member_count, and two sample member page titles.\n"
        "\n"
        "Task:\n"
        "1) Discard any remaining meta/community/maintenance categories you recognize semantically.\n"
        "2) Map surviving categories into a `kinds` object. Each kind must have:\n"
        "   - a snake_case key\n"
        "   - `minWikilinks` as an int 0..3 (higher for more interconnected entity types)\n"
        "   - a one-sentence `description` that explicitly names the required frontmatter sub-fields for that kind.\n"
        "     Required fields by kind:\n"
        f"{required_block}\n"
        "3) For each kind, emit exactly 2 `seedPages` items using the exact page title strings from the input `sample_members`/original members list.\n"
        "   - Titles must be verbatim (no paraphrasing).\n"
        "   - `summary` must be one sentence describing the entity's role in the game engine (no lore).\n"
        "\n"
        "Return ONLY raw JSON (no markdown, no prose) in this exact shape:\n"
        "{\n"
        '  "kinds": {\n'
        '    "<snake_case_kind>": {"minWikilinks": 1, "description": "..."}\n'
        "  },\n"
        '  "seedPages": [\n'
        '    {"title": "Exact Title", "kind": "<snake_case_kind>", "summary": "..."}\n'
        "  ]\n"
        "}\n"
        "\n"
        "Input categories JSON:\n"
        f"{json.dumps(trimmed, ensure_ascii=False)}"
    )


def _parse_json_with_retry(provider: str, prompt: str) -> dict[str, Any]:
    last_err: Exception | None = None
    for attempt in range(2):
        content = _llm_call(provider, prompt if attempt == 0 else (prompt + "\n\nOutput raw JSON only, no markdown."))
        try:
            return json.loads(content)
        except Exception as e:  # noqa: BLE001
            last_err = e
    assert last_err is not None
    raise ValueError(f"LLM returned invalid JSON after retry: {last_err}")


def _validate_output(result: dict[str, Any], categories: list[dict[str, Any]]) -> dict[str, Any]:
    if not isinstance(result, dict):
        raise ValueError("Expected result to be a JSON object.")
    if "kinds" not in result or "seedPages" not in result:
        raise ValueError('Expected top-level keys: "kinds" and "seedPages".')
    if not isinstance(result["kinds"], dict):
        raise ValueError('"kinds" must be an object.')
    if not isinstance(result["seedPages"], list):
        raise ValueError('"seedPages" must be an array.')

    allowed_titles: set[str] = set()
    for cat in categories:
        for title in cat.get("members") or []:
            if isinstance(title, str):
                allowed_titles.add(title)

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

    normalized_seed_pages: list[dict[str, Any]] = []
    for sp in result["seedPages"]:
        if not isinstance(sp, dict):
            continue
        title = sp.get("title")
        if not isinstance(title, str) or title not in allowed_titles:
            raise ValueError(f'Seed page title must be verbatim from input members. Bad title: {title!r}')
        kind = sp.get("kind")
        kind_str = str(kind) if kind is not None else ""
        kind_norm = kind_key_map.get(kind_str, _snake_case(kind_str))
        summary = str(sp.get("summary", "")).strip()
        normalized_seed_pages.append({"title": title, "kind": kind_norm, "summary": summary})

    return {"kinds": normalized_kinds, "seedPages": normalized_seed_pages}


def analyze_taxonomy(categories: list[dict]) -> dict:
    provider, _ = _detect_provider()
    prompt = _build_prompt(categories)
    raw = _parse_json_with_retry(provider, prompt)
    return _validate_output(raw, categories)


def _main() -> int:
    try:
        categories = json.loads(sys.stdin.read() or "[]")
        if not isinstance(categories, list):
            raise ValueError("stdin JSON must be a list (output of phase0_fetch.fetch_taxonomy()).")
        result = analyze_taxonomy(categories)
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

