# E5-C immutable publication catalog and read-back

**Status:** normative.

## Catalog ownership

`wow-store` owns physical object/catalog transactions and retention. `wow-service` owns the publication use case and validation plan. `wow-recognizers`/`wow-graph` own semantic and producer-partition validation.

No raw SQL, connection, table, row ID, filesystem path, object-store key, or transaction callback crosses the public service seam.

## Publish request

```text
exact CorePackArtifact/attestation/signature IDs and digests
publication catalog/store/schema/profile
expected absence or prior publication guard
OperationId + CanonicalRequestDigest
authorization receipt
retention/audit/budget/cancellation profiles
```

There is no current/default mutation in this request.

## Publication protocol

```text
register durable operation
-> acquire exact artifact/attestation/signature and authorization
-> validate current revocation/license/privacy state
-> materialize immutable objects
-> commit one catalog record as PublishedInactive
-> persist effect receipt
-> admit retention
-> close writer resources
-> open fresh read snapshot by exact publication ID
-> verify object bytes/digests/signatures/attestations/catalog/schema
-> invoke recognizer and graph validation catalogs
-> verify producer namespace and deactivation/closure plan
-> persist read-back validation
-> mark ValidatedInactive only through a distinct guarded effect
-> close resources
```

A physical commit without successful read-back remains `PublishedInactive`, failed, or quarantined—not validated.

## Immutable states

State progression is represented by new append-only transition records around one immutable publication identity. Artifact/attestation/signature bytes never change.

Allowed lifecycle states include:

```text
Building
PublishedInactive
ValidatedInactive
CanaryAssigned
CanaryActive
RolloutPaused
Active
Superseded
Revoked
Quarantined
GCEligible
```

State transitions require exact expected prior state and authorization/profile guards. Unsupported transitions fail.

## Idempotency

Same operation ID and request digest returns/reconciles the same publication. Same ID/different digest fails. If response is lost after object/catalog commit, query by operation/request/artifact digest and recover the exact record; do not duplicate publication.

## NoChange

Valid only when the exact eligible publication already exists with identical artifact/attestation/signature/catalog/profile request and its required state/read-back closure. Same pack name or digest alone is insufficient.

## Catalog queries

Queries are exact and deterministic:

```text
by publication ID
by exact artifact/attestation/signature tuple
by exact execution compatibility profile
by exact state under retained catalog snapshot
```

Catalog list results never select a publication for activation. Pagination binds exact catalog snapshot, filters, order, budgets, and last stable key.

## Read-back failures

Digest mismatch, missing object, invalid signature, incomplete SBOM/license, catalog inconsistency, graph/recognizer incompatibility, stale deactivation plan, or unavailable required validation causes failure/quarantine. No repair in place; build/publish a new identity if inputs change.

## Retention

Before advertising a publication handle, retain artifact, attestation, signature, submission/evidence lineage, catalog record, read-back report, deactivation/closure plans, and required audit. Active/canary/LKG/rollback/evidence references prevent GC.

## Internal-only boundary

E5-C catalog records are internal framework publication artifacts. They are not public download URLs, release packages, updater manifests, or distribution-channel entries. Those remain E7.