# E4-B public operations

**Status:** normative transport-independent operation contracts.

Every operation accepts exact retained generation/profile IDs, closed typed inputs, finite budgets and cancellation. No operation accepts raw source paths, SQL, SearchStore connections, fuzzy natural-language roots, executable predicates, callbacks, plugins, model prompts or mutable current handles.

## Profile and universe operations

### `validate_lineage_profiles`

Validates the immutable relation/change/migration/impact registry, producer schemas, proof ceilings, blocking/component/review/coverage/security/store profiles and their compatibility.

Returns a validation report; it never fills unknown fields, raises proof ceilings or selects a physical profile without executable evidence.

### `bind_lineage_universe_set`

Binds exact compatible before/after owner publications, GraphSnapshots, source/reference profiles and optional E4-A SearchShard views into one immutable `LineageUniverseSet`.

It never resolves `current`, selects nearest builds or mixes universe classes.

### `validate_lineage_input_partitions`

Validates producer partition scope, schema, exact generation bindings, entity/evidence/source/coverage/conflict closure, proof ceiling and budgets before candidate generation.

## Candidate and proposal operations

### `generate_lineage_candidate_components`

Runs reviewed bounded blocking stages over exact before/after entity inputs and producer candidates, emits immutable `LineageProposal` records, builds deterministic bipartite candidate components and records every overflow/skip/ambiguity state.

It performs no unrestricted all-pairs comparison and accepts no proposal as truth.

### `validate_lineage_proposals`

Validates relation kinds, endpoints, producer classes, evidence, confidence ceiling, component membership, coverage/conflicts and candidate identity. Search/fingerprint proposals remain Candidate.

### `apply_lineage_review_decisions`

Applies exact attested deterministic/manual review decisions under the minimum proof-ceiling rule. It emits accepted/rejected/deferred/conflicted/superseded results without modifying proposals.

## Publication operations

### `plan_lineage_graph_publication`

Produces a complete immutable logical partition membership and registered `wow-store` operation/validation plan for proposals, components, reviews, accepted assertions, conflicts, change/absence/migration records and indexes.

### `publish_lineage_graph_snapshot`

Builds an inactive lineage generation, reopens it through a fresh exact read snapshot, runs logical/integrity/golden validation and seals one immutable `LineageGraphSnapshot`.

No global current pointer is required or resolved in E4-B.

### `open_lineage_graph_view`

Opens one exact retained sealed snapshot with generation lease and registered bounded read/query catalogs.

## Lineage and change operations

### `compare_entity_generations`

Given exact before/after entity refs or one accepted assertion, returns all accepted/Candidate/Possible lineage/replacement assertions, typed changes, producer proposals, reviews, conflicts, coverage and detail handles.

### `classify_generation_changes`

Compares exact typed owner/reference/relation states for accepted entity pairs and evaluates scoped before-only/after-only absence decisions. It preserves Missing/Null/Unknown/Unsupported/Omitted/Conflict/NotEvaluated distinctions.

### `trace_lineage`

Traverses explicit accepted lineage relations over an exact generation sequence. Candidate/Possible relations are opt-in and remain labeled. Multi-hop continuity remains an ordered path unless a separate reviewed derived assertion exists.

### `explain_lineage_assertion`

Returns complete proposal/producer/evidence/source/reference/search/review/proof-ceiling/coverage/conflict arithmetic and nonclaims for one assertion.

## Migration operations

### `propose_migration_candidates`

Creates bounded advisory candidates from explicit transition/deprecation/replacement records, accepted changes and Candidate search/shape evidence. It cannot emit a validated recipe or edit.

### `validate_migration_recipe`

Read-only validation of exact source/target scope, governing assertions, preconditions, typed transformation steps, constraints, postconditions, validation requirements, proof ceiling, coverage/conflicts, privacy/license/security and canonical bytes.

## Static-impact operations

### `plan_static_impact`

Validates exact change roots and target graph snapshots and emits a deterministic bounded traversal plan over reviewed relation kinds/directions/confidence classes.

### `run_static_impact`

Executes the plan, returning direct and bounded transitive affected entities with exact reason paths, confidence caps, evidence, coverage, conflicts, omissions, budgets and continuation.

### `explain_static_impact`

Returns the exact root/change/lineage assertion, every direct path edge, producer/evidence/coverage/conflict record, confidence cap and explicit static/runtime nonclaims.

## Validation operation

### `validate_lineage_graph_snapshot`

Read-only validation of:

- exact universe/generation/profile closure;
- registry and complete producer partitions;
- proposal/component/review/assertion/conflict integrity;
- proof ceilings and ambiguity handling;
- change/absence/replacement/migration semantics;
- impact indexes/queries and direct/path distinction;
- store/object/index/retention closure;
- privacy/license/security/budget/cancellation state;
- canonical IDs/digests and golden queries.

It never repairs, picks a candidate, changes a review, fills coverage or executes a migration.

## Common request requirements

```text
exact operation/request ID
exact LineageUniverseSet and profile set
exact snapshot/base/input generation guards
exact root entity/proposal/assertion/change/migration IDs where applicable
closed relation/change/facet/impact filters
proof/confidence/provenance/coverage policy
finite pair/component/traversal/output/time/memory budgets
cancellation and optional exact continuation
```

## Common result requirements

Every result reports:

- exact operation/universe/snapshot/profile IDs;
- input and producer partition manifests;
- accepted/Candidate/Possible/NotEvaluated distinctions;
- evidence/provenance/confidence/proof ceiling;
- coverage/conflicts/ambiguity/omissions/truncation;
- budgets/cancellation/continuation;
- validation and canonical result identity;
- explicit nonclaims for replacement/migration/impact/runtime where relevant.

## Status model

```text
Complete
Partial
CandidateOnly
ConflictBlocked
NotEvaluated
Truncated
NoChange
Cancelled
Failed
Invalid
```

Empty output is never implicitly complete or authoritative.

## Idempotency

Build/publication operations use exact operation ID plus canonical request digest. Same ID/same digest resumes/returns exact durable state; same ID/different digest is rejected. Read/query operations are pure for exact inputs.

## E4-C orchestration boundary

[`wow-service/e4`](../../wow-service/e4/README.md) owns symbolic current resolution, fixed-order acquisition/release of project/reference/search/lineage/context views, review-authorization adapter calls, idempotency/retention, explicit candidate selection, search-to-context handoff, and public service/CLI envelopes.

`wow-graph` continues to accept only exact inputs and never imports or calls `wow-service`. The E4-C contract cannot change graph proof ceilings, choose a lineage candidate, repair a snapshot, apply a migration, or reinterpret static impact as runtime truth.
