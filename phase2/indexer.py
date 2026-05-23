"""Phase 2 indexer: Repomix XML to Chroma collections plus graph sidecar.

Reads `build/repomix-output.xml`, splits each vault note into
frontmatter / Description / Behavioral Mechanics / References, and upserts:

- `vault_prose`     -- Description text, semantic recall
- `vault_mechanics` -- Behavioral Mechanics text, exact-logic recall

into a local Chroma persistent store. Also writes `build/graph.json`, a
sidecar adjacency map `{id: [outbound_depends_on_ids, ...]}` sourced from
each note's `depends_on:` frontmatter (which mirrors `[[wikilinks]]` in
the body per the Phase 1 wikilink invariant).

Embeddings use a local sentence-transformers model (default
`BAAI/bge-small-en-v1.5`, 384 dim, ~120MB) loaded via Chroma's built-in
`SentenceTransformerEmbeddingFunction` so retrieval reuses the exact same
model at query time.

CLI::

    python phase2/indexer.py                          # full run
    python phase2/indexer.py --dry-run                # parse + graph only
    python phase2/indexer.py --repomix path/to.xml    # custom input
"""

from __future__ import annotations

import argparse
import json
import re
import sys
from pathlib import Path
from typing import Any

_REPO_ROOT = Path(__file__).resolve().parent.parent
_SCRIPTS_DIR = _REPO_ROOT / "scripts"
if str(_SCRIPTS_DIR) not in sys.path:
    sys.path.insert(0, str(_SCRIPTS_DIR))

from frontmatter import frontmatter  # noqa: E402

DEFAULT_REPOMIX = _REPO_ROOT / "build" / "repomix-output.xml"
DEFAULT_CHROMA_DIR = _REPO_ROOT / "chroma"
DEFAULT_GRAPH_PATH = _REPO_ROOT / "build" / "graph.json"
DEFAULT_EMBED_MODEL = "BAAI/bge-small-en-v1.5"

_FILE_BLOCK_RE = re.compile(
    r'<file path="(?P<path>[^"]+)">\n(?P<body>.*?)\n</file>',
    re.DOTALL,
)
_FRONTMATTER_RE = re.compile(r"\A---\s*\n.*?\n---\s*\n", re.DOTALL)
_HEADING_RE = re.compile(r"^##\s+(.+?)\s*$", re.MULTILINE)


def parse_repomix(xml_text: str) -> list[tuple[str, str]]:
    """Return ``[(path, body), ...]`` extracted from a repomix XML dump.

    Regex-based because the file bodies are Markdown and may contain
    angle brackets (`<gallery>`, raw HTML) that confuse strict XML parsers.
    The `<file path="...">...</file>` envelope is well-defined by repomix.
    """
    return [(m.group("path"), m.group("body")) for m in _FILE_BLOCK_RE.finditer(xml_text)]


def split_sections(body: str) -> dict[str, str]:
    """Split a vault note body into description / mechanics / references.

    Returns empty strings for missing sections. The frontmatter block is
    stripped first so the section walk only sees Markdown headings.
    """
    sections = {"description": "", "mechanics": "", "references": ""}
    fm_match = _FRONTMATTER_RE.match(body)
    text = body[fm_match.end() :] if fm_match else body

    headings = list(_HEADING_RE.finditer(text))
    for i, m in enumerate(headings):
        name = m.group(1).strip().lower()
        start = m.end()
        end = headings[i + 1].start() if i + 1 < len(headings) else len(text)
        chunk = text[start:end].strip()
        key = _classify_heading(name)
        if key is not None:
            sections[key] = chunk
    return sections


def _classify_heading(name: str) -> str | None:
    if "description" in name:
        return "description"
    if "mechanic" in name or "behavioral" in name:
        return "mechanics"
    if "reference" in name:
        return "references"
    return None


def build_records(repomix_path: Path) -> list[dict[str, Any]]:
    """Parse the repomix XML and return one record per valid vault note.

    Notes missing the ``id`` frontmatter field (or with unparseable
    frontmatter) are skipped with a warning so a bad single file does not
    abort the whole index build.
    """
    xml_text = repomix_path.read_text(encoding="utf-8")
    out: list[dict[str, Any]] = []
    for path, body in parse_repomix(xml_text):
        fm, errs = frontmatter(body)
        if errs:
            print(f"warn: skipping {path}: {errs[0]}", file=sys.stderr)
            continue
        note_id = fm.get("id")
        if not isinstance(note_id, str) or not note_id:
            print(f"warn: skipping {path}: missing or non-string id", file=sys.stderr)
            continue
        sections = split_sections(body)
        out.append(
            {
                "id": note_id,
                "path": path,
                "type": str(fm.get("type") or ""),
                "subtype": str(fm.get("subtype") or ""),
                "frontmatter": fm,
                "description": sections["description"],
                "mechanics": sections["mechanics"],
                "references": sections["references"],
            }
        )
    return out


