# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Communication Preferences

1. No filler openings. No "Great question", "Certainly", "Of course". Start with the answer.
2. No em dashes. Use commas, parentheses, or periods instead.
3. Straight to the point, efficient, concise.
4. Structured format. Use sections, headers, bullets when they help.
5. Match length to complexity. Short questions get short answers.
6. No restatement of the question, no closing recap sentence.
7. Flag uncertainty explicitly. Never invent facts, stats, dates, or technical specifics. If unsure, say so before answering.
8. Do not over-explain things dom already knows. Do not skip context dom needs.

## The Most Important Rules

1. **Ask, don't assume.** If something is unclear, ask before writing a single line. Never make silent assumptions about intent, architecture, or requirements. If a file is referenced, read it first — never claim things about code you haven't opened.
2. **Simplest solution first.** Always implement the simplest thing that could work. Do not add abstractions or flexibility that weren't explicitly requested.
3. **Don't touch unrelated code.** If a file or function is not directly part of the current task, do not modify it, even if you think it could be improved. If you notice something worth fixing elsewhere, mention it at the end. Don't touch it.
4. **Flag uncertainty explicitly.** If you are not confident about an approach or technical detail, say so before proceeding. Confidence without certainty causes more damage than admitting a gap.

## About Dom

Student developer. Goes by dom.
Background: Python, some APIs, some C++, some JavaScript, has built a couple websites, uses multiple LLMs in CLI, has worked with multiple databases.
Learning stage: most things. Treat as a developing student, not a beginner, not an expert.

## Current Project

Goal: build a game-agnostic wiki-to-code pipeline. Given any public Fandom-style wiki, the pipeline produces a structured Obsidian vault and then generates a playable clone of that game. All three phases are now live: Phase 2 codegen has been spent against the Anthropic API and has produced a real Rust/Bevy crate under `game/` (shared `Health`/`UnitStats` foundation, deterministic `tick_rng` + checksum desync detection in `game/src/sim.rs`, plus generated buildings, units, infected, wonders, and game_mechanics). Individual games (e.g. They Are Billions) are used as reference deployments, but no part of the repo should be hard-coded to one game.

- Repo root: `C:\Users\dominic\Documents\GitHub\cloneGame`
- Obsidian project folder (design notes, MEMORY.md, ERRORS.md): `cloneGame/` in the vault, reachable via the Obsidian MCP server. **This is not the same as the repo's `vault/` directory** — that one is Phase 1's generated output (gitignored, consumed by Phase 2).

## Tech Context

Comfortable with: Python, basic API work, C++ basics, JavaScript, simple web stacks, LLM CLIs, basic database use.
Still learning (relevant to this repo): Rust + Bevy (Phase 2 target), vector indexing (Chroma) and retrieval design, MediaWiki API quirks, prompt engineering for headless CLIs, JSON-Schema / per-kind contract design.
Avoid: overly complex architectures, premature abstraction, enterprise patterns, heavy frameworks where a simple script fits.

## What this repo is

A wiki-to-code pipeline structured as three sequential phases, all implemented. Phase 0 and Phase 1 are Python. **Phase 2 (codegen) is now live and has been run against the Anthropic API** (`phase2/{indexer,retrieval,baseline,codegen,system_map}.py` plus the orchestration drivers `phase2/driver.py` (one task, one file) and `phase2/loop_driver.py` (N goals, multi-file tree, gated behind `cargo build`)). Generated game source lives in `game/` (a Rust/Bevy crate). The pipeline must work for any wiki — a specific game serves as the reference but never as a hard-coded target. Phase 2 codegen output is per-game data, so do not hard-code anything about the current reference game into the pipeline modules.

`DEPLOYMENT_GUIDE.md` is the architecture spec. Per-phase implementation status and open questions live in the Obsidian vault at `cloneGame/plan.md`; active near-term work lives at `cloneGame/todo.md`. The decision log lives in the Obsidian vault at `cloneGame/MEMORY.md` — read it at the start of every session.

