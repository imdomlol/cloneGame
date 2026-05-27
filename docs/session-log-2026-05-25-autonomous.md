# Autonomous session log — 2026-05-25

Dom away ~8 hours. Explicit yes for all work. **Do not push** (local commits only, for review).
Goal: finish the in-flight loop tasks, then harden each phase so it runs autonomously start-to-end with no errors, and lay infrastructure for upcoming steps.

This is a running log of changes + decisions, newest entries appended at the bottom. Final summary at the end.

---

## Starting state (from handoff)

- Last batch: `loop_driver.py --from-vault --kinds unit --limit 6` → {skipped: 3, implemented: 4, pending: 2}.
- Landed: calliope, mutant, thanatos, titan (in `game/src/units/`).
- Pending (cargo gate reverted byte-clean): lucifer (E0277, query iterated by value), sniper (E0277, marker in Default bundle lacks Default).
- `game/` compiles with 7 units.

## Plan for this session

1. [done-pending-verify] Diagnose the two E0277 failures.
2. Add a per-engine compile-correctness nudge (engine-agnostic infra via per-engine data file).
3. Re-run the two failures through the loop.
4. Flip "Loop driver C upgrade" to [x] in plan.md + todo.md.
5. Log the batch to MEMORY.md.
6. Harden each phase (0/1/2) for autonomous end-to-end runs; build infra for upcoming steps.

---

## Log

### 1. Diagnosed lucifer + sniper E0277 (decision: per-engine nudge)
- lucifer: `lucifer_death_explosion_system` iterated `for ... in lucifers` (owned Query with `&mut` access) instead of `&mut lucifers`. Isolated slip — same file does it right 6x. No rule added (re-run likely fixes).
- sniper: `SniperBundle` derives `#[derive(Bundle, Default)]` but field `sniper: Sniper` is a bare `#[derive(Component)]` marker with no `Default`. Structural, will recur on any Default-bundle unit. Rule added.
- Architecture decision (dom approved): compile-correctness guidance lives in the existing per-engine data file `prompts/engine_determinism/<engine>.md`, NOT in Python. Generalizes to every engine (each gets its own file). Relabeled the baseline template section `## Determinism rules` → `## Engine rules` so the file can hold determinism + correctness without mislabeling. No code path touched.
- Files: `prompts/engine_baseline.template.md` (header), `prompts/engine_determinism/bevy.md` (new rule 8). Baseline re-renders at 2653/2800 tokens, under cap.

### 2. Loop re-run result (lucifer + sniper)
- **sniper: IMPLEMENTED** — the rule-8 Default-bundle nudge worked. `game/src/units/sniper.rs` now compiles. 8 units total.
- **lucifer: FAILED again, but a NEW error.** codex regenerated lucifer from scratch with a totally different design; new failure is E0308 `const ATTACK_COOLDOWN_TICKS: u16 = SIM_HZ / 2;` (i32 expr assigned to a u16 const). The original by-value-Query bug is gone. This is codegen variance, not a recurring class — re-running lucifer once more (in progress) rather than adding a rule for a generic non-Bevy slip. Lucifer is the most complex unit (barrel, burning DoT, cone attack, fire-through-wall, death explosion), so it stresses the model hardest.

