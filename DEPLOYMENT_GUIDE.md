# Deployment Guide

Architecture spec for the wiki-to-code pipeline. All three phases are
implemented. Phase 2 has been driven against live LLM CLIs and produced a
real Rust/Bevy crate under `game/` (84 modules across 5 kinds as of
2026-06-04, `cargo build` clean).

See `docs/plan.md` for implementation status and the active per-kind
backlog. See `CLAUDE.md` for agent-facing conventions and gotchas.

---

## 0. Pipeline shape

```
┌────────────────┐  Phase 0   ┌──────────────────┐  Phase 1   ┌─────────────┐  Phase 2   ┌──────────┐
│  Fandom Wiki   │ ─────────► │ game-config.json │ ─────────► │   vault/    │ ─────────► │ Game code│
│  (live URL)    │  taxonomy  │   (human-OK'd)   │   ingest   │ Obsidian MD │   codegen  │  game/   │
└────────────────┘            └──────────────────┘            └─────────────┘            └──────────┘
```

Each arrow is a separate Python entry point. All three phases are implemented;
Phase 2 codegen output lives at `game/` (Rust/Bevy crate, committed).

Target wiki for the reference deployment: **They Are Billions**
(`they-are-billions.fandom.com`).

---

## 1. Phase 0 — Taxonomy discovery

### 1.1 Purpose

Replace human guesswork about "what kinds of entities does this game have"
with an LLM-mediated read of the wiki's actual category structure. Output is
a `game-config.json` that drives Phase 1's category-to-kind routing.

### 1.2 Implementation

| Step | File | What it does |
| --- | --- | --- |
| Orchestrator | `scripts/phase0.py` | CLI entry; argparse, dispatches fetch → analyze → write |
| Category fetch | `scripts/phase0_fetch.py` | Paginated `action=query&list=allcategories`; top-2 `categorymembers` per category; maintenance-category filter |
| LLM analysis | `scripts/phase0_analyze.py` | Shells out to `claude -p` or `codex exec`; verbatim-title enforcement; JSON retry on malformed output; snake_case kind normalization |
| Proposal write | `scripts/phase0_write.py` | Emits `game-config.proposed.json`; prints ANSI-coloured unified diff; two-stage terminal confirmation that flips `human_approved: true` |

### 1.3 CLI

```powershell
python scripts/phase0.py [--wiki-url URL] [--min-members N] [--dry-run]
python scripts/phase0.py --llm-mode codex --model <model-id>
```

`--wiki-url` defaults to `game.wiki_base_url` in `game-config.json`.
`--min-members` (default 3) filters out micro-categories before the LLM call.
`--dry-run` prints the proposed JSON to stdout and skips the write/confirm.

### 1.4 `game-config.json` schema

```jsonc
{
  "version": 1,
  "game": { "name": "...", "wiki_base_url": "https://....fandom.com/wiki/" },
  "human_approved": false,         // Phase 0 LLM proposes; human flips to true
  "defaultKind": "concept",
  "kinds": {
    "<kind>": {
      "minWikilinks": 1,
      "description": "..."
    }
    // ... one entry per LLM-discovered kind
  },
  "categories": [
    { "name": "<Wiki Category>", "kind": "<kind>", "member_count": 0 }
    // ... one entry per category Phase 1 should ingest
  ],
  "seedPages": []
}
```

`human_approved: true` is a hard gate. `phase1_ingest.py` warns when the flag
is `false`; treat warning as a stop in production.

### 1.5 Reference deployment (They Are Billions)

15 categories routed to 8 kinds: `building` (Buildings, Wonders),
`unit` (Units), `enemy` (Infected, Special Infected, Runners, Walkers),
`mechanic` (Mechanics), `system` (Research), `location` (Campaign Maps,
Locations, Survival Maps), `npc` (Characters), `organization`
(Organizations, New Empire).

---

## 2. Phase 1 — Wiki → Vault

### 2.1 Purpose

Turn every page in every approved category into a strict, AI-readable
Markdown note. Each note has a typed YAML frontmatter block plus three
prose sections (`## Description`, `## Behavioral Mechanics`, `## References`).
Failures route to `vault/_quarantine/` rather than corrupting the vault.

### 2.2 Implementation

Single-file ingest at `scripts/phase1_ingest.py`. Pipeline per page:

```
MediaWiki API ──► wikitext + revid ──► cache lookup (SHA-256)
                                          │
                                  miss ───┴─── hit
                                   │            │
                         claude -p / codex exec │
                                   │            │
                                   ▼            ▼
                              raw markdown ─► cache write
                                          │
                                  frontmatter parse (hand-rolled YAML subset)
                                          │
                              jsonschema validate (universal + per-kind)
                                          │
                            ┌───── errors? ─────┐
                          yes                  no
                            │                   │
              vault/_quarantine/<slug>.md   vault/<kind>/<slug>.md
              with validation_errors: block
```

