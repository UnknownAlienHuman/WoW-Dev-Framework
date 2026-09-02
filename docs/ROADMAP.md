# Roadmap

**Status:** operational documentation and implementation routing.

```text
documentation frontier: E7-A complete
next documentation package: E7-B public packaging, distribution, update and support lifecycle
implementation frontier: not started
```

No Rust workspace, `Cargo.toml`, `.rs` files, or CI workflows exist.

## Milestones

| Milestone | Documentation | Implementation |
|---|---:|---:|
| E0 diagnostic vertical slice | Complete | Not started |
| E1 Reference Pack stack | Complete | Not started |
| E2 graph/recognizers/project/ProjectStore | Complete | Not started |
| E3 Blizzard UI source/context/service | Complete | Not started |
| E4 search/lineage/migration/impact/service | Complete | Not started |
| E5-A calibration owner | Complete | Not started |
| E5-B durable review/holdout/submission | Complete | Not started |
| E5-C core-pack publication lifecycle | Complete | Not started |
| E6-A external candidate owner bridge | Complete | Not started |
| E6-B external orchestration/mapping/context/CLI | Complete | Not started |
| E7-A registry/session/daemon/LSP/MCP | Complete | Not started |
| E7-B public packaging/distribution/update/support | Next | Not started |

## Implementation remains E0-first

Documentation maturity never reorders implementation. Executable work begins with `wow-core` E0-A, then the frozen E0 reference/analyzer/project/rules/service/CLI slice. Every later package remains blocked until its prerequisite implementation commits, exact fixtures/profiles, probes, benchmarks, adapters, runtime/client evidence, and checksums exist.

The shortest route to a runnable repository is defined in [`LAUNCH_GATES.md`](LAUNCH_GATES.md). E5 governance, optional E6, and frontend/release completeness do not block the first E0 executable.

## E0–E4 product foundation

```text
E0 first deterministic status/check executable
E1 immutable Reference Pack build and validation
E2 exact addon project/load/graph/recognizer publication
E3 Blizzard UI source + Project Map/L0/L1/context
E4 exact-generation search, lineage, migration and static impact
```

E0 provides the first runnable binary. E1–E3 provide the first useful real-addon internal alpha. E4 plus a minimal implemented E7-A frontend provides the first external developer preview.

## E5 governed recognizer evolution

### E5-A

`wow-recognizers` owns exact candidate-source/corpus/provenance/label/split/pack validation, shadow matching, anti-overfitting mutations, graph receipts, per-case-first metrics, candidate artifacts, and deactivation plans.

### E5-B

`wow-service` and `apps/wow` own exact retained acquisition, durable `OperationId + CanonicalRequestDigest`, response-loss reconciliation, independent reviewer authorization, sealed-holdout access/audit/consumption, and immutable `PromotionSubmission`.

### E5-C

E5-C independently revalidates one exact submission, creates a distinct immutable `CorePackArtifact`, records provenance/SBOM/license/notices, obtains detached signatures, publishes `PublishedInactive`, performs fresh read-back, executes exact scoped canary and finite rollout, activates by profile-specific CAS, explicitly designates LKG, and performs exact rollback/revocation/deactivation with stale producer-partition closure.

No candidate relabeling, signature-as-proof, publication-side-effect activation, missing-signal pass, inferred previous/newest LKG, rollback history rewrite, stale partition, blind effect retry, or public distribution occurs in E5.

## E6 optional external semantic candidates

### E6-A

`wow-cbm` validates reviewed provider descriptors/capabilities and explicit `StableExternalGeneration | ObservedMutableGeneration | OpaqueExternalState`, sends closed bounded allow-listed queries, preserves raw-field loss/conflicts, and emits only `semantic_candidate + Candidate` results with provider-local scores and `UnverifiedProviderLocator` records.

### E6-B

`wow-service` resolves an exact provider configuration, acquires a secret-isolated host session, registers durable query identity, invokes E6-A, publishes immutable result artifacts, asks `wow-project`/`wow-reference` owners to map locators, records an explicit `Selected | Rejected | Deferred`, and passes one exact mapped root to normal context owners with a separate Candidate sidecar.

Provider labels/rank/score/repetition/stable state/zero result/mapping/selection/context inclusion never verify provider interpretation. Provider failure is lane-local; no hidden fallback or local capability downgrade.

## E7-A frontend operation registry and transports

Contracts:

- [`../crates/wow-service/e7/README.md`](../crates/wow-service/e7/README.md)
- [`../apps/wow/e7/README.md`](../apps/wow/e7/README.md)

### Closed registry

Every visible method/tool/command/resource is generated from an immutable reviewed `FrontendOperationRegistry` bound to exact service request/result/error schemas, owner implementation capabilities, effect/authorization classification, privacy/license policy, and protocol profile.

Negotiation can narrow but cannot add an operation or advertise missing implementation. There is no runtime reflection, generic `call_service`, MCP tool proxy, arbitrary RPC, shell, script, plugin, or model escape hatch.

### Session and workspace model

A frontend session binds exact client, protocol, registry, consumer policy, explicit workspace registrations, document-overlay heads, operation tickets, response journals, and close state.

Workspace roots are explicit untrusted inputs. No cwd, Git parent, editor, addon-folder, or WoW-installation inference. Unsaved documents are immutable project-owned overlay snapshots with strict versions, UTF-8 content digests, and negotiated UTF-16/UTF-8 position conversion. Stale/out-of-order changes require resynchronization.

### Supported protocol profiles

```text
cli-one-shot-v1
wow-local-jsonrpc/1 over current-user named pipe or Unix socket
LSP 3.18 over stdio
MCP 2025-11-25 over stdio
MCP 2025-11-25 local Streamable HTTP, explicit and disabled by default
```

The initial LSP profile supports lifecycle, explicit workspace folders, incremental synchronization, pull diagnostics, negotiated push compatibility, hover, definition, references, document/workspace symbols, completion, signature help, guarded code actions and call hierarchy. It deliberately omits rename, formatting, semantic tokens, inlay hints, generic execute-command, and automatic edit application.

The initial MCP profile exposes fixed implemented read-only tools and exact immutable resources. It omits prompts, sampling, elicitation, tasks, server-requested roots, generic tools, effecting provider/calibration/publication/edit/release operations, and model-controlled authorization.

### Lifecycle and isolation

Disconnect is not cancellation. Progress is nonauthoritative. Delivery loss does not alter service completion. Response replay returns exact retained bytes and never reexecutes service. Backpressure is bounded and prioritizes final results/errors/state changes over progress/logs.

Sessions isolate workspaces, overlays, authorization, private source, provider access, operations, results and journals. Unsaved source is memory-only by default. Local daemon/HTTP endpoints never become default remote listeners.

### E7-A implementation gate

Before E7-A Rust:

```text
implemented/frozen owner/service capabilities that will be advertised
exact LSP 3.18 MCP 2025-11-25 and local-daemon adapter/library pins
complete immutable registry and request/result/error schema digests
workspace/path/overlay/position/session/isolation profiles
canonical CLI/daemon/LSP/MCP wire and lifecycle vectors
supported client/platform fixtures
cancellation/disconnect/reconnect/backpressure/crash/redaction tests
measured resource/concurrency/latency/memory limits
all SHA-256 manifests
```

A minimal preview may enable only CLI plus one complete LSP or MCP profile. Disabled profiles are not advertised.

## Next — E7-B public release lifecycle

E7-B must define:

```text
supported OS/architecture/client/WoW-profile matrix
pinned Rust toolchain/dependency and reproducible build profiles
exact binary/data/config/cache/log/package layout
release manifest, SHA-256, signatures, SBOM and provenance attestations
portable archive and any installer/package formats
install/uninstall/upgrade/update/rollback/revocation/retirement
Reference Pack/core-pack/provider adapter compatibility manifests
stable/beta/nightly channel policy if enabled
secure update verification and downgrade prevention
privacy/telemetry/crash/log/data retention policy
incident response, support windows and compatibility policy
release candidate evaluation on real addon repositories/clients
public GitHub release and CI only after real commands exist
```

E7-B cannot call an internal E5 publication a public binary release, reuse private signing material in repository/CI logs, package development-only files into end-user artifacts, silently mutate editor settings, auto-discover user projects, or claim unsupported OS/client/profile compatibility.

After E7-B documentation is frozen, the documentation roadmap is complete and the next work package becomes E0-A Rust implementation.

## Discipline

Stable contracts link the current external WoW engineering KB rather than copy patch-sensitive facts. Missing tools/probes/benchmarks/authorization/signing/provider/protocol/runtime/client evidence are blocked or `NotEvaluated`, never passed. Architecture changes require an ADR and concrete failure of the accepted design. No CI/workflow without an explicit owner, real command, and release gate.