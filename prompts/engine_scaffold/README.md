# engine_scaffold

Per-engine **foundation file tree** rendered into `game/` by `phase2/scaffold.py`
at the start of every fresh-run. The pipeline owns these files so a new wiki +
new engine never needs hand-written game code.

## How dispatch works

`phase2/scaffold.py` reads `chosen_engine.name` from `game-config.json`,
lowercases it, and looks for a matching directory under
`prompts/engine_scaffold/<engine>/`. If found, every file under that directory
is mirrored into `game/` at the same relative path.

Matching `prompts/engine_determinism/<engine>.md` (Phase 2 codegen rules) and
`chosen_engine.module_registration` + `chosen_engine.entrypoint` blocks in
`game-config.json` complete the per-engine surface.

## What goes in the scaffold

Engine-specific, **not** game-specific. The scaffold contains things every game
on this engine needs, in identical shape:

- **Manifest**: `Cargo.toml`, `pyproject.toml`, `Project.godot`, depending on
  the engine. Pin the language/runtime version and the foundation deps.
- **Foundation source**: types and resources the per-kind determinism rules
  name (e.g. Bevy's `Health`, `UnitStats`, `SimChecksumState`, `SimEventsPlugin`,
  the fixed-point RNG). The bevy.md rules already reference these types; the
  scaffold has to put them on disk so the rules' references are valid.
- **Entrypoint**: `main.rs`, `main.gd`, etc. The window, camera, sim tick loop.
  Calls into the auto-generated plugin aggregator (`crate::app_plugins::add_all`
  for Bevy); no per-game enumeration.
- **Smoke test**: a runtime-side gate that constructs the app, ticks
  `FixedUpdate` (or the engine's equivalent), and reaches the end without
  panicking. The loop driver runs this after every codegen turn.
- **Aggregator stub**: an empty `add_all` that compiles before any plugin
  exists. `phase2/entrypoint.py` regenerates it from `impl Plugin for X`
  declarations as soon as the first leaf lands.

## What does NOT go here

- **Per-game data**: unit stats, building costs, faction lore, map layouts.
  These come from the wiki via Phase 1 → vault → Phase 2 codegen.
- **String substitution placeholders**: the scaffold is rendered byte-exact
  (no `{name}` or Jinja syntax). Per-game variation flows through the codegen
  loop or `game-config.json`, not the scaffold.

## Adding a new engine

1. Pick the engine name (lowercased) — the same key used for
   `prompts/engine_determinism/<engine>.md` and the value of
   `chosen_engine.name` in `game-config.json`.
2. Create `prompts/engine_scaffold/<engine>/` mirroring the target project
   layout (e.g. `Cargo.toml`, `src/lib.rs`, `src/main.rs`, `src/sim.rs`,
   `tests/app_smoke.<ext>`).
3. Author the determinism rules at `prompts/engine_determinism/<engine>.md`.
   These rules will be inlined into every Phase 2 codegen prompt; reference
   the foundation types you put in the scaffold by their real names.
4. Optionally extend `chosen_engine.module_registration` and
   `chosen_engine.entrypoint` in `game-config.json` so the loop driver
   knows how to wire new modules and which file is the aggregator.
5. Run `python phase2/scaffold.py --dry-run` to preview, then `--force` to
   apply. `cargo build` (or the engine's equivalent) should pass before any
   codegen runs.

## Idempotency + hand-edits

The renderer never overwrites a file whose current content differs from the
scaffold unless `--force` is set. This preserves operator hand-edits during
development. Files that match the scaffold byte-exact are skipped (no mtime
churn — keeps the build cache hot).

Engine scaffold files that get rendered ARE committed to the repo as the
result of a real pipeline run. They're not gitignored. The `--force` workflow
plus a re-render is how you regenerate when you upgrade the foundation.
