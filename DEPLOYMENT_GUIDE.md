# Game Reverse-Engineering Pipeline — Deployment Guide

A production deployment specification for the two-phase wiki-to-code pipeline.
Phase 1 sanitizes a community wiki into an AI-optimized Obsidian Vault using
`llm-wiki-compiler`. Phase 2 bundles the vault with `repomix` and serves
chunks to a code-generation LLM through a 4-layer context filter.

---

## Part 1 — Phase 1 Parsing Specification

### 1.1 Pipeline Overview

```
┌────────────────┐    llmwiki      ┌────────────────┐    llmwiki     ┌──────────────────────┐
│  Fandom Wiki   │ ──── ingest ──> │  Raw HTML +    │ ── compile ──> │  Obsidian Vault      │
│  (live URL)    │                 │  scrape cache  │                │  (schema-conformant) │
└────────────────┘                 └────────────────┘                └──────────────────────┘
```

The ingest stage is a deterministic HTML-to-Markdown crawl. The compile stage
is the LLM-driven reformat that enforces the vault schema below. Compile is the
only step that consumes tokens in Phase 1, and it runs once per page lifetime
(re-runs only on wiki diff).

### 1.2 Directory Layout (Vault Output)

`llmwiki compile` must emit into a fixed structure so that Phase 2's Vault
Index can route tasks deterministically:

```
vault/
├── _index.md                     # Auto-generated graph manifest
├── items/
│   ├── weapons/
│   │   └── Iron_Sword.md
│   ├── armor/
│   └── consumables/
├── skills/
│   ├── active/
│   └── passive/
├── enemies/
├── mechanics/                    # Damage formulas, status effects, etc.
├── locations/
├── npcs/
├── quests/
└── systems/                      # Crafting, economy, leveling
```

One concept = one file. Disambiguate by full taxonomic path, never by suffix
(`items/weapons/Iron_Sword.md`, never `Iron_Sword_weapon.md`).

### 1.3 Required YAML Frontmatter Schema

Every `.md` file MUST start with a typed frontmatter block. The `type` field
selects which sub-schema is mandatory:

```yaml
---
# Universal fields (required on every file)
id: iron_sword                          # snake_case, unique within vault
name: "Iron Sword"                      # display name
type: item                              # item | skill | enemy | mechanic | location | npc | quest | system
subtype: weapon                         # type-specific discriminator
source_url: "https://wiki.example.com/wiki/Iron_Sword"
source_revision: "rev_12345"            # used by Phase 1 idempotency check
extracted_at: "2026-05-18T12:00:00Z"
confidence: 0.94                        # compiler's self-rated extraction confidence
# Type-specific fields (item/weapon example)
stats:
  damage: 12
  attack_speed: 1.1
  durability: 200
  weight: 4.0
requirements:
  level: 3
  strength: 8
crafting:
  recipe_id: iron_sword_recipe
  materials:
    - { id: iron_ingot, qty: 3 }
    - { id: wood, qty: 1 }
  station: forge
tags: [melee, one_handed, tier_2]
depends_on: [iron_ingot, wood, forge]   # MUST mirror every [[WikiLink]] in body
---
```

#### Type-specific required fields

| `type`     | Required sub-fields                                                |
| ---------- | ------------------------------------------------------------------ |
| `item`     | `stats`, `requirements`, `tags`                                    |
| `skill`    | `cost`, `cooldown`, `effects`, `scaling`                           |
| `enemy`    | `stats`, `loot_table`, `ai_profile`, `resistances`                 |
| `mechanic` | `formula`, `inputs`, `outputs`, `edge_cases`                       |
| `quest`    | `prerequisites`, `objectives`, `rewards`, `flags_set`              |
| `system`   | `controlled_entities`, `state_transitions`                         |

The schema is enforced by a JSON-Schema validator that runs as the final stage
of `llmwiki compile`. Files failing validation are emitted to
`vault/_quarantine/` with a `validation_errors:` block and excluded from the
Repomix bundle in Phase 2.

### 1.4 Body Structure (Three Mandatory Sections)

