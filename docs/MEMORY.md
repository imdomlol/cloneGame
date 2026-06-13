# MEMORY.md — cloneGame

Per-session decision log. Newest first. Each entry: what was decided, why, what was rejected (and why).

Read at the start of every session. Never contradict a logged decision without flagging it first.

Repo: `C:\Users\dominic\Documents\GitHub\cloneGame`. Project status board: `cloneGame/plan.md` in obsidian vault. Architecture spec: `docs/DEPLOYMENT_GUIDE.md`.

---

## 2026-06-13 — Lethal Company complete (160/160); 4 codegen-reliability fixes in `compile_cache.run_llm`

**Decided / done:** Drove the Lethal Company crate to **100% codegen coverage** — all entity leaves + all 9 Phase 3 systems (game_state_machine, input_handler, hud, entity_spawner, scrap_economy, combat_system, facility_timer, hazard_system, creature_ai, ship_system). `cargo build` clean, `cargo test --test app_smoke` green, `cargo run --bin clone-game` opens a window and ticks a deterministic sim (checksum advances per tick). Re-shipped to `releases/lethal-company/` with the binary bundled (the prior ship was source-only).

Getting there required fixing four backend-dispatch problems, all in `scripts/compile_cache.run_llm` (the single CLI chokepoint, so all engine/game-agnostic):

1. **Surface the real error.** On a non-zero CLI exit, fall back to the stdout tail when stderr is empty. `claude -p` prints its real error (e.g. usage-limit) to stdout, so failures were showing a blank `claude exited 1`.
2. **The claude hang root cause (confirmed via `--output-format stream-json --verbose`):** `claude -p` is a full agent and loads the user's `rust-analyzer-lsp` plugin, which injects an `LSP` tool. On a real codegen prompt the agent calls `LSP documentSymbol` to inspect sibling code; rust-analyzer indexes the whole Bevy crate and never returns → infinite hang (one call sat 6.5h). `--tools ""` does NOT cover plugin tools — only built-ins. Fix: `--disallowed-tools "LSP" --strict-mcp-config` on the claude command.
3. **Call timeout (`_CALL_TIMEOUT_SECS`).** `proc.communicate` had no timeout, so any stall froze the whole loop. Added a hard cap that kills the subprocess and fails the turn (recorded pending) so the loop keeps moving.
4. **Sized the timeout to real work: 300 → 600s.** 300s was sized for 30-90s entity leaves; complex cross-cutting systems legitimately generate 200-300s+ (measured: creature_ai 145s, ship_system 215s standalone). The 300s cap was killing ship_system under loop latency. The two "stuck" systems were never stuck — just timeout-bound.

**Why (key insight):** the LSP fix converted the *infinite* hang into a fast failure, but for the few goals complex enough to want file reads it became "spin in thinking trying to call a denied tool." The real unblock was realizing codex's `--sandbox read-only` already lets it READ sibling files (which is why its generated imports cite real sibling types), and it finishes those systems in 145-215s — so the fix for the last two was simply a timeout large enough for legitimate heavy generations, not a prompt/context change. Option 2 (inject foundation files) and "let claude read via `--tools Read`" were scoped but NOT needed once the timeout was right.

**Rejected:** (a) raising the timeout as the fix for the *claude* deadlock — disproven, a 540s claude run produced 0 bytes (genuinely stuck on denied-tool thinking, not slow). The timeout bump only helps backends that actually make progress (codex). (b) Hand-merging the diagnostic codegen output — bypasses the cargo gate; always route through the loop. (c) Building the foundation-context-injection feature (config key + prompt section) — unnecessary once codex + 600s landed both systems; revisit only if a future game has systems that genuinely exceed 600s.

**Operational notes:** claude has a hard 5-hour usage window (full reset, not rolling); codex has its own daily cap. The autonomous wall→wait→resume pattern (background quota-waiter that exits when a backend returns) worked overnight. A disk-progress watchdog (new `.rs` files vs 16-min silence) distinguishes a real hang from a quota-wall fast-fail.

**Don't reintroduce:** `--tools ""` alone for the claude codegen call (plugin tools leak past it → LSP hang). Always pair with `--disallowed-tools "LSP" --strict-mcp-config`. Don't drop the call timeout. See [[errors]] 2026-06-13 for the full diagnostic trail.

**Deferred (logged in docs/todo.md, not started):** game-specific perspective — Phase 0 should propose a `perspective` and the pipeline build a 2D or 3D world from it (Lethal Company is first-person; the scaffold is currently 2D top-down with a 2D `SimPosition`). A naive player-in-the-scaffold attempt was reverted (player/perspective is per-game, not scaffold).

---

## 2026-06-09 — `scripts/ship.py` + `scripts/load_game.py` + `releases/` convention

**Decided:** Finished games are preserved via a snapshot directory under
`releases/<slug>/`. `scripts/ship.py` writes the snapshot; `scripts/load_game.py`
restores it. Slug is auto-derived from `game-config.json -> game.name`
(lowercased, hyphenated). The convention is **independent of engine or game**:
the snapshot stores whatever the chosen engine's source tree looks like, plus
the vault and config that produced it.

What a release contains:

- `source/` — the full Cargo project (Bevy today; would be the engine's project
  shape for any other engine). Includes `src/`, `tests/`, `Cargo.toml`,
  `Cargo.lock`. `target/` is intentionally excluded (gigabytes, regenerable).
- `vault/` — Phase 1's structured notes that fed codegen.
- `game-config.json` — Phase 0 output: taxonomy + schemas + engine + system list.
- `system_map.yaml` — Phase 2 ledger: what was implemented at ship time.
- `binary/<platform>/<exe>` — compiled release binary, **gitignored**
  (`releases/.gitignore` excludes `*/binary/`); too large for git, suited for
  GitHub Releases or a separate distribution tier.
- `README.md` — generated description: shipped timestamp, engine, plugin counts,
  three ways to use the release.

**Why:** dom flagged earlier that wiping the repo to start a new game loses
the previous game entirely. The pipeline is hands-off; the operator shouldn't
have to manually copy directories. `ship` solves "I built this and want to
keep it"; `load` solves "I want to come back to it later." Both are local-only;
distribution via GitHub Releases or similar is left for a later iteration.

**Rejected:**
- (a) Git branches per game. Cleaner semantically but requires the operator to
  understand branch switching, and gigabytes of build cache from `cargo build`
  invalidate cleanly only when `target/` is in `.gitignore` per-branch.
- (b) GitHub Releases as the primary mechanism. Couples preservation to
  GitHub auth + network. Bad fit for the "user without coding experience"
  goal.
- (c) Snapshotting `build/repomix-output.xml` / `chroma/` / engine_baseline.md.
  These are regenerable from `vault/` + `game-config.json` and don't add
  signal to the release. Reduces snapshot size from ~3x to ~1x.

**Don't reintroduce:** ad-hoc rsync / manual copies. `ship` and `load` are
the entrypoints; tests in `tests/test_ship.py` + `tests/test_load_game.py`
keep them honest. 30 tests cover the full surface.

---

## 2026-06-07 (later) — Phase 3 designed + scaffolded; Phase 1 API derivation fix

**Decided:** A new "Phase 3" produces gameplay-system plugins (state machine, input, HUD, win/lose, plus per-game logic) that compose Phase 2's per-entity leaves into a playable loop. Wikis describe entities individually, not the glue that connects them, so without Phase 3 the crate compiles but doesn't play. Engine-agnostic by construction: the system list is per-game data (proposed by an LLM), and *how* to write a system lives in `prompts/engine_determinism/<engine>.md`.

**Architecture:**

