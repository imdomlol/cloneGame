"""Phase 1 wiki ingest pipeline.

Requires `jsonschema` for full schema validation; without it, this script falls
back to required-field checks so dry runs and cache-safe ingest still work.
"""

from __future__ import annotations

import argparse
import datetime as dt
import hashlib
import json
import os
import re
import subprocess
import sys
import time
import tomllib
import urllib.error
import urllib.parse
import urllib.request
from pathlib import Path
from typing import Any

try:
    import jsonschema
except Exception:  # pragma: no cover - dependency is optional at import time
    jsonschema = None

try:
    from referencing import Registry, Resource
    from referencing.jsonschema import DRAFT202012
except Exception:  # pragma: no cover - dependency follows modern jsonschema
    Registry = None
    Resource = None
    DRAFT202012 = None


def repo_root() -> Path:
    return Path(__file__).resolve().parents[1]


def read_json(path: Path) -> dict[str, Any]:
    return json.loads(path.read_text(encoding="utf-8"))


def load_config(root: Path) -> dict[str, Any]:
    return tomllib.loads((root / "phase1.config.toml").read_text(encoding="utf-8"))


def build_url(api_endpoint: str, params: dict[str, object]) -> str:
    return f"{api_endpoint}?{urllib.parse.urlencode(params, doseq=True)}"


def http_get_json(url: str, user_agent: str, timeout_s: float = 30.0) -> dict[str, Any]:
    req = urllib.request.Request(url, headers={"User-Agent": user_agent})
    with urllib.request.urlopen(req, timeout=timeout_s) as resp:
        return json.loads(resp.read().decode("utf-8"))


def request_json(url: str, user_agent: str, retry_count: int) -> dict[str, Any]:
    for attempt in range(retry_count + 1):
        try:
            return http_get_json(url, user_agent)
        except urllib.error.HTTPError as exc:
            if exc.code != 429 and exc.code < 500:
                raise
            if attempt >= retry_count:
                raise
        except (urllib.error.URLError, TimeoutError):
            if attempt >= retry_count:
                raise
        time.sleep(min(2**attempt, 30))
    raise RuntimeError("unreachable retry loop")


def category_members(
    api: str, category: str, user_agent: str, retries: int
) -> list[dict[str, Any]]:
    params: dict[str, object] = {
        "action": "query",
        "list": "categorymembers",
        "cmtitle": f"Category:{category}",
        "cmlimit": "max",
        "cmtype": "page",
        "format": "json",
    }
    out: list[dict[str, Any]] = []
    while True:
        data = request_json(build_url(api, params), user_agent, retries)
        members = data.get("query", {}).get("categorymembers", [])
        out.extend([m for m in members if isinstance(m, dict)])
        cont = data.get("continue")
        if not isinstance(cont, dict) or not cont:
            return out
        params.update(cont)


def page_wikitext(api: str, title: str, user_agent: str, retries: int) -> tuple[str, str]:
    params = {
        "action": "query",
        "prop": "revisions",
        "titles": title,
        "rvprop": "ids|content",
        "rvslots": "main",
        "formatversion": "2",
        "format": "json",
    }
    data = request_json(build_url(api, params), user_agent, retries)
    pages = data.get("query", {}).get("pages", [])
    if not pages or "missing" in pages[0]:
        raise ValueError(f"missing wiki page: {title}")
    rev = pages[0].get("revisions", [{}])[0]
    main_slot = rev.get("slots", {}).get("main", {})
    text = main_slot.get("content", rev.get("*", ""))
    return str(text), str(rev.get("revid", "unknown"))


def _remove_wikilinks_by_prefix(text: str, prefixes: tuple[str, ...]) -> str:
    lower = text.casefold()
    out: list[str] = []
    i = 0
    length = len(text)
    while i < length:
        if i + 2 <= length and text[i : i + 2] == "[[":
            prefix_match = False
            for prefix in prefixes:
                start = i + 2
                end = start + len(prefix)
                if end <= length and lower[start:end] == prefix:
                    prefix_match = True
                    break
            if prefix_match:
                depth = 1
                j = i + 2
                while j < length and depth > 0:
                    if j + 2 <= length and text[j : j + 2] == "[[":
                        depth += 1
                        j += 2
                        continue
                    if j + 2 <= length and text[j : j + 2] == "]]":
                        depth -= 1
                        j += 2
                        continue
                    j += 1
                i = j
                continue
        out.append(text[i])
        i += 1
    return "".join(out)