```markdown
## Description
Short prose summary, 1–3 sentences. Every proper noun referring to another
vault entity MUST be a [[WikiLink]] using the target file's `id`.

The [[iron_ingot|Iron Ingot]] is forged at a [[forge]]...

## Behavioral Mechanics
Bulleted, programmatic conditional logic. One bullet per branch.

- IF target has `tag:undead` THEN damage *= 1.5
- IF wielder.strength < 8 THEN attack_speed *= 0.5
- ON hit: roll 5% chance to apply [[status_bleed]] for 3s
- ON durability == 0: item is consumed, emit event `item_broken`

## References
- Source: [Fandom page](https://...)
- Related: [[steel_sword]], [[iron_armor]]
```

The Behavioral Mechanics section is the **single source of truth for codegen
logic**. It must use the imperative trigger keywords (`IF`/`THEN`/`ON`/`WHILE`)
so Phase 2's translation dictionary can lex it.

### 1.5 The Phase 1 Compile Prompt

Inject this as the system prompt for the `llmwiki compile` LLM stage:

```markdown
You are a Wiki Sanitization Agent. Convert the supplied raw HTML/Markdown
of a single wiki page into a schema-conformant Obsidian Vault note.

[OUTPUT CONTRACT]
- Emit exactly one Markdown file with YAML frontmatter, then three sections:
  ## Description, ## Behavioral Mechanics, ## References.
- The frontmatter MUST validate against the schema for `type` = {{type_hint}}.
- All numbers MUST be integers or floats — never strings, never ranges (split
  ranges into `min`/`max` fields).
- Every cross-entity reference in prose MUST be an Obsidian [[wiki_link]] using
  the target's snake_case id. Mirror every link in the `depends_on:` array.

[BEHAVIORAL MECHANICS RULES]
- One conditional per bullet. Lead with IF / THEN / ON / WHILE.
- Encode every numeric multiplier, chance, cooldown, and duration explicitly.
- If the wiki is ambiguous, emit the bullet anyway and lower `confidence`.

[FORBIDDEN]
- No trivia, no flavor lore, no patch-note history, no community speculation.
- No prose paragraphs in Behavioral Mechanics.
- No external links except the canonical Source in References.

[INPUT]
RAW_HTML:
{{stripped_html}}

PAGE_TYPE_HINT: {{type_hint}}   # set by the URL-based router in ingest
```

### 1.6 Idempotency & Incremental Recompile

- Store `source_revision` per page in `vault/_index.md`.
- Re-run `llmwiki ingest --since=<revision>` to fetch only changed pages.
- `llmwiki compile --changed-only` rewrites only affected files.
- Phase 2's Repomix bundle is regenerated on any vault change; vector index
  re-embeds only the diff (see §2.4).

---

## Part 2 — Phase 2 Python Orchestration Pipeline

### 2.1 Component Map

```
              ┌────────────────────────────────────────────────────┐
              │              Phase 2 Orchestrator (Python)         │
              │                                                     │
   vault/  ──>│  RepomixBundler ──> ChromaIndexer ──> GraphMapper  │──> Retrieval API
              │                          │                  │       │
              │                          v                  v       │
              │                    chroma/ (embeddings)  graph.json │
              └────────────────────────────────────────────────────┘
                                                │
                                                v
                                   ┌──────────────────────────┐
                                   │  Codegen LLM (4-layer    │
                                   │  context prompt)         │
                                   └──────────────────────────┘
```

### 2.2 Repomix Invocation

```bash
repomix --include "vault/**/*.md" \
        --ignore "vault/_quarantine/**" \
        --style xml \
        --output build/repomix-output.xml \
        --no-file-summary \
        --remove-comments
```

The XML output preserves each file's path inside a `<file path="...">` tag,
which the indexer uses as the primary key. Do **not** use `--style markdown`
or `--style plain`: they strip the path attribute and cause cross-file leakage
when chunks are retrieved.

### 2.3 Repomix XML Indexer (ChromaDB)

