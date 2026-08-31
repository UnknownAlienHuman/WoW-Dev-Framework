# E3-A publication and downstream context handoff

**Status:** normative cross-crate boundary.

## Publication model

E3-A uses the generic E2-D ProjectStore protocol. Platform-source data is not special-cased inside `wow-store`.

```text
validated BlizzardUiIndexCandidate
-> exact GraphPublicationPlan
-> BlizzardUiPublicationBundle
-> immutable project/graph partition versions
-> complete target generation membership
-> PublishedInactive ProjectStoreGeneration
-> fresh exact project/graph/store read-back validation
-> current-record compare-and-swap activation
-> BlizzardUiProjectView
```

## Separate store/project identity

The platform source corpus has its own `ProjectStoreId`, `ProjectId`, source universe, and current publication record. A user addon project references an exact platform-source publication in a later combined view; it does not copy or merge source rows into its own current generation.

## Coherent publication set

The semantic publication set binds:

```text
source profile/snapshot
client/build/interface/reference compatibility set
ProjectGeneration / ProjectSnapshot
AnalyzerSnapshot
core recognizer partitions
GraphGeneration / GraphSnapshot
source/package/load/fingerprint/skeleton-input manifests
coverage/conflict/truncation state
schema/operation/validation profiles
```

The store generation binds the complete semantic set afterward, preserving the E2-D noncyclic identity order.

## Validation gates

A fresh inactive-generation read must verify:

- exact source/profile/project/analyzer/recognizer/graph/store identities;
- complete configured-root/package/file/unit accounting;
- selected TOC variants and no cross-flavor merge;
- source/package/load/XML/analyzer/recognizer/graph partition manifests;
- accepted/rejected proposal closure;
- conflicts/coverage/truncation/NotEvaluated propagation;
- removed-input/stale-row/source-handle closure;
- object/license/provenance references;
- exact graph golden queries;
- skeleton-input reconstruction checks;
- cross-generation/universe leakage absence.

Only then may current activation occur.

## Failure behavior

- materialization/index candidate failure produces no bundle;
- build failure/cancellation leaves current unchanged;
- validation failure quarantines/retains target under policy;
- stale CAS leaves validated target inactive;
- provider outage after materialization has no effect;
- last-known-good/current/failed-target/validated-inactive/rollback identities remain distinct;
- no implicit fallback is reported as the target build.

## Published read view

`open_blizzard_ui_project_view` requires an exact publication identity or an explicitly resolved current record and returns one snapshot-bound read transaction/lease.

Ordinary operations are bounded:

```text
package_exact
file_exact
entity_exact
source_span_exact
load_neighbors
structural_neighbors
bounded_path
coverage_and_conflicts
```

No raw DB handle, SQL, table, whole-graph dump, or unbounded source export.

## Skeleton-input handoff to `wow-context`

`open_blizzard_ui_skeleton_input_view` exposes only structured, snapshot-bound inputs:

```text
exact source/project/graph/analyzer IDs
entity kinds and canonical display labels
signatures/types/declaration spans
package/file/load/lifecycle roles
direct relations and bounded reason paths
source/evidence/provenance/coverage/conflict records
bounded comment/doc/source-slice handles under policy
deterministic ordering and continuation
```

`wow-context` owns:

- Project Map shape;
- L0/L1 skeleton schema and rendering;
- context-query intent and root selection;
- byte/token budgets and allocation;
- relevance ordering/pruning;
- context-pack manifests;
- prompt-injection presentation boundaries;
- stop/no-new-evidence policy.

E3-A does not pre-render summaries or cache model-specific context strings.

## Combined universe set

A later context/service request may bind:

```text
exact user ProjectSnapshot
exact BlizzardUi ProjectSnapshot
exact user GraphSnapshot
exact BlizzardUi GraphSnapshot
exact ReferenceProfile/ReferenceGeneration
optional exact external candidate/runtime generations
```

The binding is a separate immutable request/context identity. Opening one source view never silently changes another source/profile generation.

## Redistribution

Default skeleton-input reads return handles and bounded local excerpts under caller policy. Publication does not create a release artifact containing Blizzard UI source. Any copied excerpt or redistributed source artifact requires explicit license/provenance handling and separate validation.
