# Roadmap

**Status:** operational documentation and implementation routing.

```text
documentation frontier: E5-C complete
next documentation package: E6-A optional external semantic-candidate bridge
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
| E6-A external candidate owner bridge | Next | Not started |
| E6-B external candidate service/CLI | Planned | Not started |
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

## Next — E6-A optional external candidate bridge

Owner: `wow-cbm`; direct dependency: `wow-core` only.

Define exact reviewed provider descriptors and external-state classes, closed bounded allow-listed candidate queries, deterministic loss-preserving normalization, hard `semantic_candidate + Candidate` authority ceiling, provider-local scores, unverified locators, zero-result negative-authority prohibition, continuation/cache, optional degradation, privacy/license/security, and an E6-B mapping/service handoff.

E6-A must not read/write provider databases, expose arbitrary MCP/tool calls, invoke models, map locators into project/reference truth, create graph/lineage/replacement proof, or make exact local workflows depend on provider availability.

## E6-B and E7

E6-B later owns configured provider/session/credential acquisition, durable external-candidate orchestration, exact project/reference owner mapping, explicit selection receipts, exact-root context handoff, and thin CLI while preserving Candidate-only authority.

E7 owns LSP/MCP transports and public release/distribution/signing/update integration after implementation gates.

## Discipline

Stable contracts link the current external WoW engineering KB rather than copy patch-sensitive facts. Missing tools/probes/benchmarks/authorization/signing/vault/observations/runtime evidence are blocked or `NotEvaluated`, never passed. Architecture changes require an ADR and concrete failure of the accepted design. No CI/workflow without explicit owner instruction.