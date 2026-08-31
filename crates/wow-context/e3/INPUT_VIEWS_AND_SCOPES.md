# E3-A exact input views, universes, and scope contract

**Status:** normative cross-crate read boundary.

## Principle

`wow-context` consumes exact immutable domain views. It does not open stores, parse source, acquire repositories, rebuild analyzer state, infer joins by name, or refresh `Current` during a request.

## Direct E3-A dependencies

```text
wow-core
wow-reference
wow-project
wow-graph
```

No active dependency on `wow-store`, `wow-emmy`, `wow-search`, `wow-cbm`, `wow-rules`, or `wow-service` in E3-A.

- `wow-project` exposes the coherent published project read view and source-detail seam.
- `wow-graph` exposes exact bounded graph queries and assertion/evidence explanations.
- `wow-reference` exposes exact selected platform API/reference facts and detail handles.
- `wow-core` provides identities, evidence, coverage, conflicts, budgets, source handles, and result primitives.

Store generation identity is carried as context, but raw SQLite/store reads remain behind the published project/graph views.

## Exact input snapshot

```text
ContextInputSnapshot
    ProjectStoreEpochId
    ProjectStoreGenerationId
    ProjectPublicationSetId
    ProjectGenerationId
    ProjectSnapshotId / ProjectViewId
    AnalyzerSnapshotId
    GraphGenerationId
    GraphSnapshotId / GraphViewId
    ProfileId
    ReferenceGenerationId / ReferenceViewId: optional exact set
    SourceUniverseManifestId[]
    project/graph/reference query catalog IDs
    capability/coverage/conflict manifest IDs
    canonical digest
```

All identifiers must describe one coherent published state. `StoreImageId` is not part of the selected E2-D model and is forbidden in current E3-A contracts.

Operational SQLite read transaction and generation lease IDs are not semantic identity but must remain valid for the request lifetime.

## Input acquisition

A higher layer acquires one exact published project state:

```text
resolve Current once or accept exact requested publication
-> acquire one project/store read snapshot and lease
-> open matching ProjectView and GraphView
-> optionally open exact matching ReferenceView
-> construct and validate ContextInputSnapshot
-> invoke wow-context with exact roots
```

After acquisition, E3-A never re-reads `Current` or silently substitutes another retained generation.

## Universe registry

### `first_party_project`

Selected package/TOC/XML/Lua source, analyzer facts, recognizer results, project graph assertions, and source handles for the exact publication.

### `declared_dependency_metadata`

Dependency package declarations and explicitly supplied metadata. It is not first-party source and cannot provide source excerpts unless a separate exact source universe is present.

### `declared_dependency_source`

Optional explicitly configured, licensed, pinned dependency source indexed under its own universe and generation. No automatic dependency download.

### `reference_platform_api`

Exact Reference Pack/API/restriction entities under the selected profile/reference generation. Reference facts remain separate from project-use facts.

### `pinned_platform_ui_source`

Pinned Blizzard UI Lua/XML/TOC source graph produced by an explicit E3-B source/project/graph producer. It includes exact acquisition provider, build/revision, source manifest, license/provenance, project/graph generation, coverage, and source handles.

E3-A can consume this universe only when already published and explicitly requested. It does not acquire, parse, index, or treat it as interchangeable with API documentation.

### Deferred universes

```text
external_implementation
semantic_candidate
runtime_probe
historical_lineage
```

These remain inactive in E3-A unless a later contract defines exact read views and confidence policies. Candidate/external/runtime records never merge into project or platform truth by name.

## Cross-universe joins

A context item may juxtapose universes only through exact registered relations or explicit request grouping. Required fields:

```text
source and target EntityKeys
source/target universe and generation
relation kind/direction
assertion/evidence/coverage/conflict IDs
confidence/provenance
reason path when not direct
```

Forbidden joins:

```text
same display name
same path suffix
same documentation prose
same model embedding
repository popularity
unreviewed alias
```

Project `uses_api` links a project occurrence to a reference API entity; it does not turn platform evidence into project-location evidence or vice versa.

## Root validation

Each requested root yields one of:

```text
ResolvedExact
AbsentWithDomainAuthority
AbsentWithoutAuthority
Partial
Conflict
UnsupportedUniverseOrKind
NotEvaluated
StaleGeneration
```

Context does not manufacture authoritative absence. It preserves the exact project/graph/reference result and its coverage.

## Query catalog requirements

The input snapshot pins compatible registered read catalogs for:

```text
project identity/package/TOC/load/source units
published analyzer facts and source coordinates
project source-handle resolution
graph entity/relation/axis/path/explain queries
reference entity/signature/restriction/detail lookup
coverage/conflict/evidence/source sidecars
```

Missing or incompatible catalog capability is scoped `NotEvaluated`; no raw fallback.

## Pinned Blizzard UI source boundary

E3-A requires E3-B or another reviewed producer to establish:

```text
exact client build/profile
acquisition provider and immutable source snapshot
package/TOC/XML/Lua inventory and content digests
source universe and project generation
analyzer/recognizer/graph producer versions
coverage/conflicts/source handles
license and redistribution policy
coherent publication/read view
```

Context can then build L0/L1 and source excerpts from selected exact roots. It cannot claim the pinned UI source is complete for APIs, runtime behavior, Secret state, taint, or client execution beyond the producer capabilities.

## View lifetime

- all views remain bound to the same exact publication/input snapshot;
- a retained old publication is valid if its lease/view still validates;
- GC/unavailability yields explicit stale/unavailable result;
- continuation cannot migrate to a newer publication;
- errors close provisional views/leases through the owning higher layer;
- context artifacts retain semantic input IDs, not operational connection handles.

## Ownership matrix

| Concern | Owner |
|---|---|
| SQLite/WAL/read transaction/lease | `wow-store` through project publication view |
| Project/current coherence and source detail seam | `wow-project` |
| Graph semantics and bounded graph queries | `wow-graph` |
| Reference/API truth and exact reference detail | `wow-reference` |
| Analyzer implementation and raw internals | `wow-emmy` |
| L0/L1/Project Map/context projection | `wow-context` |
| Root search/ranking | E4 `wow-search` or higher layer |
| Cross-component orchestration | `wow-service` |

## Required tests

- exact coherent project/graph/reference snapshot;
- mismatched epoch/store/publication/project/graph/analyzer/reference IDs;
- `Current` advances after acquisition;
- retained old publication and continuation;
- old publication collected/unavailable;
- first-party/dependency/reference/platform UI universe separation;
- project-use versus platform-contract evidence separation;
- name/path-only cross-universe join mutation;
- platform UI source without explicit producer/profile/coverage;
- reference view omitted when no platform facts requested;
- query catalog missing/incompatible capability;
- root absence under complete versus partial coverage;
- no raw store/analyzer/search fallback.

## Hard stops

- no `StoreImageId` or whole-database-generation assumption;
- no floating `Current`, profile, reference, or source snapshot after acquisition;
- no name/path/prose joins;
- no raw SQLite/analyzer/parser access;
- no automatic Blizzard/dependency/external source acquisition;
- no universe or generation collapse;
- no source/API/runtime completeness claim beyond exact producer coverage.
