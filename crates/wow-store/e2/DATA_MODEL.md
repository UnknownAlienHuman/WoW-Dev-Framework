# E2-D ProjectStore data model

**Status:** normative semantic and storage-boundary model. Concrete Rust names and SQL table names may differ; identities, ownership, states, and invariants may not.

## Object graph

```text
ProjectStoreConfiguration
├── ProjectStoreRuntimeProfile
├── ProjectStorePhysicalProfile
├── ProjectStoreSchemaSet
├── PublicationPolicy
├── CheckpointPolicy
├── RetentionPolicy
├── BackupRecoveryPolicy
└── BudgetPolicy

ProjectPublicationSet
├── ProjectIndexCandidate identity
├── GraphPublicationPlan
├── ProjectSnapshotManifest
├── GraphSnapshotManifest
├── logical partition versions
├── complete generation membership
├── object reference set
└── validation/golden-query plan

ProjectStoreEpoch
├── ProjectStoreGeneration[]
├── CurrentPublicationRecord
├── Publication/validation/failure records
├── retained generation leases/pins
└── WAL/checkpoint/operational state
```

## ProjectStore identity

```text
ProjectStoreId
    StoreKind=Project
    logical project-store namespace
    owner ProjectId
    store identity schema version
```

Absolute roots, host paths, process IDs, and clocks are not identity.

## Epoch

```text
ProjectStoreEpoch
    epoch_id
    ProjectStoreId
    physical profile ID
    SQLite runtime profile ID
    schema set ID/digest
    canonicalization profile ID
    security/limit profile ID
    database relative path
    creation/validation manifest IDs
    state: Candidate | Active | Retained | Failed | Quarantined
```

Normal project generations remain inside one compatible epoch.

## Schema set

```text
ProjectStoreSchemaSet
    store metadata bundle
    immutable partition metadata bundle
    project domain schema bundle
    graph domain schema bundle
    prepared operation catalogs
    validation catalogs
    compatibility matrix
    canonical digest
```

Store validates this set but does not interpret domain fields.

## Logical partition key

```text
LogicalPartitionKey
    owner namespace
    partition kind
    exact universe/profile/project/reference scope
    producer ID/version when applicable
    stable partition identifier
    logical schema version
```

## Partition version

```text
PartitionVersion
    partition_version_id
    LogicalPartitionKey
    canonical logical payload digest
    domain manifest ID/digest
    row/count/byte summaries
    evidence/coverage/conflict manifest refs
    object reference-set digest
    producer/schema/profile IDs
    state: Building | Sealed | Quarantined | GCEligible
```

`partition_version_id` derives from logical inputs, not SQLite row IDs or insertion order.

## Domain rows

Domain rows are immutable after partition seal and reference one `partition_version_id`. They may contain project source/TOC/XML/load/analyzer/recognizer records and graph assertions/conflicts/coverage/index rows according to owner schemas.

## Generation membership

```text
GenerationPartitionMembership
    target StoreGeneration candidate context
    complete ordered map:
        LogicalPartitionKey -> PartitionVersionId
    removed partition keys relative to base: reporting only
    reused/new partition counts
    canonical digest
```

Membership is complete, not a delta. A read never needs the base generation to discover active partitions.

## Semantic publication set

```text
ProjectPublicationSet
    publication_set_id
    exact ProjectIndexCandidate ID/digest
    ProjectGenerationId
    ProjectSnapshotId/manifest
    AnalyzerSnapshotId and fact/finding manifests
    GraphPublicationPlan ID/digest
    GraphGenerationId
    GraphSnapshotId/manifest
    accepted/rejected proposal/conflict/coverage manifests
    project and graph logical partition manifests
    object reference manifest
    schema/profile/canonicalization IDs
    capability policy
    canonical digest
```

It excludes `ProjectStoreGenerationId` to avoid an identity cycle.

## Store generation

