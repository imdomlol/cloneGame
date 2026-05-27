# TODO

Active, near-term work. See `cloneGame/plan.md` for the full phase status
and `cloneGame/MEMORY.md` for decision history.

Status keys: `[ ]` not started · `[~]` in progress · `[!]` blocked · `[-]` deferred / low priority

---

## Phase 2 — active

- [x] All four modules scaffolded with unit tests ✅ 2026-05-22
- [x] Spend the first real codegen turn ✅ 2026-05-22 (soldier, via `claude -p`
      Sonnet 4.6; `build/turn1_soldier.md`).
- [x] Single-shot driver ✅ 2026-05-22 (`phase2/driver.py`).
- [x] Update `docs/CLAUDE.md` and `docs/DEPLOYMENT_GUIDE.md` ✅ 2026-05-22.
- [x] **Cargo regression anchor** ✅ 2026-05-22. soldier extracted into `game/`,
      `cargo build` clean, Cargo.lock committed.
- [x] **Loop driver (B.parse)** ✅ 2026-05-22 (`phase2/loop_driver.py`).
- [x] **Codegen reliability hardening** ✅ 2026-05-23. The first multi-file
      loop turn (ranger) exposed and fixed four bugs — see
      `cloneGame/ERRORS.md`:
      - `claude -p --tools ""` stops plan-only / permission-asking output.
      - Explicit `=== FILE: ===` / `=== END FILE ===` output contract in the
        engine baseline; parser keys on it instead of guessing markdown.
      - Driver owns the module graph (data-driven `module_registration` in
        `chosen_engine`): LLM writes leaf modules, driver appends
        `pub mod <stem>;` to the aggregator, so shared declarations survive.
      - Byte-exact reverts (no CRLF/LF git noise); cargo resolves via
        `~/.cargo/bin` fallback; failed turns dump to `build/turns/<id>.md`.
- [x] **Second sim-path file** ✅ 2026-05-23. `game/src/units/ranger.rs`
      generated fully through the loop (codegen → parse → merge → register →
      cargo build → record). Compiles alongside the soldier. HP=60,
      fixed-point, sources header citing ranger + soldiers_center + soldier.

## Phase 2 — Bevy + lockstep follow-on (active)

- [x] **Loop driver C upgrade** ✅ 2026-05-25. Walk `vault/<kind>/*.md` to
      auto-derive goals, skip slugs already in `system_map.implemented`, run
      hands-off. Per-kind goal templates (`"implement the {slug} {kind}"`).
      End state: one command grows `game/` from two modules toward
      one-module-per-vault-note. Layered on B; B's revert + cargo gate +
      module registration carry through. First volume run drove all 9 `units`
      to a clean build (4 then 2 then sniper, lucifer on a 2nd pass).
- [x] **Crate-root module wiring** ✅ 2026-05-25. `register_modules` wires a leaf's
      full chain up to the crate root via `module_registration.crate_root`, so a new
      kind dir is declared in `lib.rs` instead of being silently orphaned. Proven by
      generating `advanced_farm` (first building).
- [x] **`--from-vault` kind filter** ✅ 2026-05-25. Walk skips dirs that aren't real
      game-config kinds (stale-dir trap closed; stale singular dirs also deleted).
- [x] **Shared `Health`/`UnitStats` contract** ✅ 2026-05-25. Foundation types in
      `sim.rs`; all 9 units regenerated to use them (no self-defined `*Health`).
- [x] **Parallel codegen** ✅ 2026-05-25. `--concurrency N` overlaps the codegen
      phase in lookahead batches; cargo gate stays serial; `N=1` byte-identical.
- [x] **Sibling-exemplar consistency** ✅ 2026-05-26. First accepted module of a kind
      is fed into every later same-kind codegen turn as a `[REFERENCE SIBLING MODULE]`
      pattern (match its public surface, adapt names/values). Lives in
      `codegen.build_user_message` (game/engine-agnostic, no name mandate); resolved
      per kind in `run_loop` (pre-seed from implemented + in-run capture);
      `_chunk_end` keeps it correct under `--concurrency`. Fixes per-turn API drift.
- [x] **Build-error repair loop** ✅ 2026-05-26. On a build failure the driver feeds
      the build tool's error + failed source back through codegen (`--repair-attempts`,
      default 2) before pending. Engine-agnostic (swap the build runner per engine);
      replaces adding a per-engine rule for each compile class. Fixed the Bevy
      15-element `Query`/`Bundle` tuple overflow on complex units (lucifer/thanatos/titan).
- [x] **Smoke test** ✅ 2026-05-26. `game/tests/unit_health.rs` asserts
      `<Unit>Bundle::default().health.max == vault HP` for all 9 units (caelus 120,
      calliope 60, lucifer 500, mutant 2000, ranger 60, sniper 150, soldier 120,
      thanatos 250, titan 800). `cargo test` green.
- [x] **Units regenerated from scratch, consistent** ✅ 2026-05-26. All 9 deleted and
      rebuilt through the loop; converged on one shape (`const <UNIT>_HP` +
      `#[derive(Bundle)] <Unit>Bundle` + hand-written `Default` → `Health::full`).
      `cargo build` clean. NB: the alphabetical first-of-kind (caelus) generates with
      no exemplar and is the fragile one (see MEMORY/ERRORS 2026-05-26).
- [ ] **Scale content (NEXT)** — run remaining kinds through the loop:
      `--from-vault --kinds buildings --concurrency 2 --repair-attempts 2` (42), then
      `infected`, etc. The sibling-exemplar + repair loop should make these land far
      more cleanly than the first unit run did. Baseline is at the 2800 token cap.
- [ ] **Runnable game** — `game/` is still a library. A `main.rs` Bevy `App` that
      wires the unit plugins, spawns entities, and ticks the sim is the step toward
      the actual playable clone.
- [ ] **First SDK turn** — only when `ANTHROPIC_API_KEY` is available.
      Validates the `cache_control: ephemeral` math from DEPLOYMENT_GUIDE §4.
      Until then `claude` CLI mode (with `--tools ""`) is production.

## Phase 1 — bulk ingest results (closed)

- [x] Phase 0 v2 re-run lands and `game-config.json` carries `frontmatter_schema` for all 8 kinds ✅ 2026-05-21
- [x] Authenticate `claude` or `codex` CLI locally ✅ 2026-05-21
- [x] `python scripts/phase1_ingest.py --dry-run` — verify page counts per category ✅ 2026-05-21
- [x] `python scripts/phase1_ingest.py` — full ingest ✅ 2026-05-21
- [x] Inspect `vault/_quarantine/` and iterate on the compile prompt or `frontmatter_schema` ✅ 2026-05-21
- [x] Split `scripts/phase1_ingest.py` into sibling modules ✅ 2026-05-21

## Phase 0 v3 — taxonomy coverage gate (closed)

- [x] Coverage gate + drop_reason discipline ✅ 2026-05-22
- [x] Mainspace-only member enumeration ✅ 2026-05-22
- [x] Phase 1 canonical_kind ✅ 2026-05-22

## Open questions to resolve

- [x] **Embedding provider** for Phase 2 Chroma index — local embedder ✅ 2026-05-21
- [ ] **Schema-proposal sampling size** — default is 2 pages per category. Tune after first multi-game runs.

## Deferred / low priority

- [-] **Concurrency in Phase 1 ingest** (deferred 2026-05-21) — `phase1.config.toml` declares `concurrency = 4` but `phase1_ingest.py` runs sequentially. Doesn't block downstream work. Revisit if bulk-ingest wait time becomes a real friction.
