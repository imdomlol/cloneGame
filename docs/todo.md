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
- [~] **Scale content** — `buildings` (42/42), `game_mechanics` (12 modules incl.
      cross-deps), and `wonders` (5/6) all done as of 2026-06-04. 84 modules
      total in `system_map.yaml`, `cargo build` clean.
- [x] **Finish infected kinds** ✅ 2026-06-04 — turned out the system_map had
      already absorbed all infected* + wonders during the prior buildings/wonders
      run (per MEMORY 2026-05-26). The backlog loop run SKIPped 37 already-done
      slugs; no new infected modules were generated this session.
- [x] **Finish the missing wonder** ✅ 2026-06-04 — already in system_map; was
      a stale count in the 2026-06-04 status table. All 6 wonders present.
- [~] **Decide on data-shaped kinds** — campaign_*/survival_maps/updates
      deferred as runtime asset-loader targets, not code (see MEMORY 2026-06-04).
      research / characters / mayors / locations / organizations sent through
      the loop: 2 organizations landed (rebels, the_new_empire). `the_great_crater`
      remains pending (5 attempts; logged in ERRORS).
- [x] **Runnable game** ✅ 2026-06-04 — `game/src/main.rs` is a visual Bevy App
      with `DefaultPlugins`, a `Camera2d`, every generated kind plugin registered,
      one of each unit type spawned, `Time::<Fixed>::from_hz(25.0)` driving
      `FixedUpdate`, and a periodic checksum log.
- [x] **Cleanup warnings** ✅ 2026-06-04 — attempted loop re-run per Q3
      preference; codex CLI crashed on all 3 (`failed to install system skills`)
      and claude mode failed `invalid_source_header` on all 3. Fell back to
      hand-edits per CLAUDE.md "simplest solution first". `swarms.rs` and
      `wood_workshop.rs` had `mut` bindings removed; `WOOD_WALL_HP` deleted
      from `stone_wall.rs`. `cargo build` clean, zero warnings. Deviation
      from Q3 logged in ERRORS 2026-06-04.
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

## 2026-06-09 deferred

- [!] **#5 Wire generated systems into `main.rs` so the game does more than
      tick.** `InputHandlerPlugin` is registered today but `main.rs` doesn't
      spawn an entity for it to control. **Attempted 2026-06-11 and reverted:**
      a player entity + camera-follow was added to the engine scaffold
      (`prompts/engine_scaffold/bevy/src/main.rs` + `game/src/main.rs`), but a
      player avatar baked into the *game-agnostic* scaffold is wrong — it would
      appear in every Bevy game (an RTS like They Are Billions has no single
      controllable character) and assumes a perspective that may not fit.
      Player representation and camera perspective are **per-game**, not
      scaffold. Superseded by the perspective TODO below.

## Game-specific perspective + 2D/3D world (deferred 2026-06-11, for a future model)

