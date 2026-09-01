# Roadmap

**Status:** operational documentation and implementation routing.

```text
documentation frontier: E3-B complete
next documentation package: E3-C service/application context orchestration
implementation frontier: not started
```

Documentation-ready does not mean executable. No Rust workspace, `Cargo.toml`, `.rs` files, or CI workflows exist yet.

## Milestone summary

| Milestone | Documentation outcome | Documentation state | Implementation state |
|---|---|---:|---:|
| E0 | Deterministic diagnostic vertical-slice contracts | Complete | Not started |
| E1 | ReferenceStore, full ReferenceView, annotations, pack build/validation | Complete | Not started |
| E2 | Graph, recognizers, full project candidate, ProjectStore publication | Complete | Not started |
| E3-A | Exact Blizzard UI source universe and SkeletonInputView producer | Complete | Not started |
| E3-B | Project Map, L0/L1, context packs and deterministic rendering | Complete | Not started |
| E3-C | Service/application context acquisition and use cases | Next | Not started |
| E4 | Search, explicit lineage, migration and impact | Planned | Not started |
| E5 | Named calibration packs with universal outputs | Planned | Not started |
| E6 | Optional Codebase Memory candidate bridge | Planned | Not started |
| E7 | LSP/MCP, release, signing, publication, rollback | Planned | Not started |

## E0 — executable diagnostic slice

Build `wow-core`, fixture `wow-reference`, `wow-emmy`, minimal `wow-project`, two bounded `wow-rules`, `wow-service`, and thin `apps/wow` only after every E0 fixture/profile/checksum is frozen.

Gate:

```text
one exact profile/reference/project/analyzer generation
merged generic and WoW findings
known API resolves; authoritative unknown API finding only under complete coverage
bounded Secret-local unsafe and guarded cases
NotEvaluated under missing/conflicted capability
project and platform evidence stay separate
1/2/N canonical output identical
no source/editor/client mutation
```

## E1 — Reference Pack

Build generic store foundation, persistent reference ingestion/evaluation/corrections/coverage, deterministic annotations, and nonrepairing pack build/validation.

Gate:

```text
exact source/profile/component pins
immutable validated ReferenceStore
raw unknowns and expired/conflicting corrections retained
negative authority honest
annotation source-map/loss/injection/editor gates pass
Ketho and consumer discrepancies classified
pack checksums/licenses/coverage close
logical rebuild determinism proven
```

## E2 — project graph and persistence

Build typed graph assertions/partitions/queries, structural recognizers, full TOC/XML/load/analyzer project indexing, incremental invalidation, and WAL manifested-partition ProjectStore publication.

Gate:

```text
producer-independent semantic identity
atomic partition replacement and coherent old/new readers
one exact TOC flavor; bounded safe XML; no source execution
no second Lua parser
recognizer precision/ambiguity/coverage retained
one-file update reuses unaffected partitions
inactive read-back validation before current CAS
crash/response-loss/recovery/lease/GC/backup gates pass
logical output deterministic; physical bytes classified separately
```

## E3-A — Blizzard UI source universe

Build a separately identified exact platform-source project with pinned materialization, package/TOC/XML/Lua/analyzer/recognizer/graph publication, coverage/license records, incremental replacement, structural fingerprints, and bounded `SkeletonInputView`.

Gate:

```text
exact source commit/tree/content/license/build profile
no flavor/profile merge
source implementation does not become API/runtime authority
separate ProjectStore/ProjectSnapshot/GraphSnapshot
removed source leaves no target records or object refs
source redistribution remains explicitly classified
bounded source and graph reads
```

## E3-B — Project Map and context

Build exact-universe binding, Project Maps, L0/L1 skeletons, closed control/effect projection, deterministic expansion/selection/pruning, source boundaries, context semantic packs, canonical JSON/Markdown, cache identities, metrics, and evaluation.

Gate:

```text
exact roots and immutable views
no search/model/parser/store internals
separate user/platform/reference identities
all claims have origin/evidence/coverage/conflict closure
mandatory records never pruned
omissions and unenumerated regions explicit
exact tokens only with frozen tokenizer/framing
source remains quoted untrusted data
no source-boundary/private-data violation
1/2/N and shuffled results byte-identical
```

## E3-C — service and application context use cases

Define and then implement exact current-resolution, retained view/lease acquisition, request normalization, context operation orchestration, renderer selection, canonical service envelopes, thin CLI commands, cancellation, status/exit codes, and resource closure.

Hard boundaries:

```text
service does not reimplement context/graph/store/parser/search/rule algorithms
applications depend on service only
symbolic current is resolved exactly once before canonical context request
no fallback/last-known-good substitution without explicit operation policy
no hidden natural-language search before E4
no background work
```

## E4 — search, lineage, migration, impact

Build exact/alias/FTS/shape/graph lanes, explicit cross-build lineage assertions, deterministic ranking explanations, and bounded impact plans. Similarity produces candidates, never authoritative replacement.

## E5 — calibration packs

Add audited pinned framework/addon calibration packs over universal normalized facts. Rename/path/repository mutations must preserve universal outputs. Pack removal changes only pack-owned partitions and coverage.

## E6 — Codebase Memory bridge

Optional MCP client emitting generation-bound external candidates. Bridge absence cannot break exact local search/context; candidates never become proven by transport or model confidence.

## E7 — production transports and releases

Thin LSP/MCP adapters, installers, release artifacts, signatures/provenance/SBOM where practical, least-privilege automation, compatibility/migration reports, last-known-good and rollback tests.

## Roadmap discipline

- Later fixtures cannot bypass earlier implementation gates.
- Architecture changes require an ADR and concrete failure of the accepted design.
- Stable framework contracts link the current external WoW engineering KB rather than copying patch-sensitive facts.
- Missing tools/probes/benchmarks/evaluations/client tests are skipped or NotEvaluated, never pass.
- The roadmap tracks outcomes and gates, not arbitrary percentages or crate counts.
