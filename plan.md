# cloneGame — Implementation Plan

Tracks progress against `DEPLOYMENT_GUIDE.md`. See `NOTES.md` for the
guide-vs-reality gap analysis and the patches applied to vendored tooling.

Status keys: `[x]` done · `[~]` partial · `[ ]` not started · `[!]` blocked

---

## Phase 0 — Taxonomy discovery (NEW; not in original guide)

The guide assumed a fixed 8-kind taxonomy. Reality: kinds should be derived
from the target wiki. Phase 0 sits in front of Phase 1.

- [x] Define the Phase 0 ↔ Phase 1 contract — `game-config.json` schema
- [x] Seed `game-config.json` with the guide's 8 kinds as a baseline
- [x] Write the Phase 0 taxonomy-discovery LLM step — `scripts/phase0_fetch.py`,
      `scripts/phase0_analyze.py`
  - Dual-query MediaWiki API pipeline: allcategories (paginated) +
    categorymembers (top 2 per category); heuristic maintenance filter
  - Multi-provider LLM call (OpenAI gpt-4o-mini → Anthropic Haiku → Gemini Flash);
    verbatim title enforcement; JSON retry; snake_case kind normalization
- [x] Proposal writer + orchestrator — `scripts/phase0_write.py`, `scripts/phase0.py`
  - Writes `game-config.proposed.json` with ANSI-coloured unified diff
  - Two-stage terminal confirmation; flips `human_approved: true` on approval
  - Prints XML dump prep instructions on completion
  - CLI: `python scripts/phase0.py [--wiki-url URL] [--min-members N] [--dry-run]`
- [ ] Human review gate — run `python scripts/phase0.py`, review diff, confirm
  - Sets `human_approved: true` in `game-config.json` after sign-off
  - `phase1_sort.py` already warns when this is `false`

---

## Phase 1 — Wiki → Vault

### Prerequisites (§4.1 of the guide)

- [x] `llm-wiki-compiler` available — vendored at `vendor/llm-wiki-compiler/`
      (atomicstrata fork, Node CLI; guide's `pipx install` was a fiction)
- [x] Schema layer patched for arbitrary kinds —
      `vendor/llm-wiki-compiler/src/schema/{types,loader,helpers}.ts`
- [x] Wiki base URL + page-type router rules — `phase1.config.toml`
      (intent) + `game-config.json` (runtime)
- [x] JSON Schemas per kind — `schemas/_universal.schema.json` +
      `schemas/{item,skill,enemy,mechanic,location,npc,quest,system}.schema.json`
- [x] Quarantine mechanism — `scripts/phase1_sort.py` (smoke-tested:
      1 sorted, 2 quarantined, exit 1 when any quarantine)
- [x] Compile-stage system prompt captured — `prompts/wiki-compile-system.md`

### Runtime ingest (the actual data work)

- [!] Batch ingest driver for `they-are-billions.fandom.com`
      - **Blocker:** `llmwiki ingest` takes one URL at a time; no built-in crawler
      - Needed: Python script that hits the Fandom MediaWiki API, enumerates
        all pages, throttles per `phase1.config.toml [ingest]`, calls
        `node vendor/llm-wiki-compiler/dist/cli.js ingest <url>` for each
- [!] `ANTHROPIC_API_KEY` set in environment (or alt provider via `LLMWIKI_PROVIDER`)
- [ ] Run `llmwiki compile` over `sources/`
- [ ] Run `python scripts/phase1_sort.py` → expect exit code 0
- [ ] Inspect `vault/_quarantine/` if non-empty; iterate on compile prompt
      or extend `kinds` in `game-config.json`

### Frontmatter validation (deferred)

The guide's §1.3 promises strict per-type validation. The patched llmwiki
tool only enforces `minWikilinks`. Bridging this:

- [ ] Wire the `schemas/*.schema.json` into `phase1_sort.py` (or a sibling
      `phase1_validate.py`) so files with valid `kind` but invalid
      sub-fields also route to `_quarantine/`
- [ ] Decide on validator: `jsonschema` package is the obvious pick

---

## Phase 2 — Vault → Code

Self-contained Python orchestration, per §2 of the guide. None of this
exists yet; everything below is greenfield once Phase 1 produces a clean vault.

- [ ] `build/repomix-output.xml` regenerated as a pre-commit hook
      - `npm install -g repomix` (or pin via `npx repomix@<ver>`)
      - Invocation per §2.2 (`--style xml`, exclude `_quarantine`)
- [ ] `pip install chromadb tiktoken pyyaml anthropic`
- [ ] `phase2/indexer.py` (per §2.3) — Repomix XML → two Chroma collections
      (`vault_prose`, `vault_mechanics`) + `graph.json`
- [ ] `phase2/retrieval.py` (per §2.4) — vector seeds + 1-hop graph expansion,
      capped at 2000 tokens
- [ ] `prompts/engine_baseline.md` written and < 2,500 tokens
      - Target engine + language not yet chosen (Godot/GDScript? Unity/C#?)
      - This is the Layer 1 cached sticky prompt
- [ ] `phase2/codegen.py` (per §2.5) — Anthropic SDK call with cache_control
- [ ] `build/system_map.yaml` rolling summarizer (per §2.6) — Haiku-driven,
      regenerated each turn

---

## Runtime guardrails (§4.3 of the guide)

- [ ] Retrieval cap enforced at 2,000 tokens (assertion in `retrieve()`)
- [ ] System mapping doc capped at 1,000 tokens (summarize-on-overflow)
- [ ] Engine baseline cache hit-rate logged per turn; alert if < 80%
- [ ] Every generated file linked back to its vault source path in a code header

---

## Decision log

| Decision | When | Why |
| --- | --- | --- |
| Use atomicstrata fork, not ussumant plugin | early Phase 1 | ussumant repo is a Claude Code plugin, no `llmwiki ingest <url>` CLI; atomicstrata is the real Node CLI matching the guide's shape |
| Patch tool's schema layer (Tier 1, ~30 LOC) instead of deep refactor | Phase 1 prereqs | Tier 2 would touch ~10 files including viewer/export; rebase pain on every upstream pull. Tier 1 keeps the patch surface inside `src/schema/` |
| Post-compile Python sorter instead of native per-kind output | Phase 1 prereqs | Obsidian wikilinks resolve by filename, so moving files between dirs doesn't break links. Sorter is reversible; tool keeps working against `wiki/concepts/` |
| Add `game-config.json` as a new schema candidate, top of priority list | Phase 1 prereqs | Phase 0 contract needs a stable file the patched loader will pick up automatically; old `.llmwiki/schema.json` still works as a fallback |
| Write `phase1.config.toml` even though no tool reads it | Phase 1 prereqs | Guide §4.1 calls it out by name; it's the canonical documentation surface for wiki base URL + routing intent |

---

## Open questions

- Target game engine for Phase 2 codegen (Godot/GDScript? Unity/C#? Bevy/Rust?)
- Should `phase1_sort.py` move into the vendored tool as a subcommand
  (`llmwiki phase1-sort`)? Pro: single CLI. Con: more upstream-divergent code.
- Phase 0 LLM — single shot taxonomy proposal, or iterative with samples?
- Embedding provider — guide says OpenAI text-embedding-3-small; do we want
  to standardize on Anthropic-only and pick a different embedder?
