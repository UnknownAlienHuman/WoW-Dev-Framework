# E2-D exact snapshots and reader leases

**Status:** normative read-consistency and retention contract.

## Read principle

Every read resolves to one exact immutable `PublicationSetId` and remains bound to it. `Current` is only a selector evaluated once.

## Selectors

```text
ExactPublicationSet(PublicationSetId)
ExactStoreGeneration(StoreGenerationId, optional exact StoreImageId policy)
Current(ProjectPublicationKey)
```

Normal project/graph consumers should prefer exact publication set once orchestration has acquired it.

## Current resolution transaction

The registry read transaction:

1. validates registry/configuration/profile state;
2. reads the exact current pointer for the publication key;
3. validates publication/generation/image state references;
4. creates or registers a read lease/root under the resolved IDs;
5. returns exact immutable lease data;
6. commits before generation open.

If lease creation is in-process rather than persistent for a profile, the profile must still prove that current/superseded generation cannot be deleted while the process holds it and must classify crash behavior.

## Lease identity

```text
ProjectStoreReadLease
    lease ID
    ProjectPublicationKey
    exact PublicationSetId
    StoreGenerationId / StoreImageId
    ProjectGenerationId / ProjectSnapshotId
    GraphGenerationId / GraphSnapshotId
    registry epoch observed
    query/runtime profile IDs
    operational owner/heartbeat/expiry state
```

Lease ID can be operational/random, but all canonical query results cite exact publication/generation identities rather than lease randomness.

## Generation open

After resolving lease:

- derive final path from validated generation ID only;
- verify path/member regular-file/security policy;
- verify generation/image/checksum manifests;
- open `project.sqlite` with exact read-only/query-only runtime profile;
- verify schema/application/runtime compatibility;
- verify database metadata matches lease IDs;
- optionally rerun bounded quick checks according to open/profile policy;
- expose only registered query interfaces.

Open failure does not silently retry another generation or current pointer.

## Reader view

```text
ProjectStoreReadView
    exact lease/publication/generation/image identities
    exact selected project and graph query bundles
    registered query execution
    manifest/capability/coverage/conflict reads
    object-handle resolution under policy
    budget/cancellation state
```

No mutable connection or raw database handle crosses the public seam.

## Snapshot stability

While a lease/view is active:

- registry current may advance;
- the view continues reading the exact old immutable generation;
- query results never combine rows from another generation;
- object resolution uses the exact generation reference set;
- source/evidence handles retain original project/store generation identity;
- refresh requires explicit close and new selector resolution.

## Query semantics

Registered query invocation includes:

```text
query bundle ID/version
query ID
exact lease/publication/generation/image
canonical bounded parameters
row/byte/time/step budgets
cancellation
expected result schema/order
```

No arbitrary SQL, callback, expression, table scan request, or query-plan hint.

## Query outcomes

```text
Found
EmptyWithAuthority       only if domain query/coverage contract permits
EmptyWithoutAuthority
Partial
Conflict
NotEvaluated
Truncated
Cancelled
Failed
```

Store empty rows never create domain negative authority by themselves.

## Deterministic pagination and continuation

Continuation binds:

- exact publication/generation/image;
- query bundle/query ID;
- normalized parameter digest;
- ordering version;
- last stable semantic key;
- budget/truncation state;
- integrity digest.

A cursor cannot continue against another current generation. Row ID/physical page/scan position is not a durable continuation key.

## Object access

Objects resolve only when:

- digest appears in exact generation reference set;
- lease and query/capability policy permits role;
- object bytes/digest/length/security/license metadata validate;
- requested range/size is bounded;
- no private path is exposed.

Object handles are read-only. No arbitrary digest enumeration or host path return by default.

## Lease renewal

Renewal is operational and does not change exact IDs. It validates:

- registry still recognizes lease/generation;
- generation is not quarantined/revoked;
- owner/session policy;
- maximum duration/heartbeat limits;
- no integrity incident.

A lease to a superseded but valid generation may renew according to retention policy. It never migrates to current.

## Lease release and expiry

Release removes the reader retention root. Expiry can make a generation GC-eligible only after the GC planner re-evaluates all roots and closure.

Clock/heartbeat fields are not generation identity. Expiry policy must account for process crash and clock anomalies.

## Integrity revocation

If corruption/image mismatch is discovered after lease acquisition:

- mark generation/publication integrity incident in registry;
- stop new leases;
- current pointer is not silently changed;
- existing reader behavior follows explicit fail-fast/quarantine policy;
- return exact error and recovery record;
- do not read another generation without a new explicit selector.

## Multi-process behavior

The physical profile must test:

- concurrent read-only opens;
- writer activating a new set while old readers remain;
- Windows file sharing/delete semantics;
- lease registration before GC eligibility;
- process crash leaving stale operational leases;
- registry busy/lock timeout and deterministic retry classification;
- no deletion/rename of an active reader's generation.

## Reader privacy

Canonical/public outputs exclude:

- absolute generation/object paths;
- SQLite filenames beyond fixed logical member names;
- process/lock owner details;
- credentials/environment values;
- raw SQL/query plans;
- source/object bytes unless explicitly requested and policy permits.

## Required tests

- exact and current selectors;
- pointer advances between selector resolution and first query;
- old reader stable across activation;
- no generation switching during pagination;
- stale/tampered continuation rejected;
- generation/image/manifest mismatch;
- query bundle/version mismatch;
- empty versus authoritative absence;
- budget/truncation/cancellation;
- object ref missing/corrupt/not authorized;
- lease renew/release/expiry/process crash;
- GC race with lease acquisition;
- Windows open/delete behavior;
- integrity revocation;
- no raw DB/SQL/path exposure.

## Hard stops

- no floating read after acquisition;
- no fallback to another current/LKG generation;
- no physical row ID pagination;
- no GC without lease/root recheck;
- no object access outside exact reference set;
- no empty-result authority invented by storage;
- no writable generation connection.
