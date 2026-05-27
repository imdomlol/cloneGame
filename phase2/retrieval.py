"""Phase 2 retrieval: vector seeds + 1-hop graph expansion, token-capped.

Layer 3 of the 4-layer codegen filter (see docs/DEPLOYMENT_GUIDE.md §3.4).
Given a task string, returns a packed sequence of ``<file path="...">...</file>``
blocks sized to fit a 2,000-token cap, prioritising direct vector hits over
graph-expanded neighbours.

Pipeline::

    task ──► embed ──► vault_prose top-k + vault_mechanics top-k
                          │
                          v
                    dedupe by id (vector hit order preserved)
                          │
                          v
                    1-hop graph expansion via build/graph.json
                          │
                          v
                    pack <file> blocks until tokens >= cap, vector hits first
                          │
                          v
                    assert packed_tokens <= cap

The cap is a hard assert, not a hint. Token counting uses tiktoken
``cl100k_base`` (best local approximation since Anthropic does not ship a
public Claude tokenizer); the gap to Claude's real tokenization is small
enough that a 2,000-token budget remains safe for the codegen context slot.

CLI::

    python phase2/retrieval.py "implement the ranger unit"
    python phase2/retrieval.py --k 8 --cap-tokens 1500 "..."
"""

from __future__ import annotations

import argparse
import json
import sys
import threading
from pathlib import Path
from typing import Any

_REPO_ROOT = Path(__file__).resolve().parent.parent
if str(_REPO_ROOT / "phase2") not in sys.path:
    sys.path.insert(0, str(_REPO_ROOT / "phase2"))

from indexer import (  # noqa: E402
    DEFAULT_CHROMA_DIR,
    DEFAULT_EMBED_MODEL,
    DEFAULT_GRAPH_PATH,
    DEFAULT_REPOMIX,
    build_records,
    parse_repomix,
)

DEFAULT_CAP_TOKENS = 2000
DEFAULT_K = 5
_TOKEN_ENCODING = "cl100k_base"

# Chroma's PersistentClient races on tenant/database init when two of them are
# constructed at once (the loop driver runs codegen concurrently, so two
# retrieve() calls land here in parallel). Building the client + embedder once
# per (dir, model) and reusing it removes that race and avoids reloading the
# embedding weights every turn. The lock guards *creation* only; Chroma queries
# are read-only and safe to run concurrently against the shared collections.
_CLIENT_LOCK = threading.Lock()
_COLLECTION_CACHE: dict[tuple[str, str], tuple[Any, Any]] = {}


def merge_vector_results(
    prose_ids: list[str],
    prose_distances: list[float],
    mechanics_ids: list[str],
    mechanics_distances: list[float],
) -> list[str]:
    """Merge two parallel id+distance lists into a single ranked, deduped id list.

    Lower distance == better match in Chroma's default cosine space. Ties
    fall back to first-collection-wins (prose before mechanics) for stable
    output across runs.
    """
    pairs: list[tuple[float, int, str]] = []
    for rank, (rid, dist) in enumerate(zip(prose_ids, prose_distances, strict=True)):
        pairs.append((dist, rank, rid))
    offset = len(prose_ids)
    for rank, (rid, dist) in enumerate(zip(mechanics_ids, mechanics_distances, strict=True)):
        pairs.append((dist, offset + rank, rid))
    pairs.sort()
    seen: set[str] = set()
    ordered: list[str] = []
    for _, _, rid in pairs:
        if rid in seen:
            continue
        seen.add(rid)
        ordered.append(rid)
    return ordered


def graph_expand(seed_ids: list[str], graph: dict[str, list[str]]) -> list[str]:
    """Return ``seed_ids`` followed by unique 1-hop neighbours.

    Seeds keep their original order; neighbours are appended in the order
    they are discovered by walking each seed in turn. Missing seed entries
    in the graph are treated as having no neighbours.
    """
    seen: set[str] = set(seed_ids)
    out: list[str] = list(seed_ids)
    for seed in seed_ids:
        for neighbour in graph.get(seed, []):
            if neighbour in seen:
                continue
            seen.add(neighbour)
            out.append(neighbour)
    return out


