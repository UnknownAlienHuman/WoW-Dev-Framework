# E5-C activation, rollout, and last-known-good

**Status:** normative.

## Execution profile

Activation is scoped to one exact execution profile:

```text
client/flavor/build compatibility
project/source/reference profile constraints
recognizer interpreter/operator/graph registry versions
producer namespace policy
privacy/license/security/resource policy
current/default record namespace
```

There is no global unqualified current pack.

## Rollout plan

A rollout plan contains a finite ordered set of stages. Each stage freezes:

```text
stage ID and expected prior stage/current record
exact cohort expansion or assignment set
required canary/previous-stage evidence
minimum signal coverage and denominators
success/fail/pause/rollback gates
maximum time/resource/attempt budget
authorization scope
project reindex/graph closure requirements
retention/audit obligations
```

No stage advances because time elapsed, a percentage was reached, or no complaint appeared.

## Rollout advance

```text
register durable operation
-> acquire exact publication/plan/prior stage/current record
-> validate authorization and all required evidence
-> validate no revocation/quarantine/profile drift
-> create exact target assignments
-> coordinate required project reindex and graph closure
-> append rollout stage receipt
-> retain/audit
-> close resources
```

A stage can complete without activating the profile-wide current record if the plan separates cohort assignment from default activation.

## Pause

`core_pack_rollout_pause` records exact scope/reason/evidence/authorization. Pause stops new advancement but does not erase active assignments or observations. Required safety policy may separately trigger rollback/deactivation.

## Current activation

`core_pack_activate` requires:

- exact `ValidatedInactive`/canary/rollout-eligible publication;
- exact execution profile;
- exact expected current record and digest;
- activation authorization;
- current signature/revocation/license/privacy/retention validity;
- required project/graph partition closure plan;
- no unresolved blocker or required `NotEvaluated` state.

Activation uses compare-and-swap. Stale base fails; no silent rebase or merge.

## Current record

The current record binds exact publication/artifact/producer namespace, execution profile, prior current record, activation authorization/validation, rollout stage, retention, and digest. It does not mutate the publication artifact.

## Last-known-good designation

A publication is designated LKG only through `core_pack_last_known_good_designate` with:

```text
exact active/retained publication
qualifying validation/canary/rollout evidence
execution profile and LKG policy
authorization
retention receipt
expected prior designation
```

Designation is append-only/superseding. `previous`, `newest`, `highest version`, `most used`, or `last active` is not LKG proof.

## Multiple profiles

The same publication may be active/LKG for one profile and inactive/failed/not-evaluated for another. Profile records never leak across client flavor, build, registry, interpreter, privacy, or project compatibility boundaries.

## Rollout completion

Rollout completion means every finite required stage and closure gate passed for the exact plan/profile. It does not imply public distribution, future-version support, ecosystem-wide runtime correctness, or permanent LKG status.

## Response loss

Activation, rollout advance/pause, and LKG designation use durable operation identity and reconciliation. If the pointer/designation may have changed but the response is lost, return `OutcomeUnknown`; do not repeat until exact state is reconciled.