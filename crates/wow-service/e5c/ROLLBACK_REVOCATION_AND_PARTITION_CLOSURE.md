# E5-C rollback, revocation, deactivation, and partition closure

**Status:** normative.

## Rollback target

A rollback target is one exact retained, signature-valid, profile-compatible, nonrevoked publication with successful required validation and an explicit qualifying LKG or rollback-target decision. Service never chooses previous/newest/oldest/highest-version automatically.

## Rollback protocol

```text
register durable rollback operation
-> acquire exact current/failed publication and expected current digest
-> acquire exact rollback target and qualification/authorization
-> validate target signatures, attestations, license/privacy and retention
-> create project reindex and recognizer/graph partition closure plan
-> guarded CAS current record to rollback target
-> persist activation/rollback effect receipt
-> coordinate new project/graph generations
-> validate stale target partitions absent and rollback-target partitions present
-> append audit/observation/revocation/deactivation records as required
-> retain evidence
-> close resources
```

If post-CAS closure fails, public result is failed/`OutcomeUnknown`/quarantined according to exact effect state—not success with a warning.

## Historical immutability

Rollback does not:

- delete or mutate the failed publication;
- rewrite prior current/rollout/canary records;
- relabel the failed target as the rollback target;
- mutate historical project/graph generations;
- erase observations, incidents, reviews, or submission evidence.

## Deactivation

Deactivation removes an exact publication from an execution profile/current assignment under authorization. It may leave no active pack only when profile policy explicitly permits and records resulting coverage/capability state. It never silently substitutes another pack.

## Revocation

A `RevocationRecord` identifies exact artifact/publication/signature/profile scope, reason/evidence, authority, required serving/activation/deactivation actions, effective ordering evidence, and audit chain.

Revocation can be triggered by signature/key compromise, semantic/security defect, license/privacy change, corrupted object/catalog state, invalid provenance, or other reviewed policy reason. It does not retroactively falsify historical validation; it changes current eligibility and required action.

## Partition closure

Core-pack execution outputs are producer-owned partitions. New project reindex generations must prove:

```text
expected active core publication/pack/producer namespace
all target pack rule partitions present or explicitly NotEvaluated
all stale replaced/revoked/deactivated pack partitions absent
foreign/core-independent/calibration shadow partitions preserved
coverage changes explicit
no old/new project/graph generation mixing
rejected/conflicted proposals retained
```

`wow-recognizers` validates pack/producer ownership, `wow-project` creates exact reindex candidates/publications, `wow-graph` validates partitions/snapshots, and `wow-store` publishes immutable generations. Service coordinates exact receipts only.

## Partial fleet/project closure

If some assigned projects/cohorts cannot reindex or close stale partitions, rollout/rollback remains partial/blocked. The profile determines whether immediate broader deactivation is required. Missing project visibility cannot be treated as successful closure.

## Response loss

Rollback, revocation, deactivation, current CAS, reindex publication, and audit append each require reconciliation. Never issue a compensating second effect merely because the caller did not receive a response.

## Emergency path

An emergency rollback profile may reduce optional evidence but cannot bypass exact target selection, authorization, signature/revocation check, current CAS guard, retention, audit, and stale partition closure. Any skipped nonessential gate is explicit and cannot be called a normal completed rollout.

## Nonclaims

Rollback restores an exact prior publication selection and corresponding new project/graph state. It does not prove the original incident cause, erase runtime effects, guarantee every client recovered, or validate unobserved profiles.