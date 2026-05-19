/**
 * Schema config loader.
 *
 * Discovers a project's schema file from a fixed list of candidate paths,
 * parses it (JSON or YAML), and merges it onto the default schema. Missing
 * files are not an error — the compiler falls back to defaults so existing
 * projects continue to work without any migration.
 */

import { existsSync } from "fs";
import { readFile } from "fs/promises";
import path from "path";
import yaml from "js-yaml";
import type {
  PageKind,
  PageKindRule,
  PartialSchemaFile,
  SchemaConfig,
  SeedPage,
} from "./types.js";
import { buildDefaultSchema } from "./defaults.js";

/**
 * Candidate schema file paths searched in priority order.
 * `game-config.json` is the Phase 0 taxonomy-discovery output and takes
 * priority over `.llmwiki/schema.json` so a human-approved game taxonomy
 * always wins over a generic schema if both are present.
 */
const SCHEMA_CANDIDATE_PATHS = [
  "game-config.json",
  ".llmwiki/schema.json",
  ".llmwiki/schema.yaml",
  ".llmwiki/schema.yml",
  "wiki/.schema.yaml",
  "wiki/.schema.yml",
];

/** Find the first existing schema candidate path under `root`, or null. */
function findSchemaPath(root: string): string | null {
  for (const candidate of SCHEMA_CANDIDATE_PATHS) {
    const absolute = path.join(root, candidate);
    if (existsSync(absolute)) return absolute;
  }
  return null;
}

/** Decide whether to parse the file as JSON or YAML based on its extension. */
function parseSchemaFile(filePath: string, content: string): PartialSchemaFile {
  const isJson = filePath.endsWith(".json");
  const parsed = isJson ? JSON.parse(content) : yaml.load(content);
  if (parsed && typeof parsed === "object") return parsed as PartialSchemaFile;
  return {};
}

/** Type-guard checking whether a value is a usable page kind name (any non-empty string). */
function isPageKind(value: unknown): value is PageKind {
  return typeof value === "string" && value.trim() !== "";
}

/** Baseline rule applied to a custom kind that has no defaults entry. */
const CUSTOM_KIND_BASELINE: PageKindRule = {
  minWikilinks: 0,
  description: "Project-defined kind from game-config.json or .llmwiki/schema.json.",
};

/** Merge a single per-kind rule from the file onto the default rule. */
function mergeKindRule(
  defaults: PageKindRule,
  override: Partial<PageKindRule> | undefined,
): PageKindRule {
  if (!override) return defaults;
  const minWikilinks = typeof override.minWikilinks === "number"
    ? override.minWikilinks
    : defaults.minWikilinks;
  const description = typeof override.description === "string"
    ? override.description
    : defaults.description;
  return { minWikilinks, description };
}

/**
 * Merge per-kind rule overrides onto the default rule table. Accepts any kind
 * name the project declares — kinds not in defaults get the
 * `CUSTOM_KIND_BASELINE` rule as their starting point before overrides apply.
 */
function mergeKinds(
  defaults: Record<PageKind, PageKindRule>,
  overrides: PartialSchemaFile["kinds"],
): Record<PageKind, PageKindRule> {
  const merged: Record<PageKind, PageKindRule> = { ...defaults };
  if (!overrides) return merged;

  const allKinds = new Set<string>([
    ...Object.keys(defaults),
    ...Object.keys(overrides),
  ]);
  for (const kind of allKinds) {
    if (!isPageKind(kind)) continue;
    const baseline = defaults[kind] ?? CUSTOM_KIND_BASELINE;
    merged[kind] = mergeKindRule(baseline, overrides[kind]);
  }
  return merged;
}

/** Validate and coerce a single seed page entry. Returns null when invalid. */
function normalizeSeedPage(entry: Partial<SeedPage>): SeedPage | null {
  if (typeof entry.title !== "string" || entry.title.trim() === "") return null;
  if (!isPageKind(entry.kind)) return null;
  const summary = typeof entry.summary === "string" ? entry.summary : "";
  const relatedSlugs = Array.isArray(entry.relatedSlugs)
    ? entry.relatedSlugs.filter((slug): slug is string => typeof slug === "string")
    : undefined;
  return { title: entry.title, kind: entry.kind, summary, relatedSlugs };
}

/** Coerce raw seed page entries into validated SeedPage objects. */
function normalizeSeedPages(entries: PartialSchemaFile["seedPages"]): SeedPage[] {
  if (!Array.isArray(entries)) return [];
  return entries
    .map(normalizeSeedPage)
    .filter((entry): entry is SeedPage => entry !== null);
}

/** Apply a parsed partial-schema onto the defaults, returning the resolved config. */
function applyOverrides(
  defaults: SchemaConfig,
  overrides: PartialSchemaFile,
  loadedFrom: string,
): SchemaConfig {
  const defaultKind = isPageKind(overrides.defaultKind)
    ? overrides.defaultKind
    : defaults.defaultKind;
  return {
    version: 1,
    defaultKind,
    kinds: mergeKinds(defaults.kinds, overrides.kinds),
    seedPages: normalizeSeedPages(overrides.seedPages),
    loadedFrom,
  };
}

/**
 * Load the schema for `root`, falling back to defaults when no file is present.
 * Throws on parse failure so the user sees a clear error rather than a silent
 * default — silent fallback would mask real config bugs.
 * @param root - Project root directory.
 * @returns Resolved schema config.
 */
export async function loadSchema(root: string): Promise<SchemaConfig> {
  const defaults = buildDefaultSchema();
  const schemaPath = findSchemaPath(root);
  if (!schemaPath) return defaults;

  const raw = await readFile(schemaPath, "utf-8");
  const parsed = parseSchemaFile(schemaPath, raw);
  return applyOverrides(defaults, parsed, schemaPath);
}

/** Expose candidate paths so the CLI `schema init` command can pick one. */
export function defaultSchemaInitPath(root: string): string {
  return path.join(root, SCHEMA_CANDIDATE_PATHS[0]);
}
