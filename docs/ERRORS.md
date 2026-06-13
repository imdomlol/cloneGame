## 2026-06-13 — Two high-fan-in Phase 3 systems won't generate: agent dead-locks on "read files first" with tools disabled

**Context:** Phase 2 `--from-vault` reached 100% (158 leaves). Phase 3 `--from-systems` landed 7 of 9 systems (game_state_machine, hud, entity_spawner, scrap_economy, combat_system, facility_timer, hazard_system). Two would not generate on any backend: `creature_ai` and `ship_system` — the two highest-fan-in systems (creature_ai depends on every creature kind; ship_system on scrap + upgrades + mission state).

**What didn't work (4 attempts, both backends):**
1. codex c=1: `creature_ai` hit the 300s timeout (killed); `ship_system` `codegen_error`.
2. codex c=1 retry: both `codegen_error` (no parseable output, error tail was just the echoed goal — codex produced nothing usable).
3. claude c=1: both hit the 300s timeout (killed).
4. claude single foreground call with a **540s** cap: `creature_ai` produced **0 bytes** in 540s.

**Root cause (confirmed via `claude -p ... --output-format stream-json --verbose` probe):** the input prompt is normal size (~5.1k tokens: baseline 2089 + user 3042, only 2 retrieved files), so it is NOT an input-size problem. The probe showed 1 `init` event, then **129 `thinking_tokens` events** (extended thinking, 3543 tokens) and only 4 `assistant` events. The assistant text was: *"Let me read the existing system files to understand the patterns and types used before writing the creature AI system"* followed by `<function_calls><invoke name=...`. The model wants to **call a tool to read sibling files before writing**, but tools are disabled (`--tools "" --disallowed-tools LSP`), so it loops in thinking/aborted tool-call attempts and never emits the `=== FILE: ===` blocks. Simple leaves (e.g. `teeth`) write directly and finish in ~40s; complex cross-cutting systems insist on inspecting code first and dead-lock when they can't.

So the LSP-disable fix (which stopped the *infinite* hang) converted "hang on a real tool call" into "spin in thinking trying to call a tool that's denied" for the few goals complex enough to want file reads. The 300s timeout then fires.

**What worked instead:** nothing yet for these 2. The game is fine without them: 158 leaves + 7 systems build clean, `cargo test --test app_smoke` green, sim ticks. They are recorded `pending` in `build/system_map.yaml`.

**Fix candidates (not yet applied — shared-path change, needs a decision):**
- (a) Add a codegen-prompt instruction: "Do not attempt to read files or call tools; emit the FILE blocks using ONLY the provided context." Forces direct output. Cheap but edits the shared system prompt (cache invalidation, affects all codegen) — test on these 2 first.
- (b) Give system goals richer retrieval (only 2 files were included) so the model has the foundation types/signatures it wants without needing to read them. Knob change in retrieval.
- (c) Split the 2 high-fan-in systems in `propose_gameplay_systems` into smaller, concrete goals the model can satisfy from local context. Phase 0 change.

**Note for next time:** don't burn blind retries on a system goal that times out with **zero** output — probe with `--output-format stream-json --verbose` first. If the events are mostly `thinking_tokens` + aborted `<function_calls>`, it's the read-first dead-lock, not quota/backend; the fix is prompt/context, not another retry. Don't exceed ~2 attempts before probing (this cost 4 long calls + a 540s probe before the cause was clear).

---

## 2026-06-09 — Phase 3 first run: StatesPlugin double-add + smoke gate scope gap

**Context:** first Phase 3 run on Lethal Company. Phase 0's `propose_gameplay_systems` produced 10 systems; the loop generated `GameStateMachinePlugin` and `InputHandlerPlugin`. cargo build passed, smoke test passed, but `cargo run --bin clone-game` panicked at startup: `Error adding plugin bevy_state::app::StatesPlugin: : plugin was already added`.

