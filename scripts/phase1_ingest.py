"""Phase 1 wiki ingest pipeline orchestrator.

Walks the categories in `game-config.json`, fetches wikitext via the
MediaWiki API, compiles it through a headless LLM CLI, validates the
resulting frontmatter, and writes each note to `vault/<kind>/<slug>.md`
(or `vault/_quarantine/` on failure).

Helpers live in sibling modules: wikitext, frontmatter, validation,
compile_cache, wiki_api, vault_index.

Requires `jsonschema` for full schema validation; without it, the
validation module falls back to required-field checks so dry runs and
cache-safe ingest still work.
"""

from __future__ import annotations

import argparse
import datetime as dt
import json
import os
import sys
import time
import tomllib
import urllib.error
from pathlib import Path
from typing import Any

# Allow bare imports of sibling scripts whether this file is run directly
# (python scripts/phase1_ingest.py) or loaded as scripts.phase1_ingest by
# unittest.
_SCRIPTS_DIR = os.path.dirname(os.path.abspath(__file__))
if _SCRIPTS_DIR not in sys.path:
    sys.path.insert(0, _SCRIPTS_DIR)

from compile_cache import (  # noqa: E402
    cache_key,
    cached_or_compile,
    compile_prompt,
    strip_llm_chatter,
)
from frontmatter import (  # noqa: E402
    frontmatter,
    repair_frontmatter_delimiter,
    replace_frontmatter_type,
)
from validation import canonical_kind, raw_kind_schema, validation_errors  # noqa: E402
from vault_index import (  # noqa: E402
    completed_source_for_kind,
    completed_source_index,
    completed_sources_for_other_kinds,
    migrate_existing_note,
    page_url,
    slugify,
    source_key,
    write_result,
)
from wiki_api import category_members, page_wikitext  # noqa: E402
from wikitext import trim_wikitext  # noqa: E402


def repo_root() -> Path:
    return Path(__file__).resolve().parents[1]


def read_json(path: Path) -> dict[str, Any]:
    return json.loads(path.read_text(encoding="utf-8"))


def load_config(root: Path) -> dict[str, Any]:
    return tomllib.loads((root / "phase1.config.toml").read_text(encoding="utf-8"))


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


def _skip_already_completed(
    ctx: dict[str, Any], cat: dict[str, Any], title: str, source_url: str, item_no: int
) -> bool:
    existing = completed_source_for_kind(ctx["source_index"], source_url, cat["kind"])
    if existing is None or ctx["force"]:
        return False
    print_status("SKIP", title, item_no)
    print_detail("reason", "already completed for this kind")
    print_detail("path", relative_to_root(existing, ctx["root"]))
    print_entry_end(item_no)
    return True


def _maybe_migrate_from_other_kind(
    ctx: dict[str, Any], cat: dict[str, Any], title: str, source_url: str, item_no: int
) -> tuple[bool, bool]:
    """Return (handled, quarantined). `handled=True` means caller should stop."""
    related = completed_sources_for_other_kinds(ctx["source_index"], source_url, cat["kind"])
    for note_kind, path in related:
        print_detail("related", f"{note_kind} -> {relative_to_root(path, ctx['root'])}")
    if not related or ctx["force"]:
        return False, False
    note_kind, related_path = related[0]
    migrated_path, errors = migrate_existing_note(
        ctx["root"], ctx["source_index"], source_url, cat["kind"], related_path
    )
    if errors:
        print_status("QUARANTINE", title, item_no)
        print_detail("result", "migration validation errors")
        print_detail("from", f"{note_kind} -> {relative_to_root(related_path, ctx['root'])}")
        print_detail("path", relative_to_root(migrated_path, ctx["root"]))
        for error in errors:
            print_detail("error", error)
        print_entry_end(item_no)
        return True, True
    print_status("MIGRATE", title, item_no)
    print_detail("reason", "reused completed source from another kind")
    print_detail("from", f"{note_kind} -> {relative_to_root(related_path, ctx['root'])}")
    print_detail("path", relative_to_root(migrated_path, ctx["root"]))
    print_entry_end(item_no)
    return True, False


def _compile_and_validate(
    ctx: dict[str, Any], cat: dict[str, Any], title: str, source_url: str
) -> tuple[str, dict[str, Any], list[str], bool, str, int, int]:
    text, rev = page_wikitext(ctx["api"], title, ctx["ua"], ctx["retries"])
    trimmed_text = trim_wikitext(text)
    source = trimmed_text + f"\n\nSOURCE_URL: {source_url}\nSOURCE_REVISION: {rev}\n"
    schema = raw_kind_schema(ctx["game_config"], cat["kind"])
    prompt = compile_prompt(ctx["system_prompt"], cat["kind"], source, schema)
    key = cache_key(prompt, ctx["model"])
    markdown, status = cached_or_compile(ctx["cache_dir"], key, prompt, ctx["mode"], ctx["model"])
    markdown = strip_llm_chatter(markdown)
    markdown = repair_frontmatter_delimiter(markdown)
    fm, errors = frontmatter(markdown)
    declared_type = fm.get("type")
    if isinstance(declared_type, str):
        canonical = canonical_kind(declared_type, ctx["kinds"])
        if canonical is not None and canonical != declared_type:
            fm["type"] = canonical
            markdown = replace_frontmatter_type(markdown, canonical)
    schema_errors, has_schema = validation_errors(
        ctx["root"], fm, cat["kind"], ctx["kinds"], ctx["game_config"]
    )
    if not has_schema:
        ctx["missing_kinds"].add(cat["kind"])
    errors.extend(schema_errors)
    return markdown, fm, errors, has_schema, status, len(text), len(trimmed_text)


def process_page(
    ctx: dict[str, Any], cat: dict[str, Any], member: dict[str, Any], item_no: int
) -> bool:
    title = str(member.get("title", ""))
    source_url = page_url(ctx["base_url"], title)
    print_next_page(cat, title, source_url, item_no)

    if _skip_already_completed(ctx, cat, title, source_url, item_no):
        return False
    handled, quarantined = _maybe_migrate_from_other_kind(ctx, cat, title, source_url, item_no)
    if handled:
        return quarantined

    markdown, fm, errors, _has_schema, status, raw_chars, trimmed_chars = _compile_and_validate(
        ctx, cat, title, source_url
    )
    slug = slugify(str(fm.get("id") or title))
    path = write_result(ctx["root"], cat["kind"], slug, markdown, errors)
    if not errors:
        ctx["source_index"].setdefault(source_key(source_url), []).append((cat["kind"], path))
    final = "QUARANTINE" if errors else "DONE"
    print_status(final, title, item_no)
    print_detail("result", "validation errors" if errors else status)
    print_detail("path", relative_to_root(path, ctx["root"]))
    print_detail("wikitext", f"{raw_chars} chars -> {trimmed_chars} chars")
    if errors:
        for error in errors:
            print_detail("error", error)
    print_entry_end(item_no)
    return bool(errors)


def _quarantine_exception(
    ctx: dict[str, Any], cat: dict[str, Any], member: dict[str, Any], item_no: int, exc: Exception
) -> None:
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
                _quarantine_exception(ctx, cat, member, item_no, exc)
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
