# E4-B owner seams, producer partitions, and publication

**Status:** normative cross-crate boundary. `wow-graph` owns lineage assertions and views; it does not import higher owner crates.

## Dependency rule

Direct framework dependencies remain:

```text
wow-core
wow-store
```

No direct dependency is added from `wow-graph` to:

```text
wow-search
wow-project
wow-reference
wow-service
wow-context
wow-rules
apps
```

Cross-owner data arrives as exact typed artifacts assembled by orchestration. E4-C `wow-service` will acquire and coordinate owner views.

## `wow-project` seam

Project/source owners provide exact generation-bound records:

```text
ProjectLineageInputView
    exact Project/Source/Graph generation IDs
    entity/source/file/package/load/unit manifests
    analyzer/recognizer fact and source-handle manifests
    stable project/source IDs under owner contracts
    structural fingerprint records
    direct project-use and relation records
    capability/coverage/conflict/license/privacy state
    bounded exact detail operations
```

For Blizzard UI source, E3-A provides the exact source/build/profile and `SkeletonInputView`-compatible structural records. Provider/repository labels remain provenance only.

`wow-project` does not decide lineage assertions or graph publication. It may emit explicit source-manifest continuity records under its own stable-ID contract.

## `wow-reference` seam

Reference owner provides:

```text
ReferenceLineageInputView
    exact from/to ReferenceProfile and ReferenceGeneration IDs
    exact API/type/event/enum/restriction entities and facts
    explicit alias/deprecation/replacement/transition records
    corrections and correction applicability
    stable reference entity keys where contractually defined
    source/evidence/coverage/conflicts/negative authority
    bounded exact detail operations
```

`wow-reference` remains authority for public API/restriction facts. `wow-graph` cannot infer an API replacement or restriction transition from implementation source alone.

## E4-A `wow-search` seam

Search supplies optional candidate evidence:

```text
LineageSearchCandidateBundle
    exact SearchUniverseSet/shard/generation IDs
    exact old/new endpoint query mappings
    SearchRequest/NormalizedQuery/Result IDs
    candidate entity and every contributing signal
    authority band, rank vector, explanation
    matched field origins
    coverage/conflicts/omissions/truncation
    canonical digest
```

Rules:

- only exact E4-A records are admitted;
- signals remain Candidate;
- raw floating search scores are diagnostic only;
- top rank has no special proof status;
- search snippets are not source evidence;
- `wow-graph` never invokes search itself;
- missing search input cannot block explicit/stable-ID lineage unless the active profile requires a candidate-recall evaluation.

## `wow-service` seam

E4-C will own:

- exact/current selector resolution before canonical requests;
- retained from/to project/source/reference/search/graph view acquisition;
- compatibility validation across independent stores;
- optional E4-A candidate generation;
- construction of exact `LineageInputBundle` and migration/impact requests;
- operation sequencing and resource release;
- canonical result envelopes and CLI projection.

`wow-service` does not implement matching, proof ceilings, change classification, migration eligibility, or impact traversal.

## Existing `wow-graph` seam

E4-B extends the graph registry/publication model with:

```text
cross-generation relation kinds
lineage proposal/review/assertion records
ambiguity groups
lineage producer partitions
LineageSnapshot/View
change/migration/impact query artifacts
```

Existing E2 semantic entity/relation assertions remain unchanged. E4-B relations connect exact generation-scoped endpoints.

## Producer classes

Initial producer partitions:

```text
lineage.explicit-owner-transition:<owner-profile>:<generation-pair>
lineage.stable-owner-id:<profile>:<generation-pair>
lineage.deterministic-structural:<profile>:<generation-pair>
lineage.search-candidate-import:<search-profile>:<generation-pair>
lineage.review-decision:<review-profile>:<generation-pair>
lineage.change-classification:<profile>:<generation-pair>
lineage.migration-evidence:<profile>:<generation-pair>
```

Static impact results are query artifacts by default, not persisted direct graph relations. A future cache/index partition must remain derived, rebuildable, and exact-request-bound.

## Partition replacement

One partition key binds:

```text
producer ID/version
exact GenerationPairId
relation/input/profile scope
source owner manifest IDs
canonical input digest
```

Replacement is atomic:

- validate entire target batch;
- remove only stale records from that producer partition;
- preserve other producer assertions/candidates/decisions;
- recompute affected ambiguity/conflict/view state;
- publish a new immutable LineageSnapshot/GraphGeneration binding;
- never modify the prior snapshot.