**Why the smoke gate missed it:** `game/tests/app_smoke.rs` constructs the app with `MinimalPlugins`, which does NOT include Bevy's `StatesPlugin`. `main.rs` uses `DefaultPlugins`, which DOES include it. The generated `GameStateMachinePlugin` called `app.add_plugins(bevy::state::app::StatesPlugin)` AND `app.init_state::<GameState>()`. With MinimalPlugins, only one registration → smoke passes. With DefaultPlugins, two registrations → runtime panic.

**Fix (immediate):** Added `GameStateMachinePlugin` to `chosen_engine.entrypoint.excluded_plugins` with a reason note. Regenerated `app_plugins.rs` aggregator. Build and run clean again.

**Fix (architectural):** Updated `prompts/engine_determinism/bevy.md` rule S1: "Do NOT call `app.add_plugins(StatesPlugin)` — `DefaultPlugins` already registers it, and a second registration panics at runtime." Future regenerations of GameStateMachinePlugin will not redo the add.

**Note for next time:**
- The smoke gate is necessarily incomplete because adding `DefaultPlugins` to the smoke would pull in winit / wgpu / a real window, which is unwanted in CI. Hence smoke uses MinimalPlugins and accepts that some `DefaultPlugins`-only conflicts slip through.
- The pattern to watch for: any system plugin that calls `add_plugins(<a Bevy stock plugin>)`. The bevy.md S1-S7 rules now explicitly call this out for `StatesPlugin`; future stock plugins re-added similarly will also need a rule.
- Operators who hit "plugin was already added" at runtime should: (a) check `app_plugins.rs` for which generated plugin re-added it, (b) add that plugin name to `excluded_plugins` with a reason, (c) update bevy.md if the offending pattern isn't already documented.

---

## 2026-06-06 — Codex CLI: invalid model id + concurrent init race + tail-truncation hid both

**Context:** fresh-run Phase 2 codegen. Every codex turn returned `codegen_error`. The truncated error message looked like a concurrency race ("failed to install system skills"), so the first fix attempt was a serialization workaround.

**What didn't work:**
1. **Running codex at `--concurrency 1`** to avoid the race — turns still failed every call. The race wasn't the root cause; it was a single-turn failure that happened to coincide with concurrency-time experiments.
2. **Adding a `--fallback-llm-mode claude`** safety net — claude DID kick in but produced `invalid_source_header` and `no_file_blocks` validation errors at a high rate. Useful as a fallback but not the right primary.
3. **Truncating exception text with `str(exc)[:240]`** — captured only the codex banner (model id, workdir, sandbox, session id) and discarded the actual error after the banner. Made every codex failure look identical regardless of root cause; took 90+ minutes to discover the real reason because I trusted the truncated message.

**What worked instead:**
1. **Reading the FULL exception in a standalone test** — `python -c "from compile_cache import run_llm; print(run_llm('say hi', 'codex', 'gpt-5.3-codex'))"` immediately surfaced `"The 'gpt-5.3-codex' model is not supported when using Codex with a ChatGPT account"`. Should have been the first diagnostic.
2. **Switching `pipeline.config.toml -> [phase2_codegen.models].codex` from `gpt-5.3-codex` to `gpt-5.5`.** Phase 0 and Phase 1 used `gpt-5.4-mini` and worked all along — only Phase 2 codegen had the bad id.
3. **A `threading.Lock` + 3-second hold around codex `Popen`** in `scripts/compile_cache.run_llm` to serialize spawn (not reasoning). Catches the real concurrency race that would surface once the model id was fixed and concurrent codex calls actually ran. Engine- and game-agnostic.
4. **Tail-truncation in `_format_backend_error`** — keeps the last 480 chars of the primary exception (and 240 of any fallback exception). Backend CLIs always banner-then-error; the tail is the signal.

**Note for next time:**
- When a CLI subprocess fails, **read the full stderr/stdout BEFORE trusting any truncation**. The pipeline now does this automatically via `_format_backend_error`, but during ad-hoc debugging, always extract the full exception.
- Codex hit its daily ChatGPT-account usage limit at ~1:30 AM PDT after 39 successful turns + repair attempts (50 LLM calls). The error surfaces as `"You've hit your usage limit. Try again at X:XX AM."` — pipeline now records this as `codegen_error` and the loop's error budget catches it. Quota resets on a rolling basis; resume the loop later with the same command and it'll pick up at the pending entries.
- Test new model ids with a single one-line CLI call before changing the pipeline config: `codex exec --model gpt-X "say hi"`. Catches "model not supported" in seconds.

