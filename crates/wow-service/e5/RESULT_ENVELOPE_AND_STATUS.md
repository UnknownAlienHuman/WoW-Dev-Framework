# E5-B result envelopes and status

**Status:** normative.

Every public operation returns one tagged `E5BServiceResultEnvelope` containing operation/request IDs, exact resolved input/output artifact IDs, owner results, service status, validation and authorization states, coverage/conflicts/blockers/omissions, holdout visibility/consumption/audit state, idempotency/response-loss/retention/closure state, privacy/license/security summaries, mandatory nonclaims, and canonical digest.

Statuses:

```text
Complete
NoChange
Partial
CandidateOnly
Blocked
ConflictBlocked
Truncated
OutcomeUnknown
NotEvaluated
Cancelled
Failed
```

Default conservative precedence:

```text
Failed
OutcomeUnknown
Cancelled
NotEvaluated
ConflictBlocked
Blocked
Truncated
Partial
CandidateOnly
NoChange
Complete
```

Validation payloads remain `Valid`, `Invalid`, or `NotEvaluated`; authorization decisions retain their domain states and are never flattened to booleans.

A completed validation can return outer `Complete` with payload `Invalid`. A complete shadow evaluation whose evidence remains candidate-only returns `CandidateOnly`. Warnings and aggregate metrics cannot hide blockers.

`NoChange` requires exact owner/store proof that the same canonical request already produced the same artifact. Empty output or unavailable capability is not no-change.

`OutcomeUnknown` means an effect may have committed but could not be reconciled. The envelope includes exact recovery identifiers and forbids blind retry.

Promotion-submission envelopes always include:

```text
not-published
not-activated
not-default
not-runtime-verified
generalization-limited-to-stated-evidence
E5-C-independent-validation-required
```

Canonical serialization excludes host paths, process IDs, terminal state, raw credentials, raw signatures, vault tokens, and incidental timings.