Disabling a candidate producer removes only its candidate records and downgrades candidate-lane coverage. It cannot remove accepted explicit assertions owned by another producer.

## Review publication

A review decision is a new immutable input. Accepting a Candidate requires qualifying evidence appropriate to the desired proof class or retains Candidate/Possible status. Review cannot bypass relation schema or proof ceiling.

A superseding review references the prior decision. The old decision remains traceable.

## Publication protocol

```text
validate GenerationPair and owner input bundles
-> build/validate producer proposal partitions
-> solve bounded ambiguity components
-> apply explicit reviewed decisions
-> validate final lineage assertions and conflicts
-> derive change-set capability records
-> build graph/store registered logical operation plan
-> commit PublishedInactive graph/lineage target through wow-store
-> reopen exact retained snapshot
-> validate assertion/partition/evidence/coverage/change golden queries
-> activate through the accepted graph/project publication protocol or publish an exact sidecar lineage snapshot, as frozen by implementation profile
```

The physical integration profile must be frozen before Rust implementation. It must preserve exact existing project/graph generations and avoid mixed lineage/source views.

## Selected logical publication shape

E4-B documentation selects a **lineage sidecar snapshot bound to exact retained graph generations** as the default logical model:

```text
LineageSnapshot
    separate immutable logical publication
    exact references to from/to GraphSnapshot and owner generations
    own producer partitions/current selector at service catalog level
```

Rationale:

- lineage compares two immutable generations and should not rewrite either;
- multiple generation pairs may coexist;
- rebuilding candidate/review partitions does not republish source graphs;
- project/reference/source current pointers remain independent;
- E4-C can acquire exact lineage snapshots explicitly.

The physical store can share `wow-store` infrastructure, but raw tables/transactions remain private.

## No implicit current

`LineageSnapshot` identity contains exact endpoints and has no floating current. A catalog may map an exact pair/profile to a validated latest lineage computation, but service resolves that mapping once and records the exact snapshot ID.

## Change-set publication

A `GenerationChangeSet` is derived from one exact lineage snapshot and owner endpoint manifests. It can be stored as a sealed derived artifact or rebuilt. It never becomes an owner fact and never mutates endpoint generations.

## Migration evidence publication

Migration evidence/recipe candidates are exact request/profile artifacts. Accepted owner transition assertions remain graph facts; project-use-specific migration plans do not become universal graph edges.

## Store boundary

`wow-graph` owns logical schemas, registered operations, validation catalogs, manifests, and query semantics.

`wow-store` owns:

- SQLite/file/object/transaction lifecycle;
- one-writer and stale-base behavior;
- staging/finalization;
- read snapshots and leases;
- integrity/recovery/retention/GC;
- physical indexes.

No raw SQL, table, row ID, PRAGMA, ATTACH, connection, transaction callback, or filesystem path crosses E4-B public APIs.

## Retention and GC

Retain exact endpoint and lineage snapshots for:

- active readers;
- continuation/review queue;
- change/migration/impact artifacts;
- evaluation/golden corpora;
- explicit evidence/debug/rollback policy.

GC requires closure:

```text
LineageSnapshot
-> generation pair
-> endpoint project/reference/source/graph snapshots
-> producer partitions and assertions/candidates/decisions
-> evidence/source/reference handles
-> change/migration/impact artifacts and continuations
```

No age-only deletion.

## Failure behavior

- owner input mismatch: no publication;
- candidate component budget failure: explicit partial/NotEvaluated partition;
- store build/validation failure: current/catalogn mapping unchanged;
- response loss: idempotent operation receipt/query, no duplicate publication;
- stale activation/catalog CAS: target remains validated inactive/exact; no rebase;
- endpoint GC during acquisition: fail exact operation, no fallback;
- review conflict: preserve both and block dependent acceptance;
- cancellation: no background continuation.

## Validation

- no reverse dependency added;
- every imported owner/search record exact and generation-compatible;
- candidate proof ceiling preserved;
- producer partitions independently replaceable;
- prior endpoint/lineage snapshots immutable;
- sidecar snapshot references exact retained endpoints;
- fresh read-back reproduces assertions/change queries;
- no cross-generation leakage;
- retention/GC closure complete;
- deterministic logical publication identity.