def count_tokens(text: str, encoding_name: str = _TOKEN_ENCODING) -> int:
    """Token count via tiktoken; small wrapper so tests can monkeypatch."""
    import tiktoken

    encoding = tiktoken.get_encoding(encoding_name)
    return len(encoding.encode(text))


def _render_block(path: str, body: str) -> str:
    """Match the repomix XML envelope so codegen sees identical formatting."""
    return f'<file path="{path}">\n{body}\n</file>'


def pack_files(
    ordered_ids: list[str],
    id_to_path: dict[str, str],
    path_to_body: dict[str, str],
    cap_tokens: int = DEFAULT_CAP_TOKENS,
    token_counter: Any = count_tokens,
) -> tuple[str, list[str]]:
    """Greedily pack ``<file>`` blocks until adding the next one would exceed cap.

    Returns ``(packed_xml, included_ids)``. Ids with no known path or empty
    body are skipped silently (they correspond to graph neighbours that were
    referenced in ``depends_on`` but never produced a vault file).

    The final packed token count is asserted to be ``<= cap_tokens``; this is
    a hard contract guarded by DEPLOYMENT_GUIDE §3.4.
    """
    blocks: list[str] = []
    included: list[str] = []
    used = 0
    for rid in ordered_ids:
        path = id_to_path.get(rid)
        if not path:
            continue
        body = path_to_body.get(path, "")
        if not body:
            continue
        block = _render_block(path, body)
        block_tokens = token_counter(block)
        if used + block_tokens > cap_tokens:
            continue
        blocks.append(block)
        included.append(rid)
        used += block_tokens
    packed = "\n".join(blocks)
    assert token_counter(packed) <= cap_tokens, (
        f"packed retrieval exceeded cap: {token_counter(packed)} > {cap_tokens}"
    )
    return packed, included


def resolve_pin(
    id_to_path: dict[str, str],
    pin_id: str | None,
    pin_kind: str | None,
) -> str | None:
    """Map a goal's note slug (+ optional kind) to its record id, or None.

    Matches the record whose vault path stem equals ``pin_id`` and, when
    ``pin_kind`` is given, whose parent directory name equals it (so a
    cross-kind slug collision — a unit and a building sharing a name — pins the
    right note). Game- and engine-agnostic: it keys on the vault layout
    (``<kind>/<slug>.md``) the whole pipeline already produces, not on any
    specific game's names. Returns None when nothing matches (pin is then a
    silent no-op, never a hard failure).
    """
    if not pin_id:
        return None
    for rid, path in id_to_path.items():
        p = Path(path)
        if p.stem == pin_id and (pin_kind is None or p.parent.name == pin_kind):
            return rid
    return None


def seed_with_pin(seed_ids: list[str], pin: str | None) -> list[str]:
    """Return ``seed_ids`` with ``pin`` forced to the front (deduped).

    Putting the pinned id first makes it a vector seed (its graph neighbours
    still expand) AND guarantees it is packed before the token cap can evict it,
    so the spec for the artifact under construction is always in context.
    """
    if not pin:
        return seed_ids
    return [pin] + [s for s in seed_ids if s != pin]


def load_artifacts(
    repomix_path: Path,
    graph_path: Path,
) -> tuple[dict[str, str], dict[str, str], dict[str, list[str]]]:
    """Load id→path, path→body, and the dependency graph from disk."""
    if not repomix_path.exists():
        raise FileNotFoundError(f"{repomix_path} missing; run scripts/regenerate_repomix.py first")
    if not graph_path.exists():
        raise FileNotFoundError(f"{graph_path} missing; run phase2/indexer.py first")
    xml_text = repomix_path.read_text(encoding="utf-8")
    path_to_body = dict(parse_repomix(xml_text))
    records = build_records(repomix_path)
    id_to_path = {r["id"]: r["path"] for r in records}
    graph = json.loads(graph_path.read_text(encoding="utf-8"))
    return id_to_path, path_to_body, graph


