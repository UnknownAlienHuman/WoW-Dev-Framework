# Agent workstreams and integration order

**Status:** operational routing through documentation frontier E3-A.

This file prevents agents from coding against documentation-only prerequisites, activating every planned crate, or inventing cross-crate seams locally.

## Ownership rules

- One agent owns one work package and one primary crate.
- Shared seam changes are proposed before dependent implementation.
- A separate integration/review agent verifies each wave.
- Documentation-ready remains `implementation_state=not-started` until code, executable probes, fixtures, and checksums exist.
- Missing tools, implementations, probes, benchmarks, tokenizers, evaluations, runtime checks, or source producers are `skipped`/blocking/`NotEvaluated`, never pass.
- Current patch-sensitive WoW claims route through `wow-addon-engineering-kb`; stable framework contracts link rather than duplicate live guidance.

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

-> E3-A wow-context Project Map/L0/L1/progressive context
-> E3-B pinned Blizzard UI source producer
-> E3-C context service/application use cases if kept separate

-> E4 wow-search + lineage/impact
-> E5 calibration packs
-> E6 Codebase Memory candidate bridge
-> E7 LSP/MCP/release/publishing
```

Parallel documentation is allowed. Rust implementation follows dependency and freeze order.

## E0 vertical slice

### E0-A — `wow-core`

[`wow-core/`](wow-core/README.md): identity, generations, source handles, evidence/conflicts, coverage/NotEvaluated, findings/warnings/budgets, canonical envelopes. Blocks dependent code.

### E0-B — fixture `wow-reference`

[`wow-reference/`](wow-reference/README.md): one synthetic profile/catalog, restricted evaluation, exact lookup, complete/partial/conflict coverage, restriction fixture.

### E0-C — `wow-emmy`

[`wow-emmy/`](wow-emmy/README.md): pinned upstream adapter, analyzer actor/snapshots, normalized Lua facts, generic diagnostics, exact source coordinates. May implement with E0-B after E0-A.

### E0-D — minimal `wow-project`

[`wow-project/`](wow-project/README.md): one first-party workspace, coherent ProjectGeneration, source registry, analyzer binding, immutable project snapshot.

### E0-E — `wow-rules`

[`wow-rules/`](wow-rules/README.md): only `wow.api.exists` and one direct local Secret-value rule.

### E0-F — `wow-service` + `apps/wow`

[`wow-service/`](wow-service/README.md), [`../apps/wow/`](../apps/wow/README.md): `status`, `check`, coherent context, raw finding preservation, causal presentation, canonical result, thin CLI.

## E0 integration gate

```text
one exact profile/reference/project/analyzer generation
known fixture API resolves
unknown fixture API produces a WoW finding only under authoritative absence
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

[`wow-service/e1/`](wow-service/e1/README.md), [`../apps/wow-reference-builder/`](../apps/wow-reference-builder/README.md): build, nonrepairing validation, deterministic rebuild comparison, pack layout, thin CLI. Signing/publication remains deferred.

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

[`wow-graph/e2/`](wow-graph/e2/README.md): versioned registries, semantic keys, immutable producer assertions, conflicts/coverage, partition replacement, GraphGeneration/Snapshot, axes, bounded queries, logical store plans.

### E2-B — `wow-recognizers`

[`wow-recognizers/e2/`](wow-recognizers/e2/README.md): bounded declarative packs, typed facts, deterministic matching, universal graph proposals, ambiguity/coverage, producer partitions, mutation/precision evaluation.

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

[`wow-project/e2/`](wow-project/e2/README.md): exact materialized source snapshot, one TOC variant, bounded TOC/XML parsing, XML virtual Lua, static load model, analyzer/recognizer orchestration, graph proposal validation, incremental invalidation, immutable `NotPublishedE2C` candidate.

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

### E2-D — ProjectStore and coherent publication

Primary contract: [`wow-store/e2/`](wow-store/e2/README.md).

Selected physical profile:

```text
project-store-wal-manifested-partitions-v1
```

Owns:

- one SQLite WAL database per compatible ProjectStore epoch;
- exact SQLite/binding/platform profile and effective PRAGMAs;
- one writer, finite busy policy, stale-base rejection;
- immutable content-addressed project/graph partition versions;
- complete generation membership, no recursive delta chain;
- owner-separated store/project/graph schema, operation, validation catalogs;
- noncyclic publication/store identity;
- target commit as `PublishedInactive`;
- fresh exact read-back validation before activation;
- separate exact-base `CurrentPublicationRecord` CAS;
- durable operation ID/request digest and response-loss reconciliation;
- snapshot-bound old/new/exact readers, semantic continuations, process-local leases;
- WAL/checkpoint/reader-pressure/Windows-sharing classification;
- startup recovery, quarantine, online backup/restore, incompatible-profile epoch rebuild;
- reference- and operation-closed generation/partition/object/epoch retention and GC;
- benchmark and logical-determinism gates.

Ownership seam:

```text
wow-project
    builds ProjectPublicationSet and calls store operations

wow-graph
    builds GraphPublicationPlan/GraphGeneration/GraphSnapshot and validates persisted graph

wow-store
    persists registered plans and owns physical atomicity without interpreting either domain
```

E2-D implementation prerequisites:

```text
implemented/frozen E0/E1 store foundation
implemented/frozen E2-A graph, E2-B recognizers, E2-C project candidate
pinned SQLite library/Rust binding/platform adapter
executable WAL/read/checkpoint/crash/response-loss/Windows probes
frozen store/project/graph schema-operation-validation bundles
frozen synthetic and roth-ui benchmark corpora and thresholds
all fixture/member/bundle checksums
```

E2-D handoff gate:

```text
baseline and representative incremental candidates publish
one-file update reuses unaffected partitions and never clones the whole SQLite image
inactive generation passes exact store/project/graph validation
activation rejects changed base
same-operation retry after response loss returns existing receipt
old reader stays old; new reader sees new
current record binds coherent store/project/graph/analyzer IDs
crash/cancel yields old-current, new-current, or recoverable/quarantined inactive state
checkpoint never changes logical state; WAL stays bounded
current/LKG/reader/evidence/operation roots block GC
stale GC plan is rejected after lease/current/operation change
breaking schema/runtime builds a new epoch
no raw SQL/domain semantics/source execution/editor/client/runtime data
logical outputs deterministic; physical bytes classified separately
```

## E3-A — `wow-context`

Primary contract: [`wow-context/e3/`](wow-context/e3/README.md).

Active direct dependency slice:

```text
wow-context
├── wow-core
├── wow-reference
├── wow-project
└── wow-graph
```

`wow-store`, `wow-emmy`, and `wow-search` remain inactive direct dependencies in E3-A. Project/graph published views carry coherent store/analyzer identities and exact registered reads; callers supply exact roots.

Owns:

- exact context input snapshot and source-universe validation;
- bounded Project Map with strict compact renderer target and mandatory blocker reserve;
- L0 identity/role/owner/load/direct-relation skeletons;
- L1 signatures/members/reason paths and closed control/effect projection;
- typed `UnknownRegion`, `CollapsedRegion`, and `OmittedRegion` records;
- lane-specific progressive expansion, cycles, no-new-evidence, stopping, continuation;
- evidence/provenance/coverage/conflict/loss/omission closure;
- faithful bounded source excerpts and prompt/container/private-data security;
- structural/byte/source budgets and optional exact pinned-tokenizer accounting;
- canonical semantic JSON plus Markdown/compact renderer contracts;
- multidimensional context metrics and frozen consumer evaluation.

Identity order:

```text
exact input + request
-> plan/frontier
-> map/skeleton/control/source/evidence/loss records
-> ContextBundleCore
-> renderer artifact
-> metrics
-> evaluation report
-> outer envelope
```

Hard boundaries:

```text
no StoreImageId or whole-database-generation assumption
no floating Current/latest after acquisition
no search/ranking in E3-A
no raw store/analyzer session
no second parser/AST/CFG/SSA/data-flow engine
no inferred purpose/code/runtime order/taint/combat/Secret/safety
no name/path/prose/embedding cross-universe join
no full source/graph/evidence dump
no source prompt or comment as policy
no exact token claim without pinned tokenizer and final renderer bytes
no mandatory blocker/evidence loss to meet approximately 2 KiB
no semantic bundle backreference to renderer/metrics/evaluation
no cache/persistence/model correctness path/background continuation
```

### E3-A implementation prerequisites

```text
implemented/frozen wow-core
implemented/frozen persistent wow-reference
implemented/frozen E2-A graph
implemented/frozen E2-C project and E2-D coherent publication/read views
exact project/graph/reference/source-detail query catalogs
frozen context/map/skeleton/control/expansion/source/budget/tokenizer/security/renderer/evaluation profiles
frozen synthetic exact corpus and pinned roth-ui publication fixture
optional platform UI source remains NotEvaluated until E3-B producer exists
all fixture/member/bundle checksums
```

### E3-A handoff gate

```text
coherent epoch/store/publication/project/analyzer/graph/reference identities
Project Map and L0/L1 contain all mandatory records with evidence closure
strict default renderer meets target or returns explicit blocker/truncation state
control/effect nodes use published facts only; unknown/collapsed/omitted detail remains typed
project occurrence and platform evidence remain separate
Possible/Candidate/partial/conflict/NotEvaluated states remain honest
source excerpts are exact, licensed, private-data-safe, and injection-contained
continuation stays on exact retained generation and never resets total budget
semantic bundle/renderer/metrics/evaluation DAG is acyclic
JSON/Markdown/compact renderers preserve the same semantic records
exact tokens bind exact tokenizer and renderer bytes; estimates remain labeled
mandatory recall/evidence/security/determinism hard gates pass
repository/package/path/name mutations do not change universal semantics
1/2/N and shuffled input/query completion produce identical semantic bytes
no source/project/graph/store/editor/client mutation or background work
```

## Next documentation package — E3-B

E3-B defines the pinned Blizzard UI source producer needed for richer platform implementation context:

```text
exact acquisition provider, client build/revision, source manifest, license, and checksums
explicit source universe separate from Reference Pack API facts
TOC/XML/Lua materialization through existing project/analyzer boundaries
producer partitions and graph publication
coverage/conflict/source-handle model
incremental rebuild and patch/profile replacement
closed synthetic and pinned real source fixtures
no hidden download/current/latest or source execution
```

Source extraction/indexing does not belong in `wow-context`.

## E3-C and E4–E7

- E3-C, if separate: `wow-service` context operations, application renderer selection, retained-view acquisition, and evaluated use cases.
- E4: exact/migration/shape/FTS/graph search, explicit lineage and impact; similarity never authorizes replacement.
- E5: pinned named calibration packs emitting universal roles only, with rename/path mutations.
- E6: optional Codebase Memory MCP bridge; external results remain Candidate.
- E7: LSP/MCP transports, release signing/publication/activation and operational gates.

## Seam request format

```text
requesting work package/crate
owning crate
required operation/data contract
current workaround rejected
why orchestration/read-view/producer artifact cannot solve it
proposed smallest seam
cycle/security/evidence/license impact
fixture/mutation proving the seam
implementation/freeze prerequisite impact
```

Do not implement a missing seam in the wrong crate.