```python
# phase2/indexer.py
import re
import xml.etree.ElementTree as ET
from dataclasses import dataclass
from pathlib import Path

import chromadb
import yaml
from chromadb.utils import embedding_functions

WIKILINK_RE = re.compile(r"\[\[([a-z0-9_]+)(?:\|[^\]]+)?\]\]")
EMBED_MODEL = "text-embedding-3-small"  # 1536-dim, ~$0.02 / 1M tokens

@dataclass
class VaultChunk:
    path: str
    frontmatter: dict
    description: str
    mechanics: str
    references: str
    links_out: list[str]

def parse_repomix(xml_path: Path) -> list[VaultChunk]:
    tree = ET.parse(xml_path)
    chunks = []
    for file_el in tree.iter("file"):
        path = file_el.attrib["path"]
        body = file_el.text or ""
        fm, desc, mech, refs = _split_sections(body)
        chunks.append(VaultChunk(
            path=path,
            frontmatter=yaml.safe_load(fm) or {},
            description=desc,
            mechanics=mech,
            references=refs,
            links_out=WIKILINK_RE.findall(body),
        ))
    return chunks

def _split_sections(body: str) -> tuple[str, str, str, str]:
    # Frontmatter between --- fences, then ## Description / ## Behavioral
    # Mechanics / ## References. Missing sections become empty strings.
    parts = re.split(r"^---\s*$", body, maxsplit=2, flags=re.M)
    fm = parts[1] if len(parts) >= 3 else ""
    rest = parts[2] if len(parts) >= 3 else body
    sections = re.split(r"^## ", rest, flags=re.M)
    named = {s.split("\n", 1)[0].strip().lower(): s.split("\n", 1)[1]
             for s in sections if "\n" in s}
    return (fm,
            named.get("description", "").strip(),
            named.get("behavioral mechanics", "").strip(),
            named.get("references", "").strip())

def build_index(xml_path: Path, persist_dir: Path) -> None:
    client = chromadb.PersistentClient(path=str(persist_dir))
    embed_fn = embedding_functions.OpenAIEmbeddingFunction(model_name=EMBED_MODEL)

    # Two collections: prose for semantic recall, mechanics for exact-logic recall.
    prose = client.get_or_create_collection("vault_prose", embedding_function=embed_fn)
    mech  = client.get_or_create_collection("vault_mechanics", embedding_function=embed_fn)

    chunks = parse_repomix(xml_path)
    prose.upsert(
        ids=[c.path for c in chunks],
        documents=[c.description for c in chunks],
        metadatas=[{"path": c.path,
                    "id": c.frontmatter.get("id", ""),
                    "type": c.frontmatter.get("type", ""),
                    "links_out": ",".join(c.links_out)} for c in chunks],
    )
    mech.upsert(
        ids=[c.path for c in chunks],
        documents=[c.mechanics for c in chunks],
        metadatas=[{"path": c.path,
                    "id": c.frontmatter.get("id", ""),
                    "type": c.frontmatter.get("type", "")} for c in chunks],
    )

    # Persist the link graph for Layer 3's graph expansion.
    graph = {c.frontmatter.get("id", c.path): c.links_out for c in chunks}
    (persist_dir / "graph.json").write_text(__import__("json").dumps(graph))
```

### 2.4 Hybrid Retrieval — Vector + Graph

Layer 3 of the context filter. Returns at most 2,000 tokens of vault content
regardless of task breadth.