def _get_collections(chroma_dir: Path, model_name: str) -> tuple[Any, Any]:
    """Return the ``(prose, mechanics)`` collections, building them once per key.

    Construction (client + embedder + collections) is serialized behind
    ``_CLIENT_LOCK`` and memoized by ``(chroma_dir, model_name)``. This is the
    fix for the concurrent ``PersistentClient`` tenant-init race; it is engine-
    and game-agnostic (the cache key is the index location, not any game data).
    """
    key = (str(chroma_dir), model_name)
    with _CLIENT_LOCK:
        cached = _COLLECTION_CACHE.get(key)
        if cached is not None:
            return cached
        import chromadb
        from chromadb.utils import embedding_functions

        client = chromadb.PersistentClient(path=str(chroma_dir))
        embedder = embedding_functions.SentenceTransformerEmbeddingFunction(
            model_name=model_name,
        )
        prose = client.get_or_create_collection(name="vault_prose", embedding_function=embedder)
        mech = client.get_or_create_collection(name="vault_mechanics", embedding_function=embedder)
        _COLLECTION_CACHE[key] = (prose, mech)
        return prose, mech


def query_collections(
    chroma_dir: Path,
    model_name: str,
    task: str,
    k: int,
) -> tuple[list[str], list[float], list[str], list[float]]:
    """Query both Chroma collections; returns parallel id+distance lists."""
    prose, mech = _get_collections(chroma_dir, model_name)
    prose_res = prose.query(query_texts=[task], n_results=k)
    mech_res = mech.query(query_texts=[task], n_results=k)
    return (
        prose_res["ids"][0],
        prose_res["distances"][0],
        mech_res["ids"][0],
        mech_res["distances"][0],
    )


def retrieve(
    task: str,
    *,
    k: int = DEFAULT_K,
    cap_tokens: int = DEFAULT_CAP_TOKENS,
    chroma_dir: Path = DEFAULT_CHROMA_DIR,
    graph_path: Path = DEFAULT_GRAPH_PATH,
    repomix_path: Path = DEFAULT_REPOMIX,
    model_name: str = DEFAULT_EMBED_MODEL,
    pin_id: str | None = None,
    pin_kind: str | None = None,
) -> tuple[str, list[str]]:
    """End-to-end retrieval: ``task → packed <file> blocks + included ids``.

    ``pin_id`` (+ optional ``pin_kind``) forces the goal's own vault note to the
    front of the bundle so the spec being implemented is always in context, even
    when the imperative task phrasing does not rank it into the vector top-k.
    See ``resolve_pin`` / ``seed_with_pin``.
    """
    id_to_path, path_to_body, graph = load_artifacts(repomix_path, graph_path)
    prose_ids, prose_dists, mech_ids, mech_dists = query_collections(
        chroma_dir, model_name, task, k
    )
    seed_ids = merge_vector_results(prose_ids, prose_dists, mech_ids, mech_dists)
    seed_ids = seed_with_pin(seed_ids, resolve_pin(id_to_path, pin_id, pin_kind))
    ordered = graph_expand(seed_ids, graph)
    return pack_files(ordered, id_to_path, path_to_body, cap_tokens=cap_tokens)


def _parse_args(argv: list[str] | None = None) -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.add_argument("task", help="task string fed to the codegen LLM")
    parser.add_argument("--k", type=int, default=DEFAULT_K)
    parser.add_argument("--cap-tokens", type=int, default=DEFAULT_CAP_TOKENS)
    parser.add_argument("--chroma-dir", type=Path, default=DEFAULT_CHROMA_DIR)
    parser.add_argument("--graph", type=Path, default=DEFAULT_GRAPH_PATH)
    parser.add_argument("--repomix", type=Path, default=DEFAULT_REPOMIX)
    parser.add_argument("--model", default=DEFAULT_EMBED_MODEL)
    return parser.parse_args(argv)


def main(argv: list[str] | None = None) -> int:
    args = _parse_args(argv)
    packed, included = retrieve(
        args.task,
        k=args.k,
        cap_tokens=args.cap_tokens,
        chroma_dir=args.chroma_dir,
        graph_path=args.graph,
        repomix_path=args.repomix,
        model_name=args.model,
    )
    print(packed)
    print(
        f"\n# included {len(included)} files, {count_tokens(packed)}/{args.cap_tokens} tokens",
        file=sys.stderr,
    )
    return 0


if __name__ == "__main__":
    sys.exit(main())
