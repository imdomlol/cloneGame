/**
 * Type definitions for the wiki schema layer.
 *
 * The schema layer turns llmwiki from a flat compiler pipeline into a shaped
 * knowledge system. It declares the kinds of pages a project supports
 * (`concept`, `entity`, `comparison`, `overview`) and the cross-link
 * expectations that lint and review enforce per kind.
 *
 * Types live in their own module so that compile, lint, CLI, and tests can
 * depend on the schema vocabulary without pulling in YAML/JSON loaders.
 */

/**
 * Page kind. Now an open string type so projects can declare custom kinds in
 * `game-config.json` or `.llmwiki/schema.json` and have them flow through the
 * pipeline without being silently dropped. Built-in defaults are still
 * available via `PAGE_KINDS`, but the type itself is no longer a closed union.
 */
export type PageKind = string;

/** Built-in default kinds, retained for backward compatibility and seeded into the schema when no project config exists. */
export const PAGE_KINDS: readonly string[] = [
  "concept",
  "entity",
  "comparison",
  "overview",
] as const;

/** Per-kind policy: minimum cross-links and a human description used in prompts. */
export interface PageKindRule {
  /** Minimum number of [[wikilinks]] a page of this kind should contain. */
  minWikilinks: number;
  /** Short human-readable description; surfaced in prompts and review output. */
  description: string;
}

/** Optional declarative seed for non-concept pages the compiler can generate. */
export interface SeedPage {
  /** Display title; also used to derive the page slug. */
  title: string;
  /** Page kind — must be one of `PAGE_KINDS`. */
  kind: PageKind;
  /** One-line summary written into frontmatter. */
  summary: string;
  /**
   * For `overview` and `comparison` kinds, the slugs the page should weave
   * together. The compiler passes these to the LLM as the source material.
   */
  relatedSlugs?: string[];
}

/** Resolved schema config the rest of the compiler reads. */
export interface SchemaConfig {
  /** Schema format version. Currently always `1`. */
  version: 1;
  /** Kind assigned to pages that don't declare a kind in frontmatter. */
  defaultKind: PageKind;
  /** Per-kind rules keyed by `PageKind`. */
  kinds: Record<PageKind, PageKindRule>;
  /** Optional seed pages the compiler should materialise on each run. */
  seedPages: SeedPage[];
  /** Path the schema was loaded from, or `null` when defaults are used. */
  loadedFrom: string | null;
}

/** Raw schema file contents — every field is optional so partial files work. */
export interface PartialSchemaFile {
  version?: number;
  defaultKind?: string;
  kinds?: Partial<Record<string, Partial<PageKindRule>>>;
  seedPages?: Array<Partial<SeedPage>>;
}
