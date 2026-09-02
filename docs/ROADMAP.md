# Roadmap

**Status:** normative implementation and launch routing.

```text
planned architecture/documentation: complete through E7-B
implementation frontier: not started
next owned work package: I0-A / wow-core E0-A
first runnable gate: R0 / E0-A through E0-F
first supported target intent: Windows x86-64 MSVC after complete evidence
```

Documentation completion does not activate a Cargo workspace, implement an operation, prove a platform, or create a release. Remaining project work is implementation, conformance, integration, and release evidence in dependency order.

## Milestone ledger

| Package | Responsibility | Documentation | Implementation | First gate |
|---|---|---:|---:|---:|
| E0-A | `wow-core` identities, evidence, coverage, results | Complete | Not started | R0 |
| E0-B | frozen `wow-reference` fixture | Complete | Not started | R0 |
| E0-C | pinned `wow-emmy` adapter | Complete | Not started | R0 |
| E0-D | minimal `wow-project` generation | Complete | Not started | R0 |
| E0-E | first bounded diagnostics | Complete | Not started | R0 |
| E0-F | `wow-service` plus `apps/wow` status/check | Complete | Not started | R0 |
| E1 | persistent Reference Pack stack | Complete | Not started | A0 |
| E2 | graph, recognizers, project indexing, ProjectStore | Complete | Not started | A0 |
| E3 | Blizzard UI source and context | Complete | Not started | A0 |
| E4 | search, lineage, migration, static impact | Complete | Not started | A1 |
| E5-A | calibration corpus and shadow candidates | Complete | Not started | B0 |
| E5-B | review, holdout, and submission orchestration | Complete | Not started | B0 |
| E5-C | core-pack publication, canary, rollout, rollback | Complete | Not started | B0 |
| E6-A | optional external Candidate owner bridge | Complete | Not started | B0 optional |
| E6-B | external session, mapping, selection, context | Complete | Not started | B0 optional |
| E7-A | sessions, overlays, CLI, daemon, LSP, MCP | Complete | Not started | A1/V1 |
| E7-B | build, evidence, signing, bundle, channel, install, update, support | Complete | Not started | V1 |

## Implementation sequence

The sequence is fixed by ownership and dependency, not documentation recency:

```text
I0-A wow-core
-> I0-B reference fixture + I0-C Emmy adapter
-> I0-D project fixture
-> I0-E rules
-> I0-F service + wow status/check

-> I1 persistent store/reference/annotations/reference builder
-> I2 graph/recognizers/project index/ProjectStore
-> I3 Blizzard UI source/context/service
-> I4 search/lineage/migration/impact/service
-> I5 governed recognizer evolution
-> I6 optional external provider lane
-> I7-A product host and editor frontends
-> I7-B public release/update/support lifecycle
```

The exact handoff is in [`IMPLEMENTATION_HANDOFF.md`](IMPLEMENTATION_HANDOFF.md). Workspace activation and dependency policy are in [`WORKSPACE_AND_BUILD_PLAN.md`](WORKSPACE_AND_BUILD_PLAN.md). Required command surfaces are in [`CONFORMANCE_COMMANDS.md`](CONFORMANCE_COMMANDS.md).

## R0 — first runnable vertical slice

Implement E0-A through E0-F only:

```text
wow-core exact primitives
+ one frozen ReferenceView fixture
+ one pinned EmmyLua adapter
+ one immutable project fixture generation
+ three bounded diagnostic paths
+ wow-service orchestration
+ wow status / wow check
```

R0 requires canonical bytes, exact profiles and generations, complete/partial/conflict/`NotEvaluated` behavior, cancellation, broken pipe, resource limits, deterministic repeats, and populated E0 fixture/checksum manifests.

R0 does not require persistent Reference Pack building, graph/search/context, calibration, external providers, LSP/MCP, update, or public distribution.

## A0 — useful internal alpha

Implement E1 through E3 after R0:

- immutable persistent ReferenceStore/View with restricted source evaluation and corrections;
- annotation generation, parity, and loss tracking;
- typed graph and universal structural recognizers;
- TOC/XML/load/Lua project indexing without repository code execution;
- WAL ProjectStore publication, read-back, current CAS, and recovery;
- separate Blizzard UI source universe;
- Project Map, L0/L1, and bounded L2 context;
- context service and CLI;
- one admitted real addon revision and exact WoW profile evaluation.

A0 may remain CLI-only.

## A1 — developer preview

Implement E4 plus the selected E7-A frontend slice:

- exact-generation search lanes and explanations;
- cross-generation lineage, change, migration, and static-impact records;
- explicit candidate selection before search-to-context;
- immutable sessions and project-owned unsaved-buffer overlays;
- one `wow` binary with direct CLI and at least one real LSP or MCP frontend;
- exact cancellation, backpressure, disconnect, and recovery behavior;
- real developer-task usefulness and latency/resource evidence.

