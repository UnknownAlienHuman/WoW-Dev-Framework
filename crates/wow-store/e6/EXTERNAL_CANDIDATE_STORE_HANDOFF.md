# `wow-store` E6-B external candidate durability handoff

**Status:** normative supporting seam; implementation not started.

## Purpose

Provide generic registered storage operations for E6-B durable operation state, immutable external candidate artifacts, mappings, selections, audit records, retention, catalog snapshots, and reconciliation without importing provider or mapping semantics into `wow-store`.

## Ownership

`wow-service`/`wow-cbm` owners define logical schemas, canonical identities, statuses, validation catalogs, and exact write/read plans.

`wow-store` owns SQLite/object storage, transactions, one-writer behavior, durability, read snapshots, retention, recovery, and GC.

## Logical record families

```text
external_operation_record
provider_dispatch_and_effect_receipt
external_candidate_result_manifest
external_candidate_artifact_manifest
external_mapping_receipt
external_selection_receipt
external_context_handoff_receipt
external_cache_record
external_audit_record
external_retention_reference_set
external_catalog_snapshot
```

Store does not interpret `Candidate`, provider score/rank, mapping status, selection origin, or context authority.

## Registered operations

```text
register_external_operation
compare_and_transition_external_operation
record_provider_dispatch_receipt
record_or_reconcile_provider_effect
insert_external_candidate_artifact_if_absent
insert_mapping_receipt_if_absent
insert_selection_receipt_if_absent
insert_context_handoff_receipt_if_absent
append_external_audit_event
open_external_catalog_snapshot
admit_external_retention_reference_set
release_external_retention_reference_set
garbage_collect_unreferenced_external_artifacts
```

No application-provided SQL.

## Idempotency

Exact `OperationId + CanonicalRequestDigest` is unique. Same ID/different digest fails. Immutable artifacts are content-addressed or exact-ID guarded. Conflicting duplicates are quarantined rather than overwritten.

## Response-loss reconciliation

Store can query exact operation/effect/artifact state after response loss. It distinguishes:

```text
no record/no effect proven under transaction profile
operation registered but not dispatched
dispatch recorded/effect unknown
effect receipt committed
artifact committed
conflicting effects/artifacts
```

Missing row is not always no effect; interpretation follows registered operation state and transaction receipts.

## Retention and GC

Retention reference sets close the race between catalog lookup and GC. A returned durable handle references exact artifacts and owner generations. GC deletes only unreferenced local E6 artifacts after validation; it never deletes provider databases/indexes or owner project/reference/context records.

## Catalog snapshots

List operations use immutable store read snapshots and deterministic ordering. Continuation binds the same snapshot. Store does not sort by newest/best/provider score unless a reviewed owner catalog profile explicitly defines a nonauthority display order.

## Security

- no raw SQL/connection/table/PRAGMA exposed;
- no ATTACH of arbitrary/provider databases;
- no provider endpoint/session/credential storage;
- no arbitrary filesystem/network/process/editor/client access;
- private locator/snippet/source data stored only under exact privacy/encryption/root policy;
- errors/logs expose stable IDs, not private payloads;
- all strings/objects/rows/bytes bounded.

## Identity exclusions

SQLite row/page/order, WAL/checkpoint state, host path, process, clock, retry count, and connection handle do not enter semantic result/candidate/mapping/selection identity.

## Recovery

On startup/recovery, validate operation state, effect receipts, artifact/catalog/retention/audit closure, and quarantine conflicts. Never relabel `OutcomeUnknown` as failed/no-effect without proof.

## Tests

- registration before effect;
- same/different request digest;
- response loss before/after transaction commit;
- duplicate conflicting artifacts;
- immutable mapping/selection receipts;
- retention/GC race;
- catalog snapshot continuation;
- crash/recovery around every write boundary;
- no provider DB attach/delete;
- deterministic logical results under physical layout/WAL changes.
