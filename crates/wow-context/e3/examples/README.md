# E3-B normative and compatibility fixtures

**Status:** closed documentation shapes; implementation-dependent IDs and SHA-256 values remain null only while implementation state is `not-started`.

## Current E3-B fixtures

- `context-universe-set.json` — exact user-project, optional Blizzard UI source, and ReferenceView binding.
- `context-request.json` — exact roots, intent, expansion, source, privacy, budget, tokenizer, and renderer profiles.
- `project-map.json` — deterministic compact navigation projection.
- `l0-skeleton.json` — bounded container-level structure.
- `l1-skeleton.json` — bounded entity/local-neighborhood detail.
- `context-semantic-pack.json` — canonical selected items, evidence, omissions, conflicts, and budgets.
- `rendered-context-artifact.json` — separately identified JSON/Markdown projection.
- `omission-coverage-cases.json` — honest partial, conflict, negative-authority, and pruning vectors.
- `cache-determinism-cases.json` — exact cache identity, stale/corrupt/privacy mismatch, and rebuild vectors.

## Inherited specialized fixtures

The existing `input-snapshot.json`, `context-bundle.json`, planning/continuation, source-security, budget/tokenizer, and evaluation fixtures remain part of E3-B through `COMPATIBILITY_ALIASES.json`. They are not a second E3-A context implementation.

## Freeze rules

- Current machine identities use `ContextUniverseSet` and `ContextSemanticPack`.
- Historical names are one-way documentation/migration aliases only.
- All roots, profiles, catalogs, expected IDs, errors, bytes, token vectors, and checksums freeze before the first Rust commit.
- Tests verify committed fixtures and never rewrite them automatically.
