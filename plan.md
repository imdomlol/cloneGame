# Implementation Plan

Tracks what's built, what's in progress, and what's next for the wiki-to-code
pipeline. See `DEPLOYMENT_GUIDE.md` for architecture; see `CLAUDE.md` for
agent-facing conventions.

Status keys: `[x]` done · `[~]` partial · `[ ]` not started · `[!]` blocked

---

## Current stage

**Phase 1 runnable but hard-coded to They Are Billions. Phase 0 v2 in progress
to make the pipeline truly game-agnostic. Phase 2 not started.**

The pipeline can take `they-are-billions.fandom.com` and produce a validated
Obsidian vault on disk. But the per-kind frontmatter contracts live as
hand-authored `schemas/*.schema.json` files — that means every new target
game requires manual schema coding. **Phase 0 v2** moves this knowledge into
data so the pipeline can reverse-engineer arbitrary games.

---

## Architectural principle (added 2026-05-19)

**Anything that varies per target game lives in `game-config.json` as data,
not in Python or as standalone JSON-Schema files.** The pipeline should be
able to ingest any wiki and produce a vault without code edits.

Stays as code (genuinely game-agnostic):

- `_universal.schema.json` — every entity needs `id`, `name`, `type`,
  `source_url`, `source_revision`, `extracted_at`, `confidence`, `depends_on`.
- Phase 0/1/2 driver scripts.
- The compile system prompt's structural contract (frontmatter + 3 body
  sections + `IF/THEN/ON/WHILE` form).

Moves to data:

- Per-kind frontmatter shape → `kinds.<kind>.frontmatter_schema` in
  `game-config.json`.
- Engine choice → `engine_candidates` + `chosen_engine` in
  `game-config.json`, LLM-proposed and human-approved.
- Translation dictionary (YAML → engine data objects) → derived from
  `chosen_engine` plus a Phase 2 baseline-prompt generator.

---

## Phase 0 — Taxonomy + schema + engine discovery

Goal: turn raw wiki signals into an approved `game-config.json` whose
`kinds`, `categories`, `frontmatter_schema` per kind, and `engine_candidates`
drive Phase 1 + Phase 2.

### Phase 0 v1 — Taxonomy (done)

- [x] MediaWiki fetcher — `scripts/phase0_fetch.py`
- [x] LLM analyzer (kinds + categories) — `scripts/phase0_analyze.py`
- [x] Proposal writer + diff/approval — `scripts/phase0_write.py`,
      `scripts/phase0.py`
- [x] Reference run approved for They Are Billions — 15 categories → 8 kinds

### Phase 0 v2 — Data-driven schemas + engine selection (in progress)

Actionable steps, in dependency order:

- [ ] **Migrate existing per-kind schemas into `game-config.json`** —
      Codex session 1. Read each of `schemas/{item,skill,enemy,mechanic,
      quest,system,location,npc}.schema.json`, inline each one's `required`
      list and `properties` map under `kinds.<kind>.frontmatter_schema`,
      delete the migrated files. Keep `_universal.schema.json` as code.
- [ ] **Refactor Phase 1 schema loader** — same Codex session 1. In
      `phase1_ingest.py:validation_errors`, stop reading
      `schemas/<kind>.schema.json` from disk; build the per-kind schema
      from `game-config.json -> kinds.<kind>.frontmatter_schema`. Keep the
      "no per-kind schema → universal-only + one-time warning" fallback for
      kinds that come back empty.
- [ ] **Add Phase 0 schema-proposal LLM step** — Codex session 2. New
      function in `phase0_analyze.py` that, given the v1 taxonomy output
      plus 1–3 sample pages per category, returns a `frontmatter_schema`
      (JSON-Schema-shaped `{required: [...], properties: {...}}`) for each
      kind. Remove the hard-coded `REQUIRED_FIELDS_BY_KIND` and
      `BASELINE_KINDS` constants — they become LLM-derived per game.
- [ ] **Add Phase 0 engine-candidate proposal step** — Codex session 3.
      New function in `phase0_analyze.py` that takes high-signal data
      (entity counts per kind, category names hinting at real-time/turn-
      based/multiplayer, total page count) and proposes 2–4 engine
      candidates as `[{name, language, pros, cons, fit_score, links}]`.
      Human picks one by setting `game-config.json -> chosen_engine` post-
      approval.
- [ ] **Wire Phase 0 v2 through `phase0.py` + `phase0_write.py`** — after
      the three Codex sessions land. Call analyze functions in sequence
      (taxonomy → schemas → engines), merge into one proposal shape,
      render `frontmatter_schema` and `engine_candidates` blocks in the
      diff for human review.
- [ ] **Re-run Phase 0 v2 against `they-are-billions.fandom.com`** — the
      reference deployment. Verify LLM-proposed schemas match (or improve
      on) the migrated hand-written ones; verify engine candidates look
      sane (expect Bevy/Rust, Unity DOTS, Godot+ECS at the top — see
      reference below).

### Engine candidates reference — They Are Billions multiplayer

The user's stated goal: solve the multiplayer gap in They Are Billions,
with tens of thousands of enemy entities active on the map at all times.
That problem is dominated by two constraints — **deterministic simulation**
(can't replicate 30k entity states across the wire; must lockstep on
inputs only) and **ECS-shaped data layout** (entity counts that high
demand data-oriented design). Phase 0 v2's engine proposer should surface
these or better; the list below is reference, not hard-coded.

