# Implementation Plan

Tracks what's built, what's in progress, and what's next for the wiki-to-code
pipeline. See `DEPLOYMENT_GUIDE.md` for architecture; see `CLAUDE.md` for
agent-facing conventions.

Status keys: `[x]` done · `[~]` partial · `[ ]` not started · `[!]` blocked · `[-]` deferred / low priority

---

## Current stage

**Phase 0 v3 + Phase 1 wired. Phase 2 loop driver mature: codegen → parse →
merge → driver-owned module registration (incl. crate-root wiring) →
`cargo build` gate → record/revert, with `--from-vault` (kind-filtered) and
opt-in `--concurrency` parallel codegen. All 9 `units` + 1 building compile
in `game/`; every unit uses the shared `crate::sim::Health`/`UnitStats`
contract. Next: smoke test (now unblocked), then scale remaining kinds, then
a runnable `main.rs`. See todo.md.**

Per-kind frontmatter contracts live as data in `game-config.json` under
`kinds.<kind>.frontmatter_schema`. Phase 0 grows two LLM passes that
propose schemas and engine candidates per target game, and a coverage
gate that forces every input category to be mapped or explicitly dropped.
Phase 1's validator sources schemas from the config and resolves
singular/plural drift in `type:` against the configured kind names.

Phase 2 layout: `phase2/{indexer,retrieval,baseline,codegen,system_map,driver,loop_driver}.py`,
template at `prompts/engine_baseline.template.md`, per-engine determinism
rules under `prompts/engine_determinism/<engine>.md`. Renders to `build/`.
Generated code lands at `game/` and is gated by `cargo build`. Aggregator
files (mod.rs) are driver-owned via `chosen_engine.module_registration`
data — the LLM writes only leaf modules.

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
- Per-engine determinism rules → `prompts/engine_determinism/<engine>.md`,
  picked by lowercased `chosen_engine.name`. Adding a new engine is a
  markdown file, not a code change.
- Module-graph registration → `chosen_engine.module_registration`
  (`{aggregator, declaration}` templates). The loop driver renders these
  rather than hard-coding `pub mod`; engines without the block skip the
  step. Added 2026-05-23.

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

### Phase 0 v2 — Data-driven schemas + engine selection (wired)

- [x] **Migrate existing per-kind schemas into `game-config.json`** —
      `schemas/{item,skill,enemy,mechanic,quest,system,location,npc}.schema.json`
      inlined under `kinds.<kind>.frontmatter_schema`. Files deleted.
      `_universal.schema.json` remains as code.
- [x] **Refactor Phase 1 schema loader** — `phase1_ingest.validation_errors`
      now sources per-kind schemas from `game_config["kinds"][kind].get(
      "frontmatter_schema")`. Universal-only fallback + warning preserved.
- [x] **Phase 0 schema-proposal LLM step** — `propose_frontmatter_schemas`
      in `phase0_analyze.py`. Hard-coded `REQUIRED_FIELDS_BY_KIND` and
      `BASELINE_KINDS` constants removed.
- [x] **Phase 0 engine-candidate proposal step** —
      `propose_engine_candidates` in `phase0_analyze.py`. Returns 2–4
      candidates ranked by fit_score.
- [x] **Wire Phase 0 v2 through `phase0.py` + `phase0_write.py`** — both
      proposers invoked after `analyze_taxonomy`; results merged into the
      proposal shape; `engine_candidates` block written to
      `game-config.proposed.json`; `chosen_engine` preserved across runs.
- [x] **Schema proposer reads real wikitext, not titles** —
      `phase0_fetch.fetch_sample_pages_by_category` fetches the top-2
      pages' wikitext per category (truncated to 4000 chars each) via
      `action=query&prop=revisions&rvslots=main`, and `phase0.py` feeds
      that to `propose_frontmatter_schemas` instead of bare titles.
      Falls back to titles-only if every wikitext fetch fails.
- [x] **Universal `source_revision` accepts integer or string** —
      MediaWiki returns revids as integers; the prior `type: "string"`
      constraint failed every page. Schema now allows both.
