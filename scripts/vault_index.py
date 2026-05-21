"""Vault file naming, source-URL indexing, and migration between kinds.

`source_key` normalizes URLs so the index can de-duplicate completed
work regardless of percent-encoding, case, or trailing slash. The index
is consulted before every compile to skip already-ingested sources and
to migrate notes whose kind has been re-routed.
"""

from __future__ import annotations

import json
import os
import re
import sys
import urllib.parse
from pathlib import Path

# Allow `from frontmatter import ...` regardless of whether this module is
# loaded as scripts.vault_index (test path) or via direct script execution.
_SCRIPTS_DIR = os.path.dirname(os.path.abspath(__file__))
if _SCRIPTS_DIR not in sys.path:
    sys.path.insert(0, _SCRIPTS_DIR)

from frontmatter import frontmatter, replace_frontmatter_type  # noqa: E402


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


def migrate_existing_note(
    root: Path,
    source_idx: dict[str, list[tuple[str, Path]]],
    source_url: str,
    kind: str,
    existing_path: Path,
) -> tuple[Path, list[str]]:
    markdown = existing_path.read_text(encoding="utf-8")
    markdown = replace_frontmatter_type(markdown, kind)
    fm, errors = frontmatter(markdown)
    slug = slugify(str(fm.get("id") or existing_path.stem))
    path = write_result(root, kind, slug, markdown, errors)
    if not errors:
        source_idx.setdefault(source_key(source_url), []).append((kind, path))
    return path, errors