| Engine | Pros | Cons |
| --- | --- | --- |
| **Bevy / Rust** | ECS-native, Rust gives binary-deterministic math out of the box (fixed-point is straightforward), `bevy_replicon` / `lightyear` for lockstep, open source, code-only surface keeps codegen scope small | Rust learning curve; engine still pre-1.0, breaking changes |
| **Unity DOTS / C#** | Mature DOTS handles 100k+ entities, Netcode for Entities is production-grade, mainstream tooling and asset ecosystem | C# IL2CPP floating-point determinism is fragile across platforms — needs `Unity.Mathematics.fixed` or third-party deterministic math; engine is closed-source |
| **Godot 4 + ECS bolt-on** | Open source, fast iteration, scripting close to Python, decent built-in multiplayer | No native ECS — must integrate `godot-ecs` or similar; not battle-tested at 10k+ entity counts; networking is state-replication first, not lockstep |
| **Custom (Flecs + SDL/Vulkan)** | Maximum control over determinism + entity count ceiling | Massive engineering cost; only worth it if codegen output is the *only* source |

Phase 2 reads `chosen_engine` and generates code targeting that engine's
idioms.

---

## Phase 1 — Wiki → Vault

Goal: turn each approved category's pages into schema-conformant Obsidian
vault notes under `vault/<kind>/<slug>.md`.

### Driver (done)

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
- [x] Inline validator with `jsonschema` (Draft 2020-12) and required-fields
      fallback when the library isn't installed
- [x] Quarantine routing — failures written to `vault/_quarantine/<slug>.md`
      with a `validation_errors:` block prepended
- [x] One-time warning when an approved kind has no per-kind schema
- [~] Per-kind schemas — currently 5 of 8 kinds have files
      (`enemy`, `mechanic`, `system`, `location`, `npc` + seed `item`,
      `skill`, `quest`). `building`, `unit`, `organization` are missing.
      **Will be obsoleted by Phase 0 v2** — per-kind schemas move into
      `game-config.json`. Do not hand-author the 3 missing files; the
      Phase 0 v2 LLM proposer will generate them along with the rest.

### Compile prompt

- [x] `prompts/wiki-compile-system.md` enforces YAML frontmatter +
      `## Description` / `## Behavioral Mechanics` / `## References` body
      sections, `IF / THEN / ON / WHILE` form, and `[[wiki_link]]` →
      `depends_on:` mirroring

### Outstanding before bulk ingest

- [ ] Phase 0 v2 lands and `game-config.json` carries per-kind
      `frontmatter_schema` for all 8 kinds
- [ ] Authenticate `claude` or `codex` CLI locally
- [ ] `python scripts/phase1_ingest.py --dry-run` — verify page counts
- [ ] `python scripts/phase1_ingest.py` — full ingest
- [ ] Inspect `vault/_quarantine/` and iterate on the compile prompt or
      `frontmatter_schema` if the quarantine rate is non-trivial

---

## Phase 2 — Vault → Code

Goal: take the sanitized vault, retrieve relevant chunks per task, and
generate production-ready game code with a Claude codegen loop.

**Status: not started. No longer blocked on a hard-coded engine choice —
engine selection becomes a Phase 0 v2 LLM proposal + human approval. Phase
2 reads `chosen_engine` from `game-config.json` and adapts.**

### Engineering tasks (greenfield)

- [ ] `npm install -g repomix` (or pin via `npx repomix@<ver>`)
- [ ] `pip install chromadb tiktoken anthropic`
- [ ] `build/repomix-output.xml` regeneration on vault change
      (pre-commit hook), excluding `vault/_quarantine/**`
- [ ] `phase2/indexer.py` — Repomix XML → two Chroma collections
      (`vault_prose`, `vault_mechanics`) + `graph.json` for `[[wikilink]]`
      adjacency
- [ ] `phase2/retrieval.py` — vector seeds + 1-hop graph expansion,
      hard-capped at 2,000 tokens
- [ ] `prompts/engine_baseline.template.md` — Layer 1 sticky cache,
      < 2,500 tokens. **Template** with placeholders that a small
      generator fills in from `chosen_engine` + `kinds.*.frontmatter_schema`,
      producing the per-game engine baseline at first Phase 2 run.
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
| Custom hand-rolled YAML frontmatter parser | Phase 1 ingest | The compile prompt produces a constrained YAML subset; a small parser avoids a `pyyaml` dependency. If the compile prompt drifts, switch to `pyyaml` rather than growing the parser. |
| `human_approved` gate in `game-config.json` | Phase 0 | Cheap insurance against an LLM-proposed taxonomy slipping into production ingest. Phase 1 warns when the flag is `false`. |
| **Per-kind schemas as data in `game-config.json`, not hand-coded files** | 2026-05-19 | Original design coupled schemas to code, requiring manual authoring per game. Goal is to reverse-engineer arbitrary games — anything game-specific must live in `game-config.json`. Phase 0 v2 LLM proposes; human approves via existing diff gate. |
| **Engine selection as LLM proposal + human approval, not hard-coded** | 2026-05-19 | Different games need different engines (RTS with 30k entities ≠ turn-based card game). Engine choice cascades into Phase 2 codegen, so it belongs alongside taxonomy in `game-config.json`, gated by the same `human_approved` flag. |

---

## Open questions

- **Embedding provider** for the Phase 2 Chroma index — the guide assumes
  OpenAI `text-embedding-3-small`. Standardise on Anthropic-only via a local
  embedder, or accept the OpenAI dependency?
- **Concurrency in Phase 1 ingest** — `phase1.config.toml` declares
  `concurrency = 4` but `phase1_ingest.py` runs sequentially. Wire up
  parallel fetch + compile, or leave sequential until bulk-ingest perf is
  actually a problem?
- **Schema-proposal sampling size** — how many representative pages per
  category should the Phase 0 v2 schema proposer see? Too few and the
  schema underfits; too many and the prompt blows the context budget.
  Default 2; tune after first multi-game runs.
