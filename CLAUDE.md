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

- **Phase 0** queries `allcategories` + `categorymembers`, asks an LLM to map raw wiki categories to engine-relevant `kinds`, and writes `game-config.proposed.json` with a colored diff. A human must approve the diff; approval flips `human_approved: true` in `game-config.json`. Phase 1 warns when this is false.
- **Phase 1** walks the approved `categories` array, pulls wikitext via the MediaWiki API, runs each page through a headless LLM CLI (`claude -p` or `codex exec`), validates the resulting YAML frontmatter against `schemas/*.schema.json`, and writes either `vault/<kind>/<slug>.md` or `vault/_quarantine/<slug>.md`. Compile output is cached by SHA-256 of `(wikitext + system prompt + model id)`, so unchanged pages skip the LLM call on re-runs.
- **Phase 2** is greenfield. The target engine has not been chosen (Godot/GDScript? Unity/C#? Bevy/Rust? — see "Open questions" in `plan.md`). Do not invent Phase 2 code without explicit user direction.

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

There is **no test suite, no linter, no build step, no CI**. Validation happens inline during Phase 1 ingest — non-zero exit code from `phase1_ingest.py` means files were quarantined. Inspect `vault/_quarantine/*.md` (each has a `validation_errors:` block at the top) and either fix the compile prompt or extend `kinds` in `game-config.json`.

## Key files for navigation

- `DEPLOYMENT_GUIDE.md` — full architectural spec.
- `plan.md` — phase-by-phase implementation status (`[x] [~] [ ] [!]`), decision log, and open questions.
- `game-config.json` — Phase 0 output / Phase 1 input. `kinds` defines the taxonomy; `categories` defines what Phase 1 fetches; `human_approved` gates production runs.
- `phase1.config.toml` — Phase 1 runtime config (wiki API endpoint, retry/throttle, LLM mode + model, cache dir).
- `prompts/wiki-compile-system.md` — system prompt for the Phase 1 compile LLM. Edits to this invalidate the SHA-256 cache.
- `schemas/_universal.schema.json` — required-on-every-file frontmatter fields. Per-kind schemas extend this via `allOf`.
- `schemas/{item,skill,enemy,mechanic,location,npc,quest,system}.schema.json` — per-kind validators. Currently missing: `building`, `unit`, `organization` (the Phase-0-approved kinds for They Are Billions). Pages with these kinds get universal-only validation plus a one-time warning.

## Conventions & gotchas

- **Outputs are gitignored**: `vault/`, `build/`, `.phase1_cache/`. Don't commit them; don't write source code under those paths.
- **LLM access is via headless CLI, not the Anthropic/OpenAI SDK.** Both Phase 0 and Phase 1 shell out to `claude -p` or `codex exec`. If you need a different provider, route through `_run_llm_command` in `phase0_analyze.py` or `run_llm` in `phase1_ingest.py` rather than introducing a new SDK dependency. Cache keys include the model id, so model swaps invalidate cached compiles automatically.
- **Frontmatter parsing is hand-rolled** in `phase1_ingest.py` (`parse_yaml_map` / `parse_block`). It handles the subset of YAML the compile prompt produces; don't assume full YAML compatibility. If frontmatter is getting mis-parsed, fix the compile prompt before reaching for a full YAML library.
- **Wikilink invariant**: every `[[wiki_link]]` in a vault file body must be mirrored in `depends_on:` in the frontmatter. The compile system prompt enforces this; Phase 2's graph expansion will rely on it.
- **The taxonomy lives in `game-config.json`, not in code.** Adding a new `kind` means: (1) add it to `kinds` in `game-config.json`, (2) optionally add `schemas/<kind>.schema.json`, (3) re-run Phase 0 or hand-edit `categories` to route wiki categories to it.
- **The directory name `cloneGame` is aspirational.** No game code exists yet. Treat any task framed as "fix the game" as a request to work on the pipeline, unless the user explicitly says they've started Phase 2.

## Development Guidelines

### Code Style & Standards

- Files must be smaller than 400 lines excluding comments. Once 400 is exceeded, initiate a refactor.
- Functions must be smaller than 40 lines excluding comments and the catch/finally blocks of try/catch sections. If a function exceeds that, refactor it.

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

- Include a substantial JSDoc comment at the top of each file. For python files, use google style docstrings
- Write clear comments for complex logic
- Document public APIs and functions
- Use JSDoc comments for functions
- Keep comments up-to-date with code changes
- Document any non-obvious behavior

### Pre-Commit Checks

This repo is Python-only. Use the Python tools installable via `pip install -r requirements-dev.txt`. Configuration lives in `pyproject.toml`.

1. `ruff check scripts/` — lint + complexity (mccabe, max 10). Preview auto-fixes with `ruff check scripts/ --fix --diff`, apply with `ruff check scripts/ --fix`.
2. `ruff format scripts/` — formatter. Preview with `ruff format scripts/ --diff`.
3. `vulture scripts/` — dead-code detection. Report-only; fix flagged items manually or add to a whitelist with explicit justification.

Do not commit until all three report clean.

## General Rules

- First think through the problem, read the codebase for relevant files.
- Make every task and code change you do as simple as possible. We want to avoid making any massive or complex changes. Every change should impact as little code as possible. Everything is about simplicity.
- Never speculate about code you have not opened. If the user references a specific file, you MUST read the file before answering. Make sure to investigate and read relevant files BEFORE answering questions about the codebase. Never make any claims about code before investigating unless you are certain of the correct answer - give grounded and hallucination-free answers.

