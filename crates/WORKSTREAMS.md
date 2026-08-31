# Agent workstreams and integration order

**Status:** operational routing through documentation frontier E2-D.

This file prevents agents from coding against documentation-only prerequisites, activating every planned crate, or inventing cross-crate seams locally.

## Ownership rules

- One agent owns one work package and one primary crate.
- Shared seam changes are proposed before dependent implementation.
- A separate integration/review agent verifies each wave.
- Documentation-ready remains `implementation_state=not-started` until code, executable probes, fixtures, and checksums exist.
- Missing tools, probes, benchmarks, runtime checks, or dependency implementations are `skipped`/blocking, never pass.
- Current patch-sensitive WoW claims route through `wow-addon-engineering-kb`; stable framework contracts do not duplicate live guidance.

## Global implementation order

```text
E0-A wow-core
-> E0-B wow-reference fixture + E0-C wow-emmy
-> E0-D wow-project fixture generation
-> E0-E wow-rules
-> E0-F wow-service + apps/wow

-> E1-A wow-store foundation
-> E1-B persistent wow-reference
-> E1-C wow-annotations
-> E1-D Reference Pack build/validate

-> E2-A wow-graph
-> E2-B wow-recognizers
-> E2-C wow-project TOC/XML/load/incremental candidate
-> E2-D ProjectStore + coherent graph/project publication

-> E3 wow-context + Blizzard/project graph/skeletons/Project Map
-> E4 wow-search + lineage/impact
-> E5 calibration packs
-> E6 Codebase Memory candidate bridge
-> E7 LSP/MCP/release/publishing
```

Parallel documentation is allowed. Rust implementation still follows the dependency/freeze order.

## E0 vertical slice

### E0-A — `wow-core`

[`wow-core/`](wow-core/README.md): identity, generations, source handles, evidence/conflicts, coverage/NotEvaluated, findings/warnings/budgets, canonical envelopes. Blocks all dependent code.

### E0-B — fixture `wow-reference`

[`wow-reference/`](wow-reference/README.md): one synthetic profile/catalog, restricted evaluation, exact lookup, complete/partial/conflict coverage, restriction fixture. Requires implemented E0-A.

### E0-C — `wow-emmy`

[`wow-emmy/`](wow-emmy/README.md): pinned upstream adapter, analyzer actor/snapshots, normalized Lua facts, generic diagnostics, exact source coordinates. May implement in parallel with E0-B after E0-A.

### E0-D — minimal `wow-project`

[`wow-project/`](wow-project/README.md): one first-party workspace, coherent ProjectGeneration, source registry, analyzer binding, immutable project snapshot.

### E0-E — `wow-rules`

[`wow-rules/`](wow-rules/README.md): only `wow.api.exists` and one direct local Secret-value rule.

### E0-F — `wow-service` + `apps/wow`

[`wow-service/`](wow-service/README.md), [`../apps/wow/`](../apps/wow/README.md): `status`, `check`, coherent context, raw finding preservation, structured causal presentation, canonical result, thin CLI.

## E0 integration gate

```text
one exact profile/reference/project/analyzer generation
known fixture API resolves
unknown fixture API only produces a WoW finding under authoritative absence
generic finding remains visible
Secret unsafe/guarded cases classify correctly
missing/conflicted capability produces NotEvaluated
reference and project evidence remain separate
1/2/N output deterministic
no source/editor/client mutation
```

## E1 Reference Pack

### E1-A — `wow-store`

[`wow-store/`](wow-store/README.md): schema/migrations, immutable ReferenceStore, object store, validation, generic publication foundation.

### E1-B — persistent `wow-reference`

[`wow-reference/e1/`](wow-reference/e1/README.md): pinned source, restricted APIDocumentation evaluator, raw metadata, corrections, exact coverage, ReferenceStore plan, ReferenceView.

### E1-C — `wow-annotations`

[`wow-annotations/e1/`](wow-annotations/e1/README.md): semantic model, explicit type lowering, deterministic inert rendering, source maps/loss, Ketho parity, EmmyLua/LuaLS probes.

### E1-D — Reference Pack build/validation

[`wow-service/e1/`](wow-service/e1/README.md), [`../apps/wow-reference-builder/`](../apps/wow-reference-builder/README.md): cross-component build, nonrepairing validation, deterministic rebuild comparison, pack layout, thin CLI. Signing/publication remains deferred.

## E1 integration gate

```text
exact source/profile/component pins
immutable ReferenceStore validated/read-only
raw unknowns and correction expiration retained
negative authority honest
annotation loss/injection/editor gates pass
oracle/consumer discrepancies classified
pack validator recomputes checksums/coverage/licenses
rebuild determinism classified correctly
no signing/upload/CI
```

## E2 project, graph, recognizers, persistence

### E2-A — `wow-graph`

[`wow-graph/e2/`](wow-graph/e2/README.md): versioned registries, semantic keys, immutable producer assertions, conflicts/coverage, partition replacement, GraphGeneration/Snapshot, axes, bounded queries, logical store plan.

