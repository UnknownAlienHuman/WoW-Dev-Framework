# E5-C result envelopes and status

**Status:** normative.

Every public operation returns one tagged `E5CServiceResultEnvelope` containing operation/request IDs, exact submission/artifact/attestation/signature/publication/canary/rollout/current/LKG/rollback/revocation/closure IDs, operation payload, conservative status, validation and authorization states, signal/coverage/conflicts/blockers/omissions, idempotency/response-loss/retention/audit/closure state, privacy/license/distribution state, nonclaims, and canonical digest.

Statuses:

```text
Complete
NoChange
Partial
PublishedInactive
ValidatedInactive
CanaryOnly
RolloutPaused
Active
RolledBack
Revoked
Deactivated
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
Revoked
RolledBack
Deactivated
RolloutPaused
CanaryOnly
PublishedInactive
ValidatedInactive
Active
NoChange
Complete
```

Operation-specific payloads preserve independent validation, authorization, signature, canary, rollout, current CAS, LKG, rollback and closure states. A completed validation may have payload `Invalid`; outer completion does not turn invalid into valid.

`PublishedInactive` text must not say active/default. `CanaryOnly` must not say generally safe. `Active` applies only to the exact execution profile/current record. `RolledBack` reports exact target and unresolved project/fleet closure. `Revoked` reports scope and required actions.

`NoChange` requires exact owner proof of an already-existing identical effect and state. Empty output or unchanged display name is insufficient.

`OutcomeUnknown` maps to unsafe-to-retry and includes exact recovery IDs. Warnings cannot hide a blocker or failed mandatory signal.

Mandatory nonclaims vary by operation but include as applicable:

```text
not-publicly-distributed
signature-is-not-semantic-proof
canary-is-not-global-runtime-proof
active-only-for-exact-profile
rollout-does-not-prove-future-builds
rollback-does-not-erase-history
static-evidence-not-rewritten
```

Default errors/output redact private signing/authorization material, cohort membership, source bodies, credentials, private paths, and raw owner handles.