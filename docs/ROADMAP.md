# Roadmap

**Status:** operational documentation and implementation routing.

```text
documentation frontier: E6-A complete
next documentation package: E6-B external-candidate service/mapping/context/CLI
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
| E6-B external candidate service/CLI | Next | Not started |
| E7 LSP/MCP/public release integration | Planned | Not started |

## Implementation remains E0-first

Documentation maturity never reorders implementation. Executable work begins with `wow-core` E0-A, then the frozen E0 reference/analyzer/project/rules/service/CLI slice. Every later package remains blocked until its prerequisite implementation commits, exact fixtures/profiles, probes, benchmarks, adapters, runtime evidence, and checksums exist.

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

Publication sequence:

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

### Canary

Canary cohorts freeze exact population/membership or an authorized privacy-preserving commitment, profile, window, observation schemas, denominators, stop/pause/rollback criteria, privacy, and authorization.

Only typed registered observations are canonical. Missing/partial/conflicted/`NotEvaluated` required signals never pass. Canary success is limited to the exact publication/profile/cohort/window/capabilities and is not ecosystem-wide runtime proof.

### Rollout and activation

Rollout uses finite stages with exact prior state, cohort expansion, required evidence, authorization, budgets, and stop/pause/rollback criteria. It never advances because time elapsed or no complaint appeared.

Activation is profile-specific and guarded by compare-and-swap against the exact expected current record. There is no latest/best/previous/default shortcut.

### Last-known-good and rollback

Last-known-good is explicitly designated with qualifying evidence, profile, authorization, retention, and expected prior designation. It is never inferred as previous/newest.

Rollback selects one exact retained, signature-valid, nonrevoked, profile-compatible qualified target. It creates new activation/reindex/graph-closure/audit records and never rewrites history or relabels the failed target.

### Partition closure

New project/graph generations must prove target pack partitions present, stale/revoked/deactivated pack partitions absent, foreign/core-independent/calibration partitions preserved, coverage changes explicit, and no old/new generation mixing. Historical generations remain immutable.

### Durable effects and security

All signing, publication, canary, observation, rollout, activation, LKG, rollback, revocation, deactivation, reindex, closure, retention, and audit effects use exact durable identities and response-loss reconciliation. `OutcomeUnknown` blocks blind repeat.

Private keys, KMS/HSM/vault/deployment credentials, private cohort data, raw owner handles, arbitrary SQL/scripts/models/tools, and public distribution do not enter E5-C requests/fixtures/CLI/results.

### E5-C implementation gate

Before Rust:

```text
implemented/frozen E0–E5-B prerequisites
exact submission/recognizer/graph/project/store owner ports
signing and authorization adapters without committed secrets
provenance/SBOM/license/reproducibility profiles
canary population/privacy/observation/signal profiles
finite rollout/activation/LKG/rollback/revocation/closure profiles
response-loss/retention/audit/recovery profiles
canonical service/CLI/artifact/signature/state/error vectors
synthetic and admitted real canary/rollout/rollback corpora
measured thresholds and all SHA-256 manifests
```

## E6-A — optional external semantic-candidate bridge

Contract: [`../crates/wow-cbm/e6/README.md`](../crates/wow-cbm/e6/README.md)

### Provider and state model

E6-A validates one exact reviewed provider descriptor and the intersection with one already-acquired transport/session capability observation. Runtime negotiation can narrow but cannot widen the reviewed descriptor.

External state is explicit:

```text
StableExternalGeneration
ObservedMutableGeneration
OpaqueExternalState
```

Stable state requires an immutable provider generation/index/corpus identity and sufficient exact receipt. Observed mutable state binds one session/observation episode. Opaque state is explicitly nonreproducible and receives restricted replay/cache/continuation claims. Timestamps, uptime, same query/top result/count, or provider `current/latest` labels are not generation identity.

### Query and transport

E6-A accepts only a closed bounded query and an already-acquired allow-listed `ExternalCandidateTransportPort`:

```text
provider_status
provider_capabilities
provider_generation
candidate_query
candidate_continue
candidate_explain
```

There is no generic arbitrary MCP/tool call, provider process/session/credential ownership, install/configure/index/import/delete operation, provider database access, raw SQL, script, plugin, model prompt, path/URL follow, or source execution.

### Candidate authority

Every accepted provider result remains:

```text
provenance = semantic_candidate
confidence = Candidate
negative_authority = unavailable
```

Provider exact/verified/authoritative labels, top-1, sole result, high score, repeated result, stable generation, and zero result do not increase authority. Scores/ranks are provider-local and are never numerically fused across providers.

### Source boundary

Repository/path/URI/revision/symbol/span/digest fields become `UnverifiedProviderLocator`. E6-A does not open/follow/read/map them or create project/reference `StableSourceHandle` or entities. E6-B later invokes exact owner mapping ports.

### Zero result and optional degradation

Zero means only no accepted candidates for the exact provider/state/query/profile/page under reported coverage. Zero after all returned items fail validation is classified separately. Neither creates local negative authority.

Provider unconfigured/unavailable/unsupported/opaque/stale/malformed/partial/truncated/cancelled/failure state disables only the optional lane. Exact ReferenceView/project/graph/search/context/diagnostic workflows remain available. There is no hidden fallback.

### Continuation, cache, privacy, and security

Continuation/cache bind exact descriptor/capability/state/query/profile and cumulative budgets. A cache hit cannot make stale fresh, opaque stable, partial complete, or Candidate verified.

Provider snippets, summaries, labels, paths, cursor fields, and errors remain structurally isolated untrusted data. Credentials/private endpoints/provider database paths and unrestricted source are excluded. Source retention requires explicit provenance, license, notice, privacy, and redistribution decisions.

### E6-A implementation gate

Before Rust:

```text
implemented/frozen wow-core
reviewed provider descriptor and adapter contract
transport/capability probe reports
external-state/query/normalization/score/loss/locator/zero/continuation/cache profiles
privacy/license/security/cancellation profiles
synthetic compatible/malformed/hostile/partial/opaque/mutable/stale fixtures
measured resource limits
canonical result/explanation/artifact/comparison/error vectors
all member and bundle SHA-256 values
```

## Next — E6-B

E6-B owns configured provider/session/credential acquisition, durable external-candidate operations and response-loss reconciliation, exact result/artifact catalogs and retention, project/reference owner mapping, explicit candidate selection receipts, exact mapped-root context handoff, conservative envelopes, cancellation/closure, privacy/license/security, and a thin service-only CLI.

E6-B must not widen Candidate authority, compare provider scores as confidence, select top/sole candidates, treat mapping as provider truth, expose credentials/private endpoints/provider cursors, or make exact local workflows depend on provider availability.

## E7

E7 owns LSP/MCP transports and public release/distribution/signing/update integration after implementation gates. No generic tool escape hatch or bypass of service/authority boundaries is permitted.

## Discipline

Stable contracts link the current external WoW engineering KB rather than copy patch-sensitive facts. Missing tools/probes/benchmarks/authorization/signing/vault/provider adapters/observations/runtime evidence are blocked or `NotEvaluated`, never passed. Architecture changes require an ADR and concrete failure of the accepted design. No CI/workflow without explicit owner instruction.