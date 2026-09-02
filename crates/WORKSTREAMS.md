# Agent workstreams and integration order

**Status:** operational routing through documentation frontier E7-A.

Documentation-ready remains `implementation_state = not-started` until executable code, exact fixtures/checksums, probes, benchmarks, authorization/signing/provider/protocol adapters, runtime/client evidence, and evaluations exist.

## Global order

```text
E0-A wow-core
-> E0-B wow-reference fixture + E0-C wow-emmy
-> E0-D wow-project fixture
-> E0-E wow-rules
-> E0-F wow-service + apps/wow diagnostics
-> E1 Reference Pack stack
-> E2 graph/recognizers/project/ProjectStore
-> E3 Blizzard UI source/context/service
-> E4 search/lineage/migration/static-impact/service
-> E5-A calibration owner
-> E5-B durable calibration review/holdout/submission
-> E5-C core-pack publication/signing/canary/rollout/rollback
-> E6-A wow-cbm external semantic candidates
-> E6-B service/CLI mapping/context orchestration
-> E7-A operation registry, sessions, overlays, daemon, LSP and MCP
-> E7-B reproducible packaging/distribution/update/support lifecycle
-> E0-A implementation begins after documentation freeze
```

## Global rules

- One agent owns one primary package/crate or one explicitly named cross-crate seam.
- Missing implementation/tool/probe/benchmark/authorization/signing/vault/provider/protocol/runtime/client evidence is blocked or `NotEvaluated`, never pass.
- Patch-sensitive WoW claims route through the current external KB and exact source/runtime evidence.
- Repository/addon/owner/path/provider/popularity/model/client identity never becomes hidden semantics.
- Applications and frontend transports invoke `wow-service` only.
- No CI/workflow without explicit owner instruction, real executable commands, and a launch/release gate.

## First executable workstream — E0

Executable work begins here regardless of later documentation maturity:

```text
wow-core E0-A identities/evidence/results
-> frozen ReferenceView fixture + pinned EmmyLua adapter
-> minimal exact project generation
-> bounded diagnostics
-> wow-service status/check
-> thin apps/wow CLI
```

Before each crate activation, populate the package's required implementation/profile/fixture/checksum values. Do not activate all documented crates at once. The R0 completion gate is defined in [`../docs/LAUNCH_GATES.md`](../docs/LAUNCH_GATES.md).

## E5 governed recognizer lifecycle

Primary routes:

- [`wow-recognizers/e5/`](wow-recognizers/e5/README.md)
- [`wow-service/e5/`](wow-service/e5/README.md)
- [`wow-service/e5c/`](wow-service/e5c/README.md)

```text
E5-A
    exact admitted corpus/provenance/labels/splits
    shadow-only packs, mutations, metrics, graph receipts
    candidate/deactivation artifacts

E5-B
    exact retained acquisition and durable runs
    independent review authorization
    sealed holdout access/audit/consumption
    immutable PromotionSubmission

E5-C
    independent submission revalidation
    distinct CorePackArtifact
    provenance/SBOM/license/signing
    PublishedInactive + fresh read-back
    exact canary + finite rollout
    profile-specific activation/LKG/rollback/revocation/closure
```

Hard stops remain: no commit-pin-only admission, donor-name semantics, label/split leakage, candidate relabeling, signature-as-proof, publication-side-effect activation, missing-signal pass, inferred LKG, rollback history rewrite, stale-partition retention, blind effect retry, or public distribution in E5.

E5 implementation waits for all prerequisite owner implementations, real admitted corpora, independent reviews/holdout, signing/authorization/canary adapters, measured thresholds, response-loss tests, and checksum closure.

## E6 optional external Candidate lane

Primary routes:

- [`wow-cbm/e6/`](wow-cbm/e6/README.md)
- [`wow-service/e6/`](wow-service/e6/README.md)
- [`../apps/wow/e6/`](../apps/wow/e6/README.md)

```text
reviewed provider descriptor/state/query
-> E6-A loss-preserving semantic_candidate + Candidate
-> UnverifiedProviderLocator
-> E6-B exact provider/session/result catalog
-> exact project/reference owner mapping
-> explicit Selected | Rejected | Deferred receipt
-> exact mapped root to normal context owner
-> separate ExternalCandidateSidecar
```

### Active E6 owners

```text
wow-cbm: descriptor/state/query/normalization and Candidate ceiling
wow-service: configuration/session orchestration and durable effects
wow-project/wow-reference: exact locator mapping
wow-context: exact-root context only
wow-store: generic immutable objects/effects/retention
apps/wow: transport only
host adapters: secret-isolated provider session construction
```

Hard stops remain: no generic provider/MCP tool; no provider database/index lifecycle; no source/path opening by service; no cross-provider score fusion; no zero-result negative authority; no implicit mapping/selection; no provider metadata in exact context truth; no hidden fallback or local capability downgrade; no secret material in public seams.

