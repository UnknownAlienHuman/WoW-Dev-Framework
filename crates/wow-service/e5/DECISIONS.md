# E5-B decisions

**Status:** normative.

## S5B-001 — E5-B orchestrates; E5-A evaluates

All corpus, split, matcher, mutation, metric, candidate, and deactivation semantics remain in `wow-recognizers` E5-A.

## S5B-002 — Durable operation identity precedes effects

Every effecting operation is keyed by exact `OperationId + CanonicalRequestDigest` before owner calls.

## S5B-003 — Response loss is not effect absence

A timeout or lost response after an owner/store effect yields `OutcomeUnknown` until exact receipt reconciliation.

## S5B-004 — Exact catalogs never imply latest

Catalog queries return `UniqueEligible`, `None`, `MultipleConflicting`, `NotEvaluated`, or `Failed`. Service never chooses by chronology or score.

## S5B-005 — Metric eligibility is not authorization

`PromotionEligibleByMetrics` cannot authorize review, holdout access, submission, publication, or activation.

## S5B-006 — Reviewer authorization is external and narrow

A `ReviewAuthorizationPort` verifies principal, role, scope, attestation, expiry, revocation, and replay. It does not judge recognizer correctness.

## S5B-007 — Holdout authorization is a separate capability

Review permission does not imply access to sealed holdout membership, labels, facts, source, or detailed results.

## S5B-008 — Candidate and run freeze before holdout access

Candidate pack bytes, implementation/profile IDs, evaluator, run request, and sealed generation are immutable before authorization.

## S5B-009 — Holdout disclosure is minimized

Default output returns aggregate/per-case classifications sufficient for gates without disclosing raw labels or source. Expanded disclosure requires a distinct authorization/profile.

## S5B-010 — Holdout consumption is lineage-aware

Once results can influence a descendant candidate, the holdout generation is consumed for that candidate lineage and cannot be represented as untouched evidence.

## S5B-011 — Audit records are append-only semantic evidence

Access, review, run, and submission audit records are immutable and superseded by linked records, never edited in place.

## S5B-012 — Promotion submissions are requests, not effects

A valid submission is input to E5-C. It does not publish or activate a pack.

## S5B-013 — Review and graph validation remain independent

Authorized review cannot repair graph-invalid output. Graph-valid output cannot bypass reviewer authorization.

## S5B-014 — Candidate bytes cannot change during review

Any pack, profile, corpus, split, implementation, threshold, or label change creates a new candidate and invalidates prior run/review bindings as defined by compatibility rules.

## S5B-015 — Hard-gate failures remain blockers

Aggregate metrics cannot hide mandatory false positives, leakage, named-condition, security, graph, determinism, or deactivation failures.

## S5B-016 — Unknown and partial states remain explicit

`Unknown`, `Possible`, `Candidate`, `Partial`, `Truncated`, `Conflict`, `NotEvaluated`, and `Cancelled` are never coerced to Negative or pass.

## S5B-017 — Repository identity does not grant trust

Repository ownership, stars, author, organization, GitHub role, or user account does not admit a corpus, authorize review, or approve promotion.

## S5B-018 — Durable artifacts use exact immutable IDs

Runs, case results, metric reports, review decisions, access grants, audit logs, and submissions bind exact digests and owner generations.

## S5B-019 — No hidden rerun on retry

Retry behavior is explicit. Read retries may repeat exact reads; effect retries require durable state reconciliation and never silently duplicate work.

## S5B-020 — Cancellation closes synchronously

No background evaluation, unseal, review, or submission task continues after a cancelled public call returns.

## S5B-021 — Privacy and license can only narrow output

Higher orchestration layers cannot expose source, labels, reviewer material, or holdout details prohibited by an owner/profile.

## S5B-022 — E5-C is the only publication/activation owner

E5-B has no core-pack catalog, default profile mutation, canary assignment, rollout percentage, current pointer, or rollback operation.

## S5B-023 — Applications remain transport-only

`apps/wow` invokes one service operation and does not resolve catalogs, authorize principals, open holdout vaults, or inspect E5-A artifacts directly.

## S5B-024 — Patch-sensitive WoW facts stay outside orchestration

E5-B stores exact profile/source/publication references but no hard-coded current API, event, Secret, taint, or runtime behavior.