```text
ProjectStoreGeneration
    store_generation_id
    epoch_id
    publication_set_id
    exact base store generation/current record: optional
    project/graph/analyzer/source identities
    complete generation membership ID/digest
    logical data manifest ID/digest
    object reference-set ID/digest
    schema/operation/validation catalog IDs
    state:
        Building
        PublishedInactive
        ValidatedInactive
        Active
        Superseded
        Failed
        Quarantined
        GCEligible
    canonical digest
```

## Current publication record

```text
CurrentPublicationRecord
    record schema/version
    ProjectStoreId / EpochId
    current StoreGenerationId
    ProjectPublicationSetId
    ProjectGenerationId / ProjectSnapshotId
    GraphGenerationId / GraphSnapshotId
    AnalyzerSnapshotId
    profile/reference generation
    previous current StoreGenerationId: optional
    activation validation report ID
    CAS base record digest
    canonical digest
```

This is the single current coherence reference for one epoch.

## Inactive validation report

```text
InactiveGenerationValidationReport
    target epoch/store generation/publication set
    SQLite runtime/effective profile
    schema and migration ledger checks
    partition membership/row/manifest checks
    project validation catalog results
    graph validation catalog results
    object/reference closure
    exact golden reads and graph queries
    cross-generation leakage checks
    budget/cancellation
    status and canonical digest
```

## Writer lease

```text
ProjectStoreWriterLease
    epoch/store identity
    owner instance identity: operational
    acquisition method/result
    base current record
    state
    cancellation/recovery information
```

Owner/process/time fields are noncanonical.

## Read snapshot and lease

```text
ProjectStoreReadSnapshot
    epoch/store/current-record IDs
    exact SQLite read transaction/snapshot identity
    store/project/graph/analyzer generation IDs
    schema/runtime profile IDs
    registered read catalog
    generation lease ID
    capability/validation state
```

```text
GenerationReadLease
    leased epoch/store generation
    holder class
    process-local state
    opened/closed status
```

The lease is operational, not semantic evidence.

## Checkpoint report

```text
ProjectStoreCheckpointReport
    epoch ID
    policy/profile ID
    mode requested/effective
    WAL frames before/checkpointed/remaining
    active reader/writer state
    busy/cancel/failure classification
    logical generation unchanged assertion
```

## Retention pin

```text
GenerationRetentionPin
    store generation
    reason:
        current
        last_known_good
        active_reader
        evidence
        debug
        recovery
        policy
    owner/reference ID
    state
```

## GC plan/report

```text
ProjectStoreGcPlan
    exact current/retention/lease snapshot
    generations eligible
    partition versions reachable/unreachable
    object references reachable/unreachable
    domain registered delete plans
    budgets/cancellation
```

```text
ProjectStoreGcReport
    deleted/skipped/quarantined generations
    deleted/skipped partition versions
    object deletion candidates/results
    validation results
    current generation unchanged assertion
    canonical semantic digest
```

## Backup and recovery

```text
ProjectStoreBackupManifest
    exact epoch/current/generation set
    SQLite backup file digest/length
    schema/runtime profile
    object reference closure
    validation report
    original identities
```

```text
ProjectStoreRecoveryReport
    discovered DB/WAL/SHM/epoch/current/inactive state
    integrity and manifest checks
    recoverable inactive generations
    selected action
    prior/current/target identities
    status
```

## Outer epoch registry

```text
ProjectStoreRegistryRecord
    ProjectStoreId
    active EpochId/database relative path
    previous EpochId: optional
    epoch manifest digest
    validation report
    record digest
```

Only incompatible profile/schema changes use this outer atomic record.

## Canonical ordering

```text
schema bundles by namespace/version/ID
partition keys by namespace/kind/scope/producer/ID
partition versions by key/version ID
generation membership by partition key
domain rows by owner canonical key
validation checks by catalog/check ID
retention pins by generation/reason/owner ID
GC records by generation/partition/object ID
```

No database row/page/insertion/thread order is semantic.