---

## 2026-06-05 — Fresh-run Phase 1 quarantined 70%+ of pages on string-vs-integer type mismatch

**Context:** running the fresh Phase 0 → Phase 1 → Phase 2 pipeline on a wiped repo, code-agnostic goal.

**What didn't work:** Phase 0's schema proposer typed every per-kind frontmatter property as `{"type": "string"}` (because wikitext tables present numbers in string-y context). The compile LLM correctly extracted numeric fields as integers (`hit_points: 125`, `build_time: 60`, `defense_life: 50`). Phase 1's validator then rejected every page that had any numeric field: 69 of ~97 pages quarantined with `"125 is not of type 'string'"` and the like. After a few minutes of running it was clear the pipeline was producing 28% usable output — unusable for any practical game.

**What worked instead:** added `_widen_property_type` in `scripts/validation.py` to widen any leaf scalar declaration (`type: "string"` / `"integer"` / `"number"` / `"boolean"`) to accept the full YAML scalar set when assembling the per-kind validation schema. The fix is **engine- and game-agnostic** by construction: it operates on the schema shape, not on field names or property semantics. Object and array shape constraints are preserved (so a property declared as `object` still has to arrive as a dict); only scalar typing is widened. The same code path runs for any game's `chosen_engine`.

**Why this is the right tactical fix, not a long-term one:** the real bug is in Phase 0's proposer prompt — it should emit `["string", "integer", "number"]` for numeric fields, not `"string"`. The validator widening is a permissive coercion that papers over noisy schema proposals. The 2026-05-19 decision said per-kind validation "enforces types as a sanity check" — that intent stands; widening just acknowledges that LLM-proposed types are noisier than hand-authored ones and the validator must tolerate that. When a stricter schema is genuinely needed (categorical enums, structured objects), the validator's `_widen_property_type` only acts on scalar types — those constraints survive.

**Note for next time:** if Phase 1 quarantine rate is >10%, look at quarantine `validation_errors:` blocks first. The pattern `"N is not of type 'string'"` is the proposer-over-types case and the validator fix handles it. If the quarantine reasons are about missing fields or shape (object/array), the universal schema's `required` list is the right knob, not the per-kind schema. The right long-term fix is in `scripts/phase0_analyze.py` `propose_frontmatter_schemas` — instruct the LLM to use union types for numeric fields. Until that lands, the lenient validator catches the drift downstream.

---

## 2026-06-04 — Backlog loop run: codex CLI init crash + low-confidence lore slug unrecoverable

**Context:** running `--from-vault --kinds infected,...,locations,organizations` with the default codex backend to close the backlog.

**What didn't work:**
1. **codex CLI bailed before generating** on 3 turns (the_great_crater, rebels, the_new_empire). Error: `failed to install system skills: io error w`. Not a model output issue — the CLI itself crashed during init, so the codegen.py `_dispatch` recorded `codegen_error` with no source. Re-running serially (c=1) reproduced exactly the same crash on each of the three. This is a codex 0.133.0 install-skill path issue, not a transient.
2. **Falling back to `--llm-mode claude` landed 2/3.** rebels and the_new_empire generated cleanly. the_great_crater still failed — this time on the `invalid_source_header` check. The dumped turn (`build/turns/the_great_crater.md`) shows the claude CLI produced a meta-narration ("I'm not receiving output from any read/shell tools…") instead of the `=== FILE: ===` contract. Same class as ERRORS 2026-05-23 ("free-form markdown output is unreliable"), this time triggered by a low-confidence (0.18) sparse vault note that gives the model nothing to anchor on.

**What worked instead:** for the 2 organizations slugs, swapping the dispatch mode (codex → claude) was enough. For the_great_crater, nothing worked across 5 attempts; recorded as `pending` with `blocked_by: invalid_source_header`. Decision: defer this slug rather than burn more turns.

