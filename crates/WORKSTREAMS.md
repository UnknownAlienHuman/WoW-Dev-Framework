# Agent workstreams and integration order

**Status:** operational routing through documentation frontier E3-B.

This file prevents agents from coding against documentation-only prerequisites, activating every planned crate, inventing cross-crate seams, or reporting missing probes as passes.

## Ownership rules

- One agent owns one work package and one primary crate.
- Shared seam changes are proposed before dependent implementation.
- A separate integration/review agent verifies each wave.
- Documentation-ready remains `implementation_state=not-started` until code, executable probes, fixtures, and checksums exist.
- Missing implementations, probes, benchmarks, tokenizers, evaluations, runtime checks, or source snapshots are blocking/`NotEvaluated`, never pass.
- Current patch-sensitive WoW claims route through `wow-addon-engineering-kb`; stable framework contracts link rather than duplicate live guidance.

## Global order

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
-> E2-C addon-project TOC/XML/load/incremental candidate
-> E2-D ProjectStore + coherent graph/project publication

-> E3-A wow-project Blizzard UI source universe/index
-> E3-B wow-context Project Map/L0/L1/context packs
-> E3-C wow-service/application context acquisition and use cases, if separate

-> E4 wow-search + lineage/impact
-> E5 calibration packs
-> E6 Codebase Memory candidate bridge
-> E7 LSP/MCP/release/publishing
```

Parallel documentation is allowed. Rust implementation follows dependency and freeze order.

## E0 vertical slice

- **E0-A `wow-core`:** identities, generations, source handles, evidence/conflicts, coverage/NotEvaluated, findings/warnings/budgets, canonical envelopes.
- **E0-B `wow-reference`:** one synthetic profile/catalog, exact lookup, complete/partial/conflict coverage, restriction fixture.
- **E0-C `wow-emmy`:** pinned analyzer adapter, actor/snapshots, normalized facts, generic diagnostics, exact coordinates.
- **E0-D `wow-project`:** one first-party workspace, coherent generation, source registry, analyzer binding, immutable snapshot.
- **E0-E `wow-rules`:** `wow.api.exists` and one bounded direct-local Secret-value rule.
- **E0-F `wow-service`/`apps/wow`:** `status`, `check`, coherent context, raw finding preservation, causal presentation, thin CLI.

E0 gate: exact profile/reference/project/analyzer identity, honest negative authority, separate platform/project evidence, Secret guarded/unsafe cases, NotEvaluated behavior, deterministic 1/2/N output, and no mutation.

## E1 Reference Pack

- **E1-A `wow-store`:** schema/migrations, immutable ReferenceStore, objects, validation, generic publication foundation.
- **E1-B `wow-reference`:** pinned source, restricted evaluator, raw metadata, corrections, exact coverage, persistent plan, ReferenceView.
- **E1-C `wow-annotations`:** semantic model, type lowering, inert rendering, source maps/loss, parity and consumer probes.
- **E1-D `wow-service`/builder:** build, nonrepairing validation, deterministic rebuild comparison, pack layout. Signing/publication deferred.

E1 gate: exact pins, immutable validated store, unknown/correction/negative-authority honesty, annotation injection/editor gates, component checksums/licenses, deterministic rebuild, no CI/upload.

## E2 project, graph, recognizers, persistence

- **E2-A `wow-graph`:** versioned registries, semantic keys, producer assertions/partitions, conflicts/coverage, snapshots, axes, bounded queries.
- **E2-B `wow-recognizers`:** bounded declarative packs over typed facts, universal proposals, ambiguity/coverage, mutation precision.
- **E2-C `wow-project`:** exact source snapshot, one TOC variant, bounded TOC/XML, virtual Lua, static load, analyzer/recognizer orchestration, invalidation, `NotPublishedE2C` candidate.
- **E2-D `wow-store`:** one WAL epoch, immutable partition versions, complete generation membership, inactive build/read-back validation/current CAS, snapshot readers, recovery/backup/retention/GC.

E2 gate: representative baseline/incremental candidates; stale partition removal; coherent store/project/graph/analyzer identity; response-loss idempotency; old/new reader isolation; crash/cancel classification; bounded WAL; protected GC roots; logical determinism; no source execution/raw SQL/domain leakage.

## E3-A — exact Blizzard UI source index

Owner: [`wow-project/e3/`](wow-project/e3/README.md).

Produces:

```text
exact materialized Blizzard UI source snapshot
separate blizzard_ui_source ProjectId/ProjectSnapshot
separate GraphGeneration/GraphSnapshot
package/TOC/XML/load/analyzer/recognizer structural records
source fingerprints for later E4 comparison only
bounded exact SkeletonInputView
E2-D coherent published store state
```

Hard boundaries:

```text
no floating repository/current/latest
no source/repository-script execution
no merging Retail/PTR/Beta/Classic variants
no source implementation as API/runtime/Secret/taint/combat authority
no Project Map/L0/L1/context rendering
no lineage/migration/impact claims
no source redistribution claim without exact license policy
```

E3-A implementation gate: implemented/frozen E0-E2, exact source materializer/provider/build/license manifest, parser/analyzer/recognizer/graph/store profiles, source/rename/removal/authority/security vectors, benchmarks, and all checksums.

## E3-B — `wow-context`

Owner: [`wow-context/e3/`](wow-context/e3/README.md).

Active direct dependencies:

```text
wow-core
wow-reference
wow-project
wow-graph
```

Owns:

- exact `ContextUniverseSet` binding and compatibility;
- separate user-project and Blizzard UI Project Maps;
- L0 container/navigation skeletons;
- L1 exact entity/local-neighborhood skeletons;
- reviewed typed expansion profiles and exact-root planning;
- deterministic candidate selection, dependencies, deduplication, pruning, stopping, and continuation;
- exact source/reference evidence acquisition through owner views;
- source/instruction structural boundary plus privacy/license/consumer policy;
- byte budgets and exact/estimated/upper-bound token accounting;
- immutable `ContextSemanticPack`;
- canonical JSON and deterministic Markdown `RenderedContextArtifact`;
- cache keys/validation, not physical cache storage;
- evidence/provenance/confidence/coverage/conflict/omission closure.

Hard boundaries:

```text
no fuzzy or natural-language root resolution
no direct store/analyzer/recognizer session
no second parser/raw-source semantic inference
no model/embedding/LLM correctness path
no source comments as instructions or framework facts
no user/platform/reference universe collapse
no generation switch after binding
no direct edge fabricated from a reason path
no authority/confidence upgrade
no mandatory evidence/boundary/coverage omission to fit
no exact token claim without a frozen exact tokenizer
no unbounded source/graph export
no diagnostics/remediation/edit planning/runtime truth
no context persistence or external side effects
```

E3-B implementation prerequisites:

```text
implemented/frozen wow-core/reference/project/graph and E2-D views
implemented/frozen E3-A Blizzard UI source/SkeletonInputView when platform context is required
exact read/source-slice catalogs
frozen universe/map/L0/L1/intent/expansion/selection/budget/tokenizer/privacy/boundary/renderer/cache profiles
synthetic, roth-ui, and Blizzard UI context fixtures
hostile-source, high-fanout, cancellation, continuation, omission, token, cache, and determinism vectors
all member and bundle checksums
```

E3-B handoff gate:

```text
one coherent exact universe set per request
Project Map/L0/L1 mandatory records have origin/evidence closure
Possible/conflict/partial/NotEvaluated/truncation remain visible
source excerpts resolve exact handle/digest/range and are structurally isolated
optional pruning is deterministic and fully reported
mandatory closure over budget fails instead of lying
continuation stays on exact generations/profiles/total budget
cache keys reject stale/cross-privacy/cross-tokenizer artifacts
canonical JSON and Markdown preserve the same semantic claims
1/2/N workers and shuffled owner result order yield identical canonical outputs
no source/project/graph/reference/store/editor/client mutation or background work
```

## Next packages

### E3-C, if separate

`wow-service` acquires exact retained views, resolves application policy, invokes E3-B operations, chooses renderers, and returns public envelopes. Applications remain thin. No duplicate context selection logic.

### E4

`wow-search` adds exact/migration/shape/FTS/graph lanes and explicit lineage/impact. Similarity never authorizes replacement. Search supplies exact roots to context; it does not become context authority.

### E5-E7

Pinned calibration packs emit universal roles only; Codebase Memory remains Candidate; LSP/MCP/release/publishing activate only after executable gates.

## Seam request

```text
requesting work package/crate
owning crate
required operation/data contract
why existing read-view/orchestration/artifact is insufficient
proposed smallest seam
cycle/security/privacy/license/evidence impact
fixtures and mutations proving the seam
implementation/freeze impact
```

Do not implement a missing seam in the wrong crate.
