# ProjectStore read snapshots, selectors, continuations, and generation leases

**Status:** normative v1 reader model for the selected WAL/partition-version profile.

## Supported topology

V1 supports readers inside the owning framework process. External processes opening the epoch database are unsupported because they bypass generation leases, retention roots, runtime profiles, registered query catalogs, and domain coverage semantics.

## Selectors

```text
Current(ProjectStoreId)
ExactPublicationSet(ProjectPublicationSetId)
ExactStoreGeneration(EpochId, ProjectStoreGenerationId)
```

`Current` is evaluated once. After acquisition every operation uses exact IDs. No `latest`, fuzzy fallback, or automatic last-known-good selector exists.

## Current snapshot acquisition

Acquisition uses the process lifecycle/lease-admission guard shared with GC:

```text
acquire read-lease admission guard
open selected epoch database under exact runtime profile
BEGIN read transaction
read and validate CurrentPublicationRecord
validate epoch/schema/runtime/publication/store/project/graph/analyzer IDs
validate exact generation membership and publication-set closure
register process-local generation lease while the transaction remains open
release admission guard
construct project/graph views through registered reads
```

If any step fails, close the transaction and remove the provisional lease. GC cannot snapshot roots between current resolution and lease registration because it must acquire the same admission guard through the writer owner.

## Exact retained-generation acquisition

Debug, comparison, rollback validation, and recovery callers may request an exact retained publication/store generation. They supply exact epoch, store generation, publication set, project snapshot, and graph snapshot IDs and pass retention/integrity checks. No fallback occurs when the target is missing, collected, quarantined, or incompatible.

## Snapshot invariants

- activation after acquisition cannot change the reader's current record or visible rows;
- every domain read resolves through exact complete generation membership and immutable partition versions;
- every query includes exact generation and partition scope;
- project and graph views use the same publication set and SQLite snapshot;
- object resolution validates exact generation references;
- stale, collected, quarantined, or integrity-failed generation open is explicit failure;
- partial/conflicted/truncated coverage is preserved;
- checkpoint and new writes cannot change logical results in an active read transaction.

## Generation lease

```text
GenerationReadLease
    process-local opaque lease ID
    exact EpochId / StoreGenerationId / PublicationSetId
    exact ProjectGenerationId / ProjectSnapshotId
    exact GraphGenerationId / GraphSnapshotId
    exact AnalyzerSnapshotId and profile/reference identities
    holder class and bounded purpose
    opened/closed state
```

Lease ID, timing, thread, and process details are operational and excluded from canonical query output.

## Lease lifecycle

- acquire before exposing the view;
- release on close, error, cancellation, panic containment, or owner shutdown;
- leaked leases are visible through owner diagnostics;
- process crash ends SQLite transactions and process-local leases;
- GC re-evaluates current, operation, retention, and lease roots under the admission guard;
- a lease never migrates to a newer current publication.

## Query catalogs

Store executes only registered read operations and returns typed rows to domain adapters. No raw connection, cursor, statement, SQL, table/index name, query plan, PRAGMA, or physical row ID escapes.

Registered query invocation binds:

```text
query catalog ID/version
query ID
exact publication/store generation and membership
canonical bounded parameters
expected result schema and ordering version
row/byte/time/step budgets
cancellation
```

## Query outcomes

```text
Found
EmptyWithAuthority       only when the domain coverage contract permits
EmptyWithoutAuthority
Partial
Conflict
NotEvaluated
Truncated
Cancelled
Failed
```

Empty storage rows never create domain negative authority by themselves.

## Deterministic pagination and continuation

A continuation cursor binds:

```text
exact epoch/publication/store/project/graph generations
query catalog/query ID
normalized parameter digest
semantic ordering version
last stable semantic key
prior truncation/budget state
cursor integrity digest
```

A cursor cannot continue against another current generation, query version, parameter set, or ordering profile. Physical row ID, page number, WAL frame, scan offset, and statement state are forbidden continuation keys.

Continuation resumes through a new exact read snapshot only if the original generation is still retained and validates. The result remains explicitly part of the original snapshot/query sequence.

## Object access

Objects resolve only when:

- the exact retained generation reference set includes the object;
- lease/query/capability policy permits the role;
- logical and encoded digests, lengths, codec, security, and license metadata validate;
- requested bytes/range are bounded;
- no private host path is exposed.

No default object enumeration or arbitrary digest lookup.

## Long readers and WAL pressure

Profiles bound concurrent readers, reader-lifetime classes, transaction duration, result size, and cancellation. Long readers may pin WAL frames and delay checkpoint/partition GC. The system reports pressure and may reject new long-reader admissions according to policy; it does not switch a valid reader to another generation or revoke it merely to make checkpoint/GC succeed.

## Integrity revocation

If an integrity incident is discovered:

- block new normal leases for the affected publication/generation;
- record exact recovery/quarantine state;
- existing reader behavior follows the explicit fail-fast/quarantine policy;
- never substitute another publication silently;
- current changes only through explicit validated activation/rollback.

## Windows behavior

Normal generation reads use one epoch database, so generation replacement does not rename/delete a database under readers. Windows sharing tests still cover:

- epoch replacement/GC;
- backup/restore files;
- content-addressed object deletion;
- process termination and handle cleanup.

A sharing violation is retryable operational state after root/lease recheck, not proof of corruption. No rename/delete loop and no valid-reader revocation.

## Required tests

- current and exact selectors;
- pointer advances during acquisition;
- lease/GC admission race;
- old reader stays old across activation;
- new reader sees new;
- project/graph/analyzer IDs remain coherent;
- exact retained generation opens;
- collected/quarantined/unknown generation fails;
- cancellation/error/panic releases provisional and active leases;
- long reader pins WAL frames without state corruption;
- semantic continuation succeeds on retained generation;
- stale/tampered/cross-generation continuation fails;
- no query without membership scoping;
- empty versus authoritative absence;
- bounded object access and missing/corrupt object;
- integrity revocation;
- Windows epoch/object sharing violations;
- no raw store handle/path leakage.

## Hard stops

- no floating read after acquisition;
- no fallback to current or last-known-good;
- no generation switch during pagination;
- no physical row/page/WAL continuation;
- no GC without lease-admission/root recheck;
- no object access outside exact reference closure;
- no storage-created negative authority;
- no writable handle or arbitrary query surface exposed.
