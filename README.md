# cloneGame

Point this at any public Fandom-style wiki and it produces a playable clone
of that game. The pipeline reads the wiki, builds a structured note vault,
and codegens a working Rust + Bevy crate that you can run with `cargo run`.

The project assumes no Rust knowledge — if you can run a few commands in a
terminal, you can make a game.

## What you need installed

You only set these up once.

1. **Rust toolchain** — `rustup` from <https://rustup.rs>. Pick stable.
   Verify with `rustc --version` and `cargo --version`.
2. **Python 3.11+** — verify with `python --version`.
3. **Node.js + npm** — needed for the `repomix` tool that packs the note
   vault before retrieval. `node --version` and `npm --version` should both
   work.
4. **`repomix`** — `npm install -g repomix`. Verify with `repomix --version`.
5. **One of two LLM CLIs** (both use a subscription, no API keys):
   - **codex** — install from the official OpenAI CLI release. Log in with
     `codex login`. The pipeline defaults to codex because it has the
     highest daily quota and the fastest per-turn latency.
   - **claude** — `npm install -g @anthropic-ai/claude-code` then
     `claude /login`. Used as a fallback when codex is unavailable.
6. **Pipeline Python dependencies** —
   ```powershell
   pip install -r requirements-phase2.txt
   pip install -r requirements-dev.txt
   pip install jsonschema
   ```

## Run the pipeline against the bundled example

The repo ships pre-configured for *They Are Billions*. Run the steps in this
order from the repo root in PowerShell or any terminal:

```powershell
# Phase 0 — taxonomy + per-kind schemas + engine candidates (LLM, ~2 min)
python scripts/phase0.py

# Pick an engine if Phase 0 didn't choose one.
# Open game-config.json, find "chosen_engine": null, replace with the
# desired entry from "engine_candidates" (Bevy is the supported one today).

# Phase 1 — wiki → vault (LLM, ~30–60 min, ~120 pages)
python scripts/phase1_ingest.py

# Phase 2 prelude
python phase2/scaffold.py          # render engine foundation files
python scripts/regenerate_repomix.py  # pack the vault for retrieval
python phase2/indexer.py           # build Chroma index + graph.json
python phase2/baseline.py          # render the engine baseline prompt
python phase2/system_map.py init   # initialize the implementation ledger

# Phase 2 codegen (LLM, ~30–60 min depending on vault size)
python phase2/loop_driver.py --from-vault --concurrency 3 --repair-attempts 3

# Verify
cargo build --manifest-path game/Cargo.toml
cargo test  --manifest-path game/Cargo.toml --test app_smoke
cargo run   --manifest-path game/Cargo.toml --bin clone-game
```

The window should open, sim ticks should print every second, and five colored
demo sprites confirm the renderer is alive while the generated unit /
building plugins quietly run.

## Run against a different wiki

Three things change between games:

1. **The wiki URL.** Edit `game-config.json` once before the first Phase 0
   run, or pass it at the command line:

   ```powershell
   python scripts/phase0.py --wiki-url https://your-wiki.fandom.com/wiki/
   ```

   Phase 0 will re-derive the taxonomy, per-kind schemas, engine candidates,
   and a `codegen: true|false` flag per kind (so map / lore / patch-notes
   kinds get auto-skipped in Phase 2).

2. **The engine choice.** Phase 0 leaves `chosen_engine: null` for you to
   pick from `engine_candidates`. Today only **Bevy** has its scaffold and
   determinism rules wired (`prompts/engine_scaffold/bevy/` +
   `prompts/engine_determinism/bevy.md`). Other engines require adding a
   matching scaffold tree and determinism markdown — see
   `prompts/engine_scaffold/README.md`.

3. **Wipe before re-running on a different game.** The vault, build
   artifacts, and `game/` source tree from the previous run will collide
   with a new wiki's content:

   ```powershell
   Remove-Item -Recurse -Force vault, .phase1_cache, build, game/src, game/tests, game/Cargo.toml, game/Cargo.lock, game-config.proposed.json
   # Then re-create a minimal game-config.json with the new wiki URL:
   ```

   ```json
   {
     "version": 1,
     "game": {"name": "<your game>", "wiki_base_url": "https://your-wiki.fandom.com/wiki/"},
     "human_approved": false,
     "kinds": {},
     "categories": []
   }
   ```

   And re-run the steps above.