- [x] **Re-run Phase 0 v2 against `they-are-billions.fandom.com`** — the ✅ 2026-05-21
      reference deployment.

### Phase 0 v3 — Coverage gate + mainspace filter (done 2026-05-22)

- [x] **Coverage gate** — `_validate_output` raises
      `IncompleteCoverageError` when the LLM omits an input category;
      `analyze_taxonomy` re-prompts once with an explicit omission list.
      Closes a silent-drop regression where mapped categories were
      disappearing from the proposal with no signal.
- [x] **`drop_reason` discipline** — explicit drops are allowed but
      restricted to wiki-administrative portal/index categories or
      strict subsets of another mapped category. The proposal carries
      a `dropped_categories` block for offline review; `phase0_write`
      prints a re-include hint per drop.
- [x] **Mainspace-only member enumeration** — `cmnamespace=0` on
      `categorymembers`, `acprop=size|hidden` on `allcategories`,
      hidden-cat skip. Categories with zero mainspace pages drop out
      of the proposal entirely.
- [x] **Phase 1 canonical_kind** — `validation.canonical_kind` resolves
      singular/plural drift between Phase 0's proposed kind names and
      Phase 1's extracted `type:` frontmatter (e.g. `game_mechanic` →
      `game_mechanics`). Rewrites both the parsed frontmatter and the
      serialized markdown.

### Engine choice — decided 2026-05-19: Bevy + lockstep

**Chosen:** Bevy (Rust) with **deterministic lockstep networking** and a
**fixed-tick sim decoupled from an interpolated render tick** for smooth
multiplayer feel at 30k+ entities.

Recorded in `game-config.json -> chosen_engine`. Binding determinism
rules for Phase 2 codegen are in `CLAUDE.md > "Target engine"`. The
machine-readable copy that flows into the engine baseline lives at
`prompts/engine_determinism/bevy.md`.

**Why Bevy over the runner-up (Godot 4):** the Phase 0 v2 proposer
scored Godot higher (0.87 vs 0.82) but did so from titles-only samples
and missed the multiplayer goal — its own con list flags networking as
"not a blocker for single-player focus," which is exactly the blocker
here. For 30k entities on the wire, only lockstep survives bandwidth,
which requires bit-identical sim across clients. Bevy's ECS + Rust gives
that foundation natively; Godot would require rewriting the sim layer
in C++ extensions with manual fixed-point math, which is "stop using
most of Godot."

**Tradeoffs accepted:**
- Bevy is pre-1.0 — pin the version in `Cargo.toml`, bump deliberately.
- Rust skills required — fewer drop-in hires than C# / GDScript.
- UI ecosystem less mature — colony-resource HUD will need more custom
  code than a Unity or Godot equivalent.

**Architecture sketch (encoded in CLAUDE.md, applied by Phase 2 codegen):**
- Sim: Bevy `FixedUpdate` schedule, 20–30Hz, deterministic system order,
  fixed-point math, seeded RNG, no `HashMap` iteration, no transcendentals.
- Render: Bevy `Update` schedule, vsync rate, interpolates between the
  last two sim states. Floats and `HashMap` are fine here.
- Networking: only player inputs cross the wire. Periodic state checksum
  broadcast — first mismatch = desync detected, log offending tick.

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
- [x] **Per-kind frontmatter schemas as data** — moved to
      `game-config.json -> kinds.<kind>.frontmatter_schema`. Standalone
      schema files deleted. Phase 0 v2 proposer authors per-kind schemas
      for new target games.
- [x] **Canonical type resolution** — `canonical_kind` accepts plural or
      singular variants of the declared `type:` and rewrites to the
      configured kind name; bypassed only on no-variant-match.

### Compile prompt

- [x] `prompts/wiki-compile-system.md` enforces YAML frontmatter +
      `## Description` / `## Behavioral Mechanics` / `## References` body
      sections, `IF / THEN / ON / WHILE` form, and `[[wiki_link]]` →
      `depends_on:` mirroring

### Closed pre-bulk-ingest