## Phase architecture

```
Phase 0  MediaWiki API ──► game-config.json (taxonomy: categories → kinds)
Phase 1  game-config.json + wiki pages ──► vault/<kind>/<slug>.md (Obsidian vault)
Phase 2  vault/ ──► repomix XML ──► Chroma index ──► codegen LLM ──► game code
```

- **Phase 0** queries `allcategories` + `categorymembers`, asks an LLM to (a) map raw wiki categories to engine-relevant `kinds`, (b) propose per-kind `frontmatter_schema` blocks from wikitext samples, (c) propose engine candidates, and auto-promotes the result into `game-config.json` (with `human_approved: true`). A mirror is written to `game-config.proposed.json` so `git diff` can be used for post-hoc review (this file is gitignored, so the diff is only meaningful between local runs, not across commits).
- **Phase 1** walks the approved `categories` array, pulls wikitext via the MediaWiki API, **trims it** (HTML comments, `[[Category:]]` tags, `[[File:]]`/`[[Image:]]` links, image-only `<gallery>` blocks, repeated blank lines, see `trim_wikitext` in `phase1_ingest.py`) to save tokens while keeping infoboxes/stat tables/formulas/wikilinks intact, builds a compile prompt that includes the per-kind `frontmatter_schema` so the LLM uses canonical field names, runs the prompt through a headless LLM CLI (`claude -p` or `codex exec`), and validates the resulting YAML frontmatter against `schemas/_universal.schema.json` plus per-kind property types. Presence of per-kind fields is NOT enforced (only the universal schema's `required` list is). Files write to `vault/<kind>/<slug>.md` on success or `vault/_quarantine/<slug>.md` on validation failure. Compile output is cached by SHA-256 of `(rendered compile prompt + model id)`, so any change to wikitext, the trim function, the system prompt, or a kind's `frontmatter_schema` invalidates the cache cleanly. Production config in `phase1.config.toml` currently pins `llm_mode = "codex"` + `model = "gpt-5.4-mini"`.
- **Phase 2** is live. The engine for any given run comes from `chosen_engine` in that game's `game-config.json`. The reference deployment chose **Bevy (Rust)** with **deterministic lockstep** — see "Reference deployment engine" below for the binding determinism rules that apply when that engine is selected. The pipeline lives in `phase2/` (indexer → retrieval → baseline → codegen → system_map), driven by `driver.py` (single task → single file) or `loop_driver.py` (a goals list → multi-file tree, each turn gated behind `cargo build game/` with revert-on-failure and `record_pending`). Per-engine determinism rules are data files under `prompts/engine_determinism/<engine>.md`, keyed by lowercased `chosen_engine.name`. Generated code lands in `game/src/`. Per Default Behavior 12, every live Anthropic codegen turn still needs an in-session yes before it runs.

## Reference deployment engine (chosen 2026-05-19): Bevy + lockstep

This section binds Phase 2 codegen **only when `chosen_engine = Bevy + lockstep`**. A different `chosen_engine` would replace this guidance. Engine choice is per-game data, not a repo-wide constant — see MEMORY.md.

**Architecture:**
- **Sim tick** runs deterministic at 20–30Hz inside Bevy's `FixedUpdate` schedule.
- **Render tick** is decoupled (`Update` schedule, vsync/native rate) and interpolates between the last two sim states so visible motion stays smooth even though the sim is slower.
- **Networking** is lockstep: only player inputs cross the wire. Each client runs the same sim from the same seed + input stream and converges on identical entity state tick-for-tick. State replication is forbidden (bandwidth doesn't survive at this entity count).

**Determinism rules (binding on every sim-path file Phase 2 generates under this engine):**

1. **Fixed-point math for sim state.** Positions, velocities, damage, RNG seeds (anything that affects what the next tick computes) must use `fixed` / `sfixed` crates, not `f32`/`f64`. Floats are fine for render-only state (camera, particle visual offsets, UI animation).
2. **No transcendentals in the sim path.** `sin`, `cos`, `sqrt`, `atan2` on floats vary across CPUs and compiler versions. Use fixed-point approximations or precomputed lookup tables.
3. **Seeded RNG everywhere.** Replace `rand::thread_rng()` with a per-tick `ChaCha8Rng` seeded from `(game_seed, tick_number)`. Every random draw in the sim path uses it. Render-side cosmetic randomness can use thread_rng.
4. **Deterministic system order.** Bevy's scheduler parallelizes by default (concurrent systems run in non-deterministic order). Force order on sim systems with `.chain()` or explicit `before()`/`after()` constraints.
5. **No `HashMap` iteration in sim code.** Rust's default `HashMap` randomizes its hash seed per-process; iteration order differs across runs. Use `BTreeMap`, sorted `Vec`, or `IndexMap` with a fixed `BuildHasher`.

**Validation:** every sim tick should compute a checksum of the canonical state (entity transforms + key game state). Periodically broadcast the hash; first mismatch across clients = desync detected, log the offending tick and the diverging entity set. Build this in from Phase 2 day one (desync bugs surface immediately, not three weeks into integration testing).

## Common commands

Phase 0 (taxonomy discovery, requires `claude` or `codex` CLI on PATH):
```powershell
python scripts/phase0.py                            # full run with confirmation prompt
python scripts/phase0.py --dry-run                  # print proposed JSON, skip write
python scripts/phase0.py --wiki-url <url> --min-members 5
python scripts/phase0.py --llm-mode codex --model <model-id>
```

Phase 1 (wiki ingest, requires `claude`/`codex` CLI authenticated + `game-config.json` with `human_approved: true`):
```powershell
python scripts/phase1_ingest.py --dry-run           # enumerate category page counts only
python scripts/phase1_ingest.py --limit 1           # one page per category, smoke test
python scripts/phase1_ingest.py                     # full ingest
pip install jsonschema                              # optional; enables full Draft 2020-12 validation instead of the required-fields fallback
```

Phase 2 (vault → codegen, requires `pip install -r requirements-phase2.txt`):
```powershell
python phase2/indexer.py                            # rebuild Chroma + graph.json
python phase2/baseline.py                           # render build/engine_baseline.md
python phase2/system_map.py init                    # one-time state init
python phase2/codegen.py --dry-run "implement X"    # assemble prompt, skip API
python phase2/codegen.py "implement X"              # live Anthropic call (DB12 gate)
python phase2/driver.py <id> src/x.rs "implement X" # one task -> one file + system_map
python phase2/loop_driver.py --goals-file goals.txt # N goals, multi-file, cargo-gated
python phase2/loop_driver.py --dry-run "implement X" # assemble only, no API/build
```

Phase 2 game crate (the generated Rust/Bevy code; requires the Rust toolchain):
```powershell
cargo build  --manifest-path game/Cargo.toml        # what loop_driver gates each turn on
cargo test   --manifest-path game/Cargo.toml        # integration tests in game/tests/
```

Tests (unit tests for Phase 1 + Phase 2 helpers; the Chroma-touching path in `phase2/indexer.py` is not exercised, only the pure-Python parsing/packing/state helpers):
```powershell
python -m unittest discover tests             # run everything under tests/
python -m unittest tests.test_phase1_ingest   # one module
python -m unittest tests.test_phase2_retrieval
```

There is **no build step and no CI**. `ruff`/`vulture` (see "Pre-Commit Checks" below) and the `unittest` suite are the full local gate. Pipeline validation happens inline during Phase 1 ingest: a non-zero exit code from `phase1_ingest.py` means files were quarantined. Inspect `vault/_quarantine/*.md` (each has a `validation_errors:` block at the top) and either fix the compile prompt or extend `kinds` in `game-config.json`.

## Key files for navigation

- `DEPLOYMENT_GUIDE.md` — full architectural spec.
- `cloneGame/plan.md` (Obsidian vault) — phase-by-phase implementation status (`[x] [~] [ ] [!]`) and open questions.
- `cloneGame/todo.md` (Obsidian vault) — active near-term TODOs distilled from `plan.md`.
- `cloneGame/MEMORY.md` (Obsidian vault) — decision log. Read at session start; never contradict without flagging.
- `cloneGame/ERRORS.md` (Obsidian vault) — failed-approach log (per Default Behaviors #20). Check before suggesting approaches to similar tasks.
- `game-config.json` — Phase 0 output / Phase 1 input. `kinds` defines the taxonomy; `categories` defines what Phase 1 fetches; `chosen_engine` drives Phase 2; `human_approved` gates production runs.
- `phase1.config.toml` — Phase 1 runtime config (wiki API endpoint, retry/throttle, LLM mode + model, cache dir).
- `prompts/wiki-compile-system.md` — system prompt for the Phase 1 compile LLM. Edits to this invalidate the SHA-256 cache.
- `schemas/_universal.schema.json` — required-on-every-file frontmatter fields. Per-kind frontmatter contracts live as data in `game-config.json -> kinds.<kind>.frontmatter_schema`, not as files.
- `game-config.json -> kinds.<kind>.frontmatter_schema` — per-kind frontmatter contract (`{properties: {...}}`). Inlined here so schemas travel with the game config and Phase 0 can LLM-propose them per target game. Per-kind validation only enforces property types on fields that happen to be present — presence is gated solely by `schemas/_universal.schema.json`'s `required` list. Do not add a per-kind `required` array; it would be ignored. Kinds without a `frontmatter_schema` get universal-only validation plus a one-time warning.

## Conventions & gotchas

- **Outputs are gitignored**: `vault/`, `build/`, `.phase1_cache/`. Don't commit them; don't write source code under those paths.
- **The repo `vault/` is not the Obsidian design-notes vault.** `vault/` in this repo is Phase 1's generated, gitignored output consumed by Phase 2. Design notes / MEMORY.md / ERRORS.md live in the Obsidian vault under `cloneGame/`, reachable via the Obsidian MCP.
- **LLM access is via headless CLI, not the Anthropic/OpenAI SDK.** Both Phase 0 and Phase 1 shell out to `claude -p` or `codex exec`. If you need a different provider, route through `_run_llm_command` in `phase0_analyze.py` or `run_llm` in `phase1_ingest.py` rather than introducing a new SDK dependency. Cache keys include the model id, so model swaps invalidate cached compiles automatically.
- **Frontmatter parsing is hand-rolled** in `phase1_ingest.py` (`parse_yaml_map` / `parse_block`). It handles the subset of YAML the compile prompt produces; don't assume full YAML compatibility. If frontmatter is getting mis-parsed, fix the compile prompt before reaching for a full YAML library.
- **Wikilink invariant**: every `[[wiki_link]]` in a vault file body must be mirrored in `depends_on:` in the frontmatter. The compile system prompt enforces this; Phase 2's graph expansion will rely on it.
- **The taxonomy AND per-kind schemas live in `game-config.json`, not in code.** Adding a new `kind` means: (1) add it to `kinds` in `game-config.json`, (2) optionally add a `frontmatter_schema` block under that kind, (3) re-run Phase 0 or hand-edit `categories` to route wiki categories to it. Phase 0 v2's LLM proposer can do (1) and (2) automatically for a new target game.
- **Game code now exists under `game/`.** A task framed as "fix the game" is real: it means the generated Rust/Bevy crate, not the pipeline. Still distinguish the two — "fix the pipeline" / "fix codegen" means `scripts/` or `phase2/`. When in doubt about which layer a task targets, ask.
- **Engine-specific fixes are a last resort** No good solution requires a game or engine-specific fix since this repo aims to be able to mix and match any and all of those options. All solutions need to be broad and able to actually apply for many cases.

## Development Guidelines

### Code Style & Standards

- Files must be smaller than 400 lines excluding comments. Once 400 is exceeded, initiate a refactor.
- Functions must be smaller than 40 lines excluding comments and the catch/finally blocks of try/catch sections. If a function exceeds that, refactor it.
- `scripts/phase1_ingest.py` is the Phase 1 orchestrator (`process_page`, `ingest`, `build_context`, `main`, dry-run + print helpers). Helpers live in sibling modules: `wikitext.py` (trimming), `frontmatter.py` (YAML parsing), `validation.py` (schema gate), `compile_cache.py` (prompt render + SHA cache + LLM dispatch), `wiki_api.py` (MediaWiki retry/pagination), `vault_index.py` (paths, source-key index, migration). Inter-script imports use bare `from <module> import ...` plus a `sys.path` bootstrap so both direct execution and unittest discovery work.
- `scripts/phase0_analyze.py` is ~398 lines, right at the line. Treat any net-add as a refactor trigger.
- `phase2/` is the Phase 2 codegen pipeline. Modules: `indexer.py` (repomix XML → Chroma + graph.json), `retrieval.py` (vector merge + 1-hop graph, 2k-token cap), `baseline.py` (renders `prompts/engine_baseline.template.md` with per-engine rules from `prompts/engine_determinism/<engine>.md`, 2.5k-token cap), `codegen.py` (Anthropic SDK call with `cache_control: ephemeral` on baseline, cache-hit logging, `// Sources:` header validation), `system_map.py` (rolling `build/system_map.yaml`, 1k-token cap), `driver.py` (single-shot: one task → one validated file + a `system_map` mutation), `loop_driver.py` (runs N goals, accepts a multi-file tree, merges into `game/` with backup, gates each turn on `cargo build`, reverts the whole turn on failure, stops after `--error-budget` failures). Imports use the same bare `from <module> import ...` + `sys.path` bootstrap pattern as scripts/.
- `game/` is the generated Rust/Bevy crate (Phase 2 output, committed). `game/src/sim.rs` holds the shared determinism foundation (`tick_rng`, `SimChecksumState`, `Health`, `UnitStats`, `SimPosition`, damage/kill/noise events) that generated units and buildings build on; `game/src/{units,buildings,infected,wonders,game_mechanics}/` hold the per-entity modules. `game/tests/` holds integration tests. `Cargo.toml`/`Cargo.lock` are pinned regression anchors — `loop_driver.py` deliberately skips them when merging codegen output.

### Comments and Documentation

- This is a Python-only repo. Use Google-style docstrings at the top of each module and on every public function/class. No JSDoc.
- Write clear comments for complex logic; comment the *why*, not the *what*.
- Keep comments up-to-date with code changes.
- Document any non-obvious behavior (cache-key inputs, validation fallbacks, etc.).

### Pre-Commit Checks

This repo is Python-only. Use the Python tools installable via `pip install -r requirements-dev.txt`. Configuration lives in `pyproject.toml`.

1. `ruff check scripts/ tests/ phase2/` — lint + complexity (mccabe, max 10). Preview auto-fixes with `--fix --diff`, apply with `--fix`.
2. `ruff format scripts/ tests/ phase2/` — formatter. Preview with `--diff`.
3. `vulture scripts/ phase2/` — dead-code detection. Report-only; fix flagged items manually or add to a whitelist with explicit justification. (`tests/` is excluded — unittest discovery treats top-level test methods as entry points and vulture cannot see those callers.)
4. `python -m unittest discover tests` — unit tests for Phase 1 + Phase 2 helpers.

Do not commit until all four report clean.

## Default Behaviors

1. Before significant or open-ended tasks, show 2 or 3 approaches and wait for dom to pick. Skip this for narrow, well-specified tasks.
2. Treat dom as a developing student. Explain trade-offs, do not lecture on basics already in dom's stack.
3. Prefer simple, well-supported tools over cutting-edge or complex ones.
4. Build small working pieces first. Avoid large upfront architecture.
5. For this repo specifically, favor:
   - Headless LLM CLI (`claude -p` / `codex exec`) over SDK dependencies.
   - Per-game data in `game-config.json` over hard-coded Python branches.
   - Hand-rolled small parsers over heavyweight libraries while the prompt-side contract stays constrained (switch to the library when the contract drifts, not before).
   - Cache-key correctness over speed — a stale compile cache hides bugs.
   - Quarantine on validation failure over silent skips; let `vault/_quarantine/` be the visible signal.
   - Pipeline must remain game-agnostic — never hard-code field names, category names, or engine choices that should live in `game-config.json`.
6. When writing code or docs on dom's behalf, match the writing style above. No em dashes. Structured. Concise.
7. Use the Obsidian MCP when relevant context lives in the vault. Do not duplicate vault notes into the repo.
8. Never create documentation files unless dom asks.
9. No unsolicited commentary, recaps, or feature suggestions beyond the task.
10. Before making any change that significantly alters content dom has already created (rewriting sections, removing paragraphs, restructuring flow, changing tone): stop. Describe exactly what you're about to change and why. Wait for confirmation before proceeding.
11. Before deleting any file, overwriting existing code, dropping database records, or removing dependencies: stop. List exactly what will be affected. Ask for explicit confirmation. Only proceed after dom says yes in the current message. "You mentioned this earlier" is not confirmation.
12. The following require explicit in-session confirmation, no exceptions: deploying or pushing to any environment, running migrations or schema changes, sending any external API call, executing any command with irreversible side effects. Dom must say yes in the current message.
13. After any coding task, end with: Files changed (list every file touched) / What was modified (one line per file) / Files intentionally not touched / Follow-up needed.
14. Never send, post, publish, share, or schedule anything on dom's behalf without explicit confirmation in the current message. This includes emails, calendar invites, document shares, or any action outside this conversation. Dom must say yes in the current message.
15. For architecture decisions, performance tradeoffs, debugging complex issues, non-trivial features, or long-term technical decisions: use extended thinking mode and the sequential-thinking MCP server. Work through the problem step by step, show reasoning, surface tradeoffs dom hasn't considered, flag assumptions that might not hold at scale, identify where you're uncertain, then implement or recommend.
16. Maintain `MEMORY.md` in `cloneGame/` in the Obsidian vault. After any significant decision, add an entry: What was decided / Why / What was rejected and why. Read MEMORY.md at the start of every session. Never contradict a logged decision without flagging it first.
17. When dom says "session end", "wrapping up", or "let's stop here": write a session summary to MEMORY.md. Include: Worked on / Completed / In progress / Decisions made / Next session priorities.
18. Maintain `ERRORS.md` in `cloneGame/` in the Obsidian vault. When an approach takes more than 2 attempts to work, log it: What didn't work / What worked instead / Note for next time. Check ERRORS.md before suggesting approaches to similar tasks.
19. If the Obsidian project folder does not exist, create one (path: `cloneGame/`). If you cannot reach the Obsidian server, create a temporary vault file at the repo root for dom to manually move later. Any markdown that does not need to be in the GitHub repo should live in the Obsidian vault (repo-root docs like `CLAUDE.md`, `README.md`, and `DEPLOYMENT_GUIDE.md` stay in repo).
20. Keep the repo-root design docs (`DEPLOYMENT_GUIDE.md`) and the Obsidian project folder current as the project progresses. When project state changes (Phase 0/1/2 milestones reached, decisions made, schemas updated, model swapped, new sub-systems added): update the relevant repo-root doc and replace TBD placeholders with actual values. Stale design docs are worse than missing ones. This complements rules 16 and 18, which still apply for MEMORY.md and ERRORS.md.
