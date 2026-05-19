# Implementation Plan

Tracks what's built, what's in progress, and what's next for the wiki-to-code
pipeline. See `DEPLOYMENT_GUIDE.md` for architecture; see `CLAUDE.md` for
agent-facing conventions.

Status keys: `[x]` done · `[~]` partial · `[ ]` not started · `[!]` blocked

---

## Current stage

**Phase 1 complete (end-to-end runnable). Phase 2 not started.**

The pipeline can take `they-are-billions.fandom.com` and produce a validated
Obsidian vault on disk. The next decision — and the gate to Phase 2 — is
choosing the target game engine. No vault data has been ingested in bulk yet;
the smoke tests cover `--dry-run` and `--limit 1`.

---

## Phase 0 — Taxonomy discovery

Goal: turn raw MediaWiki categories into an approved `game-config.json` whose
`kinds` and `categories` arrays drive Phase 1.

- [x] Phase 0 ↔ Phase 1 contract defined — `game-config.json` schema
- [x] Baseline `kinds` seeded from `DEPLOYMENT_GUIDE.md` §1.3
- [x] MediaWiki fetcher — `scripts/phase0_fetch.py`
      - Paginated `allcategories` + top-2 `categorymembers` per category
      - Heuristic maintenance-category filter
- [x] LLM analyzer — `scripts/phase0_analyze.py`
      - Headless `claude -p` or `codex exec` (configurable)
      - Verbatim-title enforcement, JSON retry, snake_case kind normalization
- [x] Proposal writer + orchestrator — `scripts/phase0_write.py`,
      `scripts/phase0.py`
      - Emits `game-config.proposed.json` with ANSI-coloured unified diff
      - Two-stage terminal confirmation; flips `human_approved: true` on approval
- [x] Human review gate — completed for They Are Billions
      - `human_approved: true` is set in `game-config.json`
      - 15 categories routed to 8 kinds (`building`, `unit`, `enemy`,
        `mechanic`, `system`, `location`, `npc`, `organization`)

Phase 0 is closed for the current target wiki. Re-run only if the target
changes or the wiki's category structure shifts significantly.

---

## Phase 1 — Wiki → Vault

Goal: turn each approved category's pages into schema-conformant Obsidian
vault notes under `vault/<kind>/<slug>.md`.

### Driver

- [x] Single-file Python ingest — `scripts/phase1_ingest.py`
      - MediaWiki API enumeration via `categorymembers` paging
      - Wikitext fetch via `revisions` + `rvslots=main`
      - Headless LLM compile (`claude -p` / `codex exec`)
      - SHA-256 cache keyed on `(wikitext + system prompt + model id)`
      - Exponential-backoff retry on 429 / 5xx / `URLError` / `TimeoutError`
      - `--dry-run` enumerates page counts per category
      - `--limit N` ingests N pages per category for smoke testing

### Validation

- [x] Universal schema — `schemas/_universal.schema.json`
- [x] Per-kind schemas (5 of 8) — `enemy`, `mechanic`, `system`, `location`,
      `npc` plus `item`, `skill`, `quest` carried from the seed taxonomy
- [x] Inline validator with `jsonschema` (Draft 2020-12) and a required-fields
      fallback when the library isn't installed
- [x] Quarantine routing — failures written to `vault/_quarantine/<slug>.md`
      with a `validation_errors:` block prepended
- [x] One-time warning when a Phase-0-approved kind has no per-kind schema
- [~] Per-kind schemas for `building`, `unit`, `organization` — **missing**
      - These three kinds currently validate against the universal schema only
      - Required to close the validation gap before bulk ingest

### Compile prompt

- [x] `prompts/wiki-compile-system.md` enforces:
      - YAML frontmatter + `## Description` / `## Behavioral Mechanics` /
        `## References` body sections
      - `IF / THEN / ON / WHILE` imperative form in Behavioral Mechanics
      - `[[wiki_link]]` mirroring into `depends_on:`

### Outstanding before bulk ingest