**Note for next time:**
- If a slug fails codex with `failed to install system skills`, swap to `--llm-mode claude` rather than re-running codex.
- Slugs with frontmatter `confidence < 0.3` and a one-paragraph Description are weak codegen inputs. Either accept they may never compile cleanly, or skip them. They are not a pipeline bug.
- Don't burn >3 attempts on a single low-confidence lore slug. The system_map `pending` entry is the correct end state.

---

## 2026-05-22 — Phase 2 codegen first turn: Unicode + Claude Code preamble

**Didn't work (attempt 1):** `python phase2/codegen.py "implement the soldier unit"` crashed with `UnicodeEncodeError: 'charmap' codec can't encode character '→'` (the `→` arrow) when `print(result["response_text"])` hit Windows cp1252 stdout. Response was generated successfully but couldn't be displayed.

**Worked (attempt 2):** Added `--output PATH` flag + `sys.stdout.reconfigure(encoding="utf-8", errors="replace")` to `phase2/codegen.py:main`. With `--output build/turn1_soldier.md`, the response writes directly to a utf-8 file. Stdout fallback also fixed for any --output-less call.

**Note for next time:** any Phase 2 module that prints model output on Windows must either reconfigure stdout to utf-8 or write to a file. The cp1252 default eats common code-doc glyphs (→, ←, ✓, em-dash, smart quotes). Same rule applies if you ever wire a CI script that captures codegen output.

**Also surfaced:** `claude -p` invoked from inside `phase2/codegen.py` spawns a real Claude Code subagent that tries to use Write/Edit on the host filesystem, gets blocked, and prepends a "Permissions are blocking writes. Here is the complete implementation..." paragraph to its response. This is not a code bug; it's how Claude Code's print-mode falls back when sandboxed. The actual code is intact below the preamble. Do not strip the preamble blindly — `// Sources:` headers live inside the generated code blocks, not the preamble.


## 2026-05-23 — Phase 2 loop driver: `claude -p` codegen output is fragile

**What didn't work (took ~6 attempts on the ranger turn):**
1. Parsing whatever markdown `claude -p` emitted. The output structure is non-deterministic run-to-run: sometimes a plan-only summary ("grant write permissions and I'll write..."), sometimes `**`path`**` + fenced code, sometimes a valid `// Sources:` header with no parseable code blocks. turn1 (soldier) only worked by luck.
2. Letting the LLM regenerate aggregator files. Asked to implement the ranger, the model rewrote `src/units/mod.rs` as just `pub mod ranger; pub mod soldier;`, silently dropping the shared `pub struct Infected;` marker that `soldier.rs` imports → cargo `E0432` → revert. This would hit on every unit turn.
3. Reverting with `read_text`/`write_text`. The round-trip flipped CRLF↔LF on Windows and left files dirty in git even when content was identical.
4. `shutil.which("cargo")` alone. rustup adds `~/.cargo/bin` to PATH via the registry, but a shell that started before the install (or a subprocess with a stale PATH) does not see it → "cargo not found".

**What worked instead:**
1. Invoke `claude -p --tools ""` (disable all tools). Without it the CLI behaves like an interactive agent that tries to Write, gets sandbox-blocked, and asks for permission. With no tools it just emits text. Fix lives in `scripts/compile_cache.run_llm`, so Phase 1 benefits too.
2. An explicit machine-parsed output contract in the engine baseline: `=== FILE: <path> ===` / `=== END FILE ===`, raw content between, no fences. `loop_driver.parse_codegen_output` keys on that. Don't hope for consistent markdown — mandate a rigid delimiter.
3. Driver owns the module graph. The LLM produces only leaf modules; the driver skips any emitted aggregator (by basename) and appends `pub mod <stem>;` to the existing aggregator itself (idempotent, byte-preserving). Kept game-agnostic by driving it from `game-config.json -> chosen_engine.module_registration` data, not Python literals.
4. Byte-exact revert (`read_bytes`/`write_bytes`) and a cargo resolver that falls back to `~/.cargo/bin`.