def _is_image_listing_line(line: str) -> bool:
    stripped = line.strip()
    if not stripped:
        return True
    lowered = stripped.casefold()
    return (
        lowered.startswith("[[file:")
        or lowered.startswith("[[image:")
        or lowered.startswith("file:")
        or lowered.startswith("image:")
    )


def trim_wikitext(text: str) -> str:
    trimmed = re.sub(r"<!--.*?-->", "", text, flags=re.S)
    trimmed = _remove_wikilinks_by_prefix(trimmed, ("category:", "file:", "image:"))

    template_keywords = (
        "nav",
        "navbar",
        "navigation",
        "footer",
        "stub",
        "cleanup",
        "spoiler",
        "toc",
        "clear",
        "wikia",
        "fandom",
    )
    lines: list[str] = []
    for line in trimmed.splitlines():
        stripped = line.strip()
        lowered = stripped.casefold()
        is_template_line = lowered.startswith("{{") and lowered.endswith("}}")
        if is_template_line and any(keyword in lowered for keyword in template_keywords):
            continue
        lines.append(line)
    trimmed = "\n".join(lines)

    gallery_pattern = re.compile(r"<gallery\b[^>]*>(.*?)</gallery>", re.I | re.S)

    def _replace_gallery(match: re.Match[str]) -> str:
        body = match.group(1)
        if all(_is_image_listing_line(line) for line in body.splitlines()):
            return ""
        return match.group(0)

    trimmed = gallery_pattern.sub(_replace_gallery, trimmed)
    trimmed = re.sub(r"\n[ \t]*\n(?:[ \t]*\n)+", "\n\n", trimmed)
    return trimmed.strip()


def slugify(value: str) -> str:
    slug = re.sub(r"[^a-z0-9]+", "_", value.casefold()).strip("_")
    return slug or "untitled"


def page_url(base_url: str, title: str) -> str:
    return base_url.rstrip("/") + "/" + urllib.parse.quote(title.replace(" ", "_"))


def source_key(url: str) -> str:
    parsed = urllib.parse.urlsplit(url.strip())
    path = urllib.parse.unquote(parsed.path).replace(" ", "_").rstrip("/")
    return urllib.parse.urlunsplit(
        (
            parsed.scheme.casefold(),
            parsed.netloc.casefold(),
            path,
            "",
            "",
        )
    )


def completed_source_index(root: Path) -> dict[str, list[tuple[str, Path]]]:
    vault = root / "vault"
    out: dict[str, list[tuple[str, Path]]] = {}
    if not vault.exists():
        return out
    for path in vault.rglob("*.md"):
        try:
            relative = path.relative_to(vault)
        except ValueError:
            continue
        if relative.parts and relative.parts[0] == "_quarantine":
            continue
        data, errors = frontmatter(path.read_text(encoding="utf-8"))
        if errors:
            continue
        source = data.get("source_url")
        if isinstance(source, str) and source:
            note_kind = data.get("type")
            if not isinstance(note_kind, str) or not note_kind:
                note_kind = path.parent.name
            out.setdefault(source_key(source), []).append((note_kind, path))
    return out


def completed_source_for_kind(
    index: dict[str, list[tuple[str, Path]]], url: str, kind: str
) -> Path | None:
    for note_kind, path in index.get(source_key(url), []):
        if note_kind == kind:
            return path
    return None


def completed_sources_for_other_kinds(
    index: dict[str, list[tuple[str, Path]]], url: str, kind: str
) -> list[tuple[str, Path]]:
    return [
        (note_kind, path) for note_kind, path in index.get(source_key(url), []) if note_kind != kind
    ]