- [ ] Write `schemas/{building,unit,organization}.schema.json`
- [ ] Authenticate `claude` or `codex` CLI locally (the `[compile] llm_mode`)
- [ ] Run full ingest: `python scripts/phase1_ingest.py`
- [ ] Inspect `vault/_quarantine/` and iterate on the compile prompt or
      extend `kinds` if the quarantine rate is non-trivial

---

## Phase 2 — Vault → Code

Goal: take the sanitized vault, retrieve relevant chunks per task, and
generate production-ready game code with a Claude codegen loop.

**Status: not started. Blocked on target engine selection.**

### Engineering tasks (greenfield)

- [ ] **Target engine decision** — Godot/GDScript? Unity/C#? Bevy/Rust?
      Drives `prompts/engine_baseline.md` content and the codegen output
      directory layout.
- [ ] `npm install -g repomix` (or pin via `npx repomix@<ver>`)
- [ ] `pip install chromadb tiktoken anthropic`
- [ ] `build/repomix-output.xml` regeneration on vault change
      (pre-commit hook), excluding `vault/_quarantine/**`
- [ ] `phase2/indexer.py` — Repomix XML → two Chroma collections
      (`vault_prose`, `vault_mechanics`) + `graph.json` for `[[wikilink]]`
      adjacency
- [ ] `phase2/retrieval.py` — vector seeds + 1-hop graph expansion,
      hard-capped at 2,000 tokens
- [ ] `prompts/engine_baseline.md` — Layer 1 sticky cache, < 2,500 tokens
- [ ] `phase2/codegen.py` — Anthropic SDK call with `cache_control: ephemeral`
      on the engine baseline
- [ ] `build/system_map.yaml` — rolling Haiku-summarised state of generated
      code, regenerated each turn

### Runtime guardrails

- [ ] Retrieval cap asserted at 2,000 tokens
- [ ] System map capped at 1,000 tokens (summarise-on-overflow)
- [ ] Engine-baseline cache hit-rate logged per turn; alert if < 80%
- [ ] Every generated source file links back to its vault source path in a
      code header

---

## Decision log

| Decision | When | Why |
| --- | --- | --- |
| Headless LLM CLI (`claude -p` / `codex exec`) instead of SDK | Phase 0 / Phase 1 | Avoids an SDK dependency tree and reuses the user's existing auth. `run_llm` is the single chokepoint to swap if needed. |
| Cache key = SHA-256 of `(wikitext + system prompt + model id)` | Phase 1 ingest | Catches all three inputs that change compile output. Unchanged source pages can be re-ingested with no LLM cost; prompt edits or model swaps invalidate cleanly. |
| Missing per-kind schemas → universal-only validation + one-time warning | Phase 1 ingest | Phase 0 approved `building`, `unit`, `organization` for They Are Billions, but per-kind schemas weren't written before the first run. Keeps the pipeline runnable while flagging the gap. |
| Custom hand-rolled YAML frontmatter parser | Phase 1 ingest | The compile prompt produces a constrained YAML subset; a small parser avoids a `pyyaml` dependency. If the compile prompt drifts, switch to `pyyaml` rather than growing the parser. |
| `human_approved` gate in `game-config.json` | Phase 0 | Cheap insurance against an LLM-proposed taxonomy slipping into production ingest. Phase 1 warns when the flag is `false`. |
| Hold Phase 2 until target engine is chosen | now | Engine choice cascades into every Phase 2 component (baseline prompt, codegen output paths, translation dictionary). Starting Phase 2 before that decision means rework. |

---

## Open questions

- **Target game engine** for Phase 2 codegen — Godot/GDScript, Unity/C#, or
  Bevy/Rust? This is the active blocker.
- **Embedding provider** for the Phase 2 Chroma index — the guide assumes
  OpenAI `text-embedding-3-small`. Standardise on Anthropic-only via a local
  embedder, or accept the OpenAI dependency?
- **Concurrency in Phase 1 ingest** — `phase1.config.toml` declares
  `concurrency = 4` but `phase1_ingest.py` runs sequentially. Wire up
  parallel fetch + compile, or leave sequential until bulk-ingest perf is
  actually a problem?
