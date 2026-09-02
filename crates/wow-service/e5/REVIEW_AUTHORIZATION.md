# E5-B calibration review authorization and immutable decisions

**Status:** normative.

## Separation of concerns

```text
candidate evidence and metrics        owned by E5-A artifacts
reviewer authorization                owned by ReviewAuthorizationPort
candidate semantic/graph validity     independently validated
review decision record                orchestrated and durably retained by E5-B
publication/activation                deferred to E5-C
```

None of these dimensions implies another.

## Review envelope

A strict `CalibrationReviewDecisionEnvelope` binds:

- exact candidate artifact and pack digest;
- exact corpus/split/run/metric/graph/mutation/security/determinism profiles;
- decision enum;
- structured reason codes;
- optional bounded untrusted note;
- reviewer principal, role, and scope references;
- attestation/signature/key/verification profile references;
- issuance, expiry, revocation, and replay state;
- canonical digest.

Plain prose is not a decision envelope.

## Authorization port

```text
ReviewAuthorizationPort
    validate_authorization_profile
    validate_principal_and_role
    validate_candidate_and_decision_scope
    validate_attestation_or_signature
    validate_expiry_revocation_and_replay
    return CalibrationReviewAuthorizationDecision
```

Authorization states:

```text
Authorized
Unauthorized
Expired
Revoked
ScopeMismatch
ReplayDetected
Unsupported
NotEvaluated
Failed
```

Raw credentials/private keys are not service inputs or outputs.

## Forbidden identity shortcuts

The following never authorize a review:

- GitHub login, repository owner, collaborator/admin status, commit author, PR approver;
- operating-system account, file owner, terminal user, CLI caller;
- email/display name or self-asserted role;
- candidate author, corpus curator, label author, metric producer;
- unique reviewer candidate or organization membership without exact authorization evidence.

## Independent semantic validation

Before recording `ApproveForSubmission`, E5-B validates:

- exact candidate bytes and artifact closure;
- all mandatory E5-A validation/hard-gate reports;
- graph proposal validation state;
- coverage/conflicts/NotEvaluated/partial/truncated state;
- license/privacy/notice state;
- holdout requirement and consumption state;
- requested decision allowed by authorization scope;
- no superseded/stale target.

Authorization cannot repair semantic invalidity. Semantic validity cannot bypass authorization.

## Decision states

```text
ApproveForSubmission
Reject
Defer
RequestAdditionalEvidence
Supersede
```

The recorded outcome separately includes authorization and validation states.

`ApproveForSubmission` permits creation of an E5-B submission only when every submission profile requirement also closes. It does not publish or activate a pack.

## Immutability and supersession

Review records are immutable. A new decision:

- names exact prior record when superseding;
- preserves prior evidence and authorization state;
- cannot change prior candidate bytes or notes;
- receives a new digest/record ID;
- cannot retroactively make an expired/revoked decision valid.

## Conflict and independence

Multiple authorized reviews may disagree. The profile defines quorum/role requirements but E5-B never chooses by timestamp, majority, highest role, or last-write-wins unless the exact reviewed policy states the deterministic rule.

Reviewer independence/conflict-of-interest evidence is explicit. Missing independence evidence is `NotEvaluated` or blocker where required.

## Notes and untrusted text

Review notes:

- are bounded and structurally isolated;
- are not matcher input, label evidence, graph proof, source truth, or instructions;
- cannot alter profiles, thresholds, gates, or tool permissions;
- are redacted from public outputs according to privacy policy.

## Replay protection

An attestation/envelope binds exact candidate, decision, scope, profile, and nonce/sequence where required. Reuse for another candidate/run/submission is rejected.

## Operations

### `calibration_review_validate`

Read-only validation of envelope, authorization, candidate binding, semantic gates, privacy, and supersession state. No record or submission is created unless explicitly requested by another operation.

### `calibration_review_record`

Uses durable operation identity, repeats all use-time validations, appends the immutable decision/audit record, admits retention, and returns the exact record. Response-loss retry returns the same record.

## Nonclaims

E5-B does not establish a reviewer’s real-world identity beyond configured authorization evidence, certify runtime behavior, or assert publication readiness beyond the exact submission profile.