### 3. Found + fixed a real pipeline bug: stale duplicate vault dirs
- **Symptom:** vault has BOTH singular and plural kind dirs (`unit/`+`units/`, `building/`+`buildings/`, `mechanic/`+`game_mechanics/`, etc.). game-config kinds are all PLURAL. The singular dirs are stale 2026-05-19 leftovers (`type: unit`); the plural dirs are the canonical 2026-05-21 full ingest (`type: "units"`).
- **Consequence:** the handoff's `--from-vault --kinds unit` walked the STALE singular dir. It only worked because retrieval indexes the whole vault (both spellings), so codegen pulled current content and cited `vault/units/...`. Latent correctness/quality risk: doubled corpus, possible stale-path citations.
- **Root cause:** `vault_index.migrate_existing_note` did a COPY, not a move — it wrote the note into the new kind dir on a Phase 0 kind rename but never removed the original. So re-ingest accumulated both.
- **Fix (code, reviewable, reversible):** made migration a move — delete the original on successful migration and drop its stale `source_idx` entry, guarded so (a) a no-op same-path migration never deletes what it wrote, and (b) a migration that fails validation (lands in `_quarantine`) keeps the only good copy. Updated the existing test to the move semantic + added a quarantine-keeps-original test. `scripts/vault_index.py`, `tests/test_phase1_ingest.py`. All phase1 tests pass.
- **NOT done (deliberately):** did not delete the existing stale vault dirs. vault/ is gitignored regenerable Phase 1 output, but deleting data while dom is away when a non-destructive route exists (drive the loop with canonical plural `--kinds`, leave dirs for dom) is the conservative call per Default Behavior 11. **For dom:** the stale singular dirs can be safely removed (they are superseded). Suggested cleanup is listed in the final summary.

### 4. Phase 0 + Phase 1 autonomous-path verification (PASS)
- **No interactive prompts anywhere in `scripts/`** (grep for input/getpass/confirm = none). Both phases run unattended.
- **Phase 0:** `phase0_write.write_proposal` auto-promotes (writes game-config.json directly with `human_approved: true`); the old diff-review gate is gone. `--dry-run` prints proposal and skips write. Did NOT re-run live — that would rewrite the approved reference config and spend LLM calls for no benefit; the 2026-05-21 reference run stands. Covered by `tests/test_phase0.py`.
- **Phase 1:** ran `python scripts/phase1_ingest.py --dry-run` live → exit 0, enumerated all 17 categories (135 mainspace pages) mapped to the canonical PLURAL kinds. Confirms (a) Phase 1 is non-interactive and autonomous, (b) the canonical kinds are plural, so the singular vault dirs are definitively stale. Did NOT do a full live re-ingest (135 pages of wiki+codex) — already complete from 2026-05-21 and cache would short-circuit anyway.
- Conclusion: Phases 0 and 1 already satisfy "autonomously completable start to end." The remaining autonomous-completion frontier is Phase 2 (the codegen loop).

### 5. Lucifer landed — Phase 2 units 9/9 COMPLETE
- Re-ran lucifer alone; fresh codex pass compiled. `game/src/units/lucifer.rs` landed via the loop (codegen → parse → merge → register `pub mod` → cargo build → record).
- Full `cargo build` of `game/` with all 9 units: **exit 0, clean.**
- `system_map.yaml`: implemented=9, pending=[], test_state all zero.
- Units: caelus, calliope, lucifer, mutant, ranger, sniper, soldier, thanatos, titan.
- **Phase 2 codegen loop is now autonomously completable for the `units` kind start-to-end with no errors**, which was the core ask. Took 2 codegen passes for lucifer (each a different design, different error) — expected codegen variance; the cargo gate caught both bad passes and reverted byte-clean.

### Codegen-quality finding (for dom, design call — NOT changed unattended)
The 9 generated units have **structurally inconsistent health/stats APIs**: soldier uses a shared `SimHealth` (I32F32); ranger defines its own `RangerHealth` with `max: u32`=60; sniper defines `SniperHealth`. Each turn generates in isolation with no shared component contract, so they diverge. Consequences: (a) a uniform cross-unit cargo smoke test is impractical (would couple to 9 different APIs and break on any regen); (b) systems can't share health logic. Recommendation (your call): add a shared `Health`/`UnitStats` contract to the engine baseline so units stop reinventing it. This changes the generated-code architecture, so I left it for you rather than imposing it. The todo's "smoke test" item is blocked on this decision — a brittle per-unit test now would be net-negative.