**Note for next time:**
- Any `claude -p` used as a non-interactive transform MUST pass `--tools ""` and define a rigid output delimiter. Treat free-form markdown output as unreliable.
- Never let the codegen LLM own registry/barrel/aggregator files (mod.rs, lib.rs, __init__.py, index.ts). The tool manages those; the LLM writes leaves.
- On Windows, file round-trips through text mode flip line endings — use bytes for backup/restore.
- When a turn fails, the raw output is saved to `build/turns/<note_id>.md` (gitignored) — read that before re-running the LLM to diagnose.


## 2026-05-26 — From-scratch unit regen: first-of-kind fragility + 15-element tuple limit + Chroma transient

**Context:** regenerating all 9 units from scratch through the loop with the new sibling-exemplar mechanism (`--from-vault --kinds units --concurrency 2`).

**What didn't work / surprised us:**
1. **First-of-kind generated with no exemplar diverged and failed.** The vault walk is alphabetical, so caelus ran first with no sibling to follow. It auto-derived `Default` on a bundle holding `SimPosition` (no `Default`) → `E0277` → revert/pending. The sibling mechanism only helps turns 2..N; turn 1 sets the standard with nothing to anchor it.
2. **Bevy `Query`/`Bundle` tuples cap at 15 elements.** lucifer (18 components) and thanatos built a `*_checksum` system querying every component in one 17-element tuple → `E0277` "the trait `QueryData` is not implemented for (..17..)". This is systematic for complex units (determinism rule 7 folds all components), not codegen variance — a plain re-run fails the same way.
3. **Chroma transient on a parallel turn:** `codegen_error: Could not connect to tenant default_tenant`. One turn's retrieval failed to connect (embedder/Chroma race under `--concurrency 2`), recorded codegen_error pending though the code was never generated.

**What worked instead:**
1. **Build-error repair loop** (engine-agnostic): feed the build tool's error + failed source back through codegen, retry up to `--repair-attempts` (default 2). Fixed lucifer/thanatos/titan (the 15-tuple class) by letting rustc's own error drive the fix — no Bevy-specific rule. Idiomatic fix the model applied: nest sub-tuples (`Query<(&A, &B, (&C, &D, &E), ...)>`, counts as one element).
2. **Re-run the transient failure at `--concurrency 1`.** caelus landed on a solo re-run (it picked up calliope as the seeded exemplar AND avoided the Chroma race). Transient retrieval/Chroma errors are not code problems — just re-run, ideally serial.

**Note for next time:**
- For a clean from-scratch kind run, generate (or order) the best/most-complete unit FIRST so the standard it sets is good; everything follows it.
- Don't add a per-engine rule for each compile class — the repair loop generalizes. Add a rule only for something the compiler can't surface.
- `--concurrency > 1` can race the Chroma client at startup; if a turn dies with a tenant/connection error, re-run it serially.


## 2026-05-26 Loop re-generates already-done modules / system_map shrinks

**What didn't work:** Running a kind through the loop, then re-running it, showed the second run re-implementing ~30 modules that were already done and `build/system_map.yaml` ending with only ~13 entries instead of 50+. Looked like state loss.

**Root cause:** `system_map.cap_tokens` collapses the oldest `implemented` entries into a `{summary, count}` line to keep the YAML under 1,000 tokens, and `run_loop` was persisting that capped view as the canonical state. The same `implemented` list is the idempotency ledger, so collapsed ids stopped matching in `already_implemented` and got re-generated.

**What worked instead:** Decouple the durable ledger from the prompt projection. Persist the full ledger; apply the cap only when rendering the Layer-4 block (`system_map.render_for_prompt`). Recover a truncated ledger by rebuilding `implemented` from the on-disk vault-backed leaf modules (`game/src/<kind>/<slug>.rs` where `vault/<kind>/<slug>.md` exists), reading each file's `// Sources:` header for `verified_against`.

**Note for next time:** Any state that is both injected into a token-bounded prompt AND used as a durable ledger must be stored complete and projected/capped only at render time. Do not persist a lossy projection. Regression test: `LedgerNotTruncatedTests` in tests/test_phase2_system_map.py.
