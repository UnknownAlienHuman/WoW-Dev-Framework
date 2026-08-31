# ProjectStore read snapshots and generation leases

**Status:** normative v1 reader model.

## Supported topology

V1 supports readers inside the owning framework process. External processes opening `project.sqlite` are unsupported because they bypass generation leases, retention, runtime profiles, and domain read-view contracts.

## Current snapshot acquisition

```text
open selected epoch database
BEGIN read transaction
read and validate CurrentPublicationRecord
validate exact epoch/schema/runtime profile
validate referenced StoreGeneration and ProjectPublicationSet
register process-local generation lease
construct project/graph domain views through registered reads
```

The transaction remains open for the bounded view lifetime.

## Exact generation acquisition

Debug, comparison, and recovery callers may request an exact retained store generation. They supply exact epoch/generation/publication-set IDs and pass retention/validation checks. No latest/fuzzy fallback.

## Snapshot invariants

- activation after acquisition cannot change the reader's record or rows;
- every domain read resolves through exact generation membership and partition versions;
- no query omits generation/partition scope;
- graph and project views use the same publication set;
- object resolution validates exact references;
- stale or collected generation open fails;
- a view cannot upgrade partial/conflicted coverage.

## Lease registry

```text
GenerationReadLease
    process-local opaque lease ID
    exact epoch/store generation/publication set
    holder class
    bounded purpose
    opened/closed state
```

Lease timing is operational and excluded from semantic output.

## Lease lifecycle

- acquire before exposing the view;
- release on close, error, cancellation, or owner shutdown;
- detect leaked leases through owner diagnostics;
- process crash drops process-local leases and SQLite read transactions;
- GC runs through the writer after checking the process lease registry.

## Long readers

Profiles impose maximum concurrent readers, reader lifetime classes, WAL pressure warnings, result budgets, and cancellation points. The system reports reader pressure; it does not silently switch or terminate a valid reader outside its API contract.

## Query catalogs

Store returns typed rows only to domain adapters. No raw connection, cursor, statement, SQL, table name, or row ID escapes.

## Current changes

A writer may activate a new current record while old readers remain. New readers see the new publication; old readers retain the old SQLite snapshot and generation lease.

## Reader failure

Missing membership, partition, row, object, or generation mismatch fails the affected view/query and triggers validation. No fallback to another generation and no mixed result.

## Required tests

- old reader stays old across activation;
- new reader sees new;
- project and graph IDs remain coherent;
- exact retained generation opens;
- collected/unknown generation fails;
- reader lease blocks GC;
- cancellation releases lease;
- long reader delays checkpoint/GC without corrupting state;
- no query without membership scoping;
- no raw store handle leaks.
