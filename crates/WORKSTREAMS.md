# Agent workstreams and integration order

**Status: operational routing**

This file prevents agents from implementing every planned component at once, coding against documentation-only prerequisites, or inventing incompatible seams in parallel.

## Ownership model

- One agent owns one work package and one primary crate.
- A work package may read all contracts but writes only its assigned crate plus explicitly listed fixtures/routes.
- Shared contract changes are proposed before dependent implementation begins.
- A separate integration/review agent validates seams after each wave.
- No agent silently redesigns another crate to make local work easier.
- Documentation-ready is not implementation-ready when prerequisite code/pins/fixtures remain absent.
- A completed documentation package keeps `implementation_state=not-started` until code and frozen checksums actually exist.

## Global implementation order

```text
E0-A wow-core
-> E0-B wow-reference fixture + E0-C wow-emmy
-> E0-D wow-project fixture generation
-> E0-E wow-rules
-> E0-F wow-service + apps/wow

-> E1-A wow-store
-> E1-B persistent wow-reference
-> E1-C wow-annotations
-> E1-D Reference Pack build/validate app

-> E2-A wow-graph
-> E2-B wow-recognizers
-> E2-C full wow-project TOC/XML/load/incremental indexing
-> E2-D ProjectStore + graph/project publication integration

-> E3 wow-context + Blizzard/project graph/skeletons/Project Map
-> E4 wow-search + lineage/impact
-> E5 calibration packs
-> E6 Codebase Memory candidate bridge
-> E7 LSP/MCP/release/publishing surfaces
```

Parallel documentation is allowed. Rust implementation still obeys prerequisite gates.

## E0 vertical slice

### E0-A — `wow-core`

Contract: [`wow-core/`](wow-core/README.md).

Owns identity, generation, source handles, evidence/conflicts, coverage/NotEvaluated, findings/warnings/budgets, canonical result envelopes.

Implementation blocks all dependent code. Before the first Rust commit, freeze all examples/hash vectors and public consumer seams.

### E0-B — fixture `wow-reference`

Contract: [`wow-reference/`](wow-reference/README.md).

Owns one synthetic profile/catalog, restricted evaluation, exact lookup, complete/partial/conflict coverage, Secret facet fixture, and no full store/builder.

May implement only after E0-A code is frozen.

### E0-C — `wow-emmy`

Contract: [`wow-emmy/`](wow-emmy/README.md).

Owns exact upstream pin/probe, analyzer actor/snapshots, normalized Lua facts, generic diagnostics, source coordinates.

May proceed in parallel with E0-B after E0-A implementation.

### E0-D — minimal `wow-project`

Contract: [`wow-project/`](wow-project/README.md).

Owns one exact first-party workspace, project generation, source registry, analyzer snapshot binding, immutable project snapshot. No TOC/XML/graph yet.

### E0-E — `wow-rules`

Contract: [`wow-rules/`](wow-rules/README.md).

Owns only `wow.api.exists` and one direct local `wow.secret.local_operation` rule in E0.

### E0-F — `wow-service` + `apps/wow`

Contract: [`wow-service/`](wow-service/README.md) and [`../apps/wow/`](../apps/wow/README.md).

Owns `status`, `check`, coherent context, raw finding preservation, structured root-cause presentation, canonical envelopes, and thin CLI serialization.

## E0 integration gate

```text
one exact profile/reference generation
one exact project/analyzer generation
known fixture API resolves
unknown fixture API produces WoW finding only under authoritative absence
generic analyzer finding remains visible
Secret-local unsafe and guarded cases classify correctly
missing/conflicted capability produces NotEvaluated
reference and project evidence remain separate
1/2/N canonical output byte-identical
no editor/client/source mutation
```

## E1 Reference Pack

### E1-A — `wow-store`

Contract: [`wow-store/`](wow-store/README.md).

SQLite/runtime/schema/migrations, immutable ReferenceStore publication, objects, integrity, and future ProjectStore boundary. No domain authority or raw SQL public seam.

### E1-B — persistent `wow-reference`

Contract: [`wow-reference/e1/`](wow-reference/e1/README.md).

Pinned source snapshot, restricted APIDocumentation evaluator, raw metadata, normalized facts, digest-bound corrections, exact coverage/negative authority, ReferenceStore logical schema/build plan, ReferenceView.

### E1-C — `wow-annotations`

Contract: [`wow-annotations/e1/`](wow-annotations/e1/README.md).

Consumer-neutral semantic model, explicit type lowering, deterministic inert rendering, source maps/loss, Ketho semantic parity, EmmyLua/LuaLS probes.

### E1-D — pack build/validation

Contract: [`wow-service/e1/`](wow-service/e1/README.md) and [`../apps/wow-reference-builder/`](../apps/wow-reference-builder/README.md).

