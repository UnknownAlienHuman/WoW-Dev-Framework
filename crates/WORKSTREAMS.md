# Agent workstreams and integration order

**Status:** operational routing through documentation frontier E6-B.

Documentation-ready remains `implementation_state = not-started` until executable code, exact fixtures/checksums, probes, benchmarks, authorization/signing/observation/provider adapters, runtime evidence, and evaluations exist.

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
-> E7-A supported LSP/MCP/CLI-daemon/session transport
-> E7-B public packaging/distribution/update/support lifecycle
```

## Global rules

- One agent owns one primary package/crate or one explicitly named cross-crate seam.
- Missing implementation/tool/probe/benchmark/authorization/signing/vault/observation/provider/runtime evidence is blocked or `NotEvaluated`, never pass.
- Patch-sensitive WoW claims route through the current external KB and exact source/runtime evidence.
- Repository/addon/owner/path/provider/popularity/model identity never becomes hidden semantics.
- Applications and future transports invoke `wow-service` only.
- No CI/workflow without explicit owner instruction and a real frozen command.

## E5-C publication lifecycle

Primary contracts:

- [`wow-service/e5c/`](wow-service/e5c/README.md)
- [`../apps/wow/e5c/`](../apps/wow/e5c/README.md)

```text
exact E5-B PromotionSubmission
-> independent E5-C revalidation
-> distinct CorePackArtifact
-> provenance/SBOM/license/notices
-> detached signing + independent verification
-> PublishedInactive
-> fresh read-back validation
-> exact canary cohort/assignment/observations/evaluation
-> finite authorized rollout stages
-> profile-specific current-record CAS
-> explicit retained last-known-good
-> exact rollback/revocation/deactivation
-> new project/graph generations with stale partition closure
```

### Active owners

```text
wow-service: orchestration, authorization use, durable effects, envelopes
wow-recognizers: core pack semantics and producer namespace
wow-graph: graph output/partition/closure validation
wow-project: exact reindex and new project generations
wow-store: immutable objects/catalog/current/retention/GC
apps/wow: transport only
```

### Hard stops

```text
no trust in submission label without exact revalidation
no relabelled candidate as core artifact
no signature as semantic/runtime proof
no private signing/deployment material in repository/CLI/results
no publication side-effect activation
no canary percentage without exact population/membership
no untyped anecdote/issue/model signal
no missing/partial/conflict/NotEvaluated required signal as pass
no time-only or open-ended rollout
no latest/best/previous/default activation or rollback target
no stale current CAS rebase
no inferred previous/newest last-known-good
no rollback history rewrite
no historical project/graph mutation
no stale partition or hidden coverage change
no blind effect retry after response loss
no public distribution in E5-C
```

### Implementation gate

Before E5-C Rust:

```text
implemented/frozen E0–E5-B prerequisites
exact submission/artifact/recognizer/graph/project/store ports
signing and authorization adapters/profiles without committed secrets
provenance/SBOM/license/reproducibility profiles
canary population/privacy/observation/signal profiles
finite rollout/activation/LKG/rollback/revocation/closure profiles
response-loss/retention/audit/recovery profiles
canonical service/CLI/artifact/signature/state vectors
synthetic and admitted real canary/rollout/rollback corpora
measured thresholds and all SHA-256 manifests
```

## E6-A external Candidate-only bridge

Primary contract: [`wow-cbm/e6/`](wow-cbm/e6/README.md)

```text
reviewed ProviderDescriptor
+ negotiated capability intersection
+ StableExternalGeneration | ObservedMutableGeneration | OpaqueExternalState
+ closed bounded query
+ already-acquired allow-listed transport
-> bounded raw response validation
-> loss/unknown/conflict preservation
-> semantic_candidate + Candidate normalization
-> provider-local score/rank metadata
-> UnverifiedProviderLocator
-> scoped zero/partial/truncated/failure state
-> exact continuation/cache validation
-> optional lane-local result
```

### Active dependency

```text
wow-cbm -> wow-core
```

No store/project/reference/graph/search/context/service/app/provider-SDK dependency is activated.

### Hard stops

```text
no provider process/session/credential ownership
no provider install/configure/index/import/delete or database access
no generic arbitrary MCP/tool call
no authority above semantic_candidate + Candidate
no provider score fusion across providers
no provider locator converted to owner source handle
no path/URL/source follow
no zero-result negative authority
no opaque-state exact replay/freshness claim
no hidden provider/stale-cache/model/web/local-search fallback
no exact-local capability degradation from provider failure
no background work, source execution, secret material, or unrestricted source retention
```

### Implementation gate

Before E6-A Rust:

```text
implemented/frozen wow-core
reviewed provider descriptor and adapter contract
transport/capability probe reports
external-state/query/normalization/score/loss/locator/zero/continuation/cache profiles
privacy/license/security/cancellation profiles
synthetic compatible/malformed/hostile/partial/opaque/mutable/stale fixtures
measured resource limits
canonical result/explanation/artifact/comparison/error vectors
all member/bundle SHA-256 values
```

## E6-B external orchestration, mapping, selection, and context

Primary contracts:

- [`wow-service/e6/`](wow-service/e6/README.md)
- [`../apps/wow/e6/`](../apps/wow/e6/README.md)
- [`wow-project/E6_B_EXTERNAL_LOCATOR_MAPPING.md`](wow-project/E6_B_EXTERNAL_LOCATOR_MAPPING.md)
- [`wow-reference/E6_B_EXTERNAL_LOCATOR_MAPPING.md`](wow-reference/E6_B_EXTERNAL_LOCATOR_MAPPING.md)
- [`wow-context/E6_B_EXTERNAL_CONTEXT_HANDOFF.md`](wow-context/E6_B_EXTERNAL_CONTEXT_HANDOFF.md)
- [`wow-store/E6_B_EXTERNAL_CANDIDATE_STORAGE.md`](wow-store/E6_B_EXTERNAL_CANDIDATE_STORAGE.md)

```text
exact provider configuration
-> credential-use authorization reference
-> narrow provider session and E6-A transport
-> durable OperationId + CanonicalRequestDigest
-> E6-A exact descriptor/state/query/result
-> immutable result/artifact catalog and read-back
-> exact project/reference owner mapping of UnverifiedProviderLocator
-> explicit candidate selection receipt
-> exact mapped root to existing context owner
-> separate external Candidate sidecar
-> retention/audit/reconciliation/reverse closure
-> one-call CLI
```

### Active owners

```text
wow-service: configuration/session orchestration, durable effects, envelopes
wow-cbm: provider descriptor/state/query/normalization and Candidate ceiling
wow-project: exact project locator mapping
wow-reference: exact reference locator mapping
wow-graph: exact graph view for context universe
wow-context: exact-root context artifacts
wow-store: generic immutable object/catalog/effect/retention substrate
apps/wow: transport only
host adapters: credential authorization and provider session construction
```

### Hard stops

```text
no raw credential/private endpoint/process/database handle in public seams
no provider install/start/configure/index/import/delete
no generic MCP/tool/RPC/SQL/script/model/shell surface
no service-side source/path/URL inspection
no mapping by name/rank/score/proximity/order/popularity
no clean no-mapping without owner negative authority
no mapping treated as provider semantic proof
no implicit top/sole/highest-score candidate selection
no selection treated as verification/acceptance/edit authorization
no provider metadata in ContextSemanticPack truth
no recursive public service call for context
no hidden fallback or local capability downgrade
no blind repeat after OutcomeUnknown
no public success before retention/audit/reverse close
```

### Implementation gate

Before E6-B Rust:

```text
implemented/frozen E0–E6-A prerequisites
exact provider configuration/authorization/session/transport/store ports
exact project/reference mapping and graph/context acquisition ports
credential/session/quota/cancellation/reconciliation profiles without committed secrets
result/artifact/catalog/continuation/cache profiles
mapping/negative-authority/selection/context/sidecar profiles
privacy/license/security/retention/audit/recovery/close profiles
synthetic and admitted real stable/mutable/opaque/zero/ambiguous/conflict fixtures
response-loss and cancellation at every effect boundary
measured resource/quota/owner/context limits
canonical service/CLI vectors and all SHA-256 manifests
```

## Next — E7-A

Owners: thin transport/application packages over `wow-service`; no transport may import lower framework crates.

Required scope:

```text
explicit supported CLI-daemon/LSP/MCP transport profiles
schema/capability/version negotiation
project/profile/session registration
one transport request -> one service operation
bounded messages/streams/progress/backpressure
cancellation/disconnect/reconnect/response-loss/lease/close behavior
multi-client isolation and credential/configuration boundaries
no arbitrary tool/shell/RPC escape hatch
developer-preview packaging and compatibility manifest boundary
```

E7-B later owns public release artifacts, checksums/signatures/SBOM/provenance, installers/packages, update channels, rollback/retirement, support and compatibility policy.

## Launch routing

See [`../docs/LAUNCH_GATES.md`](../docs/LAUNCH_GATES.md). The shortest executable path remains E0-A through E0-F; E6 and E5 governance do not block the first runnable bootstrap.

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