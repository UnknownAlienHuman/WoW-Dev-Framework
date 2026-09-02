# Roadmap

**Status:** operational documentation and implementation routing.

```text
documentation frontier: E6-A complete
next documentation package: E6-B external-candidate service/mapping/context/CLI
implementation frontier: not started
```

No Rust workspace, `Cargo.toml`, `.rs` files, or CI workflows exist.

## Summary

| Milestone | Documentation | Implementation |
|---|---:|---:|
| E0–E4 foundation/search/lineage/context | Complete | Not started |
| E5-A calibration owner | Complete | Not started |
| E5-B review/holdout/submission | Complete | Not started |
| E5-C core publication lifecycle | Complete | Not started |
| E6-A external Candidate-only bridge | Complete | Not started |
| E6-B external service/mapping/context/CLI | Next | Not started |
| E7 LSP/MCP/public distribution | Planned | Not started |

Documentation maturity never reorders implementation; executable work begins with E0-A.

## E5-C recap

E5-C independently revalidates one exact PromotionSubmission, builds a distinct immutable CorePackArtifact, produces attestations and detached signatures, publishes inactive and reads back, gathers exact scoped canary evidence, advances finite rollout stages, activates by profile-specific CAS, explicitly designates LKG, and performs immutable rollback/revocation/deactivation plus project/graph stale partition closure. Internal publication is not public distribution.

## E6-A — optional external semantic candidates

Contract: [`../crates/wow-cbm/e6/README.md`](../crates/wow-cbm/e6/README.md)

### State model

```text
StableExternalGeneration
ObservedMutableGeneration
OpaqueExternalState
```

Stable requires an immutable provider generation/index/corpus identity and receipt. Observed mutable binds one session/observation episode. Opaque state is explicitly nonreproducible and receives restricted cache/continuation claims. Timestamps, uptime, same query/top result/count, or provider `current/latest` labels are not generation identity.

### Query and authority

E6-A validates reviewed provider descriptors/capability intersections, invokes one allow-listed narrow transport operation, bounds and validates the response, records unknown/loss/conflict/coverage, and normalizes candidates.

Every result remains:

```text
provenance = semantic_candidate
confidence = Candidate
negative_authority = unavailable
```

Provider exact/verified/authoritative labels, top-1, sole result, high score, repeated result, stable generation, and zero result do not increase authority. Scores are provider-local and never numerically fused across providers.

### Source boundary

Repository/path/URI/revision/symbol/span/digest fields are `UnverifiedProviderLocator`. E6-A does not open/follow/read/map them or create project/reference `StableSourceHandle`/entities. E6-B later invokes exact owner mapping ports.

### Zero and degradation

Zero means only no accepted candidates for the exact provider/state/query/profile/page under reported coverage. Zero after all items fail validation is classified separately. Neither creates local negative authority.

Provider unconfigured/unavailable/unsupported/opaque/stale/malformed/partial/truncated/cancelled/failure state disables only the optional lane. There is no hidden fallback or local capability downgrade.

### Security

No provider install/start/configure/index/import/delete, provider database, raw MCP/tool call, SQL/script/model prompt, path/URL follow, source execution, credential/session acquisition, private endpoint, unrestricted source retention, or background task exists in E6-A.

### E6-A implementation gate

Before Rust:

```text
implemented/frozen wow-core
reviewed provider descriptor and adapter contract
transport/capability probes
external-state/query/normalization/score/loss/locator/zero/continuation/cache profiles
privacy/license/security/cancellation profiles
synthetic compatible/malformed/hostile/partial/opaque/mutable/stale fixtures
measured limits
canonical result/explanation/artifact/comparison/error vectors
all SHA-256 manifests
```

## Next — E6-B

Owners: `wow-service` + `apps/wow`; collaborators: `wow-cbm`, `wow-project`, `wow-reference`, `wow-context`, `wow-store`.

Define provider/session/credential-port acquisition, durable operation and response-loss state, exact result/artifact catalogs and retention, project/reference owner mapping, explicit selection receipts, exact mapped-root context handoff, conservative envelopes, privacy/license/security, cancellation/closure, and thin CLI.

E6-B must not widen Candidate authority, compare provider scores as confidence, choose top/sole candidates, treat mapping as provider truth, expose credentials/private endpoints/provider cursor bytes, or make exact local workflows depend on provider availability.

## E7

Define thin LSP/MCP and public release/distribution/update integration only after implementation gates. No generic tool escape hatch or bypass of service/authority boundaries.

## Discipline

Patch-sensitive facts remain in the external WoW engineering KB. Missing tools, probes, adapters, benchmarks, authorization, runtime evidence, or client validation are blocked/`NotEvaluated`, never pass. No CI/workflow without explicit owner instruction.