Configuration lives in `phase1.config.toml`. Routing lives in
`game-config.json`'s `categories` array. The compile system prompt lives in
`prompts/wiki-compile-system.md` and is hashed into the cache key.

### 2.3 CLI

```powershell
python scripts/phase1_ingest.py --dry-run           # category page counts
python scripts/phase1_ingest.py --limit 1           # one page per category
python scripts/phase1_ingest.py                     # full ingest
pip install jsonschema                              # optional: Draft 2020-12 validation
```

Exit code = number of files that ended up in `_quarantine/`. Zero is a clean
run; non-zero means inspect quarantine and iterate.

### 2.4 Frontmatter contract

Universal fields (required on every file, enforced by
`schemas/_universal.schema.json`):

| Field | Type | Notes |
| --- | --- | --- |
| `id` | string | `^[a-z0-9_]+$`, unique within vault |
| `name` | string | display name |
| `type` | enum | one of the `kinds` declared in `game-config.json` |
| `subtype` | string | type-specific discriminator (e.g. `weapon`, `ranger`) |
| `source_url` | uri | canonical wiki URL |
| `source_revision` | string | MediaWiki revid for idempotency |
| `extracted_at` | date-time | ISO-8601 |
| `confidence` | number | compiler's self-rated extraction confidence, `[0,1]` |
| `tags` | string[] | optional |
| `depends_on` | string[] | MUST mirror every `[[wiki_link]]` in the body |

Per-kind properties live in `game-config.json -> kinds.<kind>.frontmatter_schema.properties`. They are **type-only**: validation enforces declared types on fields that happen to be present, but no per-kind field is required. The universal schema's `required` list is the only presence gate. Per-kind `required: [...]` arrays were removed because they were never enforced and only fed the compile LLM stale "priority" hints.

### 2.5 Body contract

```markdown
## Description
Short prose. Every proper noun referring to another vault entity is a
[[wiki_link]] using the target file's snake_case `id`.

## Behavioral Mechanics
Bulleted programmatic logic. One conditional per bullet, leading with
IF / THEN / ON / WHILE.

- IF target has `tag:undead` THEN damage *= 1.5
- ON hit: 5% chance to apply [[status_bleed]] for 3s

## References
- Source: <wiki URL>
- Related: [[other_id]], [[another_id]]
```

`## Behavioral Mechanics` is the single source of truth for Phase 2 codegen
logic. The imperative `IF / THEN / ON / WHILE` form exists so a future
translation dictionary can lex it deterministically.

### 2.6 Idempotency & cache

- Cache directory: `.phase1_cache/` (gitignored).
- Cache key: `SHA-256(wikitext || system_prompt || model_id)`. Editing the
  compile prompt or swapping the model invalidates cleanly; unchanged source
  pages skip the LLM call on re-runs.
- Retries: 429 / 5xx / `URLError` / `TimeoutError` retry up to
  `[ingest].retry_count` times with exponential backoff capped at 30s.

### 2.7 LLM access

Phase 1 shells out to a CLI rather than using an SDK:

```python
# claude mode
["claude", "-p", "--model", model]
# codex mode
["codex", "exec", "--model", model, "-"]
```

`run_llm` in `phase1_ingest.py` is the single chokepoint. Adding a new
provider means adding a branch there, not introducing a new SDK dependency.

---

## 3. Phase 2 — Vault → Code (live)

**Live as of 2026-05-22; has scaled to 84 modules across 5 kinds by
2026-06-04.** All five pipeline modules ship with unit tests. The
`phase2/loop_driver.py` orchestrator gates each turn behind `cargo build
--manifest-path game/Cargo.toml`, byte-exactly reverts on failure, and runs a
build-error repair loop. Engine choice is Bevy + lockstep (see `docs/plan.md`
and `docs/MEMORY.md`). Live LLM calls still require in-session user
confirmation per `CLAUDE.md > Default Behaviors #12`.

### 3.1 Component map

```
              ┌────────────────────────────────────────────────────┐
              │              Phase 2 Orchestrator (Python)         │
              │                                                     │
   vault/  ──►│  RepomixBundler ──► ChromaIndexer ──► GraphMapper  │──► Retrieval API
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

### 3.2 Repomix bundling

```bash
repomix --include "vault/**/*.md" \
        --ignore "vault/_quarantine/**" \
        --style xml \
        --output build/repomix-output.xml \
        --no-file-summary \
        --remove-comments