- [x] Phase 0 v2 lands and `game-config.json` carries per-kind ✅ 2026-05-21
      `frontmatter_schema` for all 8 kinds
- [x] Authenticate `claude` or `codex` CLI locally ✅ 2026-05-21
- [x] `python scripts/phase1_ingest.py --dry-run` — verify page counts ✅ 2026-05-21
- [x] `python scripts/phase1_ingest.py` — full ingest ✅ 2026-05-21
- [x] Inspect `vault/_quarantine/` and iterate on the compile prompt or ✅ 2026-05-21
      `frontmatter_schema` if the quarantine rate is non-trivial

### Deferred / low priority

- [-] **Concurrency in Phase 1 ingest** (deferred 2026-05-21) —
      `phase1.config.toml` declares `concurrency = 4` but
      `phase1_ingest.py` runs sequentially. Wiring up parallel fetch +
      compile saves wall-clock time on bulk ingest but doesn't block
      downstream work (Phase 1 runs in parallel with Phase 2 dev).
      Revisit if bulk-ingest wait time becomes real friction.

---

## Phase 2 — Vault → Code

Goal: take the sanitized vault, retrieve relevant chunks per task, and
generate production-ready game code with a Claude codegen loop.

**Status (2026-05-23): loop driver operational. Two units (soldier,
ranger) compile together in `game/`; the ranger was produced fully through
the automated loop (codegen → parse → merge → register `pub mod` → cargo
build → record). 159 tests passing, ruff + vulture clean. Next: the C
upgrade (auto-derive goals from a vault walk).**

### Engineering tasks

- [x] `npm install -g repomix` (or pin via `npx repomix@<ver>`) ✅ 2026-05-21
- [x] `pip install -r requirements-phase2.txt` covers chromadb, ✅ 2026-05-22
      sentence-transformers, tiktoken, anthropic, pyyaml
- [x] `build/repomix-output.xml` regeneration on vault change ✅ 2026-05-21
      (pre-commit hook), excluding `vault/_quarantine/**`
- [x] `phase2/indexer.py` — Repomix XML → two Chroma collections ✅ 2026-05-21
      (`vault_prose`, `vault_mechanics`) + `graph.json` for `[[wikilink]]`
      adjacency. Local embeddings; same model indexes and queries.
- [x] `phase2/retrieval.py` ✅ 2026-05-22 — vector merge + 1-hop graph
      expansion, packs `<file path>` blocks until the 2,000-token cap.
- [x] `prompts/engine_baseline.template.md` ✅ 2026-05-22 — Layer 1 sticky
      cache + `phase2/baseline.py` generator + per-engine determinism rules.
      Renders 2,279 / 2,500 tokens for Bevy (after the FILE-contract rules
      were added 2026-05-23).
- [x] `phase2/codegen.py` ✅ 2026-05-22 — assembles the 4-section user
      prompt, three dispatch modes (`claude` CLI default with `--tools ""`,
      `codex` CLI, `sdk` with cache), validates `// Sources:` header.
- [x] `phase2/driver.py` ✅ 2026-05-22 — single-shot turn driver.
- [x] `build/system_map.yaml` ✅ 2026-05-22 — `phase2/system_map.py` tracks
      implemented / pending / test_state across turns, capped at 1k tokens.
- [x] **Cargo regression anchor** ✅ 2026-05-22 — soldier extracted into
      `game/`, `cargo build` clean, Cargo.lock committed.
- [x] **Loop driver** ✅ 2026-05-22/23 (`phase2/loop_driver.py`) — iterates
      a goal list, parses the `=== FILE: ===` contract, merges leaf files,
      registers `pub mod` in driver-owned aggregators, runs `cargo build`,
      reverts byte-exact on failure, records implemented/pending. Error
      budget stops the loop. Output contract + `--tools ""` + module
      registration hardening landed 2026-05-23 (see ERRORS.md).
- [x] **Module registration as data** ✅ 2026-05-23 —
      `chosen_engine.module_registration` `{aggregator, declaration}`.
      The driver owns aggregator files (mod.rs) so the LLM cannot drop
      shared declarations; engine-neutral (opt-in per engine).