def raw_kind_schema(game_config: dict[str, Any], kind: str) -> dict[str, Any]:
    # Per-kind frontmatter schema as authored in game-config.json, including
    # `required` if present. Used in the compile prompt so the LLM sees the
    # full contract. Validation goes through kind_frontmatter_schema, which
    # strips `required` (see plan.md decision log).
    kinds = game_config.get("kinds", {})
    if not isinstance(kinds, dict):
        return {}
    kind_config = kinds.get(kind)
    if not isinstance(kind_config, dict):
        return {}
    schema = kind_config.get("frontmatter_schema")
    return schema if isinstance(schema, dict) else {}


def compile_prompt(template: str, kind: str, source: str, game_config: dict[str, Any]) -> str:
    schema = raw_kind_schema(game_config, kind)
    schema_json = json.dumps(schema, indent=2, ensure_ascii=False) if schema else "{}"
    prompt = template.replace("{{type_hint}}", kind)
    prompt = prompt.replace("{{kind_schema}}", schema_json)
    return prompt.replace("{{stripped_html}}", source)


def cache_key(rendered_prompt: str, model: str) -> str:
    # Hashes the FINAL rendered prompt (template + kind + schema + wikitext)
    # plus the model id. Any change to wikitext, system prompt, or per-kind
    # schema invalidates the cache automatically.
    h = hashlib.sha256()
    h.update(rendered_prompt.encode("utf-8"))
    h.update(b"\0")
    h.update(model.encode("utf-8"))
    return h.hexdigest()


def run_llm(prompt: str, mode: str, model: str) -> str:
    if mode == "claude":
        cmd = ["claude", "-p", "--model", model]
    elif mode == "codex":
        codex_cmd = "codex.cmd" if os.name == "nt" else "codex"
        cmd = [codex_cmd, "exec", "--model", model, "-"]
    else:
        raise ValueError(f"unsupported [compile] llm_mode: {mode}")
    proc = subprocess.Popen(
        cmd,
        stdin=subprocess.PIPE,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
        encoding="utf-8",
    )
    try:
        stdout, stderr = proc.communicate(prompt)
    except KeyboardInterrupt:
        proc.terminate()
        try:
            proc.wait(timeout=5)
        except subprocess.TimeoutExpired:
            proc.kill()
            proc.wait()
        raise
    if proc.returncode != 0:
        raise RuntimeError(stderr.strip() or f"{cmd[0]} exited {proc.returncode}")
    return stdout


def cached_or_compile(
    cache_dir: Path, key: str, prompt: str, mode: str, model: str
) -> tuple[str, str]:
    path = cache_dir / f"{key}.md"
    if path.exists():
        return path.read_text(encoding="utf-8"), "cached"
    output = run_llm(prompt, mode, model)
    cache_dir.mkdir(parents=True, exist_ok=True)
    path.write_text(output, encoding="utf-8")
    return output, "compiled"


def strip_llm_chatter(markdown: str) -> str:
    # Agentic CLIs sometimes wrap output in "I'll convert this..." preamble
    # or ```markdown ... ``` fences despite a strict-output prompt. Locate the
    # first '---' line and drop everything before it; trim a trailing ```.
    lines = markdown.splitlines(keepends=True)
    for i, line in enumerate(lines):
        if line.strip() == "---":
            stripped = "".join(lines[i:])
            return re.sub(r"\n```\s*\Z", "\n", stripped)
    return markdown


def repair_frontmatter_delimiter(markdown: str) -> str:
    if re.match(r"\A---\s*\n(.*?)\n---\s*(?:\n|\Z)", markdown, re.S):
        return markdown
    if not markdown.startswith("---"):
        return markdown

    heading = re.search(r"\n(#{1,6}\s+)", markdown)
    if heading is None:
        return markdown
    insert_at = heading.start() + 1
    return markdown[:insert_at] + "---\n" + markdown[insert_at:]