1. **Phase 0** gains a `propose_gameplay_systems` pass that produces a `systems` array on `chosen_engine` (fallback: top-level). Each entry: `{name, description, depends_on (list of kinds), produces (list of resources/events)}`. Always includes three universals (`game_state_machine`, `input_handler`, `hud`) and 4–10 per-game additions inferred from the kind shape.
2. **Phase 2** unchanged. Still walks the vault for entity leaves.
3. **Phase 3** is a new flag on the existing loop driver: `python phase2/loop_driver.py --from-systems`. Reads `chosen_engine.systems`, builds goals via `derive_goals_from_systems`, runs through the same codegen / build / smoke / repair gates. Goal text: `"implement the <name> system: <description>"` — the existing `derive_kind` regex picks `system` as the kind word, routing output to `src/system/<name>.rs`.
4. **Aggregator pickup is automatic.** `phase2/entrypoint.py` greps every `impl Plugin for X` under `src/`, so generated system plugins land in `app_plugins::add_all` without any aggregator change.

**Per-engine system rules (deferred):** A `## System rules` section in `prompts/engine_determinism/bevy.md` would teach the LLM how Bevy expects state machines (`States`), input (`Input<KeyCode>`), HUD (`bevy_ui` / `bevy_egui`), and how cross-entity systems integrate with the determinism rules. Deferring to a second pass once the first system turn actually runs and we see where the LLM struggles.

**Why now:** dom flagged that the current generated game is "thin" — colored sprites, no input, no win condition. He asked whether a new phase is needed; the answer is yes, and this is its design.

**Also this turn — wiki API URL derivation fix:** `phase1.config.toml` hardcoded the They Are Billions API endpoint, so swapping `game-config.json` to a different wiki silently kept Phase 1 querying TAB. Surfaced as "0 mainspace pages" for every Lethal Company category. Fix: `scripts/phase1_ingest._derive_api_endpoint` derives the API URL from `game-config.json`'s `game.wiki_base_url` (`https://x.fandom.com/wiki/` → `https://x.fandom.com/api.php`). Engine- and game-agnostic; the explicit override in phase1.config.toml is now dead config (left in place, harmless). Bug had been latent since the fresh-run because both runs were against the same wiki.

**Rejected:** (a) Hand-curating the system list per game. Loses the autonomy that the rest of the pipeline has. (b) Generating systems via per-vault walk like Phase 2. Wikis don't have system entries; the codegen would have nothing to retrieve. (c) Putting system rules in the codegen prompt directly. Belongs in per-engine determinism data alongside the existing rules.

**Don't reintroduce:** hardcoded wiki URLs in `phase1.config.toml`. The single source of truth is `game-config.json`'s `game.wiki_base_url`.

---

## 2026-06-07 — Three pipeline-gap fixes from the fresh-run retrospective

Three engine- and game-agnostic improvements landed after the 2026-06-06 fresh run paused at 83/114:

1. **Phase 0 proposer prompt now teaches union types.** `scripts/phase0_analyze._build_schema_prompt` now includes explicit `TYPE EMISSION RULES`: numeric wiki fields → `["string", "integer", "number"]`; lists → `array`; nested maps → `object`; booleans → `["string", "boolean"]`. This is the real fix behind the `_widen_property_type` validator workaround from 2026-06-05. The widening stays as a safety net for older `game-config.json` files (`{"type": "string"}` widens to all-types; the new prompt's union output passes through untouched). Cleaner schemas downstream means stricter validation when the proposer actually does emit shape constraints.

2. **`codegen: false` flag on per-kind config** lets a kind be marked as runtime data / lore / patch notes and excluded from Phase 2 walks. `phase2.loop_driver.load_valid_kinds` now drops any kind with `codegen: false`. Marked the four data-shaped kinds (`campaign_map`, `campaign_content`, `survival_map`, `update_log`) in the current `game-config.json` so the loop's `--from-vault` no longer wastes turns on them. Game-agnostic: any wiki / engine can use the same knob. Future Phase 0 proposers should classify each kind as code-vs-data; for now the flag is set by hand based on the 2026-06-04 deferral decision.

3. **Retrieval pin priority is fine** — verified by tracing `retrieve("implement the lucifer unit", pin_id="lucifer", pin_kind="unit")` directly. Lucifer's vault note ranks first in the bundle as designed. The earlier suspicion was wrong: the Academy of Immortals content in the failing prompt was a graph neighbour packed *after* lucifer; the actual failure was codex hitting its daily quota. Logged as "verified working" rather than fixed.

**Rejected:** (a) removing `_widen_property_type` now that the proposer emits unions. Keeping as backward-compat for older configs; no cost when types already pass. (b) auto-classifying kinds as data-vs-code in Phase 0 this session. Real fix but would require a new LLM call + validation; deferred to next iteration of the proposer.

**Don't reintroduce:** `{"type": "string"}` as the default schema example in the Phase 0 prompt. The prompt now leads with the union example.

---

## 2026-06-06 — Codex concurrency fixed; spawn lock + correct model id + backend fallback

**Decided / done during the from-scratch fresh-run:**

1. **`gpt-5.3-codex` is not a valid model id** for ChatGPT-account `codex exec` users; the LLM dispatcher kept emitting `codegen_error` on every Phase 2 turn until the model id was changed to `gpt-5.5` in `pipeline.config.toml -> [phase2_codegen.models]`. Phase 0 and Phase 1 already used `gpt-5.4-mini` and worked. The error was hidden behind a `[:240]` truncation that captured only the codex banner; the actual `"The 'gpt-5.3-codex' model is not supported when using Codex with a ChatGPT account"` message lived past byte 240.

2. **Codex concurrency races on `~/.codex/skills/` init** when N codex processes spawn within ~1 second of each other (codex refreshes its 172-skill plugin catalog on every start). Added a `threading.Lock` + 3-second hold in `scripts/compile_cache.run_llm` that serializes only codex spawn (not reasoning). claude / SDK paths bypass the lock. Pilot at `--concurrency 3` ran 6/6 OK after the lock + correct model id were in place.

3. **`_format_backend_error` keeps the tail of the exception**, not the head. Backend CLIs (codex, claude -p) prepend a banner before the actual error message; truncating from the start hides quota / model / network issues. The new helper tails the message and is engine-agnostic.

4. **`--fallback-llm-mode`** in `phase2/loop_driver.py` retries any turn whose primary backend raises with a secondary backend (used here as the unattended-run safety net when codex hit its daily ChatGPT quota at ~1:30 AM PDT; the loop resumed on claude). Engine- and backend-agnostic; activates on any exception from the primary.

**Why:** the autonomous run uncovered three failure modes that any from-scratch user would hit. Each fix is engine- and game-agnostic: the model id is config, the spawn lock is a runtime invariant of the CLI backend, the error tail helper applies to any subprocess-stderr exception, and the fallback works for any pair of LLM_MODES. The CHATGPT quota itself is a per-user limit nothing in the pipeline can fix; the fallback is the right escape valve.

**Rejected:** (a) running codex at `--concurrency 1` as a workaround. Slower wall-clock and didn't actually fix the race for future users at higher concurrency. (b) pre-installing all 172 codex skills serially before the loop. Doesn't fix the per-call init that still runs in parallel; codex re-validates the catalog on every start. (c) burning Claude Code quota as the default. Codex is the cheaper subscription per turn; claude is the documented fallback only.

**Don't reintroduce:** `gpt-5.3-codex` as a default codex model id — it's invalid for ChatGPT-account users. If a future model rev makes it valid, prove it with a single `codex exec --model gpt-X` first. Also don't truncate backend exceptions from the head — always keep the tail; the meaningful text is at the end of the banner.

---

## 2026-06-05 — Phase 1 validator widening for LLM-proposed scalar schemas

**Decided:** `scripts/validation.kind_frontmatter_schema` widens any scalar-typed property in a per-kind schema (`type: "string"` / `"integer"` / `"number"` / `"boolean"` / `"null"`) to accept the full JSON type set including `array` and `object`. Phase 0's LLM proposer routinely labels numeric and richer-shape fields as strings because it sees them in string-y wikitext table context; the compile LLM correctly extracts them as integers, arrays, or objects, which the strict schema then rejects. On the fresh-run the pre-fix quarantine rate was ~71% (69/97 pages); post-fix it dropped to 0 after a single cache-hit re-run.

**Why:** game- and engine-agnostic by construction (operates on schema shape only, not field names). Preserves structural constraints when the proposer is confident (`object` declarations keep their constraint); only scalars are treated as advisory. Long-term fix is teaching the proposer to emit unions for numeric fields — until then the lenient validator catches the drift downstream.

**Rejected:** (a) tightening the proposer prompt to emit unions. Cleaner architecturally but invalidates the per-game `frontmatter_schema` block on the next Phase 0 run, which forces a Phase 1 re-compile of every page. Deferred. (b) disabling per-kind validation entirely. Loses sanity-check coverage for the (rare) case where the proposer DOES emit a meaningful shape constraint.

**Don't reintroduce:** strict scalar typing on LLM-proposed per-kind schemas. If a future proposer is reliably emitting union types, the widening helper can shrink back to only the declared types.

---

## 2026-06-05 (later) — Per-engine scaffold templates close the last hand-written gap

**Decided:** Engine foundation files that were previously hand-authored — `Cargo.toml`, `src/lib.rs`, `src/sim.rs`, `src/main.rs`, `src/app_plugins.rs` (stub), `tests/app_smoke.rs` — now live as templates under `prompts/engine_scaffold/<engine>/` mirroring the target layout. `phase2/scaffold.py` is the prelude step: it reads `chosen_engine.name` from `game-config.json`, finds the matching scaffold directory, and copies every file byte-exact into `game/`. Hand-edited files are preserved by default (status `skipped_hand_edit`); `--force` overwrites; `--dry-run` previews.

Engine dispatch is by lowercased `chosen_engine.name`, the same key as `prompts/engine_determinism/<engine>.md`. Adding a new engine is a new directory under `prompts/engine_scaffold/`, not a code change.

**Why:** Dom flagged during the planned fresh-run that the "preserve" list (Cargo.toml, sim.rs, main.rs, app_smoke.rs, etc) was a pipeline gap, not foundation. The pipeline should produce a working game from `chosen_engine` + vault — no hand-written code anywhere. The scaffold step closes this for non-codegen-shaped engine scaffolding (deps, foundation primitives, entrypoint, smoke harness). Per-game variation still flows through Phase 0 (taxonomy) → Phase 1 (vault) → Phase 2 (leaves); per-engine architecture flows through scaffold templates + determinism rules. Per-game inside-engine variation (e.g. window title) is currently hard-coded in the template and is acceptable — when a per-game knob is genuinely needed, the template can grow `{placeholder}` substitution; left out for now per "simplest solution first".

**Why no string substitution in templates:** the rendered files encode engine architecture (which Bevy version, what foundation types the bevy.md rules name), not per-game data. Per-game data lives in the vault. Adding substitution now would invite drift in the foundation contract; the current setup means the bevy.md rules reference exactly the types the scaffold lays down.

**Rejected:** (a) Generating Cargo.toml and main.rs via the codegen LLM. Adds LLM cost for files that are 100% engine-pattern, none of it game-specific. (b) Keeping these as "permanent foundation" outside the pipeline. Fails the "user runs the pipeline and gets a game" goal — a new engine would require code edits to the existing hand-authored files. (c) A manifest.json-driven file mapping. Walking the scaffold tree and mirroring relative paths is simpler and removes a metadata file that could drift from the actual contents.

**Don't reintroduce:** hand-authoring engine scaffold files in `game/`. The scaffold step is the entrypoint; a forgotten file gets noticed because the scaffold dir would have a new entry but the renderer would emit `skipped_hand_edit` on it. To genuinely customise the scaffold per game (rare), use `--force` after editing the per-game file, or carve out a new engine name (e.g. `bevy-multiplayer`) under `prompts/engine_scaffold/`.

---

## 2026-06-05 — Phase 2 step D: driver-owned app aggregator + excludable plugins

**Decided:** The crate-level plugin registry (`game/src/app_plugins.rs`) is now driver-owned, the same way per-kind `mod.rs` aggregators already are. `loop_driver.regenerate_app_aggregator` walks `src/` for every `impl Plugin for X`, derives crate-relative module paths from file paths, and writes an idempotent `pub fn add_all(app: &mut App) -> &mut App` chained over chunks of ≤14 plugins (Bevy's tuple-`add_plugins` ceiling). The regen runs as part of `_try_build` on every turn, captured in the revert trail so a failed turn restores `app_plugins.rs` byte-exact. `main.rs` and `app_smoke.rs` both call `app_plugins::add_all` instead of enumerating plugins by hand; they no longer decay when new leaves land.

Per-engine data: `chosen_engine.entrypoint = {aggregator_file, aggregator_module, tuple_chunk_size, main_file, excluded_plugins}` in `game-config.json`. Opt-in: engines that have no programmatic plugin registry (Godot scenes, Unity prefabs) omit the block and the step is skipped. `excluded_plugins` is a `{name: reason}` map that holds known-broken plugins out of the aggregator while the loop regenerates them through the smoke gate (current entry: `AcademyOfImmortalsPlugin` for the existing `&mut UnitStats` B0001 conflict).

**Why:** The smoke gate (A+B from the 2026-06-04 retrospective) only catches conflicts in plugins that are *in* the aggregator. Before D, `app_smoke.rs` was hand-enumerated, so a new leaf could land without being smoke-tested — the gate decayed silently. Auto-deriving the list from `impl Plugin for X` declarations closes that. Also: `app.update()` alone does not always tick `FixedUpdate` in Bevy 0.15 (Fixed time accumulator), so most ECS conflicts (which surface inside `FixedUpdate` systems) were not actually being run. The smoke test now calls `app.world_mut().run_schedule(FixedUpdate)` twice explicitly, guaranteeing those systems fire and surface their B0001 conflicts.

**Rejected:** (a) Generating `main.rs` itself via codegen in this pass — main.rs carries window config, camera, spawn logic, and the visible-app scope dom chose; the LLM has no spec for that. Deferred to a future iteration that gates on `main.rs` being absent rather than always re-running. (b) Hard-coding the aggregator filename in Python; per the game-agnostic principle it lives in `chosen_engine.entrypoint`. (c) Silently dropping conflicting plugins from the aggregator — that would hide the bug; the exclusion list is explicit, requires a documented reason, and shows up in code review.

**Don't reintroduce:** hand-enumerating plugins in `main.rs` / `app_smoke.rs`. The aggregator is the single source of truth; touching the enumeration by hand will be overwritten on the next loop run. If you need to exclude a plugin temporarily, add it to `excluded_plugins` with the reason — the loop's smoke gate will catch the underlying breakage and the repair loop will fix it, then the exclusion can come out.

---

## 2026-06-04 — Data-shaped kinds deferred to runtime asset path; lore + research go through loop

**Decided:** Of the untouched kinds in the backlog:

- **Runtime data assets (no codegen):** `campaign_content` (7), `campaign_maps` (24), `survival_maps` (1), `updates` (1). These will be loaded as RON/JSON assets at runtime once a loader exists; not generated as Rust modules. The asset-loader implementation itself is deferred until a runnable `main.rs` exists.
- **Go through the loop driver as code:** `research` (2), `characters` (3), `mayors` (1), `locations` (1), `organizations` (2). Same loop invocation as the infected/wonders backlog.

**Why:** Per-map Rust modules don't generalize across games — a different game's maps are pure data with no code shape worth synthesizing. Patch notes (`updates`) are not code. Lore-only kinds *might* not warrant code modules either, but the marginal cost of running them through the loop is small, the system_map ledger captures the coverage, and a future "mayor" entry might surface meaningful mechanics (the existing `game_mechanics/mayors.rs` shows there is real behavior to model). Better to let the codegen LLM judge than to skip blind. Research is plausibly mechanics-shaped already.

**Rejected:** (a) skipping the lore kinds entirely — would leave the coverage table permanently incomplete with no signal of *why*. (b) generating stubs — wastes a turn and produces noise modules that don't contribute. (c) generating per-map Rust for campaign_maps — 24 turns of LLM cost for data that doesn't compose into code.

**Don't reintroduce:** treating map/scenario data as a codegen target. When the asset loader lands, it reads RON/JSON from a `assets/` path; the pipeline doesn't generate it.

---

## 2026-05-21 — Phase 2 embedding provider: local embedder, not OpenAI

**Decided:** Phase 2's Chroma index will use a local embedding model (e.g. BGE / nomic-embed via `sentence-transformers`) rather than OpenAI `text-embedding-3-small`. Concrete model pick deferred to Phase 2 kickoff; the constraint is "runs locally, no third-party API key."

**Why:** No existing OpenAI API key and no appetite for adding a second vendor billing relationship. The wiki corpus is small enough that re-embedding on local hardware during Phase 2 iteration is cheap, and the index lives on the same machine as the rest of the pipeline so there is no privacy or latency benefit to a hosted API. Keeps the project Anthropic-only for paid LLM spend.

**Rejected:** OpenAI `text-embedding-3-small`. Cheaper per call and near-zero setup, but adds a non-Anthropic vendor dependency and a second billing surface for marginal gain at this corpus size.

**Don't reintroduce:** Only revisit if a measured failure forces it — e.g. local-model retrieval quality is provably insufficient on the actual Phase 2 task, or the corpus outgrows local-GPU comfort. Do not switch on convenience alone. Whatever embedding model is chosen must be used for both indexing and querying; swapping later requires re-embedding the whole vault.

---

## 2026-05-21 — Per-kind `required` arrays fully removed from config + prompt

**Decided:** The `required: [...]` array under every kind's `frontmatter_schema` in `game-config.json` is gone. The compile system prompt no longer references `required`. Phase 0's schema proposer emits `{properties: {...}}` only; Phase 1's `kind_frontmatter_schema` no longer needs to strip anything. The universal schema's `required` list remains the only presence gate (unchanged from the 2026-05-19 decision below).

**Why:** The 2026-05-19 decision neutralised `required` for validation but kept it in the compile prompt as a "priority" hint. In practice the hint was noise — the LLM already populates declared `properties` when the wiki has data, and the placeholder-on-missing rule (`0` / `""` / `[]`) was producing empty fields that downstream Phase 2 would need to filter out anyway. Removing it entirely keeps one contract instead of two and stops the per-kind `required` from looking authoritative when it wasn't.

**Rejected:** Keeping `required` as a prompt-only hint. Removed because it created the false impression that those fields were enforced and led to placeholder-bloat in vault files.

**Don't reintroduce:** Adding a `required` array under any `kinds.<kind>.frontmatter_schema` in `game-config.json` will be silently ignored by `kind_frontmatter_schema` and is not surfaced to the compile LLM. If a per-kind field genuinely must be present, raise it to `_universal.schema.json` (and accept that it then becomes required for every kind).

---

## 2026-05-19 — Phase 0 human-approval gate removed; auto-promotes

**Decided:** Phase 0 writes `game-config.json` directly with `human_approved: true`. The proposed-config file is still written for local diffing.

**Why:** The proposed config grew past 1k lines once per-kind `frontmatter_schema` and `engine_candidates` were added. Line-by-line review is infeasible, and real failures surface during Phase 1 validation anyway.

**Rejected:** Keeping the manual approval gate. Would have blocked every Phase 0 run on a human reading 1k+ lines, with low marginal safety since Phase 1 quarantine catches the failures that matter.

---

## 2026-05-19 — Per-kind `required` dropped from validation; kept in compile prompt as guidance

**Decided:** The universal schema's `required` list (id, name, type, source_url, source_revision, extracted_at, confidence) is the only presence gate. Per-kind validation enforces property TYPES only.

**Why:** Two LLMs proposing schemas and extracting fields independently will disagree on field names even from shared wikitext. Enforcing `required` at validation time was quarantining correct pages.

**Rejected:** Enforcing per-kind `required` strictly. Caused too many false-positive quarantines from naming drift between the proposer and the extractor.

**Superseded 2026-05-21:** the prompt-side hint was also removed; see entry at top.

---

## 2026-05-19 — Compile prompt includes the per-kind frontmatter schema

**Decided:** The Phase 1 compile prompt is rendered with the kind's `frontmatter_schema` inlined. Cache key hashes the rendered prompt (not just the bare system prompt), so schema edits invalidate caches automatically.

**Why:** Phase 0 and Phase 1 LLMs were not sharing the contract. The compile LLM reinvented field names per page (`cost_gold_build` vs the schema's `cost_gold`). Injecting the schema directly closes the loop.

**Rejected:** Trusting the compile LLM to infer field names from wikitext alone. Produced unstable, non-canonical frontmatter that broke downstream consumers.

---

## 2026-05-19 — Reference deployment chose Bevy + lockstep + fixed-sim/interpolated-render

**Decided:** For the reference target (They Are Billions), `chosen_engine` is Bevy (Rust) with deterministic lockstep networking and a fixed-tick sim decoupled from an interpolated render tick. Determinism rules captured in `CLAUDE.md > Reference deployment engine`.

**Why:** 30k entities on the wire requires lockstep, which requires bit-identical sim. Bevy's ECS + Rust + native `FixedUpdate` schedule give that foundation natively. Smooth visible motion comes from running the deterministic sim at 20–30Hz and decoupling render to interpolate between sim states at vsync rate.

**Rejected:** Godot 4 (higher-scored by the proposer at 0.87 vs 0.82) and Unity DOTS. The proposer scored from titles-only samples and missed the multiplayer goal; Godot's own con list flags networking as "not a blocker for single-player focus," which is exactly the blocker here. Godot would have required rewriting the sim in C++ extensions with manual fixed-point math, i.e. abandoning most of Godot.

**Trade-offs accepted:** Bevy is pre-1.0 (pin the version, bump deliberately). Smaller Rust hiring pool than C#/GDScript. UI ecosystem less mature — colony-resource HUD needs more custom code than a Unity/Godot equivalent.

---

## 2026-05-19 — Engine selection as LLM proposal + human approval, not hard-coded

**Decided:** Engine choice is per-game data in `game-config.json` (`engine_candidates` + `chosen_engine`), proposed by an LLM and approved by the same `human_approved` gate as the taxonomy.

**Why:** Different games need different engines (an RTS with 30k entities is not a turn-based card game). Engine choice cascades into Phase 2 codegen, so it belongs alongside taxonomy in `game-config.json`, not hard-coded in Python.

**Rejected:** Hard-coding one engine in the codegen scripts. Would have made the pipeline single-target and forced a code change per new game.

---

## 2026-05-19 — Per-kind schemas as data in `game-config.json`, not hand-coded files

**Decided:** Per-kind `frontmatter_schema` blocks live as data under `kinds.<kind>` in `game-config.json`. Standalone `schemas/<kind>.schema.json` files were deleted. `_universal.schema.json` remains as code because it is genuinely game-agnostic.

**Why:** The original design coupled schemas to code, requiring manual authoring per game. The goal is to reverse-engineer arbitrary games — anything game-specific must live in `game-config.json`. Phase 0 v2 LLM proposes the schemas; the human approves via the existing diff gate.

**Rejected:** Hand-authoring a schema file per kind per target game. Would have required Python changes for every new wiki and made the pipeline non-portable.

---

## Standing decisions (pre-2026-05-19, no exact date)

### Headless LLM CLI (`claude -p` / `codex exec`) instead of an SDK

**Why:** Avoids an SDK dependency tree and reuses the user's existing auth. `_run_llm_command` / `run_llm` are the single chokepoints to swap if needed.

**Rejected:** Pulling in `anthropic` / `openai` SDKs. Would have added a dependency tree and required separate auth/credential handling.

---

### Cache key = SHA-256 of `(rendered compile prompt + model id)`

**Why:** Catches every input that changes compile output (wikitext, system prompt, per-kind schema, model). Unchanged pages re-ingest at zero LLM cost; prompt edits or model swaps invalidate cleanly.

**Rejected:** Caching on wikitext-only or page-revision-only keys. Would have served stale compiles after prompt or schema changes.

---

### Custom hand-rolled YAML frontmatter parser

**Why:** The compile prompt produces a constrained YAML subset. A small parser avoids a `pyyaml` dependency.

**Rejected:** Adding `pyyaml`. Acceptable to switch later if the compile prompt drifts beyond the subset — fix the prompt first, switch parsers second.

---

### `human_approved` gate in `game-config.json` (now `true` by default after Phase 0 auto-promote)

**Why:** Cheap insurance against an LLM-proposed taxonomy slipping into production ingest. Phase 1 warns when the flag is `false`.

**Rejected:** Removing the flag entirely. Kept the flag for the warning behaviour even though Phase 0 now auto-sets it.


## 2026-05-22 — Phase 2 scaffolded: retrieval + baseline + codegen + system_map

**Decided:** All four Phase 2 modules now exist in the repo with pure-helper test coverage:

- `phase2/retrieval.py` (vector merge across vault_prose/vault_mechanics, 1-hop graph expand, packing capped at 2,000 tokens with hard assert).
- `phase2/baseline.py` + `prompts/engine_baseline.template.md` + `prompts/engine_determinism/bevy.md` + `_generic.md` (renders to `build/engine_baseline.md`, currently 2,104 of 2,500 token budget for Bevy + lockstep).
- `phase2/codegen.py` (assembles the 4-section user prompt, sends `cache_control: ephemeral` on the engine baseline, logs per-turn cache hit rate, warns below 80%, validates `// Sources:` header against the retrieval bundle's allowed paths).
- `phase2/system_map.py` (rolling `build/system_map.yaml` with record_implementation / record_pending / update_test_state / set_baseline_hash; deterministic `cap_tokens` collapses oldest entries to a `{summary, count}` line; optional Haiku-driven `summarise_with_haiku` provided for richer compaction).

**Why:** unblocks the first real codegen turn. Indexer landed 2026-05-21 and exposed the layer-3/4 + Anthropic call as the next pieces. Picked retrieval-first so the 2k-token cap could be validated against the live vault before any SDK call was wired.

**Rejected:** stubbing retrieval and writing codegen end-to-end first. Would have hidden cap and graph-expansion bugs behind a placeholder until the first paid turn.

**Don't reintroduce:** hard-coded engine rules in Python. Determinism rules live as data under `prompts/engine_determinism/<engine>.md`, keyed by lowercased `chosen_engine.name`. Adding a new engine = new markdown file, not a code change.

**Not yet done (deferred to dom's review):**
- No live API call has been made. `phase2/codegen.py` runs `--dry-run` end-to-end (baseline 2,104 tok + user 2,100 tok + 3 retrieved vault files for the prompt "implement the soldier unit"), but actually calling `anthropic.Anthropic().messages.create` is gated on dom confirming an in-session yes per CLAUDE.md Default Behavior #12.
- `requirements-phase2.txt` was extended with `tiktoken`, `anthropic`, `pyyaml`. `pip install -r requirements-phase2.txt` should be re-run.
- `CLAUDE.md`'s "Phase 2 (codegen) is not yet implemented" line is now stale but left untouched (rule 10).
- The "Engine-baseline cache hit-rate logged; alert if &lt; 80%" guardrail is implemented in `codegen.log_usage` but only verifies after a real turn ships.


## 2026-05-22 — Follow-ups landed; first live codegen turn blocked on ANTHROPIC_API_KEY

**Decided:** Doc + driver follow-ups shipped. The Anthropic SDK auth path is the blocker for the first live turn — `ANTHROPIC_API_KEY` is not set in this environment, so `phase2/codegen.py "implement the soldier unit"` exits with `Could not resolve authentication method` before any tokens spend. No charge incurred.

**What shipped 2026-05-22 evening (after the initial scaffolding):**
- `docs/CLAUDE.md`: flipped the two "Phase 2 not implemented" lines, updated greenfield paragraph, added phase2/ to the documented lint scope, added phase2/ module map, added Phase 2 common commands block.
- `docs/DEPLOYMENT_GUIDE.md`: §3 header + §3.3 / §3.4 / §3.5 / §3.6 / §5.3 wording flipped from "to be written" / "not yet operational" to current state.
- `phase2/driver.py` + `tests/test_phase2_driver.py`: single-shot turn driver (codegen call → `// Sources:` validation → write file or record_pending → update system_map). 115 tests passing repo-wide.
- `pip install -r requirements-phase2.txt`: all deps already satisfied; no install drift.

**Why API-call mode (SDK) and not `claude -p` like Phase 1:** Phase 2's cost model depends on `cache_control: ephemeral` on the engine baseline (DEPLOYMENT_GUIDE §4 documents ~76% savings vs no cache). The CLI doesn't expose cache controls. Switching to `claude -p` here would defeat the design, not just complicate auth.

**Unblock path for next session:**
- In PowerShell (per-session): `$env:ANTHROPIC_API_KEY = "sk-ant-..."` then re-run `python phase2/codegen.py "implement the soldier unit"`.
- Persistent: `[Environment]::SetEnvironmentVariable("ANTHROPIC_API_KEY", "sk-ant-...", "User")` then restart the shell.
- Either way: expect turn 1 to write the cache (`cache_creation_input_tokens ≈ 2,104`) and turn 2 within the 5-min TTL to read it (`cache_read_input_tokens ≈ 2,104`). If turn 2's cache_read is 0, the cache-control wiring is wrong, not the math.

**Rejected:** Asking dom for an API key in chat. Would have invited pasting a secret into an LLM context against best practice.


## 2026-05-22 — Phase 2 live: first codegen turn succeeded via `claude -p` on Sonnet 4.6

**Decided:** Phase 2 codegen is operational. `phase2/codegen.py "implement the soldier unit" --output build/turn1_soldier.md` produced 18,594 bytes of real Bevy code in 5 files (Cargo.toml, src/lib.rs, src/sim.rs, src/units/mod.rs, src/units/soldier.rs). Default mode is now `claude` (Phase 1-style CLI dispatch, no Anthropic API key, billed via existing Claude Code subscription) and the default model is `claude-sonnet-4-6`.

**The 18kB of Rust honors every determinism rule from `prompts/engine_determinism/bevy.md`:** `I32F32` fixed-point throughout the sim path, `rand_chacha::ChaCha8Rng` seeded RNG, no transcendentals, sim/render split, periodic checksum hooks. YAML frontmatter numbers translated 1:1 (HP=120, MS=2.4, AD=16, armor_std=0.4, etc.). `// Sources: vault/unit/soldier.md` header present and validated.

**Multi-mode dispatch landed (codegen now supports three backends):**
- `claude` (default): shells to `claude -p`, uses Claude Code subscription auth, no API key, no token-level billing visibility, no `cache_control`. Sonnet 4.6 default model.
- `codex`: shells to `codex exec`, matches Phase 1 production (`gpt-5.4-mini`). Cheap per token.
- `sdk`: `anthropic.Anthropic().messages.create` with `cache_control: ephemeral` on the engine baseline. Sonnet 4.6 default model. Requires `ANTHROPIC_API_KEY` (not currently set; only path that realizes the §4 cost-cache savings).

**Why Sonnet, not Opus:** structured code gen under strict constraints (YAML→Rust + determinism rules + source-header rule) is well inside Sonnet's competence. Per-turn cost is ~$0.036 vs Opus ~$0.18; on the 1,500-turn campaign that's ~$54 vs ~$270. Caveat: in `claude` CLI mode the dollar math doesn't apply (subscription billing), but request rate / latency still benefit from the cheaper tier.

**Why `claude` CLI mode is default (not `sdk`):** dom has no Anthropic API key and no appetite for setting up a second billing relationship beyond the existing Claude Code subscription. `claude -p` reuses that auth at zero marginal cost. The `cache_control: ephemeral` savings from §4 disappear in CLI mode, but the alternative is paying for an API key just to save tokens on Anthropic itself, which is a wash for hobby-scale usage.

**Quirk: `claude -p` invoked from inside this codegen.py wrapper opens a real Claude Code instance which tries to use Write/Edit tools, gets blocked by sandboxing, and falls back to inline code blocks prefixed with "Permissions are blocking writes. Here is the complete implementation...". The actual generated code follows the preamble verbatim. Validators and downstream parsers should tolerate this preamble (do not strip it — the model leaves the actual `// Sources:` headers intact inside the code blocks).

**Rejected:** Defaulting to Opus. Killed by cost math + dom's pricing preference; promote per-task with `--model claude-opus-4-7` when a turn comes out wrong.

**Don't reintroduce:** Hard-coding the SDK as the only backend. `_MODEL_DEFAULTS` + `_dispatch` in `phase2/codegen.py` are the seam for adding modes; new ones (e.g. local llama, openrouter) go there.


## 2026-05-22 — Cargo regression anchor landed; turn1 output compiles verbatim

**Decided:** `build/turn1_soldier.md` was extracted into `game/` at repo root with no edits, and `cargo build` succeeded clean on the first try (3m 55s cold build, 1m 24s `cargo check`, zero warnings, zero errors). Cargo.lock committed. The five files (Cargo.toml, src/lib.rs, src/sim.rs, src/units/mod.rs, src/units/soldier.rs) are now the frozen reference unit for every subsequent Phase 2 codegen turn.

**What this confirms:** the Phase 2 pipeline (vault → repomix → Chroma → retrieval → engine baseline → codegen via `claude -p` on Sonnet 4.6) produces Rust that compiles against Bevy 0.15's real API surface. The determinism rules in `prompts/engine_determinism/bevy.md` are no longer just enforced by `// Sources:` header text; they are enforced by rustc. `I32F32::lit("0.0134")` is a valid const fn at parse time, the `.chain()`'d system order works as advertised, and `EventWriter::send` is still accepted in 0.15 (it was deprecated in favor of `write` but the alias remains).

**Rejected:** Hand-editing the turn1 output before extraction. Tempting (Bevy 0.14→0.15 API drift, missing `indexmap` pin) but it would have masked whether the codegen is actually working. If `cargo build` had failed, that failure was the signal to tune the prompt or the determinism rules; pre-patching the output silences the signal. Same logic for downloading newer Bevy: pin matches what turn1 declared, even if it's not the absolute latest.

**Don't reintroduce:** committing without `cargo build game/` clean. Every Phase 2 codegen turn from here lands alongside a clean build. The loop-driver (next todo) should grow a `cargo build` gate before `record_implementation`.

**Operational notes (Rust toolchain):**
- Installed via `rustup-init.exe` (downloaded from https://win.rustup.rs/x86_64) with `-y --default-toolchain stable --profile minimal`. Resulting versions: cargo 1.95.0, rustc 1.95.0, toolchain `stable-x86_64-pc-windows-msvc`.
- MSVC build tools already present (`vswhere` reports VC.Tools.x86.x64 in the VS BuildTools install); no separate winget step needed.
- `cargo`/`rustc` live under `%USERPROFILE%\.cargo\bin\`. rustup-init added that to user PATH via the registry; new shells pick it up. Inside an already-running shell, prepend with `$env:Path = "$env:USERPROFILE\.cargo\bin;$env:Path"` if needed.
- First-build cold time was 3m 55s and pulled ~280 dependencies (full Bevy graph). Incremental `cargo check` is 1m 24s. If iteration speed becomes an issue, narrow Bevy to its sub-crates with `default-features = false` once the soldier outgrows the prelude needs.


## 2026-05-23 — Phase 2 loop driver: driver owns the module graph (data-driven)

**Decided:** The Phase 2 loop driver (`phase2/loop_driver.py`) owns aggregator/registry files (Rust `mod.rs`, `lib.rs`); the codegen LLM produces only leaf modules. After writing a leaf like `src/units/ranger.rs`, the driver appends `pub mod ranger;` to the existing aggregator itself (idempotent, byte-preserving), and skips any aggregator the LLM emits. The aggregator path and declaration are **data**, not code: `game-config.json -> chosen_engine.module_registration = {"aggregator": "{dir}/mod.rs", "declaration": "pub mod {stem};"}`. The driver renders these templates and holds no language literals. Engines without the block (e.g. C# namespaces, Godot scenes) get None and the step is skipped — opt-in per engine.

**Why:** The first multi-file loop turn (ranger) had the LLM regenerate `src/units/mod.rs` as just `pub mod ranger; pub mod soldier;`, dropping the shared `pub struct Infected;` marker that `soldier.rs` imports → cargo E0432 → revert. This would recur on every unit turn. Letting the tool manage registry/barrel files while the LLM writes leaves is the standard codegen pattern and is deterministic rather than hoping the model preserves everything.

**Why data-driven, not hard-coded:** dom emphasised the game-agnostic principle is strict. Hard-coding `pub mod`/`mod.rs` in `loop_driver.py` would make the driver Rust-only. Putting the templates in `chosen_engine` keeps the driver neutral and matches the existing pattern (determinism rules, per-kind schemas, engine choice all live in data). The residual "concept dent" (not every engine has aggregator files) is handled by making the block optional.

**Rejected:**
- Robust multi-format markdown parser (option A) — whack-a-mole; the model can always invent a new shape.
- Feeding aggregator contents into the prompt and trusting the LLM to preserve them (option B) — it already proved it drops things; bloats every prompt.
- Hard-coding the Rust module rules in Python — violates game-agnostic principle.

**Don't reintroduce:** letting the codegen LLM own registry/barrel/aggregator files. Also see [[errors]] note 2026-05-23: `claude -p` codegen MUST run with `--tools ""` and a rigid `=== FILE: ===` output contract; free-form markdown output is unreliable.

**Known limit:** a leaf directly under the crate root (`src/sim.rs`) renders an aggregator the root may not use; that fails loud via cargo rather than corrupting the tree. Unit-by-unit codegen puts leaves in subdirectories, which the template handles.


## 2026-05-25 — Per-stage model config centralized; codegen default → codex CLI

**Decided:** All per-stage LLM mode/model defaults now live in a single user-editable `pipeline.config.toml`, read via `scripts/model_config.py` (`resolve_llm`, `default_model`, `default_llm_mode`, `default_embedding_model`, `default_compactor_model`, with a built-in `FALLBACK_CONFIG`). Explicit CLI/model args still win; config only fills unset defaults. Phase 0, Phase 1, and Phase 2 codegen now default to the **codex CLI** (`codex exec`, gpt-5.x) rather than `claude -p`. Refactor authored via codex, reviewed + integrated + committed (476bb42).

**Why:** codex CLI is cheaper per token than claude CLI and, like claude -p, uses existing CLI auth — **no API key**. The binding constraint has always been "no API keys" (which rules out `sdk` mode, the only path needing `ANTHROPIC_API_KEY`); it was never "Anthropic-only". Both CLIs satisfy the constraint, so the cheaper one is the sensible default. Centralizing the config also kills scattered model constants (codegen `_MODEL_DEFAULTS`, phase1.config.toml `[compile]`, phase0 hardcoded defaults, indexer/system_map literals) — one place to change a model.

**Supersedes:** the 2026-05-22 "Phase 2 codegen defaults to claude CLI" decision. That entry framed the choice as avoiding OpenAI billing; the real constraint is no API keys, and codex CLI uses subscription auth, not a key. `sdk` mode remains wired but unused.

**Rejected:** `sdk` mode as a default — needs `ANTHROPIC_API_KEY`, against the no-keys rule. Hardcoding models per script — the whole point of `pipeline.config.toml` is one data-driven source of truth.

**Don't reintroduce:** scattered model-default constants. New stages read from `model_config`. Also: do not let codex (or any tool) hand-write game/ unit files outside the loop driver — codex manually wrote a broken `caelus.rs` (borrow-check error) that bypassed the cargo-build gate; it was reverted and regenerated through `loop_driver.py` instead. The gate exists precisely to catch this.


## 2026-05-25 — Phase 2 units complete via the C-upgrade loop; per-engine compile rules; vault-migration move fix

**Decided / done (autonomous session while dom away, nothing pushed):**

1. **The loop-driver C upgrade (`--from-vault`) is proven and the `units` kind is 9/9.** First volume run landed 4 units (calliope, mutant, thanatos, titan), skipped the 3 already done, and routed lucifer + sniper to pending on the cargo gate. After the fixes below, sniper landed on retry and lucifer landed on a 2nd pass. `cargo build` of all 9 units is clean; `system_map` implemented=9, pending=[]. Flipped the C-upgrade checkbox to [x] in plan.md + todo.md.

2. **Compile-correctness guidance is per-engine data, not Python.** The two pending failures were E0277s in codex output. Added rule 8 to `prompts/engine_determinism/bevy.md` (a marker/component placed in a `#[derive(Bundle, Default)]` must itself derive `Default`). Renamed the baseline template section `## Determinism rules` → `## Engine rules` so each `<engine>.md` can hold determinism + compile-correctness without mislabeling. **Why:** the per-engine `<engine>.md` file is already the engine-agnostic mechanism (keyed by `chosen_engine.name`); a Bevy-only fix belongs there, and a future Godot/Unity run grows its own file. No code path branches on engine. **Rejected:** a rule for lucifer's by-value-Query slip (one-off codegen variance, not a recurring class — re-run fixed it) and a dedicated generic "gotchas" template slot (premature; revisit when a 2nd engine is real). Baseline renders 2653/2800 tokens, under cap.

3. **Phase 1 vault migration is now a MOVE, not a copy.** `vault_index.migrate_existing_note` wrote the note into the new kind dir on a Phase 0 kind rename but never deleted the original, so re-ingesting after the 2026-05-21 singular→plural kind rename left BOTH `vault/unit/` (stale, `type: unit`) and `vault/units/` (canonical) etc. Fixed: delete the original on successful migration + drop its stale `source_idx` entry, guarded against same-path no-ops and quarantined-migration data loss. **Why:** the duplicate dirs doubled the retrieval corpus and let `--from-vault --kinds unit` walk stale data (it only worked by luck because retrieval indexes the whole vault). **Not done:** did not delete the existing stale dirs (gitignored regenerable output, but Default Behavior 11 + dom away → left for dom; the fix prevents recurrence).

4. **Verified Phases 0 + 1 are autonomous** (no interactive prompts; Phase 0 auto-promotes; Phase 1 `--dry-run` runs clean). Did not re-run them live (no benefit, would churn the approved config / re-spend on 135 pages).

**Open for dom's decision:** generated units have inconsistent health/stats APIs (each turn reinvents its own `*Health`). A shared `Health`/`UnitStats` contract in the engine baseline would fix it but changes generated-code architecture — left for dom. The "smoke test" todo is blocked on this (a per-unit test now would couple to 9 divergent APIs). See `cloneGame/session-log-2026-05-25-autonomous.md` for the full running log.


## 2026-05-25 (later) — Shared Health/UnitStats foundation contract + parallel codegen

**Decided / done (dom present, approved each step; nothing pushed):**

1. **Shared `Health` + `UnitStats` foundation types.** Added to `game/src/sim.rs`: `Health { current, max: I32F32 }` (+ `Health::full(max)`) and `UnitStats { move_speed, attack_range, attack_damage, attack_speed, watch_range }`. All 9 units regenerated to use them; 0 self-defined `*Health` structs remain. **Why:** codegen had each unit reinventing its own health/stats component (soldier `SimHealth`, ranger `RangerHealth(u32)`, sniper `SniperHealth`), preventing shared logic and a uniform smoke test. **Architecture:** broad principle in the engine baseline TEMPLATE (game/engine-agnostic: "reuse shared foundation types, never redefine per unit"); concrete type named in the per-engine `bevy.md` rule 9 (engine-specific), exactly like rule 7 names `SimChecksumState`. No driver/Python change. A future engine names its own foundation types in its own determinism file. **Rejected:** giving `Health`/`UnitStats` a `Default` — there is no sensible default HP; units must construct via `Health::full(max)`. Consequence: a unit using `#[derive(Bundle, Default)]` with a `Health` field fails (caelus hit this, 8/9 did not); WATCH-ITEM for the buildings/infected runs — add a rule-8 clause (trim the now-full baseline to fit) or a guarded Default only if it recurs.

2. **Baseline is now at the 2800-token cap.** Rules 8 (Default-bundle) and 9 (shared Health/UnitStats) were added to `bevy.md`; the template section was renamed `Determinism rules` → `Engine rules` to hold compile-correctness alongside determinism. Any future prompt addition must trim or move detail into retrieval.

3. **Parallel codegen mode (`--concurrency N`, default 1).** `loop_driver.run_loop` split into phase A (`_prepare_turn`: codegen+validate, pure, parallelizable) and phase B (`_commit_turn`: merge+register+cargo gate+record, serial). N>1 prepares upcoming turns in lookahead batches concurrently; the gate stays serial so results/system_map order/error budget are byte-identical to the serial path. **Why:** overlap the dominant LLM wait on the big upcoming runs (buildings ×42, infected). **Rejected (for now):** model-racing (codex+claude per turn) — multiplies cost for marginal gain; the cargo gate + cheap re-run already recover failures. **Safe because:** `retrieval.query_collections` builds its own Chroma client + embedder per call (no shared state); concurrency is bounded by memory, not correctness.

4. **Vault hygiene:** deleted the 11 stale singular kind dirs (superseded by plural canonical kinds); rebuilt repomix + Chroma index (112 notes). Veteran promotion confirmed (dom) to exist only for soldier/ranger/sniper on the wiki — vault + code are correct, not a gap.

**State:** 9 units (shared contract) + 1 building all compile; system_map 10 implemented / 0 pending; 177 python tests green. All local, NOT pushed (awaiting dom's review). Full running log: `cloneGame/session-log-2026-05-25-autonomous.md`.


## 2026-05-26 — Codegen consistency: sibling-exemplar + engine-agnostic build-error repair loop

**Decided / done (dom present, approved each step; nothing pushed):**

1. **Sibling-exemplar mechanism (fixes per-turn API drift).** Root cause of drift: each leaf module was generated in isolation, so every unit invented its own public surface (some had `base_health()`, some didn't; HP const visibility was random). Fix: the loop driver now passes the first accepted module of a kind into every subsequent same-kind turn's codegen prompt as a `[REFERENCE SIBLING MODULE]` block, instructing the model to match its *public surface* (accessors/constructors + organization), adapting names/values. **Where it lives:** the instruction text is in `codegen.build_user_message` (game- AND engine-agnostic Python), NOT in `bevy.md` and NOT a name mandate. dom was explicit: drift can happen on any engine, the fix must be broad, and mandating uniform names is wrong (games differ). The exemplar is resolved per kind in `loop_driver.run_loop`: pre-seeded from already-implemented modules (`_seed_exemplars`) so a kept module seeds its siblings across runs, and captured in-run on the first accepted module (`_capture_exemplar`). `_chunk_end` forces the first-of-kind to commit before siblings are prepared, so the mechanism holds under `--concurrency`.

2. **Build-error repair loop (engine-agnostic compile-correctness).** dom rejected adding a Bevy-specific rule for each compile failure class ("shipped product, can't adjust per new engine"). Instead: on a `cargo build` failure the driver feeds the build tool's own error output + the failed source back through codegen (`[BUILD FAILURE — REVISE]` block), retrying up to `--repair-attempts` (default 2) before recording pending. Engine-agnostic by construction: rustc today, a Godot/Unity build check tomorrow by swapping the cargo runner; no per-engine rules needed. Lives in `loop_driver._commit_turn` (`_try_build` + `_make_repair_fn`) and `codegen._repair_section`. **This supersedes the per-engine-rule approach from the 2026-05-25 entry** (rule 8 in `bevy.md` stays but we stopped adding new ones).

3. **All 9 units regenerated from scratch through the loop; smoke test landed.** Deleted all 9 unit files + reset `units/mod.rs` to the bare `Infected` marker, ran `--from-vault --kinds units`. The 9 converged on ONE uniform shape: `const <UNIT>_HP`, `#[derive(Bundle)] pub struct <Unit>Bundle { pub health: Health, pub stats: UnitStats, ... }`, hand-written `impl Default` doing `Health::full(<UNIT>_HP)`. `game/tests/unit_health.rs` asserts `<Unit>Bundle::default().health.max == vault HP` for all 9 (caelus 120, calliope 60, lucifer 500, mutant 2000, ranger 60, sniper 150, soldier 120, thanatos 250, titan 800) — passes. The old soldier rich surface (base_health/base_stats/spawn helpers) is gone, replaced by the bundle shape; dom chose to keep the bundle shape.

**Why the first-of-kind is the fragile step (confirmed empirically):** the from-vault walk is alphabetical, so caelus was first and generated with NO exemplar → it diverged (auto-derived `Default` on a bundle holding non-`Default` `SimPosition`) → failed. calliope became the de-facto standard. Lesson: the first entry of a kind has nothing to anchor it; its quality sets the standard for all siblings. If a specific shape is wanted, seed/order that unit first.

**Rejected:** (a) a `bevy.md` rule for the 15-element tuple limit — engine-specific, doesn't scale; the repair loop handles it generically. (b) re-seeding with a chosen exemplar after the run — the bundle shape is already uniform and testable. (c) mandating uniform accessor names — games require different schemes.

**State:** 9 units + 1 building implemented, `cargo build` clean, `cargo test` green (unit_health), system_map 10 implemented / 0 pending, 185 python tests + ruff + vulture clean. All local, NOT pushed.

**Don't reintroduce:** per-engine compile-correctness rules as the default fix (use the repair loop); letting leaf modules be generated with no sibling context when siblings exist.


## 2026-05-26 system_map ledger decoupled from prompt cap

**Decided:** The persisted `build/system_map.yaml` keeps the COMPLETE `implemented` list (it is the idempotency ledger `already_implemented` reads to skip finished work). The 1,000-token cap (DEPLOYMENT_GUIDE 3.7) now applies ONLY to the Layer-4 prompt projection, via new `system_map.render_for_prompt(path)`. `run_loop` and the system_map CLI `_dispatch` no longer persist the capped state.

**Why:** `cap_tokens` was collapsing the oldest `implemented` entries into a `{summary, count}` line and that capped view was being saved to disk. Once enough modules accumulated (buildings run pushed past the cap), older ids were evicted, `already_implemented` stopped recognising them, and the loop re-generated them every run. It never converges and would be fatal for the full ~130-note game. The filesystem + full ledger are truth; the prompt view is a bounded projection.

**Rejected:** Raising TOKEN_CAP (just delays the same failure); Haiku summariser as the ledger (still lossy, costs tokens). Both keep the ledger and prompt-context concerns conflated, which is the actual bug.

**Also this session (both broad, engine/game-agnostic):**
- Retrieval now pins the goal's own vault note into the bundle (`retrieval.resolve_pin` + `seed_with_pin`, threaded as `pin_id`/`pin_kind` through codegen.generate and loop_driver). The imperative task phrasing did not reliably rank a note's own spec into the vector top-k, so the model was implementing modules without their own spec in context (caused land_mine + wood_workshop `invalid_source_header`).
- Chroma `PersistentClient` + embedder are now built once per (dir, model) behind a lock (`retrieval._get_collections`), fixing the concurrent "Could not connect to tenant default_tenant" race under `--concurrency 2` (caused advanced_quarry codegen_error) and removing the per-turn embed-weight reload.

**State:** buildings 42/42 + units 9/9 = 51 implemented (ledger rebuilt from disk after the truncation). Not committed per dom's instruction.


## 2026-05-26 (cont.) Loop run results + stopping point

**Ran through the gated loop (codex, concurrency 2, repair 2), all gated on cargo build:**
- buildings 42/42, infected 14/14, then game_mechanics 9 + wonders 6 + research 2 + characters + mayors = 19. With units (9) already done: **84 implemented, 0 pending, full crate builds green** (warnings only: unused mut, dead const in generated files).

**Within-run duplicate-id fix (broad):** `loop_driver._build_plan` now skips a note_id already attempted earlier in the same run, not just ones already implemented. A slug can appear under two kinds (`technology_tree` is in both game_mechanics and research) and was being generated twice into two files + two ledger entries. First occurrence wins. Test: `BuildPlanDedupTests`. The already-built duplicate (`technology_tree` x2 on disk + ledger) was left as-is; harmless, dom can prune.

**Module placement observation (not fixed, likely fine):** the model places leaf files by semantic fit, not strictly by kind dir, so wonders landed in `src/buildings/`, infected partly in `src/units/`, research/mayors in `src/game_mechanics/`. Everything compiles and is wired. Arguably correct (wonders are special buildings). Flagged for dom in case strict kind→dir is wanted.

**Stopped before the content/lore kinds (judgment call for dom):**
- Skip as non-code: `updates` (patch notes), `locations` (lore, conf 0.18), `organizations` (thin factions, conf 0.34), `campaign_content` (heroes that dedupe against units).
- `campaign_maps` (24) + `survival_maps` (1): legitimate game DATA (missions, map modifiers), but level/content data likely belongs in a data-asset path (RON/JSON), NOT LLM-generated Rust per map. Generating per-map Rust would not generalize across games. Left for dom to decide codegen-vs-asset before running.
