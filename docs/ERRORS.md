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