### 6. Infra for upcoming steps: crate-root module wiring (the blocker for non-unit kinds)
- **Gap found:** `register_modules` only wired a leaf into its OWN dir's `mod.rs`. For a brand-new kind like `buildings`, it created `src/buildings/mod.rs` but nothing added `pub mod buildings;` to `src/lib.rs`. Result: the module is orphaned, cargo build PASSES anyway (the dir is simply not part of the crate), so the gate gives a **false green** while the building code is never compiled. This blocked scaling the loop to every kind except `units` (which only worked because `lib.rs` had a hand-written `pub mod units;` from the regression anchor). Also: the LLM-emitted `lib.rs` was NOT protected, so a turn could clobber the hand-built crate root.
- **Fix (data-driven, engine-agnostic):**
  - Added optional `crate_root: "src/lib.rs"` to `chosen_engine.module_registration` in game-config.json.
  - `register_modules` now walks the full chain leaf → crate root: wires `pub mod wood_gate;` into `src/buildings/mod.rs` AND `pub mod buildings;` into `src/lib.rs`. Also fixes the old "known limit" (a leaf like `src/sim.rs` directly under the root now registers in `lib.rs`, not a bogus `src/mod.rs`).
  - `merge_into_game` now also skips an LLM-emitted crate-root file (`lib.rs`), same protection `mod.rs` already had.
  - Backward compatible: engines with no `crate_root` get the exact prior behavior (opt-in).
  - Added `crate_root_basename` helper + `_registration_nodes`.
- **Tests:** +5 (crate-root wires new kind into lib, leaf-under-src→lib, idempotent existing kind, lib.rs clobber-skip). Full gate green: ruff/format/vulture clean, **171 tests pass** (was 166).
- **Live proof in progress:** `--from-vault --kinds buildings --limit 1` to show a new kind dir wires into lib.rs and compiles for real.

### 7. Infra: `--from-vault` ignores non-kind directories
- Added `load_valid_kinds(game-config.json)` + a `valid_kinds` filter in `derive_goals_from_vault`. A `--from-vault` walk now skips any directory that is not a real kind in game-config. This makes a bare `python phase2/loop_driver.py --from-vault` (all kinds, hands-off) safe even while the stale singular dirs still exist — they are not in game-config `kinds`, so they produce no goals. Tests +3. Combined with the migration move-fix (§3), the stale-dir trap that bit the handoff's `--kinds unit` is now closed from both directions.
- Full gate after all loop_driver changes: ruff/format/vulture clean, **174 tests pass**.

### 8. Live cross-kind proof: PASS
- `--from-vault --kinds buildings --limit 1` → `advanced_farm` IMPLEMENTED. `lib.rs` now reads `pub mod sim; pub mod units; pub mod buildings;`; `src/buildings/mod.rs` = `pub mod advanced_farm;`. The LLM emitted both `src/lib.rs` and `src/buildings/mod.rs`; the driver SKIPPED both (clobber protection confirmed: log shows `skipped: src/lib.rs,src/buildings/mod.rs`).
- Final full `cargo build` (9 units + 1 building): exit 0, clean. The building is genuinely compiled (in-crate), not orphaned.

---

## FINAL SUMMARY (session 2026-05-25, autonomous)

**Bottom line:** all three phases are autonomously completable start-to-end with no errors. Phase 2's `units` kind is fully built (9/9) and the loop now scales to any new kind (proven with 1 building). Everything is gate-green. **Nothing committed, nothing pushed** — left as a clean working-tree diff for dom to review (per CLAUDE.md "commit only when the user asks").

**Gate at session end:** ruff clean · ruff format clean (31 files) · vulture clean · 174 unittests pass (was 166) · `cargo build` exit 0 (9 units + 1 building).

**Files changed (all local, uncommitted):**
- `prompts/engine_baseline.template.md` — section header `## Determinism rules` → `## Engine rules`.
- `prompts/engine_determinism/bevy.md` — added rule 8 (marker components in a `#[derive(Bundle, Default)]` must derive Default). Fixed the sniper failure.
- `game-config.json` — added `chosen_engine.module_registration.crate_root = "src/lib.rs"`.
- `phase2/loop_driver.py` — crate-root-aware `register_modules` (wires leaf→crate root), `crate_root_basename` + `_registration_nodes` helpers, `merge_into_game` skips the crate-root file, `load_valid_kinds` + `valid_kinds` filter in `derive_goals_from_vault`.
- `scripts/vault_index.py` — `migrate_existing_note` is now a move (deletes original + drops stale index entry, guarded).
- `tests/test_phase1_ingest.py` — migration move + quarantine-keeps-original tests.
- `tests/test_phase2_loop_driver.py` — +8 tests (crate-root wiring, lib.rs clobber-skip, valid_kinds).
- `game/src/lib.rs`, `game/src/units/mod.rs`, `game/src/buildings/` (new), + 6 new unit `.rs` files + `advanced_farm.rs` — generated by the loop, all compile.