def frontmatter(markdown: str) -> tuple[dict[str, Any], list[str]]:
    markdown = repair_frontmatter_delimiter(markdown)
    match = re.match(r"\A---\s*\n(.*?)\n---\s*(?:\n|\Z)", markdown, re.S)
    if not match:
        return {}, ["missing YAML frontmatter"]
    return parse_yaml_map(match.group(1)), []


def parse_scalar(raw: str) -> Any:
    raw = raw.strip()
    if raw in {"", "null", "~"}:
        return None
    if raw in {"true", "false"}:
        return raw == "true"
    if raw.startswith(('"', "'")) and raw.endswith(raw[0]):
        return raw[1:-1]
    if raw.startswith("[") or raw.startswith("{"):
        try:
            return json.loads(raw.replace("'", '"'))
        except Exception:
            if raw.startswith("[") and raw.endswith("]"):
                return parse_flow_sequence(raw)
            return raw
    try:
        return int(raw) if re.fullmatch(r"-?\d+", raw) else float(raw)
    except ValueError:
        return raw


def parse_flow_sequence(raw: str) -> list[Any]:
    body = raw[1:-1].strip()
    if not body:
        return []

    items: list[str] = []
    current: list[str] = []
    quote = ""
    escaped = False
    for char in body:
        if escaped:
            current.append(char)
            escaped = False
            continue
        if char == "\\" and quote == '"':
            current.append(char)
            escaped = True
            continue
        if quote:
            current.append(char)
            if char == quote:
                quote = ""
            continue
        if char in {"'", '"'}:
            quote = char
            current.append(char)
            continue
        if char == ",":
            items.append("".join(current).strip())
            current = []
            continue
        current.append(char)
    items.append("".join(current).strip())
    return [parse_scalar(item) for item in items if item]


def parse_yaml_map(text: str) -> dict[str, Any]:
    lines = [line.rstrip() for line in text.splitlines() if line.strip()]
    data, _ = parse_block(lines, 0, 0)
    return data if isinstance(data, dict) else {}


def line_indent(line: str) -> int:
    return len(line) - len(line.lstrip(" "))


def parse_block(lines: list[str], idx: int, indent: int) -> tuple[Any, int]:
    if idx >= len(lines):
        return {}, idx
    if lines[idx].strip().startswith("- "):
        return parse_list(lines, idx, indent)
    return parse_map(lines, idx, indent)


def parse_map(lines: list[str], idx: int, indent: int) -> tuple[dict[str, Any], int]:
    out: dict[str, Any] = {}
    while idx < len(lines) and line_indent(lines[idx]) >= indent:
        if line_indent(lines[idx]) != indent or lines[idx].strip().startswith("- "):
            break
        key, _, val = lines[idx].strip().partition(":")
        idx += 1
        if val.strip():
            out[key] = parse_scalar(val)
        elif idx < len(lines) and line_indent(lines[idx]) > indent:
            out[key], idx = parse_block(lines, idx, line_indent(lines[idx]))
        else:
            out[key] = {}
    return out, idx


def parse_list(lines: list[str], idx: int, indent: int) -> tuple[list[Any], int]:
    out: list[Any] = []
    while idx < len(lines) and line_indent(lines[idx]) == indent:
        item = lines[idx].strip()[2:].strip()
        idx += 1
        if ":" in item and not item.startswith(('"', "'")):
            key, _, val = item.partition(":")
            obj = {key: parse_scalar(val)} if val.strip() else {key: {}}
            if idx < len(lines) and line_indent(lines[idx]) > indent:
                extra, idx = parse_map(lines, idx, line_indent(lines[idx]))
                obj.update(extra)
            out.append(obj)
        elif item:
            out.append(parse_scalar(item))
    return out, idx


def collect_required(schema: dict[str, Any]) -> list[str]:
    required = list(schema.get("required", []))
    for part in schema.get("allOf", []):
        if isinstance(part, dict):
            required.extend(part.get("required", []))
    return required


def universal_schema(root: Path, kinds: set[str]) -> dict[str, Any]:
    schema = read_json(root / "schemas" / "_universal.schema.json")
    type_prop = schema.get("properties", {}).get("type")
    if isinstance(type_prop, dict) and isinstance(type_prop.get("enum"), list):
        type_prop["enum"] = sorted(set(type_prop["enum"]) | kinds)
    return schema


