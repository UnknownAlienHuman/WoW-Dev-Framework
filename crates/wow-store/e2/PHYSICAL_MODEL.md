# E2-D ProjectStore physical model

**Status:** normative first physical profile.

## Why file-per-generation

The E2-D workload publishes relatively infrequent coherent index generations and serves many immutable readers. The selected profile optimizes correctness and rollback clarity over in-place update density.

Benefits:

- reader snapshot is a concrete immutable artifact;
- writer never mutates a reader's database;
- rollback is head selection, not row repair;
- crash inventory is classifiable from directories/manifests;
- retained generations support evidence and regression comparison;
- logical/physical determinism can be reported separately.

Costs are explicit:

- generation file duplication;
- object/reference manifests and GC required;
- atomic rename/replace behavior must be tested per supported filesystem;
- large projects may later justify another measured profile.

## Owned root

The host adapter supplies one prevalidated owned store root. The library receives an opaque root capability, not an arbitrary path string.

```text
registry/
generations/
objects/
staging/
quarantine/
locks-or-actor-state/
```

Reject symlinks, junctions, reparse points, device paths, traversal, case collisions, and ownership ambiguity according to the profile.

## Staging database

- created exclusively under owned `staging/`;
- schema created/migrated through registered bundles;
- one writer;
- WAL/journal/synchronous settings frozen by profile;
- statement and transaction budgets enforced;
- no reader treats staging as a published generation;
- cancellation/failure leaves it cleanup-eligible or quarantined with exact report.

## Sealing

Sealing requires:

1. all registered writes completed;
2. domain and store validation catalogs pass;
3. transaction committed;
4. WAL checkpoint policy completed;
5. all SQLite handles closed;
6. file/object members flushed according to durability profile;
7. manifest/checksum members built;
8. staging members atomically materialized to a unique final generation location;
9. final members verified unchanged;
10. exact final database reopened read-only and validated.

No final generation depends on a writable sidecar journal.

## Read-only open

Open validates:

- owned canonical location;
- generation/artifact manifest;
- all mandatory member sizes and SHA-256;
- SQLite header/application/schema/user versions;
- profile and bundle IDs;
- required integrity/foreign-key/domain catalog results;
- requested generation identity.

Only then may a reviewed immutable/read-only optimization be enabled.

## Object store

Objects are keyed by cryptographic digest and typed metadata. Write flow:

```text
validate bytes/type/license/privacy/budget
-> write unique staging object
-> fsync/profile durability
-> atomically materialize if absent
-> verify existing object if already present
-> include exact object ref in generation manifest
```

No digest collision fallback by filename or last-write. Existing mismatched bytes under a digest are corruption.

## Head registry

The registry is physically independent from immutable generation databases. It supports a single atomic CAS per project head key.

The selected implementation may use a small SQLite registry or another reviewed local atomic-record mechanism. Its exact choice freezes in `ProjectStorePhysicalProfile`; it cannot be inferred from platform defaults.

The head points only to an already sealed/open-validated generation.

## Filesystem assumptions

The profile includes executable probes for:

- same-filesystem atomic materialization;
- replace/no-replace semantics;
- directory durability expectations;
- file locking and delete-while-open behavior;
- case sensitivity/collision behavior;
- read-only permissions;
- maximum path/member sizes;
- crash leftovers.

Unsupported filesystems yield typed unavailable; correctness is not approximated.

## Physical determinism

Report classes:

```text
logical_equal
sealed_member_equal
sqlite_bytes_equal
sqlite_logically_equal_physical_different
object_bytes_equal
registry_semantically_equal
not_comparable_profile_changed
```

E2-D requires deterministic logical manifests and query results. Exact SQLite byte equality is a measured property, not assumed.