**Decisions for dom (left untouched on purpose):**
1. **Stale singular vault dirs** (`unit/`, `building/`, `mechanic/`, etc.) are superseded leftovers. Safe to delete; I did not (gitignored regenerable data, you were away). Cleanup when ready:
   `Get-ChildItem vault -Directory | Where-Object { $_.Name -notin (python -c "import json;print(' '.join(json.load(open('game-config.json'))['kinds']))").Split() -and $_.Name -ne '_quarantine' } | Remove-Item -Recurse`
   (verify the list first). The migration + valid_kinds fixes mean this won't recur and won't break runs in the meantime.
2. **Inconsistent unit health/stats APIs** — each unit reinvents its own `*Health`. Recommend a shared `Health`/`UnitStats` contract in the engine baseline. This changes generated-code architecture, so it's your call. The "smoke test" todo is blocked on it.
3. **The `advanced_farm` building** is real proof-of-capability code. Keep it or drop it (it's one isolated module + its wiring).

**Next-session priorities:**
- Decide the shared-health contract (unblocks the smoke test + cleaner buildings).
- If buildings codegen quality looks good, run `--from-vault --kinds buildings` for the full 42 (and then `infected`, etc.). The loop, gate, revert, and crate-root wiring all carry through.
- Optional: incremental `system_map` save (currently saved once at loop end; a crash mid-long-run loses progress).

### 9. Stale vault dirs deleted (dom confirmed) + index rebuilt
- Deleted 11 stale singular dirs (building, campaign_map, character, infected_runner, infected_walker, location, mechanic, organization, survival_map, unit, wonder). vault/ now holds only the 17 canonical kinds + _quarantine.
- Regenerated `build/repomix-output.xml` and rebuilt the Chroma index (`phase2/indexer.py`): 112 unique notes, graph.json 112 nodes/504 edges. Confirmed no `vault/<singular>/` paths remain in the snapshot — retrieval can no longer cite a deleted path.
- Note (pre-existing, not acted on): some pages belong to multiple categories and were ingested under multiple kinds (e.g. `caelus` in units/ + characters/ + campaign_content/). Same id across kinds → indexer dedupes by id. Orthogonal to the stale-dir issue; flagging only.

### 10. Shared Health/UnitStats contract (dom approved lay + regen)
- **Veteran finding (reported to dom):** generated code is faithful to the vault, not buggy. Vault notes describe veteran promotion for soldier/ranger/sniper only, and the code implements it for exactly those three. The other six (caelus, calliope, lucifer, mutant, thanatos, titan) have no veteran mention in their notes, so the code has none. If lucifer/thanatos/titan should be promotable (likely in TAB), it's a Phase 1 / wiki data-completeness gap (those pages don't document it or extraction dropped it), NOT a codegen bug. Did not fabricate veteran data. → veteran therefore stays a per-unit component, not part of the shared contract.
- **Architecture answer:** broad principle (template, engine-agnostic) + engine-specific realization (foundation code + per-engine determinism file). Same split as the compile rules. No Python/driver change.
- **Implemented:**
  - `game/src/sim.rs`: added `Health { current, max: I32F32 }` (+ `Health::full(max)`) and `UnitStats { move_speed, attack_range, attack_damage, attack_speed, watch_range }`. cargo build clean.
  - `engine_baseline.template.md`: generic "reuse shared foundation types, never redefine per unit" rule (Content rules).
  - `bevy.md`: rule 9 naming `crate::sim::Health`/`UnitStats`, forbidding per-unit `*Health`/`*Stats`, extras stay separate.
  - Baseline now renders **2800/2800** (had to trim rule 9 + the template bullet + drop a redundant checksum clause to fit). NOTE: baseline is now AT the cap — any future addition must trim or move detail into retrieval.
