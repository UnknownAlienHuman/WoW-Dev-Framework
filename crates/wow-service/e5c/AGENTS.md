# AGENTS.md — `wow-service` E5-C

## Scope

Implement transport-independent publication lifecycle orchestration only. Do not modify E5-A/B evidence or reproduce recognizer, graph, project, store, signature-provider, authorization-provider, or canary-observation algorithms.

## Before coding

1. Read repository/crate instructions and all E5-A, E5-B, and E5-C contracts.
2. Verify exact prerequisite implementation commits and fixture bundle digests.
3. Freeze publication, artifact, signing, attestation, canary, observation, rollout, activation, last-known-good, rollback, revocation, closure, retention, security, result, CLI, and canonicalization profiles.
4. Freeze canonical machine fixtures before the first Rust commit.

## Submission discipline

- Select one exact `PromotionSubmission` with digest/profile guards.
- Revalidate every mandatory referenced artifact independently.
- Never repair, omit, reinterpret, or replace a blocker.
- A changed candidate/submission/profile creates a new publication request.

## Artifact discipline

- Build a distinct immutable `CorePackArtifact`; never relabel the E5-A candidate pack.
- Bind exact rule/operator/registry/schema/profile bytes and provenance.
- Preserve rejected/deferred/conflicted evidence and nonclaims.
- No repository/addon/owner/path/popularity condition may enter production rules.

## Signing discipline

- Use narrow authorization and signing ports.
- Private keys, KMS/HSM/vault credentials, bearer tokens, or signing secrets never enter canonical requests, fixtures, logs, CLI, or repository.
- Signature validity proves byte/key/profile binding only; it does not prove semantic or runtime correctness.

## Publication discipline

- Publish immutable `PublishedInactive` first.
- Reopen through a fresh exact read and run all validation catalogs before canary/activation.
- Never update current/default as a side effect of publication.
- Response loss becomes `OutcomeUnknown`; do not blindly republish.

## Canary and rollout discipline

- Cohorts are exact, bounded, authorized, privacy-reviewed artifacts.
- Observations are typed and scoped; missing/unknown signals are not pass.
- Canary success is not global correctness.
- Rollout uses finite explicit stages and exact stop/pause/rollback criteria.
- No percentage or stage advances from wall-clock passage alone.

## Activation and rollback discipline

- Activation uses an exact profile-specific current record and guarded CAS.
- Last-known-good is explicitly designated after required evidence and retention; it is not previous/newest.
- Rollback selects an exact retained validated publication and creates new activation/audit records; it never rewrites history or relabels failed targets.
- Revocation/deactivation state is explicit and does not delete historical evidence.

## Partition closure

- Activating or rolling back a pack schedules exact project reindexing through `wow-project`.
- `wow-recognizers` owns producer namespace/pack execution semantics.
- `wow-graph` independently validates producer partitions and stale closure.
- Historical project/graph generations remain immutable.

## Lifecycle

- Register `OperationId + CanonicalRequestDigest` before every effect.
- Persist exact receipts after every effect boundary.
- No public success before retention and reverse-order resource closure.
- No detached/background work after return.

## Completion report

```text
exact submission/artifact/publication/signature/profile IDs
canary/observation/rollout/activation/LKG/rollback/revocation IDs
owner calls and effect receipts
current-record CAS base/target
project/graph partition closure
coverage/conflicts/blockers/nonclaims
response-loss/recovery/retention/audit/closure
security/privacy/license/distribution state
tests and skipped/blocked/NotEvaluated gates
```