"""Phase 1 wiki ingest pipeline.

Requires `jsonschema` for full schema validation; without it, this script falls
back to required-field checks so dry runs and cache-safe ingest still work.
"""

from __future__ import annotations

import argparse
import datetime as dt
import hashlib
import json
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


def category_members(api: str, category: str, user_agent: str, retries: int) -> list[dict[str, Any]]:
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


def slugify(value: str) -> str:
    slug = re.sub(r"[^a-z0-9]+", "_", value.casefold()).strip("_")
    return slug or "untitled"


def page_url(base_url: str, title: str) -> str:
    return base_url.rstrip("/") + "/" + urllib.parse.quote(title.replace(" ", "_"))


def compile_prompt(template: str, kind: str, source: str) -> str:
    prompt = template.replace("{{type_hint}}", kind)
    return prompt.replace("{{stripped_html}}", source)


def cache_key(wikitext: str, system_prompt: str, model: str) -> str:
    h = hashlib.sha256()
    h.update(wikitext.encode("utf-8"))
    h.update(b"\0")
    h.update(system_prompt.encode("utf-8"))
    h.update(b"\0")
    h.update(model.encode("utf-8"))
    return h.hexdigest()


def run_llm(prompt: str, mode: str, model: str) -> str:
    if mode == "claude":
        cmd = ["claude", "-p", "--model", model]
    elif mode == "codex":
        cmd = ["codex", "exec", "--model", model, "-"]
    else:
        raise ValueError(f"unsupported [compile] llm_mode: {mode}")
    proc = subprocess.run(cmd, input=prompt, text=True, capture_output=True)
    if proc.returncode != 0:
        raise RuntimeError(proc.stderr.strip() or f"{cmd[0]} exited {proc.returncode}")
    return proc.stdout


def cached_or_compile(cache_dir: Path, key: str, prompt: str, mode: str, model: str) -> tuple[str, str]:
    path = cache_dir / f"{key}.md"
    if path.exists():
        return path.read_text(encoding="utf-8"), "cached"
    output = run_llm(prompt, mode, model)
    cache_dir.mkdir(parents=True, exist_ok=True)
    path.write_text(output, encoding="utf-8")
    return output, "compiled"


def frontmatter(markdown: str) -> tuple[dict[str, Any], list[str]]:
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
            return raw
    try:
        return int(raw) if re.fullmatch(r"-?\d+", raw) else float(raw)
    except ValueError:
        return raw


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


def validate_basic(data: dict[str, Any], schemas: list[dict[str, Any]]) -> list[str]:
    errors: list[str] = []
    for schema in schemas:
        for key in collect_required(schema):
            if key not in data:
                errors.append(f"required field missing: {key}")
    if "confidence" in data and not isinstance(data["confidence"], (int, float)):
        errors.append("confidence must be a number")
    return errors


def validate_jsonschema(data: dict[str, Any], schemas: list[dict[str, Any]], root: Path) -> list[str]:
    if jsonschema is None:
        return validate_basic(data, schemas)
    resolver = jsonschema.RefResolver(base_uri=(root / "schemas").as_uri() + "/", referrer=schemas[0])
    errors: list[str] = []
    for schema in schemas:
        validator = jsonschema.Draft202012Validator(schema, resolver=resolver)
        errors.extend(sorted(e.message for e in validator.iter_errors(data)))
    return errors


def validation_errors(root: Path, data: dict[str, Any], kind: str, kinds: set[str]) -> tuple[list[str], bool]:
    schemas = [universal_schema(root, kinds)]
    kind_path = root / "schemas" / f"{kind}.schema.json"
    has_kind_schema = kind_path.exists()
    if has_kind_schema:
        schemas.append(read_json(kind_path))
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


def category_plan(game_config: dict[str, Any]) -> list[dict[str, str]]:
    cats = game_config.get("categories", [])
    return [c for c in cats if isinstance(c.get("name"), str) and isinstance(c.get("kind"), str)]


def print_dry_run(api: str, cats: list[dict[str, str]], user_agent: str, retries: int) -> int:
    for cat in cats:
        count = len(category_members(api, cat["name"], user_agent, retries))
        print(f"{cat['name']} ({cat['kind']}): {count} pages")
    return 0


def process_page(ctx: dict[str, Any], cat: dict[str, str], member: dict[str, Any]) -> bool:
    title = str(member.get("title", ""))
    text, rev = page_wikitext(ctx["api"], title, ctx["ua"], ctx["retries"])
    source = text + f"\n\nSOURCE_URL: {page_url(ctx['base_url'], title)}\nSOURCE_REVISION: {rev}\n"
    prompt = compile_prompt(ctx["system_prompt"], cat["kind"], source)
    key = cache_key(text, ctx["system_prompt"], ctx["model"])
    markdown, status = cached_or_compile(ctx["cache_dir"], key, prompt, ctx["mode"], ctx["model"])
    fm, errors = frontmatter(markdown)
    schema_errors, has_schema = validation_errors(ctx["root"], fm, cat["kind"], ctx["kinds"])
    ctx["missing_kinds"].add(cat["kind"]) if not has_schema else None
    errors.extend(schema_errors)
    slug = slugify(str(fm.get("id") or title))
    path = write_result(ctx["root"], cat["kind"], slug, markdown, errors)
    final = "quarantined" if errors else status
    print(f"[{final}] {cat['name']} / {title} -> {path.relative_to(ctx['root'])}")
    return bool(errors)


def ingest(ctx: dict[str, Any], cats: list[dict[str, str]], limit: int | None) -> int:
    quarantined = 0
    delay = int(ctx["delay_ms"]) / 1000
    for cat in cats:
        members = category_members(ctx["api"], cat["name"], ctx["ua"], ctx["retries"])
        selected = members if limit is None else members[:limit]
        for member in selected:
            try:
                quarantined += int(process_page(ctx, cat, member))
            except Exception as exc:
                title = str(member.get("title", "unknown"))
                markdown = f"---\nid: {slugify(title)}\nname: {title}\ntype: {cat['kind']}\n---\n"
                write_result(ctx["root"], cat["kind"], slugify(title), markdown, [str(exc)])
                print(f"[quarantined] {cat['name']} / {title} -> vault/_quarantine/{slugify(title)}.md")
                quarantined += 1
            time.sleep(delay)
    return quarantined


def build_context(root: Path, config: dict[str, Any], game_config: dict[str, Any]) -> dict[str, Any]:
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
        "kinds": set(game_config.get("kinds", {}).keys()),
        "missing_kinds": set(),
        "started_at": dt.datetime.now(dt.UTC).isoformat(),
    }


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description="Ingest They Are Billions wiki pages into an Obsidian vault.")
    parser.add_argument("--dry-run", action="store_true", help="Only enumerate category member counts.")
    parser.add_argument("--limit", type=int, help="Ingest at most N pages per category.")
    args = parser.parse_args(argv)
    root = repo_root()
    config = load_config(root)
    game_config = read_json(root / "game-config.json")
    cats = category_plan(game_config)
    ctx = build_context(root, config, game_config)
    if args.dry_run:
        return print_dry_run(ctx["api"], cats, ctx["ua"], ctx["retries"])
    quarantined = ingest(ctx, cats, args.limit)
    if ctx["missing_kinds"]:
        print("warning: missing per-kind schemas: " + ", ".join(sorted(ctx["missing_kinds"])), file=sys.stderr)
    return quarantined


if __name__ == "__main__":
    raise SystemExit(main())