- [ ] **Phase 0 proposes a `perspective`, and the pipeline builds a 2D or 3D
      world from it.** Decision (dom, 2026-06-11): perspective is game-specific
      data, expressed via **config + a generated system**.

      **The gap this addresses:** the engine scaffold currently bakes in a 2D
      top-down worldview for *every* Bevy game, as an unstated assumption — not
      a choice expressed anywhere:
      - `spawn_camera` hardcodes `Camera2d`.
      - the demo sprites are 2D `Sprite`s.
      - `SimPosition { x: I32F32, y: I32F32 }` in `game/src/sim.rs` is **2D**
        (no `z`), and the `SimPosition → Transform` mirror is 2D.
      This suits an RTS (They Are Billions) and is wrong for a first-person
      game (Lethal Company, the current target, is genuinely first-person 3D).

      **Target design (config + generated system):**
      1. Phase 0's analyzer proposes a `perspective` for the game (e.g.
         `first_person_3d`, `top_down_2d`, `side_2d`) and writes it to
         `game-config.json` (likely `chosen_engine.perspective`). Game-agnostic
         data, same as the engine choice itself.
      2. The scaffold stops hardcoding `Camera2d` / 2D spawns. Camera + player
         become a **generated** per-game system (Phase 3
         `propose_gameplay_systems`), so the controller behavior is per-game,
         not in the shared scaffold.
      3. The world is built 2D **or** 3D based on the proposed perspective.

      **Foundation implication (this is the large part — read before starting):**
      A real 3D / first-person world requires `SimPosition` to become **3D**
      (`x, y, z`), all in fixed-point per the determinism rules. `SimPosition`
      is the position component every simulated entity uses, so this is a
      change to `game/src/sim.rs` (the determinism foundation) that ripples
      through the determinism rules in `prompts/engine_determinism/bevy.md`
      **and all ~90 generated entity modules** that read/write position. It is
      a deliberate foundation change, not a localized `main.rs` edit. A
      "first-person camera over the 2D sim" was explicitly rejected as a veneer
      (camera looks FP, but the world stays a flat plane of billboards with no
      verticality — doesn't deliver first-person gameplay).

      **Scope note:** the cheap half (perspective field + de-hardcoding the
      scaffold camera) is doable with no LLM cost; the 2D↔3D world build +
      generated controller is the foundation work. Plan the dimensionality
      decision (per perspective value) before touching `sim.rs`.

## Fresh-run follow-ups (2026-06-06)

- [x] **Phase 2 loop** ✅ 2026-06-07. 110 implemented, 2 stale pending (both `codegen: false` kinds). cargo build + smoke + run all green.
- [ ] **Disk drift cleanup**: codex randomly produced `src/buildings/` and `src/building/` for the same kind. Both compile; aggregator handles them. Future iteration: tell the codegen prompt to use the configured kind name as the dir.
- [ ] **Stale pending cleanup**: drop pending entries whose kinds are now `codegen: false`. Cosmetic; doesn't affect future runs.
- [x] **Retrieval pin priority** ✅ 2026-06-07. Verified working by direct tracing; earlier suspicion was wrong. Real failure was codex quota; the Academy of Immortals content was a graph neighbour after the pin, not before.
- [x] **`codegen: false` on data-shaped kinds** ✅ 2026-06-07. `campaign_map`, `campaign_content`, `survival_map`, `update_log` are now excluded from `--from-vault` walks via `phase2/loop_driver.load_valid_kinds` reading the per-kind flag. Game-agnostic config knob.
- [x] **Phase 0 schema proposer union types** ✅ 2026-06-07. `_build_schema_prompt` now teaches the LLM to emit unions for numeric fields and structural types for lists / maps. Validator widening kept as a safety net for older configs.
- [ ] **`the_great_crater` still pending across runs**: lore-only single page at confidence 0.18. Either accept as permanently pending or mark `location.codegen = false` (would also drop other locations if added). One slug; not worth a kind-wide flag.

## Phase 2 — D follow-on (active)

- [x] **App-aggregator generator** ✅ 2026-06-05. `phase2/entrypoint.py` produces
      `game/src/app_plugins.rs` from every `impl Plugin for X` under `src/`,
      chunked to ≤14 per `add_plugins` tuple. `main.rs` and `app_smoke.rs` call
      `app_plugins::add_all` instead of hand-enumerating; smoke gate now covers
      new leaves automatically. Per-engine data in `chosen_engine.entrypoint`.
- [x] **Smoke test exercises `FixedUpdate`** ✅ 2026-06-05. `app.update()` alone
      does not always tick FixedUpdate in Bevy 0.15; smoke now calls
      `world_mut().run_schedule(FixedUpdate)` twice so B0001 conflicts in
      FixedUpdate systems actually fire.
- [x] **`excluded_plugins` knob** ✅ 2026-06-05. Lets known-broken plugins (e.g.
      `AcademyOfImmortalsPlugin` B0001) stay on disk for the repair loop while
      the binary and smoke gate run. Each entry carries a reason string.
- [ ] **Regenerate AcademyOfImmortalsPlugin through the loop** so the smoke
      gate accepts it and the exclusion can be removed. Same path applies to any
      future plugin that lands in the exclusion list.
- [ ] **LLM-authored `main.rs` codegen phase** (the second half of D). The
      aggregator covers plugin enumeration; `main.rs` window/camera/spawn logic
      is still hand-authored. A future codegen step gated on `main.rs` absence
      would auto-generate the entrypoint for a fresh game. Defer until a second
      target game tests the path.

## Open questions to resolve

- [x] **Embedding provider** for Phase 2 Chroma index — local embedder ✅ 2026-05-21
- [ ] **Schema-proposal sampling size** — default is 2 pages per category. Tune after first multi-game runs.

## Deferred / low priority

- [-] **Concurrency in Phase 1 ingest** (deferred 2026-05-21) — `phase1.config.toml` declares `concurrency = 4` but `phase1_ingest.py` runs sequentially. Doesn't block downstream work. Revisit if bulk-ingest wait time becomes a real friction.
