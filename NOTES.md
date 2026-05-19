# Phase 1 Setup Notes

Captured during the Phase 1 prereq implementation so the gaps between
`DEPLOYMENT_GUIDE.md` and the real tooling are reviewable.

## Tooling reality vs. guide

| Guide wording (§4.1) | Reality | Where it lives |
| --- | --- | --- |
| `pipx install llm-wiki-compiler` | atomicstrata/llm-wiki-compiler is a **Node** CLI, not Python. Vendored from source. | `vendor/llm-wiki-compiler/` (cloned, `npm install && npm run build`) |
| `llmwiki ingest <url>` | Exists. Ingests one URL or local file at a time. No built-in full-wiki crawl. | `node vendor/llm-wiki-compiler/dist/cli.js ingest <url>` |
| `llmwiki compile` | Exists. Hash-based incremental. Requires `ANTHROPIC_API_KEY` (or alt provider). | `node vendor/llm-wiki-compiler/dist/cli.js compile` |
| Page-type router rules in `llmwiki.toml` | Tool has no URL router. Routing happens at compile time via LLM classification. | Intent captured in `phase1.config.toml` (documentation) + frontmatter `kind:` produced by the compile LLM |
| JSON-Schema files per `type` in `schemas/` | Tool does not validate frontmatter against external JSON Schemas. | `schemas/*.schema.json` written for downstream validation by a custom step |
| `_quarantine/` for failing files | Tool has no quarantine. Has a `--review` candidate flow instead. | Recreated by `scripts/phase1_sort.py` (post-compile pass) |
| Strict typed vault dirs (`vault/items/weapons/…`) | Tool writes flat to `wiki/concepts/<slug>.md`. | `scripts/phase1_sort.py` copies into `vault/<kind>/<slug>.md` based on frontmatter |

## Patch applied to the vendored tool

To accept arbitrary `kind:` values (the guide's 8 typed kinds plus future
Phase 0 discoveries), four files in `vendor/llm-wiki-compiler/src/schema/`
were patched. Patch is small (~30 LOC) and stays inside `src/schema/`:

- `types.ts` — `PageKind` is now `string` (was a closed union).
- `loader.ts` — `mergeKinds` iterates over the override keys, not a whitelist;
  `isPageKind` accepts any non-empty string; `game-config.json` is the
  highest-priority schema candidate path.
- `helpers.ts` — `resolvePageKind` accepts any kind declared in the loaded
  schema, falls back to `defaultKind` otherwise.
- `defaults.ts` — unchanged (the existing `Record<PageKind, …>` shape works
  unchanged when `PageKind = string`).

To re-apply after an upstream pull:

```bash
cd vendor/llm-wiki-compiler
git fetch origin && git reset --hard origin/main
# re-apply the schema-layer patch (re-run the Edits in this conversation)
npm install && npm run build
```

Upstream tests have a pre-existing Windows bug in `test/global-setup.ts`
(uses `execFile("npx", …)` which doesn't resolve `npx.cmd` on Windows), so
`npm test` fails before any test runs. `npx tsc --noEmit` and `npm run build`
both pass with the patch applied — those are the verification gates used here.

## Phase 0 ↔ Phase 1 contract

`game-config.json` is the bridge:

```jsonc
{
  "version": 1,
  "game": { "name": "...", "wiki_base_url": "..." },
  "human_approved": false,         // Phase 0 LLM proposes; human flips to true
  "defaultKind": "concept",
  "kinds": {
    "item":   { "minWikilinks": 1, "description": "..." },
    "skill":  { "minWikilinks": 1, "description": "..." },
    // ... 8 kinds from §1.3, plus any Phase 0 discoveries
  },
  "seedPages": []
}
```

- Patched llmwiki loader reads it as a `PartialSchemaFile` (extra fields are
  ignored, custom kinds are merged in).
- `scripts/phase1_sort.py` reads its `kinds` object to learn the approved
  taxonomy and emits a warning when `human_approved !== true`.

## Phase 1 prereq checklist (§4.1) status

| Item | Status | Where |
| --- | --- | --- |
| Install llm-wiki-compiler | ✅ vendored at `vendor/llm-wiki-compiler/`, built | npm clone path |
| Wiki base URL + page-type router | ✅ captured | `phase1.config.toml` (intent), `game-config.json` (runtime) |
| JSON Schemas for each type | ✅ 9 schemas (8 typed + universal) | `schemas/` |
| `ingest && compile` produces zero quarantined files | ⏸ blocked on: real `ANTHROPIC_API_KEY`, a batch ingest driver, and `human_approved: true` | run `python scripts/phase1_sort.py` after compile; exit code 1 iff anything quarantined |

The smoke test for the sorter (item routes to `vault/item/`, unknown-kind
and missing-kind both quarantine) ran clean on three fabricated input files;
deleting the smoke-test files when you're ready for the real ingest:

```bash
rm wiki/concepts/iron_sword.md wiki/concepts/mystery_thing.md wiki/concepts/no_kind.md
rm -rf vault/
```

## Outstanding work before Phase 1 produces a real vault

1. Flip `human_approved` in `game-config.json` to `true` (after Phase 0 review).
2. Set `ANTHROPIC_API_KEY` in the shell / `.env`.
3. Write a batch ingest driver (the guide assumes one but doesn't ship it):
   walk the Fandom MediaWiki API for `they-are-billions.fandom.com`, enumerate
   pages, call `llmwiki ingest <url>` for each (throttled per
   `phase1.config.toml`'s `ingest` block).
4. Run `llmwiki compile`, then `python scripts/phase1_sort.py`.
5. Verify exit code 0 (zero quarantined). If non-zero, inspect
   `vault/_quarantine/` and either (a) update `game-config.json` kinds to
   absorb the new taxonomy, or (b) tighten the compile prompt.
