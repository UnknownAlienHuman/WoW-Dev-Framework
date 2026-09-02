# E7-B durable release effects, response-loss recovery, retention, audit, and closure

**Status:** normative.

## Effect classes

Externally observable or durable effects include:

```text
source/dependency/toolchain materialization
build submission/execution records
artifact/evidence/bundle publication
portable and platform signing
release-candidate record publication
provider object upload/read-back
channel/update-manifest publication and CAS
bundle download/materialization
installation staging and backup
store/config migration
executable/current installation activation
post-install validation and LKR designation
update rollback
release/key/artifact revocation
support retirement and incident actions
retention/audit/quarantine/cleanup records
```

Every effect has an exact owner and reconciliation operation.

## Registration

Before dispatch, persist:

```text
OperationId
CanonicalRequestDigest
operation/effect kind
exact source/plan/build/artifact/bundle/channel/install/update targets
expected prior states/digests
owner/adapter/profile and authorization references
budgets/cancellation/privacy/license policy
```

Same ID/same digest returns or reconciles the same effect. Same ID/different digest fails.

## State machine

Generic lifecycle:

```text
Planned
Registered
Authorized
Prepared
Dispatched
OwnerOutcomeUnknown
OwnerReceiptRecovered
Committed
ReadBackValidated
RetentionAdmitted
Closing
Completed
Partial
Cancelled
Failed
Quarantined
```

Operation-specific states remain in their owner records. Generic state cannot hide a partial install, failed signature, stale channel CAS or unresolved migration.

## Effect receipt

Persist after each boundary:

```text
owner operation/request/target identity
prepared plan and expected prior state
whether dispatch occurred
commit/no-effect/unknown classification
exact produced object/state/pointer IDs and digests
response/delivery state
read-back/validation state
authorization and quota/resource effects
reconciliation handle
retention/audit/close state
```

## OutcomeUnknown

Timeout, disconnect, cancellation, process crash, serialization failure or caller loss after dispatch does not prove no effect.

While unknown:

- do not redispatch;
- query exact owner/provider/installer/store by operation/request/target identity;
- inspect actual channel/current/install/migration state using exact guards;
- validate recovered receipts and bytes;
- preserve possible credential/quota/distribution/install effects;
- quarantine conflicts or duplicate objects/records;
- expose unsafe-to-retry and exact recovery identifiers.

If an owner lacks reconciliation for an effect, the effect cannot be safely exposed as retryable in a public release profile.

## Build recovery

Classify:

```text
registered but not dispatched
executor started/in progress
executor completed with retained unsigned artifacts
executor outcome unknown
artifact publication incomplete
reproducibility comparison incomplete
```

Recovery may validate retained exact artifacts. It does not start another builder or select the newest output.

## Signing recovery

Query exact target digest/key/profile/operation. Recovered signatures are independently verified. Unknown or duplicate/conflicting signatures remain quarantined; no blind re-sign.

## Distribution recovery

Reconcile provider objects, manifests, release records and channel CAS separately. An uploaded bundle with no channel record remains an orphan immutable object; it is not public current by implication. Recovery can complete read-back and exact pending CAS only when the original prepared operation/authorization remains valid and the owner contract explicitly permits it.

## Installation/update recovery

Inspect exact durable staging/current/backup/migration/helper/self-check records. Classify:

```text
verified staging only
backup complete, activation not started
executable swap prepared/in progress/complete
store/config migration prepared/in progress/complete
current pointer changed or unchanged
post-install self-check pending/failed/passed
rollback pending/in progress/complete
cleanup pending
conflict or unknown
```

Recovery never infers from directory timestamps or version text. It never deletes the old installation until rollback/LKR/retention gates allow.

## Retention graph

Retain while referenced or under policy hold:

```text
source/materialization/build plan and executor receipts
unsigned/signed artifacts and independent builds
SBOM/provenance/license/notices/checksums/signatures
release bundles/candidates/support matrices/test/benchmark reports
provider/channel/update manifests and read-back evidence
current/staged/backup/LKR installation records
store/config/data migration and backup artifacts
update/rollback/revocation/retirement/incident records
authorization/audit/reconciliation records
```

Published channel releases, current installations, LKR/rollback targets, active support windows, unresolved effects/incidents and legal/security holds prevent GC.

## Audit

Append-only hash-linked events record:

```text
source and plan validation
build/materialization/executor effects
reproducibility and tests
artifact/evidence/signing/bundle gates
candidate/channel publication/read-back/CAS
update check/download/verify/install/migrate/activate/self-check
rollback/revocation/retirement/incident actions
all authorization denials, failures, cancellations, response loss and reconciliation
retention/GC/backup/restore/close
```

Trusted ordering/time evidence is included only when the exact profile requires it. Local wall clock alone cannot prove signature/channel/revocation order where stronger evidence is required.

## Resource acquisition and close

Acquire only operation-required resources in a package-defined order, typically:

```text
durable operation registry
-> source/materialization/build catalogs
-> authorization and executor/signing/publisher/installer adapters
-> artifact/store/channel/install views
-> retention
-> audit
```

Close in reverse order. No public success before mandatory output flush/read-back, retention, audit and close results.

A close failure after an effect preserves the committed effect and changes the service outcome to failed/partial/`OutcomeUnknown`; it is not success with a warning.

## Cleanup

Staging/orphan/cache cleanup is an explicit owner operation with exact reachability and retention checks. It never runs detached after final success. Failed cleanup does not delete current/backup/LKR/evidence artifacts and remains an explicit state.

## Backup and restore

Release/install store backup/restore uses exact `wow-store` and installation-owner profiles. Restore validates identities, signatures, channel/current/LKR/install/migration/audit/retention closure. Restored records cannot refresh manifests, un-revoke releases or create new semantic IDs.

## Determinism

Canonical release records exclude host/process IDs, local paths, wall durations, provider object order, retry count, cache state, CI job ID, upload timing and delivery state except where an explicit operational audit/evidence record owns them. Exact source/artifact/build/profile identities remain stable.