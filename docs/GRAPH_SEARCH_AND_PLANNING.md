# Blizzard and addon source search

The practical search workflow indexes a selected Blizzard UI checkout and/or
addon root, then finds source text, declarations, API symbols, and useful
navigation context. Use existing `wow-project`, `wow-search`, `wow-store`, and
application owners. SQLite/FTS5 is the current local substrate; keep normal
queries on retained indexed facts rather than reparsing the same source.

## Inputs and facts

Use the configured root, client flavor, TOC order, and exact source revision.
Preserve path identity, source locations, and UTF-8 byte spans. Parse Lua/XML
and TOC data as bounded, nonexecuting input; do not run addon scripts or load
generated code. Reuse EmmyLua for Lua syntax and semantics. Keep one parser,
index, and current representation; update derived rows from source/configuration
when inputs change.

Useful query results include matching symbols or text, the owning file and
location, relevant TOC/XML relationships already represented by the index, and
a small source excerpt or Markdown context with a resolvable source location.
Keep truncation and unsupported syntax visible. Similar names or text matches
are discovery hints; they do not prove that an API replaces another or that a
pattern executes at runtime.

## Scope and evidence

The current product is local source search, symbol lookup/navigation, and
bounded context over Blizzard UI and addon text. Use the existing SQLite tables,
FTS5 indexes, project facts, graph relations, and service/front-end operations
that actually exist. Inspect owner code before describing a query as supported;
this document is a short responsibility guide, not a promise that every
relationship or UI surface is implemented.

Do not expand this into a source-lineage or migration-impact platform, review
and candidate-ranking system, structural-fingerprint service, external provider
or Codebase Memory integration, historical archive, or generic graph framework.
These are outside current scope. A concrete developer task may justify a small
search or relationship improvement when existing owners cannot answer it; name
and replace a redundant path instead of stacking another index or parser.

Preserve exact source/flavor identity and unknown results. Static source facts
do not establish runtime execution, load success, combat behavior, or taint
safety. For a changed search/navigation behavior, use one representative
positive and negative query; run a broader corpus qualification only when
claiming that coverage.
