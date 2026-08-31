# E2-D ProjectStore data model

**Status:** normative logical/physical store contract.

## Profile

```text
ProjectStorePhysicalProfile
    profile_id/version
    SQLite/runtime/library pin
    compile-option allow/deny manifest
    generation directory/path grammar
    staging journal/synchronous/checkpoint rules
    sealed read-only/open rules
    atomic materialization and registry CAS assumptions
    page/cache/mmap/busy/statement/resource limits
    checksum/hash/canonicalization profiles
    supported filesystem capability report
    canonical digest
```

## Registered bundle set

```text
RegisteredBundleSet
    bundle_set_id
    schema bundles[]
    migration bundles[]
    write operation catalogs[]
    read operation catalogs[]
    validation catalogs[]
    compatibility matrix
    canonical digest
```

Every bundle is repository-owned, versioned, nonexecuting data plus implementation-owned registered handlers. A caller cannot add code through a bundle payload.

## Build request

```text
ProjectStoreGenerationRequest
    request_id
    StoreId
    exact expected current head/store generation
    target logical generation identity
    physical profile ID
    registered bundle set ID
    ordered operation phases
    object write/reference plan
    expected logical manifests/counts/digests
    required validation catalog invocations
    durability/budget/cancellation policy
```

## Staging generation

```text
StagingGeneration
    staging_id
    request/target IDs
    owned canonical staging location handle
    transaction state
    applied operation phase/invocation IDs
    object staging records
    validation state
    cancellation/fault state
```

Staging location is internal and noncanonical.

## Registered invocation

```text
RegisteredOperationInvocation
    invocation_id
    catalog/operation ID and version
    phase and semantic ordinal
    bounded typed payload bytes + schema ID
    expected affected record/partition IDs
    expected row/object/count/digest deltas
    prerequisite invocation IDs
    canonical digest
```

## Generation manifest

```text
ProjectStoreGenerationManifest
    StoreId
    ProjectStoreGenerationId
    physical profile and registered bundle IDs
    exact project/graph publication bundle ID
    logical domain manifest IDs
    ordered operation manifest
    object-reference manifest
    schema/migration state
    logical counts/digests
    validation report IDs
    sealed artifact ID/checksums
    eligibility/state
    canonical digest
```

## Artifact

```text
ProjectStoreArtifact
    ProjectStoreArtifactId
    generation manifest ID
    relative owned artifact members
    byte lengths/SHA-256
    SQLite header/profile/open facts
    physical reproducibility classification
    seal report
```

## Open view

```text
ProjectStoreReadHandle
    handle_id
    exact generation/artifact/profile
    GenerationLeaseId
    read-only capability set
    open validation report
    bounded registered read executor
```

## Head registry record

Store treats the head payload as opaque typed bytes with validated schema and CAS metadata:

```text
PublicationHeadRecord
    head_key
    head_schema_id
    head_id
    payload digest
    exact referenced store generation/artifact
    expected previous head ID
    registry sequence for concurrency control only
```

The registry sequence is not project semantic identity.

## Lease

```text
GenerationLease
    lease_id
    generation/artifact
    lease owner class and opaque owner ID
    acquired registry state
    heartbeat/renewal state under profile
    explicit release state
    retention-root record
```

## Recovery inventory

```text
RecoveryInventoryEntry
    owned relative location
    observed class
    manifest/checksum/open state
    registry/head/lease/reference links
    expected action class
    errors and quarantine reason
```

Classes:

```text
staging-active
staging-abandoned
sealed-inactive
current
last-known-good
pinned
leased
orphan-object
corrupt
quarantined
unknown-owned-entry
```

## Retention and GC

```text
RetentionRoot
    root_id
    kind
    generation/object target
    source registry/manifest/evidence
    policy/version
```

```text
GcPlan
    exact inventory and head/lease snapshot IDs
    retained generations/objects with reason paths
    candidate deletions with nonreachability proof
    byte/count estimates
    validation catalog
    canonical digest
```

```text
GcExecutionReport
    plan ID
    preconditions revalidated
    deleted/quarantined/skipped members
    post-sweep integrity result
    cancellation/failure state
```

## State machines

### Generation

```text
Requested
-> Staging
-> TransactionActive
-> TransactionCommitted
-> Checkpointed
-> Closed
-> Sealed
-> OpenValidated
-> InactiveValidated
-> Headed
```

Terminal/scoped:

```text
Aborted
Cancelled
Quarantined
Corrupt
GcEligible
Collected
```

### Head

```text
Absent
-> Present(head N)
-> CAS(head N -> head N+1)
```

No partial multi-record head update.

## Canonical ordering

- bundles by kind/ID/version;
- operation phases by declared phase then semantic ordinal and invocation ID;
- object refs by digest/type;
- logical manifests by domain/kind/ID;
- validation records by catalog/check/subject/ID;
- retention roots by kind/target/root ID;
- GC candidates by generation/object key.

Filesystem enumeration, SQLite row order, transaction timing, and registry sequence do not determine canonical logical IDs.