- **Regen in progress:** cleared the 9 unit entries from system_map (backup at `build/system_map.yaml.bak`, kept advanced_farm), running `--from-vault --kinds units --error-budget 9`. Each unit's old working file is gate-protected (reverts on failure).

### Veteran: closed (dom 2026-05-25)
Dom confirmed the wiki documents veteran/promotion only for soldier, ranger, sniper — no data for the other six anywhere on the wiki. So the vault and the generated code are CORRECT as-is. Not a pipeline gap; no re-ingest needed. Veteran stays a per-unit component (soldier/ranger/sniper only).

### 11. Parallel codegen mode (dom approved: parallel codegen only, no model-race)
- **Design:** split each turn into phase A (`_prepare_turn`: codegen + validate, pure, no state/game writes) and phase B (`_commit_turn`: merge + register + cargo gate + record, serial). `run_loop` gained `concurrency` (CLI `--concurrency N`, default 1). With N>1 it prepares upcoming turns in lookahead batches of N concurrently (ThreadPool over the LLM subprocess call), then commits each serially in goal order.
- **Why this shape:** the cargo gate is inherently serial (one shared `game/` tree + cargo target — can't attribute/revert concurrent builds). Codegen is embarrassingly parallel (no shared state; `retrieval.query_collections` builds its own Chroma client + embedder per call, so threading is safe — each worker just loads its own embedder, so concurrency is bounded by memory not correctness). So we overlap only the LLM wait.
- **Invariants preserved:** default `concurrency=1` is byte-identical to the old loop (results order, error budget, max_turns). Determinism: commit runs in sorted goal order regardless of N, so `system_map` order is stable. Error-budget waste is bounded to one lookahead batch (verified by test).
- **Tests +4** (parallel==serial results, order-with-skips, budget-bounds-wasted-calls). Removed `_process_goal` (replaced by prepare/commit). ruff/format/vulture clean, loop_driver suite 50 tests pass.
- **Biggest payoff:** the upcoming buildings ×42 / infected runs, where sequential LLM waits dominate. Usage e.g. `python phase2/loop_driver.py --from-vault --kinds buildings --concurrency 4`.

### 12. Regen result: 8/9 adopted the shared contract on first pass
- soldier, ranger, sniper, calliope, lucifer, mutant, thanatos, titan: all regenerated using `crate::sim::Health` + `UnitStats`, none define their own `*Health` struct. The per-unit health/stats divergence is eliminated for these 8.
- **caelus FAILED:** put the shared `Health` in a `#[derive(Bundle, Default)]`, but `Health`/`UnitStats` intentionally have NO `Default` (no sensible default HP — must be constructed via `Health::full(max)`). Same class as rule 8, now hitting a foundation type. 8/9 constructed explicitly, so it's codegen variance, not systematic — re-running caelus (gate-protected; reverted to its prior working self-defined-health version meanwhile).
- Decision: did NOT add new prompt guidance for this — baseline is at the 2800 cap, and 1/9 variance that a re-run fixes doesn't justify trimming. WATCH-ITEM: if the buildings/infected runs show "Health in Default bundle" recurring, add a rule 8 clause (and trim to fit) or give Health a guarded Default. Not now.

### 13. Shared contract migration COMPLETE
- caelus re-ran and adopted the shared types on the 2nd pass. All 9 units now use `crate::sim::Health` + `UnitStats`; **0 self-defined `*Health` structs remain**. The per-unit divergence is fully resolved.
- system_map: 10 implemented (9 units + advanced_farm), 0 pending.
- Final verification: `cargo build` exit 0 (9 shared-contract units + 1 building); ruff/format/vulture clean; **177 tests pass**.
- Note: this session's regen used the sequential loop (it was launched before the parallel-codegen refactor landed; the refactor only affects future runs). Parallel mode is tested and ready for the next big run.
