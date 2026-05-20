# TODO

## Phase 1 Wikitext Trimming

### Goal

Reduce Codex token usage and compile time by cleaning raw MediaWiki wikitext
before it is inserted into the Phase 1 compile prompt.

### Where It Belongs

This belongs in **Phase 1**, not Phase 0.

Current pipeline:

```text
Phase 0: wiki categories -> game-config.json
Phase 1: game-config.json + wiki pages -> vault/<kind>/<slug>.md
Phase 2: vault/ -> generated game code
```

The trimming step should be inserted inside Phase 1:

```text
page_wikitext(...)
  -> trim_wikitext(...)
  -> compile_prompt(...)
  -> Codex
  -> schema validation
  -> vault file or quarantine file
```

### Why Not Phase 0

Phase 0 only discovers taxonomy and writes configuration. It does not compile
individual wiki pages into vault notes, so page-level cleanup would be outside
its responsibility.

### Current Behavior

`scripts/phase1_ingest.py` currently fetches raw page content from the
MediaWiki API:

```python
rvprop = "ids|content"
rvslots = "main"
```

That avoids scraping rendered HTML, but it still sends raw wiki markup to the
LLM. The prompt variable is named `{{stripped_html}}`, but the content is not
actually stripped yet.

### Proposed Change

Add a `trim_wikitext(text: str) -> str` helper in `scripts/phase1_ingest.py`.

Call it in `process_page` immediately after fetching the page:

```python
text, rev = page_wikitext(ctx["api"], title, ctx["ua"], ctx["retries"])
trimmed_text = trim_wikitext(text)
source = trimmed_text + f"\n\nSOURCE_URL: {source_url}\nSOURCE_REVISION: {rev}\n"
```

### Conservative First Pass

Start with safe removals only:

- HTML comments
- `[[Category:...]]` tags
- file/image links such as `[[File:...]]` and `[[Image:...]]`
- obvious nav/footer templates
- empty repeated lines
- gallery sections if they contain only image listings

Avoid removing:

- infoboxes with stats
- tables with costs, health, damage, range, or unlock data
- bullet lists
- formulas
- links to other gameplay entities

### Observability

Print raw vs trimmed size for each page during ingest:

```text
[compiled] Buildings / Tesla Tower -> vault/building/tesla_tower.md
  source: https://they-are-billions.fandom.com/wiki/Tesla_Tower
  wikitext: 18420 chars -> 11290 chars
```

This makes token savings visible while keeping the current cache and validation
flow easy to reason about.

### Cache Impact

The cache key is based on the final rendered prompt plus model id. Adding a
trimming step will change prompts and invalidate existing `.phase1_cache`
entries. That is expected.

### Follow-Up Option

After trimming is stable, consider batching multiple pages into one LLM call.
Batching may reduce repeated prompt overhead, but it is riskier because output
framing, validation, retries, and quarantine handling become more complicated.
