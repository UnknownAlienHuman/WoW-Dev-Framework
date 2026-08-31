# Agent workstreams and integration order

**Status: operational routing**

This file prevents agents from implementing every planned component at once, binding code to documentation-only prerequisites, or inventing incompatible cross-crate seams.

## Ownership model

- One agent owns one work package and one primary crate.
- Shared contracts change before dependent implementation starts.
- A separate integration/review agent validates each wave.
- Documentation-ready is not code-ready while prerequisite implementations, pins, and checksum freezes are absent.
- No agent implements another crate's responsibility as a local workaround.
- No package adds CI, workflows, source execution, editor mutation, or runtime claims by convention.

## Global order

```text
E0-A wow-core
-> E0-B wow-reference fixture + E0-C wow-emmy
-> E0-D minimal wow-project
-> E0-E wow-rules
-> E0-F wow-service + apps/wow

-> E1-A wow-store
-> E1-B persistent wow-reference
-> E1-C wow-annotations
-> E1-D Reference Pack build/validation

-> E2-A wow-graph
-> E2-B wow-recognizers
-> E2-C wow-project TOC/XML/load/invalidation candidate
-> E2-D ProjectStore + coherent project/graph publication

-> E3 wow-context + Blizzard/project graph/skeletons/Project Map
-> E4 wow-search + lineage/impact
-> E5 calibration packs
-> E6 Codebase Memory candidate bridge
-> E7 LSP/MCP/release/publishing
```

Parallel documentation is allowed. Rust implementation obeys the prerequisite gates.

## E0 vertical slice

### E0-A — `wow-core`

Identity, generation, source handles, evidence/conflicts, coverage/NotEvaluated, findings, warnings, budgets, and canonical envelopes.

### E0-B — fixture `wow-reference`

One synthetic profile/catalog, restricted evaluation, exact lookup, coverage, and Secret-facet fixture.

### E0-C — `wow-emmy`

Exact upstream pin/probe, analyzer actor/snapshots, normalized Lua facts, diagnostics, and coordinates.

### E0-D — minimal `wow-project`

One closed first-party Lua workspace, source registry, project generation, analyzer binding, and immutable snapshot.

### E0-E — `wow-rules`

Only `wow.api.exists` and one direct local Secret operation rule.

### E0-F — `wow-service` + `apps/wow`

`status`, `check`, coherent context, raw finding preservation, structured root-cause presentation, and thin CLI.

## E1 Reference Pack

### E1-A — `wow-store`

Domain-neutral SQLite/object persistence and immutable ReferenceStore publication.

### E1-B — persistent `wow-reference`

Pinned source, restricted APIDocumentation evaluation, raw metadata, normalization, corrections, coverage, ReferenceStore schema/build plan, and ReferenceView.

### E1-C — `wow-annotations`

Semantic model, explicit type lowering, deterministic inert rendering, source maps/loss, parity, and consumer probes.

### E1-D — pack build/validation

Cross-component build, independent nonrepairing validation, deterministic rebuild comparison, assembly, and thin builder CLI.

## E2 project and graph

### E2-A — `wow-graph`

Versioned registries, semantic keys, evidence-bearing assertions, conflicts/coverage, producer partitions, immutable graph snapshots, explicit axes, and bounded queries.

### E2-B — `wow-recognizers`

Bounded declarative pack schema, normalized fact input, deterministic matching, core TOC/XML/frame/signal/hook/library/state rules, proposed assertions, ambiguity/coverage, producer versioning, and mutation evaluation.

### E2-C — `wow-project` index candidate

Contract: [`wow-project/e2/`](wow-project/e2/README.md).

Owns source snapshot validation, one TOC variant, bounded TOC/XML parsing, static load model, virtual Lua units, analyzer/recognizer orchestration, graph-proposal validation, and dependency-driven invalidation.

Output remains:

```text
ProjectIndexCandidate
persistent_publication_state = NotPublishedE2C
```

### E2-D — ProjectStore and integrated publication

Primary physical contract: [`wow-store/e2/`](wow-store/e2/README.md).

Coordinator contract: [`wow-project/e2d/`](wow-project/e2d/README.md).

#### `wow-store` owns

```text
file-per-generation ProjectStore physical profile
one-writer staging transaction
registered schema/operation/validation catalogs
content-addressed object references
sealed immutable store generation
read-only snapshot handles and leases
generation registry and atomic head compare-and-swap primitive
recovery classification
retention roots and mark-and-sweep GC
physical/logical integrity reports
```

#### `wow-project` owns

```text
exact E2-C candidate selection
coherent ProjectGeneration/AnalyzerSnapshot/Recognizer/Graph input closure
project and graph logical write plans
ProjectPublicationBundle
post-seal ProjectView and GraphView golden validation
ProjectSnapshotManifest and GraphSnapshotManifest coherence
single ProjectPublicationHead
last-known-good and failed-target reporting
```

#### Selected E2-D physical baseline

```text
one immutable SQLite database per ProjectStore generation
shared content-addressed object store with generation reference manifests
staging WAL allowed only before sealing
published generation opened read-only
one atomic coherent head record, never separate project and graph current pointers
```

Row-versioned single-database publication remains a deferred measured alternative, not an E2-D implementation choice.

#### E2-D publication sequence

```text
validate exact candidate/base/head
-> compose registered project + graph write/validation plans
-> build ProjectPublicationBundle
-> wow-store creates one staging generation
-> one transaction writes all logical partitions and manifests
-> validate, commit, checkpoint, close, seal, and atomically materialize immutable generation
-> reopen exact generation read-only
-> wow-project and wow-graph run golden validation
-> construct exact snapshot manifests
-> compare-and-swap one ProjectPublicationHead
-> old readers continue on leased old generation
```

No reader observes mixed project/analyzer/recognizer/graph/store generations.

#### E2-D handoff gate

```text
one exact ProjectIndexCandidate and expected current head
one exact graph registry/base snapshot and complete proposal-validation report
store plan uses only registered operations; no SQL/callbacks cross the seam
all project and graph logical records written in one generation transaction
post-seal read-only reopen succeeds
ProjectView and GraphView reproduce candidate/manifests/golden queries
one head record binds store/project/graph/analyzer/recognizer/profile/reference identities
head CAS conflict never overwrites another publication
failure before head advance leaves prior head unchanged
sealed inactive generation is classified and recoverable only by exact revalidation
old reader lease remains stable after new head publication
retention/GC never removes current, last-known-good, pinned, leased, recovery, or evidence-rooted generations/objects
crash/cancel/fault injection at every phase yields old-or-new, never mixed
1/2/N and shuffled logical inputs produce identical logical generation/manifests
no raw SQL, in-place mutation, age-only GC, last-known-good relabel, editor/client/network/source execution, or CI
```

## E3–E7 routing

### E3

`wow-context`, full project/Blizzard UI graph inputs, L0/L1 skeletons, Project Map, and bounded context metrics.

### E4

`wow-search`, explicit lineage, migration and impact. Similarity never authorizes replacement.

### E5

Pinned audited calibration packs emitting universal roles only.

### E6

Optional Codebase Memory candidate bridge; external results remain Candidate.

### E7

LSP/MCP transports, release signing/publication/activation, and operational automation after explicit owner approval.

## Seam request format

```text
requesting work package/crate
owning crate
required operation/data contract
current workaround rejected
why orchestration/read-view cannot solve it
smallest proposed seam
cycle/security/evidence impact
fixture/mutation proving it
implementation/freeze prerequisite impact
```