```python
# phase2/retrieval.py
import json
import tiktoken
from pathlib import Path
import chromadb

ENC = tiktoken.encoding_for_model("gpt-4o")
MAX_TOKENS = 2000

def retrieve(task: str, persist_dir: Path, seed_ids: list[str] | None = None
             ) -> list[dict]:
    client = chromadb.PersistentClient(path=str(persist_dir))
    graph = json.loads((persist_dir / "graph.json").read_text())

    # Step 1 — vector seeds. Pull top-k from each collection and dedupe by path.
    prose = client.get_collection("vault_prose")
    mech  = client.get_collection("vault_mechanics")
    hits = {}
    for col in (prose, mech):
        res = col.query(query_texts=[task], n_results=6)
        for path, meta in zip(res["ids"][0], res["metadatas"][0]):
            hits.setdefault(path, meta)

    # Step 2 — graph expansion. For each seed, pull one hop of [[wikilinks]].
    expanded = dict(hits)
    for path, meta in hits.items():
        for linked_id in graph.get(meta.get("id", ""), []):
            # naive id->path resolve via metadata scan
            linked = prose.get(where={"id": linked_id}).get("metadatas", [])
            for lm in linked:
                expanded.setdefault(lm["path"], lm)

    # Step 3 — pack under MAX_TOKENS, prioritizing direct vector hits.
    ordered = list(hits.items()) + [(p, m) for p, m in expanded.items()
                                    if p not in hits]
    packed, used = [], 0
    for path, meta in ordered:
        doc_prose = prose.get(ids=[path])["documents"][0]
        doc_mech  = mech.get(ids=[path])["documents"][0]
        block = f'<file path="{path}">\n{doc_prose}\n\n## Mechanics\n{doc_mech}\n</file>'
        cost = len(ENC.encode(block))
        if used + cost > MAX_TOKENS:
            continue
        packed.append(block)
        used += cost
    return packed
```

### 2.5 Layer Assembly & Codegen Call

```python
# phase2/codegen.py
from anthropic import Anthropic

ENGINE_BASELINE = Path("prompts/engine_baseline.md").read_text()  # ~2k tokens, cached
client = Anthropic()

def generate(task: str, system_map: str, persist_dir: Path) -> str:
    vault_chunks = "\n\n".join(retrieve(task, persist_dir))
    resp = client.messages.create(
        model="claude-opus-4-7",
        max_tokens=4096,
        system=[
            # Layer 1 — sticky prompt cache, hits every call.
            {"type": "text",
             "text": ENGINE_BASELINE,
             "cache_control": {"type": "ephemeral"}},
        ],
        messages=[{
            "role": "user",
            "content": (
                f"[CURRENT RECREATION PROGRESS]\n{system_map}\n\n"
                f"[SANITIZED OBSIDIAN VAULT SPECIFICATION (VIA REPOMIX XML)]\n"
                f"{vault_chunks}\n\n"
                f"[TRANSLATION CONSTRAINTS]\n"
                f"- Trust YAML frontmatter as absolute truth for numbers.\n"
                f"- No placeholders, no shorthand, no external deps.\n\n"
                f"[DEVELOPMENT GOAL]\n{task}"
            ),
        }],
    )
    return resp.content[0].text
```

**Caching note:** Layer 1 (`ENGINE_BASELINE`) is marked `cache_control:
ephemeral`. After the first call, subsequent calls within the cache TTL pay
only the ~10% cache-read rate for those tokens, which is the main mechanism
that makes the 4-layer scheme economical (see §3).

### 2.6 Layer 4 — System Mapping Document

Compress the multi-turn codegen history into a single rolling document after
each successful build. Wipe raw turn history; keep only this summary.

```yaml
# build/system_map.yaml — regenerated after every codegen turn
implemented:
  - id: iron_sword
    file: src/items/iron_sword.gd
    hash: a3f...
    verified_against: vault/items/weapons/Iron_Sword.md
pending:
  - id: steel_sword
    blocked_by: [iron_ingot]   # vault dep not yet implemented
test_state:
  passing: 47
  failing: 2
  failing_ids: [iron_sword_durability_test, iron_sword_bleed_proc_test]
last_engine_baseline_hash: 9c2...
```

The orchestrator regenerates this file via a cheap summarizer call (Haiku, ~$0.001
per turn) rather than pasting raw history into the next prompt. This is the
single biggest token saver across long sessions.

---

## Part 3 — Token Bookkeeping Comparison

Baseline scenario: a medium game wiki (1,200 pages, ~2.5M tokens of raw HTML),
recreating ~400 game mechanics over a 30-day dev cycle (~1,500 codegen turns).

### 3.1 Per-Turn Token Cost