def kind_frontmatter_schema(game_config: dict[str, Any], kind: str) -> dict[str, Any] | None:
    # Per-kind schema for VALIDATION. Strips `required` so per-kind fields are
    # optional — universal schema's required list is the only presence gate;
    # per-kind validation only enforces TYPE on fields that happen to be
    # present. See plan.md decision log.
    schema = raw_kind_schema(game_config, kind)
    properties = schema.get("properties") if isinstance(schema, dict) else None
    if not isinstance(properties, dict) or not properties:
        return None
    return {"type": "object", "properties": properties}


def validate_basic(data: dict[str, Any], schemas: list[dict[str, Any]]) -> list[str]:
    errors: list[str] = []
    for schema in schemas:
        for key in collect_required(schema):
            if key not in data:
                errors.append(f"required field missing: {key}")
    if "confidence" in data and not isinstance(data["confidence"], (int, float)):
        errors.append("confidence must be a number")
    return errors


def validate_jsonschema(
    data: dict[str, Any], schemas: list[dict[str, Any]], root: Path
) -> list[str]:
    if jsonschema is None:
        return validate_basic(data, schemas)
    registry = schema_registry(root)
    errors: list[str] = []
    for schema in schemas:
        validator = (
            jsonschema.Draft202012Validator(schema, registry=registry)
            if registry is not None
            else jsonschema.Draft202012Validator(schema)
        )
        errors.extend(sorted(e.message for e in validator.iter_errors(data)))
    return errors


def schema_registry(root: Path) -> Any | None:
    if Registry is None or Resource is None or DRAFT202012 is None:
        return None

    registry = Registry()
    schema_dir = root / "schemas"
    if not schema_dir.exists():
        return registry

    for path in schema_dir.glob("*.schema.json"):
        schema = read_json(path)
        uri = str(schema.get("$id") or path.as_uri())
        resource = Resource.from_contents(schema, default_specification=DRAFT202012)
        registry = registry.with_resource(uri, resource)
        registry = registry.with_resource(path.as_uri(), resource)
    return registry


def validation_errors(
    root: Path,
    data: dict[str, Any],
    kind: str,
    kinds: set[str],
    game_config: dict[str, Any],
) -> tuple[list[str], bool]:
    schemas = [universal_schema(root, kinds)]
    kind_schema = kind_frontmatter_schema(game_config, kind)
    has_kind_schema = kind_schema is not None
    if kind_schema is not None:
        schemas.append(kind_schema)
    return validate_jsonschema(data, schemas, root), has_kind_schema


def prepend_errors(markdown: str, errors: list[str]) -> str:
    block = ["validation_errors:"]
    block.extend(f"  - {json.dumps(error)}" for error in errors)
    return "\n".join(block) + "\n---\n" + markdown


def write_result(root: Path, kind: str, slug: str, markdown: str, errors: list[str]) -> Path:
    base = root / "vault" / ("_quarantine" if errors else kind)
    base.mkdir(parents=True, exist_ok=True)
    path = base / f"{slug}.md"
    path.write_text(prepend_errors(markdown, errors) if errors else markdown, encoding="utf-8")
    return path


def category_plan(game_config: dict[str, Any]) -> list[dict[str, Any]]:
    cats = game_config.get("categories", [])
    return [c for c in cats if isinstance(c.get("name"), str) and isinstance(c.get("kind"), str)]


def configured_member_count(cat: dict[str, Any]) -> int | None:
    count = cat.get("member_count")
    return count if isinstance(count, int) else None


def dry_run_count(api: str, cat: dict[str, Any], user_agent: str, retries: int) -> tuple[int, str]:
    try:
        return len(category_members(api, cat["name"], user_agent, retries)), "api"
    except (TimeoutError, urllib.error.URLError) as exc:
        count = configured_member_count(cat)
        if count is not None:
            return count, "configured"
        raise RuntimeError(f"could not enumerate {cat['name']}") from exc


