# E6-B external-candidate storage seam

**Status:** normative cross-crate seam; implementation has not started.

`wow-store` provides generic immutable object/catalog/effect/retention mechanisms for E6-B. It does not interpret provider, candidate, mapping, selection, or context semantics.

## Logical objects supplied by owners/service

```text
provider configuration receipt references
query operation and dispatch receipts
bounded raw provider response object
E6-A result set and candidate artifact
result validation record
mapping request/result record
selection request/receipt/supersession record
context sidecar and combined-result manifest
reconciliation/quarantine/audit references
```

Domain/service code supplies registered schemas, canonical bytes, logical IDs/digests, prepared operations, validation callbacks through narrow typed contracts, and retention edges.

## Store operations

Using existing generic store primitives:

```text
publish immutable object
publish append-only catalog/state record
read exact object/catalog record
read snapshot-bound list page
compare-and-swap only where an explicit owner state machine requires it
admit/release retention
record/reconcile durable operation effect
validate backup/restore and GC reachability
```

There is no global current/default external result or candidate selection pointer in E6-B.

## Hard boundaries

- `wow-store` does not call a provider or own sessions/credentials.
- It does not parse provider responses or validate Candidate authority.
- It does not map locators, choose candidates, build context, or interpret privacy/license meaning.
- It exposes no raw SQL, connection, table, row ID, filesystem path, object-store key, transaction callback, or arbitrary serializer to service/applications.
- It depends directly only on `wow-core`.

## Publication/read-back

E6-B service prepares exact objects, publishes through store, closes writer resources, reacquires a fresh read snapshot, and invokes domain/service validation. Store commit alone does not mean semantically valid.

## Retention and GC

Reachability edges must preserve unresolved operations, result/artifact continuation lineage, mapping owner evidence, selections, context results, incidents, and legal/privacy/license holds. GC cannot remove referenced evidence.

## Response loss

Store operations reconcile by exact operation/request/object/catalog identity. Same operation ID with different request digest fails. Possible commit with lost response remains unknown until exact reconciliation; service cannot duplicate the effect.