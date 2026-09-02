# Roadmap

**Status:** operational documentation and implementation routing.

```text
documentation frontier: E6-B complete
next documentation package: E7-A supported transport/session and developer-preview release boundary
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
| E7-A LSP/MCP/CLI-daemon/session developer preview | Next | Not started |
| E7-B public packaging/distribution/update/support | Planned | Not started |

## Implementation remains E0-first

Documentation maturity never reorders implementation. Executable work begins with `wow-core` E0-A, then the frozen E0 reference/analyzer/project/rules/service/CLI slice. Every later package remains blocked until its prerequisite implementation commits, exact fixtures/profiles, probes, benchmarks, adapters, runtime evidence, and checksums exist.

The shortest route to a runnable repository is defined in [`LAUNCH_GATES.md`](LAUNCH_GATES.md). E5 governance and the optional E6 provider lane are not prerequisites for the first E0 executable.

## E5-A — calibration evidence owner

`wow-recognizers` owns exact candidate-source/corpus/provenance/label/split/pack validation, shadow matching, anti-overfitting mutations, graph receipts, per-case-first metrics, candidate artifacts, and deactivation plans.

No commit-pin-only admission, donor-name semantics, split leakage, hidden Negative coercion, confidence above `Derived`/`Possible`, default graph publication, or core activation.

## E5-B — durable review, holdout, and submission

`wow-service` and `apps/wow` own exact retained acquisition, durable `OperationId + CanonicalRequestDigest`, response-loss reconciliation, reviewer authorization, sealed-holdout access/audit/consumption, immutable `PromotionSubmission`, conservative envelopes, and thin transport.

Review authorization, graph validity, holdout authorization, disclosure, submission, publication, activation, and runtime correctness remain separate.

## E5-C — immutable core-pack publication lifecycle

Contracts:

- [`../crates/wow-service/e5c/README.md`](../crates/wow-service/e5c/README.md)
- [`../apps/wow/e5c/README.md`](../apps/wow/e5c/README.md)

### Artifact and publication

E5-C independently reacquires and revalidates one exact E5-B submission and all mandatory E5-A/B evidence. It creates a distinct immutable `CorePackArtifact` with `trust_class=core`; the calibration candidate is never relabeled.

```text
submission revalidation
-> core artifact build/validation
-> provenance/SBOM/license/notices
-> detached signing + verification
-> PublishedInactive catalog record
-> fresh exact read-back validation
-> ValidatedInactive
```

Publication does not activate current/default. A valid signature proves bytes/key/profile binding only.

### Canary, rollout, activation, and rollback

Canary freezes exact population/membership, profile, window, typed observation schemas, denominators, criteria, privacy, and authorization. Missing/partial/conflicted/`NotEvaluated` required signals never pass.

Rollout uses finite stages with exact prior state, required evidence, authorization, budgets, and stop/pause/rollback criteria. Activation is profile-specific and guarded by exact current-record CAS. Last-known-good is explicitly designated, never inferred as previous/newest.

Rollback selects one exact retained, signature-valid, nonrevoked, compatible qualified target. It creates new activation/reindex/graph-closure/audit records and never rewrites history. New project/graph generations must prove stale target partitions absent, expected target partitions present, foreign partitions preserved, and coverage changes explicit.

### Durable effects and security

Signing, publication, canary, observation, rollout, activation, LKG, rollback, revocation, deactivation, reindex, closure, retention, and audit use exact durable identities and response-loss reconciliation. Private signing/deployment material, arbitrary SQL/scripts/models/tools, and public distribution do not enter E5-C requests/fixtures/CLI/results.

## E6-A — optional external semantic-candidate bridge

Contract: [`../crates/wow-cbm/e6/README.md`](../crates/wow-cbm/e6/README.md)

### Provider and state model

E6-A validates one exact reviewed provider descriptor and the intersection with one already-acquired transport/session capability observation. Runtime negotiation can narrow but cannot widen the descriptor.

```text
StableExternalGeneration
ObservedMutableGeneration
OpaqueExternalState
```

Stable state requires an immutable provider generation/index/corpus identity and sufficient receipt. Observed mutable state binds one session/observation episode. Opaque state is explicitly nonreproducible. Timestamps, uptime, same query/top result/count, or provider `current/latest` labels are not generation identity.

### Query, authority, and source boundary

E6-A accepts only closed bounded queries through an allow-listed `ExternalCandidateTransportPort`:

```text
provider_status
provider_capabilities
provider_generation
candidate_query
candidate_continue
candidate_explain
```

Every accepted result remains:

```text
provenance = semantic_candidate
confidence = Candidate
negative_authority = unavailable
```

Provider exact/verified/authoritative labels, top-1, sole result, high score, repetition, stable generation, and zero result do not increase authority. Scores are provider-local.

Repository/path/URI/revision/symbol/span/digest fields become `UnverifiedProviderLocator`. E6-A does not open, follow, read, or map them into project/reference truth.

### Zero, cache, degradation, and security

Zero means only no accepted candidates for the exact provider/state/query/profile/page under reported coverage. Zero after validation loss is separate. Neither creates local negative authority.

Continuation/cache bind exact descriptor/capability/state/query/profile and cumulative budgets. Cache cannot make stale fresh, opaque stable, partial complete, or Candidate verified.

Provider failure disables only the optional lane. There is no generic tool call, provider lifecycle/database access, raw SQL, script, model prompt, path/URL follow, source execution, hidden fallback, or local capability downgrade.

## E6-B — external orchestration, exact mapping, selection, and context

Contracts:

- [`../crates/wow-service/e6/README.md`](../crates/wow-service/e6/README.md)
- [`../apps/wow/e6/README.md`](../apps/wow/e6/README.md)
- [`../crates/wow-project/E6_B_EXTERNAL_LOCATOR_MAPPING.md`](../crates/wow-project/E6_B_EXTERNAL_LOCATOR_MAPPING.md)
- [`../crates/wow-reference/E6_B_EXTERNAL_LOCATOR_MAPPING.md`](../crates/wow-reference/E6_B_EXTERNAL_LOCATOR_MAPPING.md)
- [`../crates/wow-context/E6_B_EXTERNAL_CONTEXT_HANDOFF.md`](../crates/wow-context/E6_B_EXTERNAL_CONTEXT_HANDOFF.md)
- [`../crates/wow-store/E6_B_EXTERNAL_CANDIDATE_STORAGE.md`](../crates/wow-store/E6_B_EXTERNAL_CANDIDATE_STORAGE.md)

### Configuration and durable query

E6-B resolves one exact provider configuration, obtains one nonsecret credential-use authorization receipt, acquires one narrow reviewed session, registers `OperationId + CanonicalRequestDigest`, invokes E6-A, and publishes an immutable result/artifact catalog with fresh read-back validation.

No public request contains credential bytes, private endpoints, commands, process handles, provider database paths, or arbitrary MCP/RPC/tool input. Service never installs, starts, configures, indexes, imports, deletes, or mutates a provider.

Provider dispatch, response receipt, result publication, mapping, selection, context publication, retention, and audit are separate effects. Any uncertain effect becomes `OutcomeUnknown`; blind repeat is forbidden.

### Exact owner mapping

`wow-project` and `wow-reference` alone map an owner-neutral bounded locator projection into one exact retained generation. Results remain:

```text
ExactMapped
MultipleMappings
NoMappingWithOwnerAuthority
NoMappingPartial
Conflict
NotEvaluated
Failed
```

Service never maps by same name, nearest path, rank, score, order, popularity, or floating current. Clean no-mapping requires owner negative authority. `ExactMapped` proves locator-to-owner-record identity only.

### Explicit selection

The caller supplies exact result, candidate, mapping, intended use, and `Selected | Rejected | Deferred`. Service records the decision and never chooses top/sole/highest-score candidates. Selection is not verification, acceptance, lineage, replacement, edit authorization, or core promotion.

### Exact-root context handoff

Context build requires retained `ExactMapped` plus `Selected`. Service reacquires exact project/reference/graph views and invokes one existing `wow-context` operation with the exact mapped root.

Normal context artifacts contain only exact local owner evidence. Provider result fields remain in a separate `ExternalCandidateSidecar`; they never become `ContextSemanticPack` facts.

### Optional degradation

Provider/session/query failure remains lane-local. Exact local reference/project/graph/search/context/diagnostics/rules remain independently usable. No hidden fallback is permitted.

### E6-B implementation gate

Before Rust:

```text
implemented/frozen E0–E6-A prerequisites
exact provider configuration/authorization/session/transport/store ports
exact project/reference mapping and graph/context acquisition ports
credential/session/quota/cancellation/reconciliation profiles
result/artifact/catalog/continuation/cache profiles
mapping/negative-authority/selection/context/sidecar profiles
privacy/license/security/retention/audit/recovery/close profiles
synthetic and admitted real provider/mapping/context fixtures
response-loss and cancellation at every effect boundary
measured resource/quota/owner/context limits
canonical service/CLI vectors and all SHA-256 manifests
```

## Next — E7-A supported frontend/session transports

E7-A must define concrete supported transport/application packages over `wow-service` only:

```text
CLI daemon and local session protocol
LSP capability/session/request mapping
MCP server/tool schemas and exact service mapping
schema/version/capability negotiation
project/profile/session registration
bounded messages/streaming/progress/backpressure
cancellation/disconnect/reconnect/response-loss/lease/close
multi-client isolation, configuration and credential boundaries
no arbitrary tool/shell/RPC escape hatch
developer-preview packaging and compatibility manifest
```

Transport success cannot upgrade service evidence. One transport request maps to one service operation unless an explicit higher-level workflow is itself a documented service operation.

## E7-B public release lifecycle

E7-B later owns reproducible packaging, release artifacts, signatures/checksums/SBOM/provenance, installers/packages, update channels, compatibility/support policy, rollback/retirement, incident response, and public distribution. It cannot bypass E5 signing/publication distinctions or service authority boundaries.

## Launch gates

See [`LAUNCH_GATES.md`](LAUNCH_GATES.md) for the exact distinction between first runnable E0 bootstrap, useful internal alpha, developer preview, governed beta, and public supported v1.

## Discipline

Stable contracts link the current external WoW engineering KB rather than copy patch-sensitive facts. Missing tools/probes/benchmarks/authorization/signing/provider adapters/mapping/runtime/client evidence are blocked or `NotEvaluated`, never passed. Architecture changes require an ADR and concrete failure of the accepted design. No CI/workflow without explicit owner instruction.