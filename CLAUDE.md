# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## What this repo is

A wiki-to-code pipeline that reverse-engineers a video game from a public Fandom wiki. Target game is **They Are Billions** (`they-are-billions.fandom.com`). The repo is structured as three sequential phases — Phase 0 and Phase 1 are implemented in Python; **Phase 2 (codegen) is not yet implemented**, so the repo currently contains no game source code, just the data-extraction pipeline that will feed it.

`DEPLOYMENT_GUIDE.md` is the architecture spec. `plan.md` tracks implementation status, the active blocker, and the decision log. Read both before making non-trivial changes.

## Phase architecture

```
Phase 0  MediaWiki API ──► game-config.json (taxonomy: categories → kinds)
Phase 1  game-config.json + wiki pages ──► vault/<kind>/<slug>.md (Obsidian vault)
Phase 2  vault/ ──► repomix XML ──► Chroma index ──► codegen LLM ──► game code
```

- **Phase 0** queries `allcategories` + `categorymembers`, asks an LLM to (a) map raw wiki categories to engine-relevant `kinds`, (b) propose per-kind `frontmatter_schema` blocks from wikitext samples, (c) propose engine candidates, and auto-promotes the result into `game-config.json` (with `human_approved: true`). A mirror is written to `game-config.proposed.json` so `git diff` can be used for post-hoc review — note this file is gitignored (`*.proposed.json` in `.gitignore`), so the diff is only meaningful between local runs, not across commits.
- **Phase 1** walks the approved `categories` array, pulls wikitext via the MediaWiki API, **trims it** (HTML comments, `[[Category:]]` tags, `[[File:]]`/`[[Image:]]` links, image-only `<gallery>` blocks, repeated blank lines — see `trim_wikitext` in `phase1_ingest.py`) to save tokens while keeping infoboxes/stat tables/formulas/wikilinks intact, builds a compile prompt that includes the per-kind `frontmatter_schema` so the LLM uses canonical field names, runs the prompt through a headless LLM CLI (`claude -p` or `codex exec`), and validates the resulting YAML frontmatter against `schemas/_universal.schema.json` plus per-kind property types (presence of per-kind fields is NOT enforced — only the universal schema's `required` list is). Files write to `vault/<kind>/<slug>.md` on success or `vault/_quarantine/<slug>.md` on validation failure. Compile output is cached by SHA-256 of `(rendered compile prompt + model id)`, so any change to wikitext, the trim function, the system prompt, or a kind's `frontmatter_schema` invalidates the cache cleanly. Production config in `phase1.config.toml` currently pins `llm_mode = "codex"` + `model = "gpt-5.4-mini"`.
- **Phase 2** is greenfield. Target engine is **Bevy (Rust)** with **deterministic lockstep** networking — see "Target engine" section below for the binding determinism rules. Do not invent Phase 2 code without explicit user direction.

## Target engine (chosen 2026-05-19): Bevy + lockstep

Phase 2 codegen targets **Bevy (Rust)** with **deterministic lockstep networking**. This is the load-bearing decision — every Phase 2 generated sim-path system must honor the rules below or multiplayer silently desyncs. The chosen-engine record is in `game-config.json -> chosen_engine`.

**Architecture:**
- **Sim tick** runs deterministic at 20–30Hz inside Bevy's `FixedUpdate` schedule.
- **Render tick** is decoupled (`Update` schedule, vsync/native rate) and interpolates between the last two sim states so visible motion stays smooth even though the sim is slower.
- **Networking** is lockstep: only player inputs cross the wire. Each client runs the same sim from the same seed + input stream and converges on identical entity state tick-for-tick. State replication is forbidden — bandwidth doesn't survive at this entity count.

**Determinism rules (binding on every sim-path file Phase 2 generates):**

1. **Fixed-point math for sim state.** Positions, velocities, damage, RNG seeds — anything that affects what the next tick computes — must use `fixed` / `sfixed` crates, not `f32`/`f64`. Floats are fine for render-only state (camera, particle visual offsets, UI animation).
2. **No transcendentals in the sim path.** `sin`, `cos`, `sqrt`, `atan2` on floats vary across CPUs and compiler versions. Use fixed-point approximations or precomputed lookup tables.
3. **Seeded RNG everywhere.** Replace `rand::thread_rng()` with a per-tick `ChaCha8Rng` seeded from `(game_seed, tick_number)`. Every random draw in the sim path uses it. Render-side cosmetic randomness can use thread_rng.
4. **Deterministic system order.** Bevy's scheduler parallelizes by default — concurrent systems run in non-deterministic order. Force order on sim systems with `.chain()` or explicit `before()`/`after()` constraints.
5. **No `HashMap` iteration in sim code.** Rust's default `HashMap` randomizes its hash seed per-process; iteration order differs across runs. Use `BTreeMap`, sorted `Vec`, or `IndexMap` with a fixed `BuildHasher`.

**Validation:** every sim tick should compute a checksum of the canonical state (entity transforms + key game state). Periodically broadcast the hash; first mismatch across clients = desync detected → log the offending tick and the diverging entity set. Build this in from Phase 2 day one — desync bugs surface immediately, not three weeks into integration testing.

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
python scripts/phase1_ingest.py --limit 1           # one page per category — smoke test
python scripts/phase1_ingest.py                     # full ingest
pip install jsonschema                              # optional; enables full Draft 2020-12 validation instead of the required-fields fallback
```

Tests (unit tests for `phase1_ingest` helpers — `trim_wikitext`, frontmatter parsing, validation, completed-source indexing):
```powershell
python -m unittest discover tests             # run everything under tests/
python -m unittest tests.test_phase1_ingest   # one module
```

There is **no build step and no CI** — `ruff`/`vulture` (see "Pre-Commit Checks" below) and the `unittest` suite are the full local gate. Pipeline validation happens inline during Phase 1 ingest: a non-zero exit code from `phase1_ingest.py` means files were quarantined. Inspect `vault/_quarantine/*.md` (each has a `validation_errors:` block at the top) and either fix the compile prompt or extend `kinds` in `game-config.json`.

## Key files for navigation

- `DEPLOYMENT_GUIDE.md` — full architectural spec.
- `plan.md` — phase-by-phase implementation status (`[x] [~] [ ] [!]`), decision log, and open questions.
- `game-config.json` — Phase 0 output / Phase 1 input. `kinds` defines the taxonomy; `categories` defines what Phase 1 fetches; `human_approved` gates production runs.
- `phase1.config.toml` — Phase 1 runtime config (wiki API endpoint, retry/throttle, LLM mode + model, cache dir).
- `prompts/wiki-compile-system.md` — system prompt for the Phase 1 compile LLM. Edits to this invalidate the SHA-256 cache.
- `schemas/_universal.schema.json` — required-on-every-file frontmatter fields. Per-kind frontmatter contracts live as data in `game-config.json -> kinds.<kind>.frontmatter_schema`, not as files.
- `game-config.json -> kinds.<kind>.frontmatter_schema` — per-kind frontmatter contract (`{required: [...], properties: {...}}`). Inlined here so schemas travel with the game config and Phase 0 can LLM-propose them per target game. `required` is read by the compile LLM (as field-name guidance) but NOT enforced by Phase 1 validation; per-kind validation only enforces property types on fields that happen to be present. Kinds without a `frontmatter_schema` get universal-only validation plus a one-time warning.

## Conventions & gotchas

- **Outputs are gitignored**: `vault/`, `build/`, `.phase1_cache/`. Don't commit them; don't write source code under those paths.
- **LLM access is via headless CLI, not the Anthropic/OpenAI SDK.** Both Phase 0 and Phase 1 shell out to `claude -p` or `codex exec`. If you need a different provider, route through `_run_llm_command` in `phase0_analyze.py` or `run_llm` in `phase1_ingest.py` rather than introducing a new SDK dependency. Cache keys include the model id, so model swaps invalidate cached compiles automatically.
- **Frontmatter parsing is hand-rolled** in `phase1_ingest.py` (`parse_yaml_map` / `parse_block`). It handles the subset of YAML the compile prompt produces; don't assume full YAML compatibility. If frontmatter is getting mis-parsed, fix the compile prompt before reaching for a full YAML library.
- **Wikilink invariant**: every `[[wiki_link]]` in a vault file body must be mirrored in `depends_on:` in the frontmatter. The compile system prompt enforces this; Phase 2's graph expansion will rely on it.
- **The taxonomy AND per-kind schemas live in `game-config.json`, not in code.** Adding a new `kind` means: (1) add it to `kinds` in `game-config.json`, (2) optionally add a `frontmatter_schema` block under that kind, (3) re-run Phase 0 or hand-edit `categories` to route wiki categories to it. Phase 0 v2's LLM proposer can do (1) and (2) automatically for a new target game.
- **The directory name `cloneGame` is aspirational.** No game code exists yet. Treat any task framed as "fix the game" as a request to work on the pipeline, unless the user explicitly says they've started Phase 2.

## Development Guidelines

### Code Style & Standards

- Files must be smaller than 400 lines excluding comments. Once 400 is exceeded, initiate a refactor.
- Functions must be smaller than 40 lines excluding comments and the catch/finally blocks of try/catch sections. If a function exceeds that, refactor it.
- **Known outlier:** `scripts/phase1_ingest.py` is currently ~811 lines and exceeds the 400-line rule. A split is pending; do not let this file motivate a drive-by refactor — but also do not pile more code into it without first carving out a module (candidates: `trim_wikitext`, frontmatter parsing, validation, cache I/O).
- `scripts/phase0_analyze.py` is ~398 lines — right at the line. Treat any net-add as a refactor trigger.

### Clean Code Rules

- Meaningful Names: Name variables and functions to reveal their purpose, not just their value.
- One Function, One Responsibility: Functions should do one thing.
- Avoid Magic Numbers: Replace hard-code values with named constants to give them meaning.
- Use Descriptive Booleans: Boolean names should state a condition, not just its value.
- Keep Code DRY: Duplicate code means duplicate bugs. Try and reuse logic where it makes sense.
- Avoid Deep Nesting: Flatten your code flow to improve clarity and reduce cognitive load.
- Comment Why, Not What: Explain the intention behind your code, not the obvious mechanics.
- Limit Function Arguments: Too many parameters confuse. Group related data into objects.
- Code Should Be Self-Explanatory: Well-written code needs fewer comments because it reads like a story.

### Comments and Documentation

- This is a Python-only repo. Use Google-style docstrings at the top of each module and on every public function/class. No JSDoc.
- Write clear comments for complex logic; comment the *why*, not the *what*.
- Keep comments up-to-date with code changes.
- Document any non-obvious behavior (cache-key inputs, validation fallbacks, etc.).

### Pre-Commit Checks

This repo is Python-only. Use the Python tools installable via `pip install -r requirements-dev.txt`. Configuration lives in `pyproject.toml`.

1. `ruff check scripts/ tests/` — lint + complexity (mccabe, max 10). Preview auto-fixes with `ruff check scripts/ tests/ --fix --diff`, apply with `ruff check scripts/ tests/ --fix`.
2. `ruff format scripts/ tests/` — formatter. Preview with `ruff format scripts/ tests/ --diff`.
3. `vulture scripts/` — dead-code detection. Report-only; fix flagged items manually or add to a whitelist with explicit justification. (`tests/` is excluded — unittest discovery treats top-level test methods as entry points and vulture cannot see those callers.)
4. `python -m unittest discover tests` — unit tests for `phase1_ingest` helpers.

Do not commit until all four report clean.

## General Rules

- First think through the problem, read the codebase for relevant files.
- Make every task and code change you do as simple as possible. We want to avoid making any massive or complex changes. Every change should impact as little code as possible. Everything is about simplicity.
- Never speculate about code you have not opened. If the user references a specific file, you MUST read the file before answering. Make sure to investigate and read relevant files BEFORE answering questions about the codebase. Never make any claims about code before investigating unless you are certain of the correct answer - give grounded and hallucination-free answers.