Cross-component build, independent nonrepairing validation, deterministic rebuild comparison, pack assembly, thin CLI. Signing/publication remains deferred.

## E1 integration gate

```text
exact source/profile/component pins
immutable ReferenceStore published/reopened/read-only
raw unknown metadata retained
corrections digest-bound and expiration tested
ReferenceView negative authority honest
annotation projection has no silent loss/injection/editor mutation
Ketho/EmmyLua/LuaLS gates classified
pack validator recomputes all member/checksum/coverage/license gates
1/2/N rebuild comparison stable by declared determinism class
no signing/upload/activation/CI
```

## E2 project and graph

### E2-A — `wow-graph` typed assertion core

Contract: [`wow-graph/e2/`](wow-graph/e2/README.md).

Owns versioned registries, semantic entity/relation keys, producer assertions, conflicts/coverage, atomic producer-partition replacement, immutable graph snapshots, explicit axes, bounded exact queries, and logical store operations.

Prerequisites before code:

```text
implemented/frozen wow-core
implemented/frozen wow-store and selected ProjectStore profile
frozen graph registry and fixtures
```

### E2-B — `wow-recognizers` core structural rules

Contract: [`wow-recognizers/e2/`](wow-recognizers/e2/README.md).

Owns:

- bounded canonical JSON core pack schema;
- recognizer-owned typed fact input envelope;
- deterministic non-Turing-complete matching;
- TOC/XML/frame/mixin/event/callback/hook/library/SavedVariables rules;
- proposed graph assertions only;
- confidence/ambiguity/coverage/NotEvaluated;
- producer partition/version/replacement contract;
- mutation and precision evaluation.

Hard boundaries:

```text
no source parser or raw-text fallback
no wow-project dependency
no repository/addon/path semantic conditions
no LLM correctness path
no graph publication/final IDs
no diagnostic severity/safety/autofix claims
native frame events, EventRegistry frame bridges, custom callbacks and CVar callbacks remain distinct
custom RegisterCallback requires exact TriggerEvent producer for confirmed custom relation
hook recognition never means taint/combat/protected/managed/runtime safe
SavedVariables roots require TOC declarations
```

Prerequisites before code:

```text
implemented/frozen wow-core
implemented/frozen wow-emmy facts/pin/probe
implemented/frozen wow-graph registry/proposal seam
frozen project TOC/XML fact adapter profile for real integration
frozen core pack/rules/evaluation/budget fixtures and checksums
```

E2-B handoff gate:

```text
all active rules use normalized facts only
all proposals validate against exact graph registry
all outputs Derived/Possible with complete evidence closure
positive/near-negative/partial/dynamic cases for every rule
repository/path/local-name mutations prove no overfitting
decisive convention-literal mutations change only intended rules
custom/native signal separation holds
hook safety claims absent
SavedVariables TOC authority holds
partial/truncated/cancelled partitions never publish complete
rule update/disable removes only owned producer assertions
1/2/N and shuffled inputs produce byte-identical outputs
```

### E2-C — full `wow-project` TOC/XML/load/index contract

Next documentation work package.

Must define:

```text
bounded TOC parser and flavor/variant selection
streaming XML facts and embedded-script source ownership
project fact adapter profiles consumed by recognizers
load/dependency/optional/LOD/bootstrap model
SavedVariables declarations and state-root seeds
incremental invalidation across files/TOC/XML/analyzer/recognizer partitions
coherent ProjectGeneration + GraphGeneration target publication
no parser duplication inside recognizers or graph
```

### E2-D — ProjectStore and integrated publication

After E2-A/B/C code seams exist, choose/freeze the measured ProjectStore physical model and atomic publication sequence. Readers never observe mixed project/analyzer/recognizer/graph generations.

## E3–E7 routing

### E3

Full project/Blizzard UI graph inputs, `wow-context` L0/L1 skeletons, Project Map, bounded context metrics.

### E4

`wow-search` exact/migration/shape/FTS/graph ranking, explicit lineage, patch-impact traversal. Similarity never authorizes replacement.

### E5

Named calibration packs from pinned audited repositories. They emit universal roles only and must survive repository/path/name mutations. Pack removal reduces coverage only.

### E6

Optional Codebase Memory MCP bridge. External semantic results remain Candidate and never bypass local evidence/graph/search authority.

### E7

LSP/MCP transports, release signing/publication/activation policy, final packaging and operational gates. No automation is introduced earlier by convention.

## Seam request format

When an agent cannot continue without another crate change, report:

```text
requesting work package/crate
owning crate
required operation/data contract
current workaround rejected
why orchestration/read-view cannot solve it
proposed smallest seam
cycle/security/evidence impact
fixture/mutation that proves the seam
implementation/freeze prerequisite impact
```

Do not implement a missing seam in the wrong crate while waiting.