### E2-B — `wow-recognizers`

[`wow-recognizers/e2/`](wow-recognizers/e2/README.md): bounded declarative packs, normalized typed facts, deterministic matching, universal graph proposals, ambiguity/coverage, producer partitions, mutation/precision evaluation.

Hard boundaries:

```text
no second parser/raw-source fallback
no repository/addon/path semantic conditions
no LLM correctness path
no graph publication/final IDs
native frame events, EventRegistry bridges, custom callbacks, and CVar callbacks remain distinct
hook recognition never proves taint/combat/protected/runtime safety
SavedVariables roots require TOC declarations
```

### E2-C — `wow-project` full index candidate

[`wow-project/e2/`](wow-project/e2/README.md): exact materialized source snapshot, one TOC variant, bounded TOC/XML parsing, XML virtual Lua, static load model, analyzer and recognizer orchestration, graph proposal validation, incremental invalidation, immutable `NotPublishedE2C` candidate.

Hard boundaries:

```text
no floating repo/installed-addon scan
no source or repository-script execution
no automatic dependency download
no second Lua parser
no runtime readiness/safety claims from static load order
no SavedVariables contents
no ProjectStore/current pointer/final GraphGeneration
```

### E2-D — `wow-store` ProjectStore and coherent publication

Primary contract: [`wow-store/e2/`](wow-store/e2/README.md).

Selected physical profile:

```text
project-store-wal-manifested-partitions-v1
```

Owns:

- one SQLite WAL database per ProjectStore epoch;
- exact SQLite/binding/platform profile and effective PRAGMAs;
- one writer with finite busy policy and stale-base rejection;
- immutable content-addressed project/graph partition versions;
- complete generation membership maps with no recursive delta chain;
- owner-separated store/project/graph schema, operation, and validation catalogs;
- noncyclic `ProjectPublicationSet` and `ProjectStoreGeneration` identity;
- target commit as `PublishedInactive`;
- fresh exact read-back validation before current activation;
- separate compare-and-swap `CurrentPublicationRecord` transaction;
- snapshot-bound old/new/exact readers and process-local generation leases;
- WAL/checkpoint/reader-pressure/durability classification;
- startup recovery, online backup, restore, and incompatible-profile epoch rebuild;
- reference-based generation/partition/object/epoch retention and GC;
- benchmark and logical-determinism gates.

Ownership seam:

```text
wow-project
    builds ProjectPublicationSet and calls store operations

wow-graph
    builds GraphPublicationPlan/GraphGeneration/GraphSnapshot and validates persisted graph

wow-store
    never interprets either domain; it persists registered plans and owns physical atomicity
```

E2-D implementation prerequisites:

```text
implemented/frozen E0/E1 wow-store foundation
implemented/frozen E2-A graph
implemented/frozen E2-B recognizers
implemented/frozen E2-C project candidate
pinned SQLite library/Rust binding/platform adapter
executable WAL/read/checkpoint/crash probes
frozen store/project/graph schema-operation-validation bundles
frozen synthetic and roth-ui benchmark corpora and thresholds
all fixture/member/bundle checksums
```

E2-D handoff gate:

```text
one baseline and representative incremental candidate publish
one-file update reuses unaffected partitions without recursive reads
inactive generation opens and passes store/project/graph golden validation
activation rejects a changed base
old reader stays old; new reader sees new
current record always binds coherent store/project/graph/analyzer IDs
crash/cancel at every phase yields old-current, new-current, or recoverable inactive state
checkpoint never changes logical state and WAL remains bounded under declared policy
current/LKG/reader/evidence pins block GC
orphan generation/partition/object reclamation is closed and transactional
breaking schema/runtime changes build a new epoch
no raw SQL, domain semantics, source execution, editor/client access, or runtime data
logical outputs deterministic; physical SQLite/WAL bytes classified separately
```

## Next documentation package — E3

E3 should begin with `wow-context` and the graph-to-context contract:

```text
L0/L1 skeleton schema
Project Map generation and strict size budget
progressive detail handles and source resolution
project plus pinned Blizzard UI graph scopes
context capability/coverage/conflict state
bounded neighborhood selection and no-source-dump defaults
context metrics and deterministic output
```

Full Blizzard UI source extraction belongs to explicit reference/project graph producers, not to `wow-context`.

## E4–E7

- E4: exact/migration/shape/FTS/graph search, explicit lineage and impact; similarity never authorizes replacement.
- E5: pinned named calibration packs emitting universal roles only, with rename/path mutations.
- E6: optional Codebase Memory MCP bridge; all external results remain Candidate.
- E7: LSP/MCP transports, release signing/publication/activation and operational gates.

## Seam request format

```text
requesting work package/crate
owning crate
required operation/data contract
current workaround rejected
why orchestration/read-view cannot solve it
proposed smallest seam
cycle/security/evidence impact
fixture/mutation proving the seam
implementation/freeze prerequisite impact
```

Do not implement a missing seam in the wrong crate.
