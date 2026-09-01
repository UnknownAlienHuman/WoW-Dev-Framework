# E4-A normative fixture shapes

These files are closed documentation fixtures. Implementation-dependent IDs, bytes, expected ranks, benchmark values, and SHA-256 fields may remain `null` only while `implementation_state = not-started`.

- `search-universe-set.json` — exact user-project, optional Blizzard UI, and Reference SearchShard binding.
- `shard-build-plan.json` — owner projection, immutable partition membership, logical index, FTS, validation, and publication shape.
- `search-request.json` — structured exact and mixed query forms plus safe FTS AST.
- `lane-results.json` — exact, text, similarity, shape, and graph signal separation.
- `ranked-results.json` — authority bands, integer/ordinal ranking tuple, complete explanation, and no-lineage nonclaims.
- `miss-continuation-cases.json` — exact/partial/approximate misses, result-set pages, retained exact continuation, and budget behavior.
- `security-cases.json` — query/FTS/SQL/source/privacy/corruption/resource/cancellation adversarial cases.
- `evaluation-cases.json` — hard authority gates, top-k metrics, collisions, ablation, and deterministic replay.
- `CHECKSUMS.json` — prerequisite/profile/corpus/vector/member freeze gate.

Tests verify committed fixtures. They never rewrite them automatically.