- [x] **Second sim-path file (ranger)** ✅ 2026-05-23 — first unit produced
      end-to-end through the loop; compiles alongside the soldier.

### Runtime guardrails

- [x] Retrieval cap asserted at 2,000 tokens (`phase2.retrieval.pack_files`)
- [x] System map capped at 1,000 tokens (`phase2.system_map.cap_tokens`)
- [x] Engine-baseline cache hit-rate logged per turn; warn if < 80%
      (SDK mode only; CLI mode is subscription-billed).
- [x] Every generated source file starts with `// Sources: vault/...`;
      post-check flags missing or hallucinated paths.
- [x] **`cargo build` gates every loop turn.** A turn that does not compile
      is reverted byte-exact and recorded pending, never committed.

### Open follow-ups

- [x] **Loop driver C upgrade** ✅ 2026-05-25 — walk `vault/<kind>/*.md` to
      auto-derive goals, skip slugs already in `system_map.implemented`,
      run hands-off. Per-kind goal templates. Layered on the working loop;
      revert + cargo gate + module registration carry through. First volume
      run (`--from-vault --kinds unit`) drove all 9 units to a clean build.
- [ ] **Smoke test** — a `cargo test` covering one unit invariant per unit
      (soldier HP=120, ranger HP=60) to catch silent constant regressions.
- [ ] **First SDK turn** — only when `ANTHROPIC_API_KEY` is available.
      Validates the `cache_control: ephemeral` math from DEPLOYMENT_GUIDE §4.
      Until then `claude` CLI mode (with `--tools ""`) is production.

---

## Decision log

Migrated to the Obsidian vault: `cloneGame/MEMORY.md`. Add new entries there (format: What was decided / Why / What was rejected and why). This file keeps only phase status and open questions.

---

## Open questions

- **Embedding provider** for the Phase 2 Chroma index — **resolved 2026-05-21**: local embedder (e.g. BGE / nomic-embed via `sentence-transformers`), no OpenAI dependency. See `cloneGame/MEMORY.md`.
- **Schema-proposal sampling size** — how many representative pages per
  category should the Phase 0 v2 schema proposer see? Too few and the
  schema underfits; too many and the prompt blows the context budget.
  Default 2; tune after first multi-game runs.


---

## 2026-05-26 status update (supersedes stale checkboxes above)

Two engine-agnostic codegen mechanisms landed, all 9 units regenerated from
scratch into one consistent shape, and the smoke test is done. See
`cloneGame/MEMORY.md` 2026-05-26 for the full decision record.

- **Sibling-exemplar consistency** (fixes per-turn API drift): the first
  accepted module of a kind is fed to every later same-kind codegen turn as a
  `[REFERENCE SIBLING MODULE]` pattern. Lives in `codegen.build_user_message`
  (game/engine-agnostic, no name mandate), resolved per kind in
  `loop_driver.run_loop`, kept correct under `--concurrency` by `_chunk_end`.
- **Build-error repair loop** (engine-agnostic compile-correctness): a failed
  build feeds the build tool's own error + failed source back through codegen,
  `--repair-attempts` (default 2) before pending. Replaces the prior
  per-engine-rule approach; swap the build runner for a new engine and it still
  works. Fixed the Bevy 15-element `Query`/`Bundle` tuple overflow.
- **Smoke test done**: `game/tests/unit_health.rs` asserts
  `<Unit>Bundle::default().health.max == vault HP` for all 9 units; `cargo test`
  green. The "Open follow-ups > Smoke test" item above is now complete.
- **Units shape**: all 9 share `const <UNIT>_HP` + `#[derive(Bundle)]
  <Unit>Bundle` + hand-written `Default` → `Health::full(<UNIT>_HP)`. The old
  soldier `base_health`/`spawn` helpers are gone (replaced by the bundle).

**Next:** scale content — `--from-vault --kinds buildings --concurrency 2
--repair-attempts 2` (42 buildings), then `infected`. The new mechanisms should
make these land far more cleanly than the first unit run.
