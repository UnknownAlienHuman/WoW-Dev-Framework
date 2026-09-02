# E5-C decisions

**Status:** normative.

## P5C-001 — Submission is input, not publication

An E5-B `PromotionSubmission` requests consideration. E5-C independently revalidates it before any artifact or catalog effect.

## P5C-002 — Core artifact is distinct from calibration candidate

A published core pack has its own immutable identity, trust class, execution profile compatibility, attestations, and producer namespace. Candidate bytes are never relabeled in place.

## P5C-003 — Publication and activation are separate

`PublishedInactive` is the first catalog state. Current/default execution records remain unchanged until a distinct authorized CAS operation.

## P5C-004 — Signature proves bytes, key, and policy only

A valid signature does not prove matcher semantics, graph validity, runtime behavior, absence of false positives, or authorization to activate.

## P5C-005 — Signing credentials stay outside canonical data

Only nonsecret key/attestation references and detached signatures enter artifacts. Private material is never stored by the repository or CLI.

## P5C-006 — Read-back is independent

After publication, open exact stored bytes/catalog records through a fresh read path and rerun artifact, signature, provenance, graph, recognizer, license, and store validation before canary or activation.

## P5C-007 — Canary cohorts are exact artifacts

Cohort membership/selection/profile is frozen, bounded, privacy-reviewed, and independently authorized. A percentage without exact deterministic membership is invalid.

## P5C-008 — Canary observations are typed evidence

Only registered observation schemas/ports are accepted. Free-form anecdotes, repository stars, issue counts, or model summaries cannot satisfy a signal gate.

## P5C-009 — Canary evidence is scoped

A canary pass applies only to the exact pack/profile/cohort/observation window/capabilities. It is not ecosystem-wide or future-build runtime proof.

## P5C-010 — Rollout is finite and stage-based

Every stage has exact eligibility, authorization, cohort expansion, signal, stop, pause, timeout, and rollback criteria. No open-ended automatic rollout.

## P5C-011 — Activation is profile-specific CAS

One exact current/default record is updated only against its expected prior digest. No silent rebase, newest selection, or cross-profile activation.

## P5C-012 — Last-known-good is explicit

A retained publication becomes LKG only through an authorized designation with exact evidence/profile/retention. Previous or newest is not automatically good.

## P5C-013 — Rollback is a new effect

Rollback creates new operation, activation, audit, closure, and observation records while reselecting an exact retained publication. History is never rewritten.

## P5C-014 — Failed target retains identity

A failed, paused, revoked, quarantined, or rolled-back publication keeps its original IDs and evidence. It is never relabeled as LKG or another target.

## P5C-015 — Revocation and deactivation are distinct

Revocation declares a publication/signature/profile unfit for further use under a policy. Deactivation changes active selection. Either can require the other but they are separate records.

## P5C-016 — Historical generations remain immutable

Pack changes cause new project/recognizer/graph generations. Stale producer partitions disappear only from new target generations; old evidence remains retained.

## P5C-017 — Partition closure is independently validated

`wow-project`, `wow-recognizers`, and `wow-graph` retain their owner semantics. Service coordinates and verifies receipts; it does not delete rows or invent graph state.

## P5C-018 — Response loss is not no effect

Publication, signing, canary start, observation append, activation, rollout, rollback, revocation, deactivation, and LKG designation all require idempotency and exact reconciliation.

## P5C-019 — Public distribution is deferred

E5-C owns an internal immutable publication/catalog lifecycle. Public release channels, downloads, update manifests, and package distribution remain E7.

## P5C-020 — Authorization scopes remain separate

Submission review, signing, publication, canary, activation, rollout, rollback, revocation, and distribution use separate exact scopes. One grant does not imply another.

## P5C-021 — Missing signals never pass

Unknown, partial, conflicted, truncated, unavailable, cancelled, or `NotEvaluated` required canary/closure evidence blocks advancement.

## P5C-022 — Runtime observations cannot rewrite static evidence

Observations may gate rollout and create scoped evidence; they do not mutate E5-A/B labels, metrics, graph proofs, or candidate history.

## P5C-023 — No latest/best/previous shortcuts

Artifact, publication, activation, LKG, rollback, and revocation selectors are exact and guard their expected digests/profiles.

## P5C-024 — Applications remain transport-only

`apps/wow` invokes one E5-C service operation and cannot sign, publish, select cohorts, authorize, activate, or rollback locally.