# E4-B lineage publication and query model

**Status:** normative.

## Publication inputs

```text
LineagePublicationRequest
    exact LineageUniverseSet and profile set
    exact base LineageGraphSnapshot/generation: optional
    complete producer input partition manifests
    complete proposal/component/review/conflict manifests
    accepted assertion/change/absence/migration manifests
    logical store schema/operation/validation catalogs
    expected counts/digests/golden queries
    retention/object references
    budgets/cancellation
```

No raw SQL, physical path, current selector, source parser, SearchStore connection or arbitrary callback.

## Publication pipeline

```text
validate exact before/after owner and graph views remain retained
-> validate registry/profile compatibility
-> validate complete producer partitions
-> generate/validate candidate components and proposals
-> apply deterministic and reviewed promotion decisions
-> derive accepted assertions, conflicts, change sets, absence decisions and migration records
-> build complete lineage logical partition membership
-> construct registered wow-store operation plan
-> commit target as PublishedInactive
-> open fresh exact read snapshot
-> validate record/evidence/coverage/index/object closure and golden queries
-> seal immutable LineageGraphSnapshot
```

A separate E4-C service/store policy may later manage a convenience current comparison pointer. E4-B core publication is exact-ID-addressed and does not require or resolve a global current lineage snapshot.

## Atomicity

Readers see either the prior exact LineageGraphSnapshot or the new exact snapshot. They never see partial replacement of:

- producer inputs/proposals;
- candidate components;
- review decisions;
- accepted assertions;
- conflicts;
- change/absence records;
- migration records;
- indexes/manifests.

Build/validation/cancellation failure leaves prior snapshots unchanged. A failed target retains its own identity and cannot relabel last-known-good.

## Producer partition replacement

Each producer partition replaces exactly its own scoped inputs/proposals. Updating/disabling `search_lineage_candidate` cannot delete project/reference/review evidence. Dependent accepted assertions/change/migration/impact indexes are recomputed atomically.

Stale base/profile/input generation is rejected; no silent rebase or merge.

## Snapshot identity

`LineageGraphSnapshotId` derives from:

- exact lineage universe/comparison bindings;
- relation and profile registry IDs;
- ordered complete producer partition manifests;
- proposals/components/reviews/assertions/conflicts;
- change/absence/migration manifests;
- logical store schema/operation/validation catalogs;
- capability/coverage manifests;
- contract/canonicalization versions.

It excludes physical row/page/order, WAL/checkpoint state, process/host/time, reviewer note rendering, cache state and mutable current pointers.

## Read view

```text
LineageGraphView
    exact snapshot/generation/universe/profile IDs
    registered read/query catalogs
    capability/coverage/conflict summaries
    exact retained owner/graph/reference/search generation guards
```

A read view never switches snapshots.

## Public query operations

### `compare_entity_generations`

Returns exact before/after entity refs, accepted lineage/replacement assertions, Candidate/Possible proposals, typed changes, conflicts, coverage, migration records and detail handles.

### `trace_lineage`

Traverses accepted lineage relations over an explicit generation sequence under depth/path/budget constraints. Candidate/Possible inclusion is opt-in and remains labeled. Multi-hop paths remain paths.

### `explain_lineage_assertion`

Returns every producer proposal, evidence/source/reference/search record, proof-ceiling calculation, review decision, conflict and coverage record that supports or limits an assertion.

### `classify_generation_changes`

Returns an exact `GenerationChangeSet` or a scoped partial/NotEvaluated result. It never regenerates owner facts.

### `propose_migration_candidates`

Returns bounded advisory candidates from accepted transition/change evidence and Candidate search/shape signals. It cannot claim replacement or recipe validation.

### `validate_migration_recipe`

Read-only validation of one exact recipe under exact source/target generations and profiles. It does not execute edits/tests/client probes.

### `plan_static_impact`

Produces a deterministic bounded query plan over exact change roots and target graph snapshots.

### `run_static_impact`

Executes the plan and emits reason-path-preserving static impact results.

### `explain_static_impact`

Returns exact root, path, relation, evidence, proof-cap, coverage/conflict and omission records.

### `validate_lineage_graph_snapshot`

Read-only full logical validation; no repair.

## Query request common fields

```text
exact LineageGraphSnapshotId
exact before/after generation guards
exact root entity/assertion/change/migration IDs
relation/change/impact whitelist
proof/confidence/provenance policy
coverage/conflict policy
finite depth/fanout/path/item/byte/time budgets
continuation: optional
cancellation
```

No fuzzy name, natural-language prompt, raw SQL, executable predicate or arbitrary graph program.

Search/service can resolve a query to exact entity/proposal IDs before invoking graph queries.

## Result states

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

`CandidateOnly` never renders as accepted lineage. Empty results never imply removal/introduction/absence without explicit `GenerationAbsenceDecision` authority.

## Ordering

Canonical ordering uses exact:

```text
universe and before/after generation roles
relation registry order
source/target generation entity keys
proof/confidence class
producer class/ID/version/partition
proposal/assertion/change/migration/impact path IDs
```

Never source/SQL/hash iteration, score completion, reviewer chronology or popularity.

## Pagination and continuation

Pages contain whole assertions/components/change records/migration records/impact paths plus mandatory explanation closure. Continuation binds:

- exact snapshot and input generations;
- normalized request/profile;
- result manifest;
- stable ordering/frontier;
- selected/omitted/conflict state;
- cumulative budgets;
- prior page-chain digest;
- integrity digest.

It never resolves current, resets budgets or hides previous ambiguity/truncation.

## Validation catalog

At minimum:

- exact generation/universe/profile closure;
- no generation-local entity mutation/merge;
- complete producer partition membership;
- proposal/evidence/source/coverage closure;
- proof ceilings and review authorization;
- ambiguity and multiplicity constraints;
- removal/introduction authority;
- change-facet typed state correctness;
- replacement/migration distinction;
- impact direct/path and confidence caps;
- reverse/index closure;
- deterministic golden compare/trace/change/migration/impact queries;
- no stale/cross-generation data;
- privacy/license/security/budget/cancellation state.

## No repair

Validation failure produces `Invalid` or typed failure/quarantine. It does not choose a candidate, modify review, weaken a relation schema, fill coverage, rewrite IDs, apply a recipe, or update a snapshot in place.