```

XML style is mandatory. `markdown` and `plain` styles strip the
`<file path="...">` attribute that the indexer uses as the primary key.

### 3.3 Indexer

`phase2/indexer.py` parses the XML, splits each file into
frontmatter / Description / Behavioral Mechanics / References, and upserts
two Chroma collections:

- `vault_prose` — embeds the Description (semantic recall)
- `vault_mechanics` — embeds the Behavioral Mechanics block (exact-logic recall)

Plus a sidecar `build/graph.json` mapping each `id` to its outbound
`[[wikilink]]` ids, sourced from `depends_on:` in frontmatter.

Embeddings use a local sentence-transformers model
(`BAAI/bge-small-en-v1.5`, 384-dim) loaded via Chroma's
`SentenceTransformerEmbeddingFunction`, so retrieval reuses the same model
at query time. Run with `--dry-run` to skip the Chroma upsert and only
write `graph.json`.

### 3.4 Retrieval

`phase2/retrieval.py` runs Layer 3 of the 4-layer filter:

1. Vector seeds — top-k from each collection, merged by distance and
   deduped by id (`merge_vector_results`).
2. Graph expansion — 1-hop along `build/graph.json` from each seed,
   seeds-first (`graph_expand`).
3. Token-cap pack — emit `<file path="...">…</file>` blocks until 2,000
   tokens, prioritising direct vector hits over graph-expanded files
   (`pack_files`).

The 2,000-token cap is asserted, not advisory: a post-pack
`assert token_counter(packed) <= cap_tokens` defends against per-block
vs joined-string drift in the tokenizer. Token counts use tiktoken
`cl100k_base` as the local approximation since Anthropic does not ship
a public Claude tokenizer.

### 3.5 Codegen call

`phase2/codegen.py` wraps the Anthropic SDK. The engine baseline (Layer 1,
rendered ahead of time by `phase2/baseline.py` into `build/engine_baseline.md`)
goes in the `system` block with `cache_control: ephemeral`:

```python
client.messages.create(
    model="claude-opus-4-7",
    max_tokens=4096,
    system=[{
        "type": "text",
        "text": ENGINE_BASELINE,                    # < 2,500 tokens
        "cache_control": {"type": "ephemeral"},     # Layer 1 cache
    }],
    messages=[{
        "role": "user",
        "content": (
            f"[CURRENT RECREATION PROGRESS]\n{system_map}\n\n"
            f"[SANITIZED OBSIDIAN VAULT SPECIFICATION]\n{vault_chunks}\n\n"
            f"[TRANSLATION CONSTRAINTS]\n"
            f"- YAML frontmatter is absolute truth for numbers.\n"
            f"- No placeholders, no shorthand, no external deps.\n\n"
            f"[DEVELOPMENT GOAL]\n{task}"
        ),
    }],
)
```

Prompt-cache hit on the engine baseline is the main cost mechanism — see §4.
Per-turn usage is logged via `log_usage(response.usage)`; the helper warns
when `cache_read / (cache_read + cache_creation + input)` drops below 80%,
which signals the baseline has drifted or the 5-minute ephemeral TTL expired
between turns. Generated output is post-checked by `validate_source_header`,
which rejects responses missing a `// Sources: vault/...` header or citing
paths outside the retrieval bundle.

### 3.6 System mapping document

`build/system_map.yaml` is Layer 4: a rolling summary of what's been
generated so far, managed by `phase2/system_map.py`. Default compaction
is deterministic (oldest `implemented` entries collapse into a single
`{summary, count}` line via `cap_tokens`); an optional
`summarise_with_haiku` path swaps in a Haiku-driven rewrite for richer
summarisation. Tracks:

```yaml
implemented:
  - { id: ..., file: ..., hash: ..., verified_against: vault/.../*.md }
pending:
  - { id: ..., blocked_by: [dep_id, ...] }
test_state:
  passing: N
  failing: N
  failing_ids: [...]
last_engine_baseline_hash: ...
```

### 3.7 The 4-layer context filter

| Layer | Contents | Token budget | Purpose |
| --- | --- | --- | --- |
| 1 — Engine baseline | Target engine + language, architectural patterns, YAML→data-object translation dictionary | 1,500–2,500 | Sticky prompt cache; ensures code uniformity |
| 2 — Vault index & task router | Repomix XML tree + task dispatch | ~500 | Picks which file paths matter for the task |
| 3 — Hybrid retrieval | Vector + graph chunks from §3.4 | ≤ 2,000 | Per-task relevant vault content |
| 4 — System map | `build/system_map.yaml` | ≤ 1,000 | Replaces raw turn history; tracks state |

---

## 4. Token bookkeeping

Reference scenario: medium game wiki (1,200 pages, ~2.5M tokens of raw HTML),
recreating ~400 mechanics over 30 days of dev (~1,500 codegen turns).

### 4.1 Per-turn token cost

