# Agent workstreams and integration order

**Status:** operational routing through documentation frontier E6-A.

Documentation-ready remains `implementation_state = not-started` until executable code, exact fixtures/checksums, probes, adapters, benchmarks, authorization and runtime evidence exist.

## Global order

```text
E0 diagnostic vertical slice
-> E1 Reference Pack stack
-> E2 graph/recognizers/project/ProjectStore
-> E3 Blizzard UI source/context/service
-> E4 search/lineage/migration/static-impact/service
-> E5-A calibration owner
-> E5-B review/holdout/submission
-> E5-C core publication/signing/canary/rollout/rollback
-> E6-A external Candidate-only owner bridge
-> E6-B provider/session/mapping/context service and CLI
-> E7 LSP/MCP/public release integration
```

## Global rules

- One agent owns one primary package/crate.
- Missing implementation/tool/probe/benchmark/authorization/credential adapter/runtime evidence is blocked or `NotEvaluated`, never pass.
- Patch-sensitive WoW claims route through the current external KB and exact source/runtime evidence.
- Repository/addon/owner/path/provider/popularity/model identity never becomes hidden semantics.
- Applications import `wow-service` only.
- No CI/workflow without explicit owner instruction.

## E6-A owner work

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

No store/project/reference/graph/search/context/service/app/MCP-provider SDK dependency is activated.

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
no background work, source execution, private credential, or unrestricted source retention
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

## Next — E6-B

Owners: `wow-service` and thin `apps/wow`; collaborators: `wow-cbm`, `wow-project`, `wow-reference`, `wow-context`, `wow-store`.

Required flow:

```text
explicit provider configuration
-> credential/session authorization and narrow transport acquisition
-> durable OperationId + CanonicalRequestDigest
-> exact E6-A descriptor/state/query execution
-> immutable result/artifact catalogs and retention
-> exact project/reference owner mapping of UnverifiedProviderLocator
-> explicit candidate selection receipt
-> exact mapped root to existing context service
-> reverse closure and conservative envelope
-> one-call CLI
```

Mapping states must preserve exact mapped, multiple, no-mapping-with-authority, partial, conflict, `NotEvaluated`, and failed. Mapping proves locator-to-owner-record identity only. Selection never occurs from top-1, sole result, score, name, path, snippet, or provider label.

E6-B must keep provider credential/session/database/index lifecycle behind narrow ports, preserve `OutcomeUnknown`, never expose provider cursors/credentials/private endpoints, and never make local exact workflows depend on the provider.

## E7

After E6 implementation gates, define thin LSP/MCP transports and public release/distribution/update integrity. Do not expose arbitrary tool calls or bypass existing service/authority boundaries.

## Seam request

State requesting/owning package, exact crossing operation/data, rejected workaround, insufficiency, smallest seam, cycle/identity/security/privacy/license/evidence impact, proving fixtures/mutations, and freeze/migration impact.