def configured_dry_run_count(cat: dict[str, Any]) -> int:
    count = configured_member_count(cat)
    if count is None:
        raise RuntimeError(f"missing configured member_count for {cat['name']}")
    return count


def print_entry_start(item_no: int) -> None:
    print()
    print(f"========== ITEM {item_no:03d} ==========")


def print_entry_end(item_no: int) -> None:
    print(f"---------- END ITEM {item_no:03d} ----------")


def print_status(status: str, message: str, item_no: int | None = None) -> None:
    prefix = f"{item_no:03d} {status:<10}" if item_no is not None else f"{status:<10}"
    print(f"{prefix} {message}")


def print_detail(label: str, value: object) -> None:
    print(f"  {label:<10} {value}")


def relative_to_root(path: Path, root: Path) -> str:
    try:
        return str(path.relative_to(root))
    except ValueError:
        return str(path)


def print_dry_run(api: str, cats: list[dict[str, Any]], user_agent: str, retries: int) -> int:
    print()
    print_status("DRY RUN", "category member counts")
    use_api = True
    for cat in cats:
        count, source = (
            dry_run_count(api, cat, user_agent, retries)
            if use_api
            else (configured_dry_run_count(cat), "configured")
        )
        use_api = source == "api"
        suffix = "" if source == "api" else " (configured)"
        print_detail(cat["name"], f"{count} pages [{cat['kind']}]{suffix}")
    return 0


def print_next_page(cat: dict[str, Any], title: str, source_url: str, item_no: int) -> None:
    print_entry_start(item_no)
    print_status("NEXT", title, item_no)
    print_detail("category", cat["name"])
    print_detail("kind", cat["kind"])
    print_detail("source", source_url)
    if sys.stdin.isatty():
        print_detail("cancel", "Ctrl+C skips this page and continues")


def process_page(
    ctx: dict[str, Any], cat: dict[str, Any], member: dict[str, Any], item_no: int
) -> bool:
    title = str(member.get("title", ""))
    source_url = page_url(ctx["base_url"], title)
    print_next_page(cat, title, source_url, item_no)

    source_idx = ctx["source_index"]
    existing = completed_source_for_kind(source_idx, source_url, cat["kind"])
    if existing is not None and not ctx["force"]:
        print_status("SKIP", title, item_no)
        print_detail("reason", "already completed for this kind")
        print_detail("path", relative_to_root(existing, ctx["root"]))
        print_entry_end(item_no)
        return False
    related = completed_sources_for_other_kinds(source_idx, source_url, cat["kind"])
    for note_kind, path in related:
        print_detail("related", f"{note_kind} -> {relative_to_root(path, ctx['root'])}")

    text, rev = page_wikitext(ctx["api"], title, ctx["ua"], ctx["retries"])
    trimmed_text = trim_wikitext(text)
    source = trimmed_text + f"\n\nSOURCE_URL: {source_url}\nSOURCE_REVISION: {rev}\n"
    prompt = compile_prompt(ctx["system_prompt"], cat["kind"], source, ctx["game_config"])
    key = cache_key(prompt, ctx["model"])
    markdown, status = cached_or_compile(ctx["cache_dir"], key, prompt, ctx["mode"], ctx["model"])
    markdown = strip_llm_chatter(markdown)
    markdown = repair_frontmatter_delimiter(markdown)
    fm, errors = frontmatter(markdown)
    schema_errors, has_schema = validation_errors(
        ctx["root"], fm, cat["kind"], ctx["kinds"], ctx["game_config"]
    )
    ctx["missing_kinds"].add(cat["kind"]) if not has_schema else None
    errors.extend(schema_errors)
    slug = slugify(str(fm.get("id") or title))
    path = write_result(ctx["root"], cat["kind"], slug, markdown, errors)
    if not errors:
        source_idx.setdefault(source_key(source_url), []).append((cat["kind"], path))
    final = "QUARANTINE" if errors else "DONE"
    print_status(final, title, item_no)
    print_detail("result", "validation errors" if errors else status)
    print_detail("path", relative_to_root(path, ctx["root"]))
    print_detail("wikitext", f"{len(text)} chars -> {len(trimmed_text)} chars")
    if errors:
        for error in errors:
            print_detail("error", error)
    print_entry_end(item_no)
    return bool(errors)


