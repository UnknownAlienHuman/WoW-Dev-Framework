# `wow-service` E4-C search, lineage, migration, impact, and context-handoff contract

**Status:** implementation-ready documentation; no Rust code yet.

**Contract ID:** `wow-service/e4-c/search-lineage-impact-use-cases`

## Mission

Coordinate exact E4-A search shards and exact E4-B lineage overlays into transport-independent public use cases without moving search, graph, project, reference, context, persistence, review authorization, or rendering algorithms into `wow-service`.

```text
symbolic or exact owner selectors
-> resolve symbolic current only at the service boundary
-> acquire exact retained project/reference/graph/search/lineage views
-> validate generation/profile/capability/privacy compatibility
-> invoke exactly one owner operation plan
-> preserve owner evidence, candidates, proof ceilings, conflicts, coverage, omissions and budgets
-> optionally validate an explicit search candidate selection
-> optionally hand the selected exact entity root to E3-B context through the existing E3-C service path
-> close all resources in reverse order
-> publish one canonical service result envelope
```

## Public E4-C operations

```text
search_index_status
search_index_build
search_index_validate
search_query
search_continue
search_explain
search_select
search_context

lineage_status
lineage_build
lineage_validate
lineage_review_validate
lineage_review_apply
lineage_compare
lineage_trace
lineage_explain

migration_candidates
migration_validate

impact_plan
impact_run
impact_continue
impact_explain
```

No operation applies a source edit, executes a migration, writes an addon, claims runtime behavior, or silently selects the highest-ranked candidate.

## Active direct dependency slice

```text
wow-service
├── wow-core
├── wow-store
├── wow-reference
├── wow-project
├── wow-graph
├── wow-search
└── wow-context
```

`wow-emmy`, `wow-recognizers`, `wow-rules`, `wow-annotations`, and `wow-cbm` remain inactive direct dependencies for E4-C. Their relevant immutable outputs can be reached only through exact owner publications.

## Ownership

`wow-service` owns:

- public request normalization and profile alias resolution;
- one-time symbolic selector resolution;
- fixed-order exact view/lease acquisition and reverse-order closure;
- cross-owner compatibility validation;
- orchestration of E4-A shard build/query operations;
- orchestration of E4-B producer, review, publication, lineage, migration and impact operations;
- explicit search-candidate selection receipts;
- exact search-result-to-context-root handoff;
- lifecycle, idempotency, continuation retention, status folding and canonical envelopes.

It does not own:

- search documents, query AST, lane execution, ranking, explanations or miss authority;
- lineage proposals, proof ceilings, ambiguity resolution, accepted assertions, change classification, migration recipes or impact traversal;
- project/reference producer facts;
- context maps, skeletons, selection, source boundaries or rendering;
- physical SQLite, shard, graph, lease, cache or file implementation;
- reviewer identity/credential verification rules;
- CLI parsing, stdout/stderr or exit codes.

## Exact selection boundary

Search and lineage owners accept exact retained views. E4-C may expose bounded convenience selectors such as `CurrentPublished`, but every alias is resolved before owner invocation and replaced by exact IDs in the canonical request.

There is no `latest`, nearest-build, same-name, last-known-good, automatically rebuilt shard, automatically selected lineage snapshot, or cross-profile fallback.

## Search-to-context boundary

```text
exact SearchResultId
+ exact SearchCandidateId
+ exact result manifest and selection guard
-> validate candidate belongs to the exact result
-> produce SearchSelectionReceipt
-> extract the candidate's exact owner EntityKey/ID
-> invoke existing E3-C context operation with that exact root
```

Rank, lane count and score are recorded as selection provenance only. They do not become context facts, lineage proof, replacement authority, or permission to edit.

## Review boundary

Review decisions arrive as strict typed `LineageReviewDecisionEnvelope` values. `wow-service` invokes a narrow `ReviewAuthorizationPort` and passes only validated exact decisions to `wow-graph`. Plain prose, GitHub username, local account name, CLI operator identity or file ownership is not review authority.

Applying a review creates a new immutable E4-B lineage snapshot; it never modifies an existing snapshot in place.

## Result states

```text
Complete
NoChange
Partial
CandidateOnly
ConflictBlocked
Truncated
NotEvaluated
Cancelled
Failed
```

Validation payloads separately report `Valid`, `Invalid`, or `NotEvaluated`. A completed invalid validation is not an internal service crash.

## Deferred

- automatic source edits or migration execution;
- rule severity or remediation changes;
- runtime/client probes and runtime impact;
- external/Codebase Memory/model/embedding candidates;
- LSP/MCP/daemon/session transport;
- release signing/publication/rollback and CI.

## Completion gate

E4-C implementation is complete only when exact/current acquisition is reproducible and honest; missing shards never trigger hidden builds; search candidates cannot be auto-selected or upgraded; review authorization and proof ceilings are enforced; lineage publication/review is immutable and idempotent; migration remains advisory; impact remains bounded static paths; continuation reopens exact retained artifacts; search-to-context passes one explicitly selected exact root; all failures close resources without publishing false success; CLI is service-only; and all synthetic, pinned addon, Blizzard UI, Reference, ambiguity, review, privacy, cancellation, response-loss, 1/2/N worker, and checksum gates pass.
