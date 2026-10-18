# `wow-core` contribution guidance

Follow repository [`AGENTS.md`](../../AGENTS.md),
[`docs/ARCHITECTURE.md`](../../docs/ARCHITECTURE.md),
[`docs/REFACTOR_QUEUE.md`](../../docs/REFACTOR_QUEUE.md), and the shared
[`crates/AGENTS.md`](../AGENTS.md).

`wow-core` is a small, dependency-light home for primitives genuinely shared by
current owners: typed IDs where needed, bounded-input helpers, common errors,
and source/path representation if consumers need one definition. Keep domain
semantics in reference, annotations, Emmy, project, graph, recognizers, rules,
search, or context owners.

Do not expand core into a universal contract or protocol framework. Avoid
cryptographic result/operation envelopes, digest-bound journaling, schema
catalogues, elaborate status/coverage taxonomies, generic admission registries,
or compatibility layers unless a present product caller requires a specific
invariant. Remove obsolete public variants and consumers together when the
current format can be rebuilt from retained source/configuration.

Preserve meaningful input validation and explicit limits. Do not silently
normalize invalid paths, spans, identifiers, or byte ranges into plausible
values. Errors should identify the actual invalid input class without leaking
private local paths or source excerpts.

Before changing a shared type, inspect its actual callers and remove/consolidate
compatibility callers in the same coherent change. Keep APIs simple enough for
the CLI and stdio LSP/MCP paths to serialize at their boundary. Add focused
coverage only where it demonstrates a meaningful behavior or regression.
