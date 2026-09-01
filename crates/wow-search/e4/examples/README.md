# E4-A search fixture shapes

All implementation-dependent IDs, exact SQLite/FTS5 profile values, result bytes, thresholds, benchmark reports, and SHA-256 values remain null only while `implementation_state = not-started`.

## Fixtures

- `search-universe-set.json` — exact separate user-project, Blizzard UI, and Reference shards.
- `shard-build-plan.json` — deterministic document partitions, generation-local FTS build, validation, and seal.
- `search-request.json` — structured exact/literal query and safe FTS AST.
- `lane-results.json` — exact/alias/prefix/text/fuzzy/shape/graph signal records and lane states.
- `ranked-results.json` — authority bands, integer contributions, stable ties, and complete explanations.
- `miss-continuation-cases.json` — exact/approximate misses, pages, cumulative budgets, and exact retained-shard continuation.
- `security-cases.json` — query injection, privacy, source-instruction, resource, cancellation, and cross-generation mutations.
- `CHECKSUMS.json` — prerequisite, owner-port, SQLite/FTS, profile, corpus, vector, threshold, artifact, and member freeze gate.

Tests validate committed fixture bytes and never rewrite them automatically.
