# Deployment Guide

Architecture spec for the wiki-to-code pipeline. Describes the system as it
exists today (Phases 0–1) plus the design for the unimplemented Phase 2.

See `plan.md` for implementation status and the active blocker (target engine
selection). See `CLAUDE.md` for agent-facing conventions and gotchas.

---

## 0. Pipeline shape

```
┌────────────────┐  Phase 0   ┌──────────────────┐  Phase 1   ┌─────────────┐  Phase 2   ┌──────────┐
│  Fandom Wiki   │ ─────────► │ game-config.json │ ─────────► │   vault/    │ ─────────► │ Game code│
│  (live URL)    │  taxonomy  │   (human-OK'd)   │   ingest   │ Obsidian MD │   codegen  │  (TBD)   │
└────────────────┘            └──────────────────┘            └─────────────┘            └──────────┘
```

Each arrow is a separate Python entry point. Phases 0 and 1 are implemented;
Phase 2 is planned but not yet built.

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

Per-kind required fields (extend the universal schema via `allOf`):

| `type`     | Required sub-fields                                                |
| ---------- | ------------------------------------------------------------------ |
| `item`     | `stats`, `requirements`, `tags`                                    |
| `skill`    | `cost`, `cooldown`, `effects`, `scaling`                           |
| `enemy`    | `stats`, `loot_table`, `ai_profile`, `resistances`                 |
| `mechanic` | `formula`, `inputs`, `outputs`, `edge_cases`                       |
| `quest`    | `prerequisites`, `objectives`, `rewards`, `flags_set`              |
| `system`   | `controlled_entities`, `state_transitions`                         |
| `location` | `map_layout`, `objectives`, `difficulty`, `enemy_spawns`           |
| `npc`      | `role`, `relationships`, `lore`, `affiliations`                    |
| `building` | _per-kind schema not yet written — universal-only validation_      |
| `unit`     | _per-kind schema not yet written — universal-only validation_      |
| `organization` | _per-kind schema not yet written — universal-only validation_  |

Closing the last three is tracked in `plan.md`.

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

## 3. Phase 2 — Vault → Code (planned)

**Not implemented.** Blocked on the target engine decision (see `plan.md`).
This section is design intent, not a code description.

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

`phase2/indexer.py` (to be written) parses the XML, splits each file into
frontmatter / Description / Behavioral Mechanics / References, and upserts
two Chroma collections:

- `vault_prose` — embeds the Description (semantic recall)
- `vault_mechanics` — embeds the Behavioral Mechanics block (exact-logic recall)

Plus a sidecar `graph.json` mapping each `id` to its outbound `[[wikilink]]`
ids, sourced from `depends_on:` in frontmatter.

### 3.4 Retrieval

`phase2/retrieval.py` (to be written) runs Layer 3 of the 4-layer filter:

1. Vector seeds — top-k from each collection, deduped by path.
2. Graph expansion — 1-hop along `graph.json` from each seed.
3. Token-cap pack — emit `<file path="...">…</file>` blocks until 2,000
   tokens, prioritising direct vector hits over graph-expanded files.

The 2,000-token cap is asserted, not advisory.

### 3.5 Codegen call

`phase2/codegen.py` (to be written) wraps the Anthropic SDK:

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

### 3.6 System mapping document

`build/system_map.yaml` (to be written) is Layer 4: a rolling summary of
what's been generated so far. Regenerated after every codegen turn by a
cheap Haiku summariser, replacing raw multi-turn history. Tracks:

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

### 5.3 Phase 2 (not yet operational)

- [ ] Target engine chosen (active blocker — see `plan.md`)
- [ ] `npm install -g repomix`
- [ ] `pip install chromadb tiktoken anthropic`
- [ ] `prompts/engine_baseline.md` written and < 2,500 tokens
- [ ] `phase2/indexer.py`, `phase2/retrieval.py`, `phase2/codegen.py` implemented
- [ ] Retrieval cap asserted at 2,000 tokens
- [ ] System map capped at 1,000 tokens (summarise-on-overflow)
- [ ] Engine-baseline cache hit-rate logged; alert if < 80%
- [ ] Every generated source file links back to its vault source path