def ingest(ctx: dict[str, Any], cats: list[dict[str, Any]], limit: int | None) -> int:
    quarantined = 0
    item_no = 0
    delay = int(ctx["delay_ms"]) / 1000
    for cat in cats:
        members = category_members(ctx["api"], cat["name"], ctx["ua"], ctx["retries"])
        selected = members if limit is None else members[:limit]
        for member in selected:
            item_no += 1
            try:
                quarantined += int(process_page(ctx, cat, member, item_no))
            except KeyboardInterrupt:
                title = str(member.get("title", "unknown"))
                print_status("SKIP", title, item_no)
                print_detail("reason", "user canceled current page")
                print_detail("source", page_url(ctx["base_url"], title))
                print_entry_end(item_no)
            except Exception as exc:
                title = str(member.get("title", "unknown"))
                slug = slugify(title)
                markdown = f"---\nid: {slug}\nname: {title}\ntype: {cat['kind']}\n---\n"
                path = write_result(ctx["root"], cat["kind"], slug, markdown, [str(exc)])
                print_status("QUARANTINE", title, item_no)
                print_detail("result", "exception")
                print_detail("path", relative_to_root(path, ctx["root"]))
                print_detail("source", page_url(ctx["base_url"], title))
                print_detail("error", str(exc))
                print_entry_end(item_no)
                quarantined += 1
            time.sleep(delay)
    return quarantined


def build_context(
    root: Path, config: dict[str, Any], game_config: dict[str, Any]
) -> dict[str, Any]:
    compile_cfg = config.get("compile", {})
    ingest_cfg = config.get("ingest", {})
    prompt_path = root / str(compile_cfg.get("system_prompt_path"))
    return {
        "root": root,
        "api": config["wiki"]["api_endpoint"],
        "base_url": game_config["game"]["wiki_base_url"],
        "ua": ingest_cfg.get("user_agent", "cloneGame-phase1/0.1"),
        "retries": int(ingest_cfg.get("retry_count", 3)),
        "delay_ms": int(ingest_cfg.get("request_delay_ms", 250)),
        "mode": compile_cfg.get("llm_mode", "claude"),
        "model": compile_cfg.get("model", "default"),
        "system_prompt": prompt_path.read_text(encoding="utf-8"),
        "cache_dir": root / config.get("cache", {}).get("dir", ".phase1_cache"),
        "force": False,
        "kinds": set(game_config.get("kinds", {}).keys()),
        "game_config": game_config,
        "missing_kinds": set(),
        "source_index": completed_source_index(root),
        "started_at": dt.datetime.now(dt.UTC).isoformat(),
    }


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(
        description="Ingest They Are Billions wiki pages into an Obsidian vault."
    )
    parser.add_argument(
        "--dry-run", action="store_true", help="Only enumerate category member counts."
    )
    parser.add_argument("--limit", type=int, help="Ingest at most N pages per category.")
    parser.add_argument(
        "--force",
        action="store_true",
        help="Reprocess pages even when their source_url already exists in the vault.",
    )
    args = parser.parse_args(argv)
    root = repo_root()
    config = load_config(root)
    game_config = read_json(root / "game-config.json")
    cats = category_plan(game_config)
    ctx = build_context(root, config, game_config)
    ctx["force"] = args.force
    if args.dry_run:
        return print_dry_run(ctx["api"], cats, ctx["ua"], ctx["retries"])
    quarantined = ingest(ctx, cats, args.limit)
    if ctx["missing_kinds"]:
        print(
            "warning: missing per-kind schemas: " + ", ".join(sorted(ctx["missing_kinds"])),
            file=sys.stderr,
        )
    return quarantined


if __name__ == "__main__":
    raise SystemExit(main())
