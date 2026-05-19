/**
 * Schema helper utilities shared by compile, lint, and CLI.
 *
 * Kept separate from `loader.ts` so callers that just need to interpret a
 * page's kind or count its wikilinks don't pull the YAML/JSON parser into
 * their dependency graph.
 */

import yaml from "js-yaml";
import type { PageKind, SchemaConfig } from "./types.js";

/** Pattern matching [[Wikilink Title]] references in markdown content. */
const WIKILINK_PATTERN = /\[\[([^\]]+)\]\]/g;

/**
 * Resolve a page's kind from its raw frontmatter value, falling back to the
 * schema default when no explicit kind is set or the value is not declared
 * in the loaded schema. A kind is considered valid when the schema has a
 * matching rule entry, so projects that declare custom kinds via
 * `game-config.json` get their kinds honoured here.
 * @param rawKind - Raw `kind` value pulled from frontmatter (untyped).
 * @param schema - Resolved schema config.
 * @returns The resolved page kind.
 */
export function resolvePageKind(rawKind: unknown, schema: SchemaConfig): PageKind {
  if (typeof rawKind === "string" && rawKind.trim() !== "" && schema.kinds[rawKind] !== undefined) {
    return rawKind;
  }
  return schema.defaultKind;
}

/**
 * Count the [[wikilinks]] in a page body.
 * Pure function so the linter can apply per-kind minimums without re-parsing.
 * @param body - Markdown body text.
 * @returns Number of wikilink references found.
 */
export function countWikilinks(body: string): number {
  const matches = body.match(WIKILINK_PATTERN);
  return matches ? matches.length : 0;
}

/**
 * Serialise a schema config to YAML for `llmwiki schema init` to write to disk.
 * The `loadedFrom` field is omitted because it's a runtime-only annotation.
 * @param schema - Resolved schema config.
 * @returns YAML string suitable for writing to a schema file.
 */
export function serializeSchemaToYaml(schema: SchemaConfig): string {
  const serializable = {
    version: schema.version,
    defaultKind: schema.defaultKind,
    kinds: schema.kinds,
    seedPages: schema.seedPages,
  };
  return yaml.dump(serializable, { lineWidth: -1, quotingType: '"' });
}