| Component                       | Direct Raw-Wiki  | Phased Pipeline    |
| ------------------------------- | ---------------: | -----------------: |
| Raw wiki HTML pasted in         |       ~45,000 tok|              0 tok |
| Engine baseline (uncached)      |        2,000 tok |              0 tok |
| Engine baseline (cached read)   |                — |         ~200 tok\* |
| System mapping doc              |                — |           ~600 tok |
| Retrieved vault chunks          |                — |         ~2,000 tok |
| Task prompt + constraints       |          ~400 tok|           ~400 tok |
| **Input subtotal**              |   **~47,400 tok**|     **~3,200 tok** |
| Output (generated code)         |        ~1,500 tok|         ~1,500 tok |
| **Total per turn**              |   **~48,900 tok**|     **~4,700 tok** |

\* Anthropic prompt-cache reads are billed at ~10% of input rate.

### 4.2 30-day campaign cost (Claude Opus pricing)

Assumptions: $15 / 1M input, $75 / 1M output, $1.50 / 1M cached-read.

| Line item                          | Direct       | Phased       |
| ---------------------------------- | -----------: | -----------: |
| Phase 1 one-time compile (Haiku)   |          $0  |        ~$8   |
| Embeddings (text-embedding-3-small)|          $0  |       ~$0.05 |
| Codegen input (uncached)           |     $1,067   |       ~$108  |
| Codegen input (cached reads)       |          $0  |        ~$5   |
| Codegen output                     |       ~$169  |       ~$169  |
| **Total**                          | **$1,236**   |  **~$290**   |
| **Savings**                        |            — | ~$946 (76%)  |

### 4.3 When this architecture doesn't pay

Phase 1 amortises at roughly **80–100 codegen turns**. Below that — quick
prototypes, single-mechanic spikes, throwaway demos — direct prompting is
cheaper because the compile cost and indexer engineering time never get
recouped. This pipeline is designed for sustained multi-week recreation
projects.

---

## 5. Deployment checklist

### 5.1 Phase 0 (one-time per target wiki)

- [ ] `game-config.json` exists with `game.wiki_base_url` set
- [ ] `claude` or `codex` CLI authenticated locally
- [ ] `python scripts/phase0.py` run; diff reviewed
- [ ] `human_approved: true` after sign-off

### 5.2 Phase 1 (per ingest run)

- [ ] `human_approved: true` in `game-config.json`
- [ ] Per-kind schema exists for every approved kind (or accept universal-only
      validation + one-time warning)
- [ ] `claude` or `codex` CLI authenticated for the configured `[compile].llm_mode`
- [ ] `python scripts/phase1_ingest.py --dry-run` shows expected page counts
- [ ] `python scripts/phase1_ingest.py` returns exit code 0, or `_quarantine/`
      contents reviewed and prompt/schema iterated

### 5.3 Phase 2 (live; 84 modules generated)

- [x] Target engine chosen (Bevy + lockstep, 2026-05-19; see `plan.md`)
- [x] `npm install -g repomix` (2026-05-21)
- [x] `pip install -r requirements-phase2.txt` covers chromadb,
      sentence-transformers, tiktoken, anthropic, pyyaml (2026-05-22)
- [x] `prompts/engine_baseline.template.md` + `prompts/engine_determinism/<engine>.md`
      render via `phase2/baseline.py` to `build/engine_baseline.md`
      (2,104 / 2,500 tokens for Bevy + lockstep)
- [x] `phase2/indexer.py`, `phase2/retrieval.py`, `phase2/baseline.py`,
      `phase2/codegen.py`, `phase2/system_map.py` all shipped with unit tests
- [x] Retrieval cap asserted at 2,000 tokens
      (`phase2.retrieval.pack_files`)
- [x] System map capped at 1,000 tokens
      (`phase2.system_map.cap_tokens`; `summarise_with_haiku` available)
- [x] Engine-baseline cache hit-rate logged; warn if < 80%
      (`phase2.codegen.log_usage`; validates only after a real turn ships)
- [x] Every generated source file should start with `// Sources: vault/...`
      (output rule in `prompts/engine_baseline.template.md`; post-check in
      `phase2.codegen.validate_source_header`)
- [x] First live codegen turn spent (soldier, 2026-05-22)
- [x] Loop driver shipped (`phase2/loop_driver.py`) — cargo-gated, byte-exact
      revert, driver-owned module registration, sibling-exemplar consistency,
      build-error repair loop, parallel codegen
- [x] 84 modules generated across `units`, `buildings`, `game_mechanics`,
      `wonders`, `infected` (2026-06-04); `cargo build` clean
- [ ] Finish remaining vault kinds (`infected_runners/special/walkers`, 1
      wonder, plus decide on data-shaped kinds — see `docs/todo.md`)
- [ ] `main.rs` Bevy `App` that wires plugins, spawns entities, ticks the sim
- [ ] First SDK-mode turn (only when `ANTHROPIC_API_KEY` is available) to
      validate the `cache_control: ephemeral` math in §4