## What if something fails?

The pipeline is **idempotent**. Anything that already succeeded gets skipped on
a re-run; failed entries get retried. Common cases:

- **`cargo build` fails after Phase 2** — codegen produced a broken module.
  The loop driver's repair loop usually fixes this automatically. Re-run
  `python phase2/loop_driver.py --from-vault` and the failed slug retries.
- **`cargo test --test app_smoke` panics** — a runtime conflict between
  generated plugins (e.g. two plugins both mutating the same component).
  See `docs/MEMORY.md` for the Bevy B0001 pattern; the most common fix is
  to add the offending plugin to
  `chosen_engine.entrypoint.excluded_plugins` in `game-config.json` and
  regenerate it through the loop.
- **`You've hit your usage limit`** — the codex (or claude) CLI hit its
  daily subscription quota. Resume the loop later when the quota refreshes,
  or pass `--llm-mode claude` (or `--llm-mode codex`) to swap backends. Run
  `python phase2/loop_driver.py --from-vault --fallback-llm-mode claude`
  to fall back automatically per-turn.
- **Phase 1 quarantines too many pages** — Phase 0's schema proposer
  over-typed some fields. The validator widens scalar declarations
  automatically; re-running Phase 1 after this fix in place should bring
  the quarantine rate near zero.

## What you'll have at the end

- `game/` — a real Rust + Bevy crate. Library + binary. Runs with
  `cargo run`. Builds clean. Generated by the pipeline; no hand edits.
- `vault/` — Phase 1's structured note vault. Markdown files with YAML
  frontmatter, one per wiki page, sorted into per-kind directories.
- `build/system_map.yaml` — the implementation ledger. Tells you what got
  generated, what's pending, and what failed.
- `game-config.json` — the per-game knob. Taxonomy, schemas, engine
  choice, kind code/data flags. All produced by Phase 0; safe to edit.

## Ship a finished game so it survives a wipe

The pipeline treats `game/`, `vault/`, `build/`, and `game-config.json` as the
in-progress working set. Switching to a different wiki, or just iterating,
wipes those. Snapshot the current game first:

```powershell
python scripts/ship.py                # snapshots + cargo build --release + bundle
python scripts/ship.py --skip-binary  # snapshot source/vault/config only (fast)
python scripts/ship.py --force        # overwrite an existing release
```

This writes `releases/<slug>/` containing the full Cargo project, the vault
that produced it, the game-config, the implementation ledger, and a release
binary for the OS you're on. The release directory is committed to git
(source + vault are small); binaries are gitignored.

To restore a shipped game back to the working tree:

```powershell
python scripts/load_game.py                  # list available releases
python scripts/load_game.py lethal-company   # restore that one
python scripts/load_game.py lethal-company --force  # overwrite working tree
```

After load, `cargo build --manifest-path game/Cargo.toml` rebuilds against
the loaded source. You can run the binary, replay the pipeline against the
same wiki, or just inspect what was generated.

## How the pipeline is shaped

Three phases, each gated and resumable:

```
Phase 0    wiki API  ─►  game-config.json   (taxonomy + schemas + engine + codegen flags)
Phase 1    wiki + game-config.json ─►  vault/<kind>/<slug>.md
Phase 2    vault/ ─►  Chroma index + engine baseline ─►  game/src/<kind>/<slug>.rs
```

The big architectural notes live in:

- `CLAUDE.md` — agent-facing development rules.
- `DEPLOYMENT_GUIDE.md` — pipeline architecture.
- `docs/plan.md` / `docs/MEMORY.md` / `docs/ERRORS.md` — running history.

## Help / where to file issues

This is a student-built project. If something breaks, check `docs/ERRORS.md`
first — most recurring failure modes are already documented there with the
fix that worked. New problems are best filed against this repo's issues
tracker with: the wiki URL, the failing command, and the last 30 lines of
output.