The canonical E7-A profiles are `wow-local-jsonrpc/1`, LSP 3.18, and MCP 2025-11-25. LSP uses incremental `textDocument/didChange`; a full-document change is an exact replacement. A preview exposes only implemented capabilities and omits the rest from the registry.

## B0 — governed recognizer evolution

Implement E5 only after its prerequisites:

```text
admitted licensed corpora and provenance groups
independent labels and leakage-safe splits
shadow packs, mutations, and per-case metrics
independent review authorization
sealed holdout authorization, audit, and consumption
PromotionSubmission
independent E5-C revalidation
immutable core artifact and attestations/signatures
PublishedInactive and read-back
exact canary and finite rollout
profile-specific activation and explicit LKG
rollback, revocation, deactivation, and partition closure
```

No metric, review, holdout, signature, or canary result automatically authorizes the next gate.

## Optional E6 lane

E6 can remain disabled without blocking the exact local product. Before enabling one provider:

- implement a reviewed descriptor and transport adapter;
- classify external state as stable, observed mutable, or opaque;
- prove the hard `semantic_candidate + Candidate` ceiling;
- preserve provider-local scores and zero-result nonauthority;
- implement exact configuration, session authorization, durable results, owner mapping, caller selection, and exact-root context sidecar;
- prove credential, privacy, license, outage, degradation, and measurable task benefit.

Provider unavailability must never block exact local workflows.

## E7-A — final product host

One planned `wow` executable provides:

```text
wow <one-shot command>
wow daemon run|status|shutdown
wow lsp --transport stdio
wow mcp --transport stdio
wow mcp --transport streamable-http-local   # explicit, disabled by default
```

Required E7-A implementation:

- content-addressed service operation registry and compatibility manifest;
- immutable sessions and exact workspace/project/profile bindings;
- project-owned overlay generations for unsaved buffers;
- LSP 3.18 incremental synchronization with exact full-replacement and resynchronization behavior;
- diagnostics, completion, signature help, hover, definitions, references, symbols, call hierarchy, and guarded code-action candidates;
- local named-pipe or Unix-socket daemon only;
- standalone LSP/MCP modes with no hidden daemon fallback;
- static read-only MCP registry by default;
- progress, artifact streams, backpressure, cancellation, reconnect, and recovery;
- multi-client isolation and no generic invoke, shell, or tool escape hatch.

## E7-B — supported release lifecycle

Implement the complete release path:

```text
exact source tree + Cargo.lock + Rust toolchain + dependency materialization
-> narrow isolated build executor
-> at least two independent unsigned builds
-> reproducibility comparison
-> artifact self-description and complete required tests
-> SBOM, provenance, license/notices, and checksums
-> portable and target-specific signatures
-> deterministic ReleaseBundle
-> exact support matrix and immutable ReleaseCandidate
-> provider-neutral publication, public read-back, and channel CAS
-> signed update manifest
-> staged verified install, migration, current CAS, and self-check
-> explicit LastKnownRunnable
-> exact rollback, revocation, retirement, and incident lifecycle
```

The first target intent is `x86_64-pc-windows-msvc`. It is not supported until exact Windows build, path/ACL, named-pipe, console, LSP/MCP, code-signing, installation/helper, migration, update, rollback, clean-machine, and real-addon suites pass.

GitHub Releases may be the first distribution adapter, but it is not release architecture, authorization, or artifact trust.

Baseline update behavior is explicit. There is no hidden startup check, background download or install, telemetry, crash upload, or remote configuration.

## CI and automation

CI is deferred until real commands exist. Once implemented, workflows may invoke the exact commands defined in [`CONFORMANCE_COMMANDS.md`](CONFORMANCE_COMMANDS.md):

```text
contract, manifest, fixture, and checksum validation
format, lint, unit, integration, mutation, and security tests
Windows target build and client/platform tests
independent reproducibility builds
SBOM, provenance, bundle, and candidate validation
manual separately authorized channel publication
```

Workflow YAML never becomes a second semantic or release engine. A skipped or unavailable required job is not pass.

## State transitions

When an implementation package completes, update from fresh evidence:

```text
package CONTRACT and CHECKSUMS implementation state
crates/MANIFEST.json
PROJECT_COMPLETION_MATRIX.md
LAUNCH_GATES.md when a gate changes
WORKSTREAMS.md when the next owner changes
release and support manifests when applicable
```

Do not report percentages. State the exact completed package or gate and remaining blockers.

## Next action

```text
I0-A / wow-core E0-A

1. freeze exact Rust toolchain and minimal dependencies;
2. create the root Cargo workspace with only crates/wow-core active;
3. implement the complete E0-A invariant types and canonical serialization;
4. finalize E0-A fixtures and checksums;
5. run all E0-A acceptance, property, mutation, resource, security, and dependency tests;
6. merge before activating I0-B or I0-C.
```

Any new architecture proposal now requires a concrete implementation-discovered failure of the accepted contract and the smallest tested seam or ADR change.