| Component                       | Direct Raw-Wiki Approach | Phased Pipeline       |
| ------------------------------- | -----------------------: | --------------------: |
| Raw wiki HTML pasted in         |              ~45,000 tok |                 0 tok |
| Engine baseline (uncached)      |                2,000 tok |                 0 tok |
| Engine baseline (cached read)   |                        — |            ~200 tok\* |
| System mapping doc              |                        — |              ~600 tok |
| Retrieved vault chunks          |                        — |            ~2,000 tok |
| Task prompt + constraints       |                  ~400 tok|              ~400 tok |
| **Input subtotal**              |           **~47,400 tok**|        **~3,200 tok** |
| Output (generated code)         |                ~1,500 tok|            ~1,500 tok |
| **Total per turn**              |           **~48,900 tok**|        **~4,700 tok** |

\* Anthropic prompt cache reads are billed at ~10% of input rate.

### 3.2 30-Day Campaign Cost (1,500 turns, Claude Opus pricing)

Pricing assumption: $15 / 1M input tokens, $75 / 1M output tokens, $1.50 / 1M
cached-read tokens.

| Line item                          | Direct Approach | Phased Pipeline |
| ---------------------------------- | --------------: | --------------: |
| Phase 1 one-time compile (Haiku)   |             $0  |          ~$8    |
| Embeddings (text-embedding-3-small)|             $0  |          ~$0.05 |
| Codegen input (uncached)           |         $1,067  |          ~$108  |
| Codegen input (cached reads)       |             $0  |          ~$5    |
| Codegen output                     |          ~$169  |          ~$169  |
| **Total**                          |      **$1,236** |       **~$290** |
| **Savings**                        |               — | **~$946 (76%)** |

### 3.3 Performance & Quality Side-Effects

| Dimension                              | Direct Approach          | Phased Pipeline               |
| -------------------------------------- | ------------------------ | ----------------------------- |
| First-byte latency per turn            | 8–14 s                   | 1.5–3 s                       |
| Context window pressure                | 47k/200k (24%) per turn  | 3k/200k (1.5%) per turn       |
| Hallucinated stat values               | High (prose ambiguity)   | Near-zero (YAML is truth)     |
| Cross-file consistency                 | Drift after ~20 turns    | Stable (graph + system map)   |
| Re-run on wiki update                  | Full re-pay              | Incremental (changed-only)    |
| Onboarding a new mechanic              | Re-ingest full HTML      | One vault file + recompile    |

### 3.4 When the Phased Pipeline Doesn't Pay

The Phase 1 amortization breaks even at roughly **80–100 codegen turns**.
Below that — quick prototypes, single-mechanic spikes, throwaway demos — the
direct approach is cheaper because you never recoup the compile cost or the
indexer engineering time. The phased pipeline is designed for sustained
multi-week recreation projects, not one-shot generation.

---

## Part 4 — Deployment Checklist

1. **Phase 1 prerequisites**
   - [ ] `pipx install llm-wiki-compiler`
   - [ ] Wiki base URL + page-type router rules committed to `llmwiki.toml`
   - [ ] JSON-Schema files for each vault `type` checked into `schemas/`
   - [ ] `llmwiki ingest <url> && llmwiki compile` produces zero quarantined files

2. **Phase 2 prerequisites**
   - [ ] `npm install -g repomix` (or pin via `npx repomix@<ver>`)
   - [ ] `pip install chromadb tiktoken pyyaml anthropic`
   - [ ] `prompts/engine_baseline.md` written and < 2,500 tokens
   - [ ] `build/repomix-output.xml` regenerated as a pre-commit hook on vault changes
   - [ ] Chroma index rebuilt incrementally via `phase2/indexer.py`

3. **Runtime guardrails**
   - [ ] Retrieval cap enforced at 2,000 tokens (assert in `retrieve()`)
   - [ ] System mapping doc capped at 1,000 tokens (summarize-on-overflow)
   - [ ] Engine baseline cache hit-rate logged per turn; alert if < 80%
   - [ ] Every generated file linked back to its vault source path in a code header
