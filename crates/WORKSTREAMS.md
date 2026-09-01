# Agent workstreams and integration order

**Status:** operational routing through documentation frontier E3-B.

This file prevents agents from coding against documentation-only prerequisites, activating every planned crate, inventing seams locally, or treating missing evidence as success.

## Ownership rules

- One agent owns one work package and one primary crate.
- Shared seam changes are proposed before dependent implementation.
- A separate integration/review pass validates each wave.
- Documentation-ready remains `implementation_state=not-started` until code, executable probes, fixtures, checksums, and required evaluations exist.
- Missing tools, implementations, probes, benchmarks, tokenizers, evaluations, runtime checks, or source producers are blocking/`NotEvaluated`, never pass.
- Patch-sensitive WoW claims route through the current `wow-addon-engineering-kb`, pinned Blizzard source/generated docs, and required runtime probes.
- No repository/addon/path/provider/popularity name becomes a hidden production rule.
- No CI is added without explicit owner instruction.

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
-> E2-C wow-project TOC/XML/load/incremental candidate
-> E2-D ProjectStore + coherent graph/project publication

-> E3-A wow-project Blizzard UI source universe
-> E3-B wow-context Project Map/L0/L1/context pack
-> E3-C wow-service/apps context acquisition/use cases

-> E4 wow-search + lineage/impact
-> E5 calibration packs
-> E6 Codebase Memory candidate bridge
-> E7 LSP/MCP/release/publishing
```

Parallel documentation is allowed. Rust implementation follows dependency and freeze order.

## E0 vertical slice

### E0-A — `wow-core`

Identity, generations, source handles, evidence/conflicts, coverage/NotEvaluated, findings/warnings/budgets, canonical envelopes. Blocks dependent code.

### E0-B — fixture `wow-reference`

One exact synthetic profile/catalog, restricted evaluation, exact lookup, complete/partial/conflict coverage, restriction fixture.

### E0-C — `wow-emmy`

Pinned upstream adapter, analyzer actor/snapshots, normalized Lua facts, generic diagnostics, exact source coordinates.

### E0-D — minimal `wow-project`

One first-party workspace, coherent ProjectGeneration, source registry, analyzer binding, immutable project snapshot.

### E0-E — `wow-rules`

Only `wow.api.exists` and one bounded direct local Secret-value rule.

### E0-F — `wow-service` + `apps/wow`

`status`, `check`, coherent context acquisition, raw finding preservation, causal presentation, canonical result, thin CLI.

## E1 Reference Pack

### E1-A — `wow-store`

Schema/migrations, immutable ReferenceStore, object store, validation, generic publication foundation.

### E1-B — persistent `wow-reference`

Pinned source, restricted APIDocumentation evaluator, raw metadata, corrections, exact coverage, ReferenceStore plan, ReferenceView.

### E1-C — `wow-annotations`

Semantic model, explicit type lowering, deterministic inert rendering, source maps/loss, Ketho parity, EmmyLua/LuaLS probes.

### E1-D — Reference Pack build/validation

Service and thin builder app coordinate build, nonrepairing validation, deterministic rebuild comparison, pack layout. Signing/publication remain deferred.

## E2 project, graph, recognizers, persistence

### E2-A — `wow-graph`

Versioned registries, semantic keys, immutable producer assertions, conflicts/coverage, partition replacement, GraphGeneration/Snapshot, axes, bounded queries, logical store plans.

### E2-B — `wow-recognizers`

Bounded declarative packs over typed normalized facts, deterministic matching, universal graph proposals, ambiguity/coverage, producer partitions, mutation/precision evaluation.

Hard stops: no second parser, repository-specific conditions, model correctness path, final graph publication, signal-class collapse, hook safety inference, or SavedVariables roots without TOC facts.

### E2-C — full `wow-project` candidate

Exact materialized addon snapshot, one TOC variant, bounded TOC/XML parsing, XML virtual Lua, static load model, analyzer/recognizer orchestration, graph proposal validation, invalidation/reuse/stale removal, immutable `NotPublishedE2C` candidate.

### E2-D — ProjectStore coherent publication

Selected profile:

```text
project-store-wal-manifested-partitions-v1
```

One SQLite WAL epoch, one writer, immutable partition versions, complete generation membership, inactive build, exact read-back validation, current-record CAS, durable operation idempotency, snapshot readers/leases, recovery/backup/retention/GC. Domain semantics remain in project/graph owners.

## E3-A — Blizzard UI source index (`wow-project`)

Primary contract: [`wow-project/e3/`](wow-project/e3/README.md).

Owns:

- one exact materialized Blizzard UI source snapshot;
- separate `blizzard_ui_source` project universe;
- explicit package/TOC/client-flavor profile;
- E2 TOC/XML/Lua/analyzer/recognizer/graph reuse;
- exact source, package, load, XML/template/mixin, analyzer, graph, fingerprint, coverage, license, and redistribution records;
- incremental update/removal closure;
- coherent E2-D publication into a separate platform-source store;
- bounded exact `SkeletonInputView` for `wow-context`.

Hard stops:

```text
no floating latest/current repository or installed-client discovery
no source/repository script execution
no API/runtime/Secret/taint/combat/performance authority from implementation source
no Retail/PTR/Beta/Classic variant merge
no Project Map/L0/L1/context rendering inside wow-project
no redistribution claim from local indexing alone
```

## E3-B — Project Map, L0/L1, and context packs (`wow-context`)

Primary contract: [`wow-context/e3/`](wow-context/e3/README.md).

Active direct dependencies:

```text
wow-core
wow-reference
wow-project
wow-graph
```

Owns:

- exact immutable `ContextUniverseSet` across user project, optional Blizzard UI, and ReferenceView;
- deterministic separate/combined Project Maps;
- L0 container/navigation skeletons without bodies;
- L1 exact entity/local-neighborhood skeletons;
- closed control/effect projection over published facts only;
- exact-root reviewed intent/expansion/selection profiles;
- mandatory closure, deterministic optional pruning, omissions, no-new-evidence, and snapshot-bound continuation;
- exact source excerpts with privacy/license/consumer trust and structural source-data boundaries;
- exact semantic/rendered bytes and honest Exact/Estimate/UpperBound/Unavailable token classes;
- immutable `ContextSemanticPack`, canonical JSON, deterministic Markdown;
- cache keys/validation only, noncanonical metrics, and frozen evaluation.

Hard stops:

```text
no second parser/CFG/SSA/data-flow engine
no raw store/analyzer/recognizer/rule access
no search/fuzzy/natural-language root resolution
no model/embedding/external tool in canonical selection
no generation switch after binding
no universe/name/path identity merge
no direct edge created from a reason path
no authority/confidence/coverage upgrade
no hidden omission or mandatory evidence pruning
no exact token claim without frozen tokenizer/framing over exact bytes
no source text as framework or agent instructions
no physical cache/persistence/service/application behavior
```

### E3-B implementation gate

```text
implemented/frozen E0-E2 prerequisites
implemented/frozen E3-A for profiles using Blizzard UI source
exact public project/graph/reference/source-slice catalogs
all context/profile/tokenizer/privacy/renderer/evaluation profiles frozen
synthetic, high-fanout, pinned roth-ui, pinned Blizzard UI, combined and adversarial corpora frozen
all IDs, expected bytes/tokens, thresholds and SHA-256 manifests populated
1/2/N and shuffled owner outputs byte-identical
mandatory claim/evidence/coverage/boundary recall complete
zero universe/generation substitution, authority upgrade, path-as-edge, hidden omission, source-boundary escape, or private-data leak
```

## Next package — E3-C (`wow-service` + `apps/wow`)

E3-C must define only orchestration and thin application behavior:

```text
symbolic current request at service boundary
-> resolve exact current project/platform/reference publication once
-> acquire coherent retained views and leases
-> resolve or require exact roots according to the operation contract
-> call wow-context map/L0/L1/context operations
-> select renderer/profile explicitly
-> validate result and close resources
-> return canonical service envelope
-> thin CLI serialization/exit code/cancellation
```

It must not implement Project Map, skeleton, expansion, source boundary, graph query, store transaction, parser, rule, search, or renderer algorithms already owned elsewhere.

Planned public use cases:

```text
context status
context map
context inspect
context build
context continue
context validate
context render
```

Natural-language search remains E4 unless E3-C requires exact root IDs.

## E4–E7

- E4: `wow-search`, explicit lineage and patch impact; similarity never authorizes replacement.
- E5: pinned named calibration packs emitting universal roles only, with rename/path mutations.
- E6: optional Codebase Memory bridge; external results remain Candidate.
- E7: LSP/MCP transports, release signing/publication/activation, operational gates.

## Seam request format

```text
requesting work package/crate
owning crate
required operation/data contract
current workaround rejected
why existing read view/artifact/service operation is insufficient
proposed smallest seam
cycle/security/evidence/privacy/license impact
fixture/mutation proving the seam
implementation/freeze prerequisite impact
```

Do not implement a missing seam in the wrong crate.
