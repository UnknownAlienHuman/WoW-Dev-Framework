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

Owns bounded declarative core packs, typed normalized fact inputs, deterministic matching, universal graph proposals, evidence/confidence/ambiguity/coverage, producer partitions, and mutation/precision evaluation.

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

### E2-C — full `wow-project` source/TOC/XML/load/incremental index

Contract: [`wow-project/e2/`](wow-project/e2/README.md).

Owns:

- exact materialized project source snapshot, roots, universes, packages, and file manifest;
- one selected TOC variant per package with no cross-variant merge;
- bounded TOC directives/files/dependencies/LOD/bootstrap/SavedVariables parsing;
- bounded streaming XML includes/templates/objects/inheritance/scripts;
- source-mapped XML inline Lua virtual units;
- static package/file/unit load model and reachability;
- generation-bound `wow-emmy` physical/virtual Main and separate Library workspaces;
- exact TOC/XML/project/analyzer adapters for E2-B;
- recognizer execution and graph-proposal validation orchestration;
- dependency-driven incremental invalidation, conservative widening, reuse proof, and stale-output removal;
- immutable Complete/Partial `ProjectIndexCandidate` and E2-D publication bundle.

Hard boundaries:

```text
consume one closed materialized source snapshot; no floating repo or installed-addon scan
no repository hook/build/test/generator/Lua/XML/TOC execution
no second Lua parser
no automatic dependency download
unknown TOC/XML syntax preserved with narrow coverage effects
LOD/Bootstrap/load order remain static metadata, not runtime success or frame readiness
SavedVariables declarations only; never contents
first-party/dependency/library/reference/external/runtime universes remain separate
recognizers produce proposals and graph independently validates them
rejected graph proposals remain visible
unknown invalidation impact widens conservatively
removed source removes all target facts/matches/proposals/handles
wow-store remains inactive and candidate state is NotPublishedE2C
no current pointer or final GraphGeneration
```

Prerequisites before E2-C code:

```text
implemented/frozen wow-core
implemented/frozen wow-emmy pin/facts/virtual-source mapping
implemented/frozen wow-graph E2-A registry/proposal seam
implemented/frozen wow-recognizers E2-B core pack
frozen materializer/TOC/XML/load/adapter/invalidation/candidate profiles
frozen synthetic fixture and one pinned user-owned addon fixture
all fixture/member/bundle checksums
```

E2-C handoff gate:

```text
one exact source snapshot validates without host-path leakage
one exact TOC variant selected; alternate flavors never fill gaps
TOC unknowns and XML unknowns survive with exact coverage
XML DTD/entities/network/execution disabled
physical and XML virtual Lua units bind exactly to wow-emmy
static load model retains dependency/order/reachability reasons without runtime claims
native/custom/CVar signal and hook proof limits survive adapters/recognizers
all graph proposal rejections/conflicts remain visible
Lua/TOC/XML/profile/rule/registry updates invalidate exact partitions or widen safely
removed inputs have complete stale-output closure
Complete/Partial candidates remain NotPublishedE2C
one synthetic and one pinned user-owned addon fixture pass with repository/path rename mutations
1/2/N and shuffled input/update sequences are byte-identical
```

### E2-D — ProjectStore and integrated publication

Next documentation package.

Must choose/freeze the measured physical ProjectStore model and define:

```text
wow-store activation in wow-project
one writer and stale-base rejection
WAL/read-snapshot/runtime profile where selected
registered project and graph logical write plans
atomic ProjectStore + GraphSnapshot + ProjectSnapshot publication
current/last-known-good/failed-target identities
crash/cancel/recovery/reopen/query validation
retention/leases/checkpoint/backup/rebuild/GC
no mixed source/analyzer/recognizer/graph/store generations
```

No E2-D code before E2-A/B/C implementations and ProjectStore benchmarks/fixtures exist.

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