def deduplicate_records(records: list[dict[str, Any]]) -> list[dict[str, Any]]:
    """First-wins dedup by ``id`` across the parsed record list.

    Phase 1's in-progress kind migration can leave the same logical entity
    at multiple paths (e.g. ``vault/building/x.md`` and ``vault/buildings/x.md``).
    Chroma requires unique ids per collection, and graph.json must map each
    id to a single adjacency list, so collapse duplicates here once and feed
    both downstream consumers the same list.
    """
    seen: set[str] = set()
    kept: list[dict[str, Any]] = []
    for rec in records:
        if rec["id"] in seen:
            print(
                f"warn: duplicate id {rec['id']!r} at {rec['path']} (keeping first occurrence)",
                file=sys.stderr,
            )
            continue
        seen.add(rec["id"])
        kept.append(rec)
    return kept


def build_graph(records: list[dict[str, Any]]) -> dict[str, list[str]]:
    """Sorted adjacency map from each note id to its `depends_on` targets."""
    graph: dict[str, list[str]] = {}
    for rec in records:
        deps_raw = rec["frontmatter"].get("depends_on")
        deps = [d for d in (deps_raw or []) if isinstance(d, str) and d]
        graph[rec["id"]] = sorted(set(deps))
    return graph


def index_chroma(
    records: list[dict[str, Any]],
    chroma_dir: Path,
    model_name: str,
) -> None:
    """Upsert prose and mechanics chunks into two Chroma collections."""
    import chromadb
    from chromadb.utils import embedding_functions

    chroma_dir.mkdir(parents=True, exist_ok=True)
    client = chromadb.PersistentClient(path=str(chroma_dir))
    embedder = embedding_functions.SentenceTransformerEmbeddingFunction(
        model_name=model_name,
    )

    for collection_name, field in (
        ("vault_prose", "description"),
        ("vault_mechanics", "mechanics"),
    ):
        coll = client.get_or_create_collection(
            name=collection_name,
            embedding_function=embedder,
        )
        ids: list[str] = []
        docs: list[str] = []
        metas: list[dict[str, Any]] = []
        for rec in records:
            if not rec[field].strip():
                continue
            ids.append(rec["id"])
            docs.append(rec[field])
            metas.append(
                {
                    "path": rec["path"],
                    "type": rec["type"],
                    "subtype": rec["subtype"],
                }
            )
        if ids:
            coll.upsert(ids=ids, documents=docs, metadatas=metas)
        print(f"{collection_name}: upserted {len(ids)} chunks")


def _parse_args(argv: list[str] | None = None) -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.add_argument("--repomix", type=Path, default=DEFAULT_REPOMIX)
    parser.add_argument("--chroma-dir", type=Path, default=DEFAULT_CHROMA_DIR)
    parser.add_argument("--graph", type=Path, default=DEFAULT_GRAPH_PATH)
    parser.add_argument("--model", default=DEFAULT_EMBED_MODEL)
    parser.add_argument(
        "--dry-run",
        action="store_true",
        help="parse + write graph.json, skip Chroma upsert (no embedding model load)",
    )
    return parser.parse_args(argv)


def main(argv: list[str] | None = None) -> int:
    args = _parse_args(argv)

    if not args.repomix.exists():
        print(
            f"error: {args.repomix} does not exist "
            "(run repomix first; see DEPLOYMENT_GUIDE.md §3.2)",
            file=sys.stderr,
        )
        return 1

    records = build_records(args.repomix)
    records = deduplicate_records(records)
    print(f"parsed {len(records)} unique notes from {args.repomix}")

    graph = build_graph(records)
    args.graph.parent.mkdir(parents=True, exist_ok=True)
    args.graph.write_text(
        json.dumps(graph, indent=2, sort_keys=True) + "\n",
        encoding="utf-8",
    )
    edge_count = sum(len(v) for v in graph.values())
    print(f"wrote {args.graph} ({len(graph)} nodes, {edge_count} edges)")

    if not args.dry_run:
        index_chroma(records, args.chroma_dir, args.model)

    return 0


if __name__ == "__main__":
    sys.exit(main())