E6 is optional and may ship disabled. Enabling it requires a real reviewed adapter, exact stable/mutable/opaque fixtures, mapping/selection/context tests, response-loss and cancellation tests, measured benefit and limits, privacy/license closure, and checksums.

## E7-A frontend sessions and transports

Primary routes:

- [`wow-service/e7/`](wow-service/e7/README.md)
- [`../apps/wow/e7/`](../apps/wow/e7/README.md)
- [`wow-project/E7_A_DOCUMENT_OVERLAYS.md`](wow-project/E7_A_DOCUMENT_OVERLAYS.md)
- [`wow-emmy/E7_A_OVERLAY_ANALYSIS.md`](wow-emmy/E7_A_OVERLAY_ANALYSIS.md)
- [`wow-rules/E7_A_LIVE_DIAGNOSTICS.md`](wow-rules/E7_A_LIVE_DIAGNOSTICS.md)
- [`wow-store/E7_A_SESSION_AND_RESPONSE_JOURNAL.md`](wow-store/E7_A_SESSION_AND_RESPONSE_JOURNAL.md)

### Contract flow

```text
implemented service capability
-> immutable FrontendOperationRegistry
-> exact protocol profile and client session
-> explicit workspace/project/profile registration
-> exact saved generation or versioned unsaved overlay
-> one CLI/daemon/LSP/MCP semantic request
-> one service operation
-> bounded progress/cancellation/backpressure
-> exact final result and delivery journal
-> explicit close/reconciliation
```

### Protocol profiles

```text
CLI one-shot
wow-local-jsonrpc/1 over current-user named pipe or Unix socket
LSP 3.18 over stdio
MCP 2025-11-25 over stdio
MCP 2025-11-25 local Streamable HTTP, explicit and disabled by default
```

### Active owners

```text
wow-service
    operation registry, sessions, workspace/document orchestration,
    transport-neutral feature requests, cancellation/reconciliation

wow-project
    explicit workspace validation and immutable document overlays

wow-emmy
    exact overlay analysis using the pinned analyzer

wow-rules/search/context/reference/graph
    existing exact owner results consumed through service

wow-store
    generic registry/session/lease/response-journal/retention substrate

apps/wow
    protocol framing, lifecycle, output and local endpoint host only
```

### Hard stops

```text
no transport importing lower framework crates
no reflection or generic call-service/tool/RPC method
no capability advertisement without implementation
no implicit cwd/Git/editor/WoW workspace discovery
no stale/out-of-order document change applied best-effort
no LSP position conversion without exact overlay/encoding
no editor-specific semantic fork or automatic edit application
no default MCP effecting tools, prompts, sampling, elicitation or tasks
no model invocation treated as user authorization
no floating current/latest MCP resource URI
no remote daemon/MCP listener by default
no disconnect treated as cancellation
no progress treated as completion
no response replay that reexecutes service
no unbounded queues, source/resource output or cross-client access
no public success before retention/audit/reverse close
```

### Implementation gate

Before E7-A Rust:

```text
implemented/frozen owner/service capabilities that will be advertised
exact LSP 3.18, MCP 2025-11-25 and local daemon adapter/library pins
complete immutable operation registry and schema digests
workspace/path/overlay/position-encoding/session/isolation profiles
canonical CLI/daemon/LSP/MCP request/result/progress/error/reconnect vectors
Windows and claimed Unix endpoint/platform fixtures
reference LSP and MCP host client fixtures
cancellation/disconnect/response-loss/backpressure/crash/redaction tests
measured frame/document/source/result/concurrency/latency/memory limits
all member and bundle SHA-256 values
```

The minimal developer preview may implement CLI plus one LSP or MCP profile first, but disabled transports must not be advertised. The complete E7-A package gates the public supported release.

## Next — E7-B release lifecycle

E7-B owns documentation for:

```text
pinned Rust toolchain and reproducible build profiles
supported OS/architecture/client/profile compatibility matrix
binary/data/config/cache/log layout
portable archive and optional installer/package layout
release manifests/checksums/signatures/SBOM/provenance attestations
Reference Pack/core-pack/provider adapter compatibility manifests
install/uninstall/upgrade/rollback/revocation/retirement
stable/beta/nightly channel policy if enabled
secure update verification and no-downgrade rules
privacy/telemetry/crash/log/data-retention policy
incident response and support lifecycle
release candidate evaluation on real addon repositories
public GitHub release and CI only after real commands exist
```

After E7-B documentation is frozen, the next repository work package is E0-A implementation—not another architecture milestone.

## Seam request format

```text
requesting package/crate
owning crate
required operation/data
rejected workaround
why existing seam is insufficient
smallest proposed seam
cycle/identity/security/privacy/license/evidence impact
fixture/mutation proving it
freeze/migration impact
```

Do not implement a missing seam in the wrong crate.