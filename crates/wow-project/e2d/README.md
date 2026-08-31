# `wow-project` E2-D integrated publication contract

**Status:** implementation-ready documentation; no Rust code.

**Contract ID:** `wow-project/e2-d/integrated-project-graph-publication`

## Mission

Atomically publish one coherent project state from an exact E2-C `ProjectIndexCandidate` and validated graph proposals into one immutable E2-D ProjectStore generation, one GraphSnapshot, one ProjectSnapshot, and one publication head.

```text
exact E2-C ProjectIndexCandidate
+ exact current ProjectPublicationHead/base store+graph snapshots
+ exact E2-A graph registry and proposal validation
+ exact E2-D ProjectStore profile
-> validate full generation/coherence closure
-> build project logical write plan
-> ask wow-graph for graph partition replacement/write plan
-> compose ProjectPublicationBundle
-> invoke wow-store staging transaction and seal
-> reopen exact sealed generation read-only
-> validate ProjectView and GraphView golden results
-> build ProjectSnapshotManifest and GraphSnapshotManifest
-> build one coherent ProjectPublicationHead
-> compare-and-swap head
```

## Direct dependencies

```text
wow-core
wow-store
wow-emmy
wow-graph
wow-recognizers
```

`wow-project` consumes E2-C candidate records and exact component identities. It does not depend on `wow-reference`, `wow-rules`, `wow-search`, `wow-context`, `wow-cbm`, `wow-service`, or applications.

## Ownership

`wow-project` owns:

- target/base/head selection policy;
- candidate and generation coherence validation;
- project logical records and snapshot manifest;
- orchestration of graph proposal-to-assertion planning;
- publication bundle and expected cross-domain manifests;
- post-seal project/graph read validation;
- coherent head payload;
- target failure, inactive generation, and last-known-good reporting.

`wow-store` owns physical transaction, artifact sealing, read handles/leases, head CAS primitive, recovery inventory, retention, and GC.

`wow-graph` owns graph semantic keys, assertions, conflicts, coverage, write plan, snapshot manifest validation, and GraphView.

## E2-D output

```text
PublishedProjectGeneration
    ProjectPublicationHead
    ProjectStoreGeneration/Artifact
    ProjectSnapshotManifest
    GraphSnapshotManifest
    AnalyzerSnapshot binding
    Recognizer result/proposal validation manifests
    capability/conflict/coverage summaries
    publication/validation reports
```

A failed target never returns a mixed or relabeled last-known-good result.

## Public operations

```text
validate_project_publication_request
validate_project_index_candidate_for_publication
resolve_project_publication_base
build_project_logical_write_plan
build_graph_partition_replacement_request
validate_graph_replacement_plan
build_project_publication_bundle
invoke_project_store_generation_build
validate_sealed_project_store_generation
build_project_and_graph_snapshot_manifests
build_project_publication_head
compare_and_swap_project_publication_head
open_published_project_view
recover_sealed_inactive_project_publication
classify_project_publication_result
```

## Completion gate

E2-D is complete only when crash/fault/cancel tests prove readers see old or new coherent heads; all project, analyzer, recognizer, graph, profile, reference, store, and artifact identities agree; stale base/head never overwrites; graph rejections/conflicts remain visible; post-seal validation is independent; inactive sealed generations require exact adoption revalidation; old reader leases remain stable; last-known-good keeps its original identity; and deterministic logical inputs produce byte-identical canonical publication bundles, snapshot manifests, and heads.
