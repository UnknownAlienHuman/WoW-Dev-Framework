# E4-A implementation plan

**Status:** normative order; implementation not started.

## Phase 0 — prerequisite and profile freeze

- implement and freeze all E0–E3 owner contracts required by E4-A;
- freeze exact owner read catalogs;
- probe and freeze SQLite/Rust binding/FTS5/tokenizer/platform behavior;
- freeze document/field/normalization/query/lane/ranking/miss/privacy/budget profiles;
- freeze synthetic and pinned corpora;
- freeze machine fixtures and expected canonical bytes;
- update implementation state only with evidence.

No Rust E4-A crate activation before this gate.

## Phase 1 — pure value types and canonicalization

Implement:

- profile IDs and closed registries;
- SearchShardSourceBinding;
- SearchDocument/FieldOrigin/Partition;
- SearchRequest/NormalizedSearchQuery/FtsQueryAst;
- CandidateSignal/Candidate/Explanation;
- Miss/ResultSet/Result/Continuation;
- typed errors.

No SQLite, graph traversal, or owner calls.

Tests: schema, canonicalization, identity, error, malicious-input vectors.

## Phase 2 — document projection

Implement exact owner adapters through reviewed read ports:

- user project;
- Blizzard UI source;
- ReferenceView.

Project only allowed fields, origins, coverage/conflicts, privacy/license, and immutable partitions. No source parsing or context import.

Tests: projection, profile isolation, alias evidence, removal closure, rename/path mutations.

## Phase 3 — in-memory exact indexes and lanes

Implement:

- exact identity;
- canonical qualified/short names;
- explicit aliases;
- namespace/member/receiver;
- case-sensitive prefix.

Freeze exact miss gates and ambiguity behavior.

## Phase 4 — immutable shard logical build

Implement complete partition membership, reuse/removal plan, logical index manifests, SearchShardId, and validation catalogs independent of physical storage.

## Phase 5 — FTS5 physical profile

After executable probe acceptance:

- add SearchStore schema/operations through `wow-store`;
- build generation-local FTS content;
- private rowid/document mapping;
- read-only reopening;
- integrity and golden queries;
- physical reproducibility classification.

No loadable extensions.

## Phase 6 — text and identifier similarity

Implement safe query AST compiler, FTS lane with local ordinals/snippets, deterministic trigrams/edit distance, budgets, cancellation, and adversarial tests.

## Phase 7 — structured shape lane

Implement typed exact/compatible/partial shape features without unknown collapse or lineage inference.

## Phase 8 — graph-assisted lane

Implement exact-seed bounded graph expansion, reason paths, confidence caps, cross-universe explicit bridges, cycle/fanout limits, and path-preservation tests.

## Phase 9 — fusion, explanations, miss, result set

Implement authority bands, integer/ordinal ranking tuple, caps/penalties, signal aggregation, explanation reconstruction, exact miss evaluation, immutable result-set manifest, and whole-candidate pages.

## Phase 10 — continuation and retention seam

Implement exact cursor validation, cumulative budgets, result/shard retention requirements, deterministic replay validation if enabled, and no-current/no-refresh tests.

## Phase 11 — evaluation and calibration

Run frozen corpora, hard zero-tolerance gates, ablations, collision/privacy/adversarial tests, 1/2/N worker and cold/warm rebuilds. Freeze accepted quantitative thresholds and RankingProfile.

## Phase 12 — final freeze and handoff

Populate:

- prerequisite implementation and fixture digests;
- all profile IDs;
- owner/store operation catalogs;
- shard/document/query/result vectors;
- canonical JSON bytes;
- accepted benchmark/evaluation reports;
- all member SHA-256 and bundle digest.

Publish E4-A implementation evidence. Only then may E4-B lineage and E4-C service work start.

## Deferred

- service/CLI search commands;
- lineage/replacement/migration/impact;
- context candidate selection;
- CBM/models/embeddings;
- runtime observations;
- LSP/MCP/release/CI.
