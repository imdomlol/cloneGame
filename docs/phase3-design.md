# Phase 3 design — gameplay-system codegen

Phase 2 produces per-entity plugins (one soldier, one farm, one scrap item) by
walking the vault. Phase 3 produces **systems** that compose entities into a
gameplay loop (wave spawner, input handler, win condition, HUD, ship departure
timer) by walking a curated list. The two phases use the same codegen
machinery; what differs is where the goal list comes from and what's in the
retrieval bundle per turn.

## Why a new phase

Wikis describe entities and mechanics individually. They don't describe the
**glue** that turns "the soldier shoots at 16 damage" + "the infected dies at
70 HP" into "soldiers attack infected when within watch range and the round
ends when all infected die". That glue is the game; the leaves are the
vocabulary. Without a phase that generates the glue, the crate compiles and
runs but you see colored sprites that don't react to anything.

## What gets generated

Each "system" is a Bevy plugin that:

- Pulls components from one or more entity kinds (soldiers + infected) and
  drives them with shared logic (find nearest target in watch_range, deal
  attack_damage per attack_speed).
- Owns a small piece of global game state (current wave, timer, score) as a
  Bevy `Resource`.
- Reads input or game state to fire actions on entities it doesn't own.

Examples per game:

| Lethal Company systems | They Are Billions systems |
|---|---|
| `ship_departure_timer` | `wave_spawner` |
| `quota_tracker` | `colony_economy` |
| `scrap_collector` | `combat_resolver` |
| `creature_aggro` | `noise_propagation` |
| `flashlight_input` | `wall_repair_system` |
| `apparatus_event` | `wonder_objective_tracker` |
| `game_state_machine` | `game_state_machine` |
| `hud` | `hud` |
| `input_handler` | `input_handler` |

Some are universal (state machine, HUD, input handler); most are per-game.

## Data shape: `chosen_engine.systems`

A new list in `game-config.json` proposed by Phase 0:

```json
"systems": [
  {
    "name": "wave_spawner",
    "description": "spawns infected waves on timer; difficulty ramps per day",
    "depends_on": ["infected", "game_mechanic"],
    "produces": ["WaveTimer", "InfectedSpawnEvent"]
  },
  {
    "name": "combat_resolver",
    "description": "every unit with UnitStats shoots the nearest hostile in range",
    "depends_on": ["unit", "infected"],
    "produces": []
  }
]
```

- `depends_on`: list of vault `kind` names this system reads from. Phase 3
  retrieval pins those kinds' notes into the codegen bundle.
- `produces`: optional list of new Bevy `Resource`s / `Event`s the system
  introduces. The aggregator picks them up via the existing pattern.
- `description`: one-line behavior statement, expanded by codegen.

## Where it goes on disk

`game/src/systems/<system_name>.rs` — one plugin per system. The driver's
existing `module_registration` mechanism declares `pub mod systems;` in
`lib.rs` and `pub mod wave_spawner;` in `systems/mod.rs`. The aggregator
(`app_plugins::add_all`) picks up `WaveSpawnerPlugin` automatically because
it greps every `impl Plugin for X` under `src/` — no special path needed.

## Pipeline integration

### Phase 0 changes

1. `scripts/phase0_analyze._build_systems_prompt` — proposes the systems
   list given the kinds, sample wikitext, and the engine's role.
2. `scripts/phase0_analyze.propose_gameplay_systems` — runs the LLM call,
   validates the output shape.
3. `scripts/phase0.py` — calls it after schemas/engines/codegen-flags,
   writes `chosen_engine.systems` into game-config.json.

### Phase 2 (no changes)

Phase 2 still walks `vault/<kind>/*.md` as before. Codegen for entity leaves
is unchanged. Systems are NOT generated here.

### Phase 3: new CLI surface

`python phase2/loop_driver.py --from-systems` (or a separate
`python phase3.py` that calls the same `run_loop`):

1. Reads `chosen_engine.systems` from `game-config.json`.
2. For each system, builds a goal: `"implement the <name> system: <description>"`.
3. For each turn, retrieval pins the vault notes for every `depends_on` kind
   (instead of one note per `pin_id`). Falls back to vector search beyond
   the pins.
4. The codegen prompt includes a new `[IMPLEMENTED ENTITIES]` block —
   the full `system_map.implemented` list with each entity's public API
   (struct names, components, events), pulled from disk. So the system
   plugin can reference the leaves correctly.
5. Goes through the same cargo build + smoke gate + repair loop as Phase 2.

### Per-engine determinism additions

`prompts/engine_determinism/bevy.md` gains a `## System rules` section:

- Game state lives in `Resource`s, not random globals.
- State transitions go through Bevy's `States` machinery for cleanliness.
- Input handling uses `Input<KeyCode>` / `MouseButton` (deterministic on a
  given client; not part of the lockstep sim).
- HUD uses `bevy_egui` or `bevy_ui`; not part of the deterministic sim path
  either.
- Cross-entity systems (combat, AI) MUST tick in `FixedUpdate` and use the
  determinism rules from earlier sections (`tick_rng`, no transcendentals,
  `BTreeMap` iteration).

These are engine-rules, not game-rules. A different engine's
`engine_determinism/<engine>.md` would carry its own conventions.

## What this DOESN'T do

- It does not invent gameplay. Phase 0 proposes the system list from the
  wiki's existing content. If the wiki has no "win condition" page, Phase 0
  may not propose one — that's a limitation of the LLM-classifier approach
  and is honestly acceptable for v1.
- It does not handle multiplayer / lockstep sync. The determinism rules
  carry through, but the netcode itself remains foundation-level work
  (out of scope for this phase).
- It does not produce art assets. Visual sprites stay placeholder; the
  pipeline doesn't generate textures.

## Risk: codegen drift across systems

A system plugin that takes `&mut Health` on every entity will conflict with
the unit plugin's own systems that take `&mut Health`. The existing smoke
gate catches B0001 panics, and the repair loop retries with the error
attached. Should hold up the same way it did for entity leaves.

If a system requires architectural cross-cutting (e.g. wave spawner needs
infected plugins to expose a "spawn point" Component the LLM didn't know
about), the repair loop's compile-error feedback may not be enough; the
system would land pending. Plan: triage pending systems with an operator
on first run.

## Build order

1. **Phase 1 finishes the Lethal Company vault.** (in progress)
2. **Phase 2 produces ~150 entity plugins.** (next, after Phase 1)
3. **Phase 0 re-run with systems pass.** Or, in the simpler implementation,
   add the systems pass as a one-shot CLI: `python scripts/phase0_systems.py`
   that reads the existing game-config + vault and proposes systems.
4. **Phase 3 loop generates systems** one at a time from the curated list.
5. **Smoke + run.** Window shows sprites that now react to input and game
   state changes.

## What I'm shipping in this iteration

While Phase 1 runs for Lethal Company:

1. `propose_gameplay_systems` + `_build_systems_prompt` + validator in
   `scripts/phase0_analyze.py`.
2. Phase 0 wiring in `scripts/phase0.py` (merges into `chosen_engine.systems`).
3. `derive_goals_from_systems` in `phase2/loop_driver.py` reading
   `chosen_engine.systems`.
4. `--from-systems` CLI flag in `phase2/loop_driver.py` analogous to
   `--from-vault`.
5. Tests for the new validator + goal derivation.
6. Documentation updates (MEMORY, plan).

The per-engine system rules in `bevy.md` and the implemented-entity context
injection in `codegen.py` are deferred to a second pass once the first system
turn actually runs and we see where the LLM struggles.
