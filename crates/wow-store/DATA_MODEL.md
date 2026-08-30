# `wow-store` E1-A data model

**Status:** normative semantic model for the storage foundation, immutable ReferenceStore, schema migration framework, and content-addressed objects.

Concrete Rust/SQLite layout may differ. Ownership, identity, publication, integrity, and security semantics may not.

## 1. Object graph

```text
StoreConfiguration
├── StoreRoot
├── SqliteRuntimeProfile
├── SchemaRegistry
├── OperationCatalogRegistry
├── PublicationPolicy
├── ObjectStoreConfiguration
├── RetentionPolicy
└── BudgetPolicy

SchemaRegistry
└── SchemaBundle[]
    └── MigrationEdge[]

ReferenceStoreBuildRequest
-> StagingStore
-> StoreValidationReport
-> StoreManifest
-> SealedStoreGeneration
-> GenerationPublicationRecord
-> ActivePointerRecord

ObjectStore
├── ObjectManifest[]
├── EncodedPayloadRecord[]
├── ObjectReferenceRecord[]
├── ObjectLeaseRecord[]
└── GarbageCollectionReport[]
```

## 2. Store kinds

```text
StoreKind
    Reference
    Project
    External
```

E1-A active: `Reference`.

`Project` and `External` are contract-declared but implementation-deferred.

## 3. Store identity

```text
StoreId
    store kind
    logical store namespace
    schema registry identity
    configured root identity
```

```text
StoreGenerationId
    store ID
    exact domain generation/profile identity supplied by consumer
    schema bundle set/digests
    canonical logical data manifest digest
    object reference-set digest
    store-generation schema version
```

Store does not interpret domain generation fields; it validates exact stable IDs and embeds them in manifests.

## 4. Store configuration

```text
StoreConfiguration
    configuration_id
    store roots by kind
    SQLite runtime profile requirement
    schema registry identity
    operation catalog registry identity
    publication policy
    object store configuration
    retention policy
    budgets/cancellation
    path/durability platform adapter identity
    configuration schema version
```

Canonical identity excludes machine-specific absolute root. A root capability/identity can be represented through configured root label plus platform adapter; public manifests use relative generation/object paths.

## 5. Store root

```text
StoreRoot
    root_id
    store kind/namespace
    configured absolute path (private runtime field)
    publication volume identity
    relative staging/published/active/quarantine/object directories
    path policy
    platform adapter identity
```

Absolute path never enters public/canonical store manifest or error by default.

## 6. SQLite runtime profile

```text
SqliteRuntimeProfile
    profile_id
    SQLite library version
    Rust binding/version/features
    compile options/capabilities digest
    open flags by store kind/state
    PRAGMA profile
    page/encoding/application/user version policy
    foreign-key behavior
    integrity-check policy
    URI/immutable/query-only support
    extension/attach/trusted-schema/defensive policy
    journal/synchronous/busy/checkpoint policy
    probe report identity
```

A profile is accepted only after executable capability/behavior probes.

## 7. Schema registry

```text
SchemaRegistry
    registry_id
    standard metadata schema bundle
    registered domain schema bundles[]
    migration graph
    operation catalogs[]
    registry schema version
    canonical digest
```

E1-A expected bundles:

```text
wow-store-metadata/e1-a/1
one wow-reference-provided E1 schema bundle (exact ID frozen later)
```

Store validates but does not interpret domain tables.

## 8. Schema bundle

```text
SchemaBundle
    bundle_id
    namespace
    version
    parent versions[]
    schema declarations[]
    index/trigger/virtual-table declarations[]
    required SQLite capabilities[]
    migration edges[]
    prepared operation catalog ID
    validation check catalog[]
    destructive-change policy
    expected canonical schema digest
    owner crate/contract/version
```

DDL/migration operations are static repository-owned artifacts. No source/user SQL.

## 9. Migration edge

```text
MigrationEdge
    migration_id
    namespace
    from version
    to version
    operation catalog/digest
    required SQLite capabilities
    preconditions
    transactional policy
    destructive/data-transform policy
    validation checks
    expected target schema digest
    owner/version
```

Graph invariants:

- acyclic;
- no duplicate from/to edge ID;
- no missing parent;
- no implicit skip;
- target path deterministic/explicit;
- migration ledger matches applied operations/digests;
- unknown/tampered state fails.

## 10. Standard metadata records

Conceptual domain-neutral records:

```text
StoreMetaRecord
    store kind/ID/generation/profile/reference/domain identity
    schema registry/bundle IDs/digests
    SQLite runtime profile ID
    creation/build tool identities
    candidate/sealed/published state

SchemaMigrationRecord
    migration ID/from/to/digest/order
    application transaction/report IDs
    result/validation IDs

StoreGenerationRecord
    generation ID
    logical manifest digest
    object reference-set digest
    file/store digest
    state

PublicationRecord
    candidate/staging/sealed/published/active IDs
    previous active generation
    platform adapter/durability level
    pointer manifest/digest
    validation report

IntegrityRecord
    check kind/version/result
    schema/foreign-key/database/application/file/object validation refs

RetentionRecord
    retained/active/last-known-good/failed/quarantined generations
    object lease/reference snapshot
    GC eligibility decision
```

Physical table names are frozen in implementation; semantic fields remain.

## 11. Prepared operation catalog

```text
PreparedOperationCatalog
    catalog_id
    schema bundle ID/version
    operation descriptors[]
    canonical digest
```

```text
PreparedOperationDescriptor
    operation_id
    kind: read | insert | update | delete | validation
    static repository-owned SQL digest
    parameter schema
    result schema
    allowed store kind/state
    transaction requirement
    budget/cardinality policy
```

Domain adapters call operation IDs with typed encoded parameters/results. No arbitrary SQL string at runtime/application boundary.

## 12. Store transaction

```text
StoreReadTransaction
    transaction_id
    store generation ID
    schema/runtime profile ID
    consistency snapshot identity
    budget/cancellation
```

```text
StoreWriteTransaction
    transaction_id
    candidate generation/build ID
    source/target schema versions
    operation catalog set
    state
    budget/cancellation
```

E1 ReferenceStore build write transactions exist only in staging. Published reference stores expose read transactions only.

## 13. Reference store build request

```text
ReferenceStoreBuildRequest
    request_id
    StoreConfiguration ID
    exact domain profile/reference generation IDs
    target schema registry/bundle set
    expected logical input manifest/digests
    domain write plan/operation invocations
    object write/reference plan
    budgets/cancellation
    requested durability level
```

The domain write plan is produced by `wow-reference`. Store treats operations/parameters as registered typed inputs and validates all catalogs.

## 14. Staging store

```text
StagingStore
    staging_id
    candidate generation ID
    private root-relative path
    SQLite file state
    active schema/migration ledger
    object temp/published references
    build transaction reports
    validation state
```

Staging ID/path never becomes published identity. Cancellation/failure leaves no active pointer change.

## 15. Validation report

```text
StoreValidationReport
    report_id
    candidate generation ID
    schema registry/digest result
    migration ledger result
    SQLite runtime/open-profile result
    foreign-key result
    quick/integrity result
    registered application validation results[]
    manifest/file digest result
    object logical/payload/reference result
    unexpected schema/sidecar result
    budget/cancellation/truncation
    overall status
    canonical digest
```

All mandatory checks must pass before sealing.

## 16. Store manifest

```text
StoreManifest
    manifest schema version
    StoreId / StoreGenerationId
    store kind/state
    domain profile/reference/generation IDs
    schema registry/bundle/migration IDs/digests
    SQLite runtime profile ID and required capabilities
    logical data manifest digest
    SQLite file digest and byte length
    object reference-set digest/count
    integrity validation report ID
    publication/retention metadata
    tool/producer versions
    canonical digest
```

Public manifest uses normalized relative paths/IDs only.

## 17. Sealed store generation

```text
SealedStoreGeneration
    generation ID
    final immutable relative path
    StoreManifest
    SQLite file
    referenced objects/manifests
    seal report/digest
```

Seal asserts no writable transaction/sidecar/pending object/unknown schema remains.

## 18. Active pointer

```text
ActivePointerRecord
    pointer schema version
    StoreId
    active StoreGenerationId
    active generation relative path/manifest digest
    previous active generation ID
    pointer digest
```

Pointer is small and atomically replaced after generation publication. Timestamp may be supplemental/noncanonical.

## 19. Object identity

```text
ObjectId
    sha256(canonical uncompressed logical bytes)
```

```text
ObjectManifest
    ObjectId
    logical media/type tag
    logical length
    logical digest algorithm
    encoded payload records[]
    canonical object metadata digest
```

Logical metadata affecting interpretation is explicit and versioned; source filename/path is not identity.

## 20. Encoded payload

```text
EncodedPayloadRecord
    payload_id
    ObjectId
    codec ID/version/parameters
    encoded payload sha256
    encoded length
    relative object path
    write/verify state
```

`payload_id` derives from logical ObjectId + encoding contract + payload digest.

## 21. Object reference

```text
ObjectReferenceRecord
    StoreGenerationId
    ObjectId
    reference kind/owner record key supplied opaquely by domain adapter
    created in publication candidate
    retained state
```

Store can count/retain references without interpreting owner key semantics.

## 22. Object lease

```text
ObjectLeaseRecord
    lease_id
    ObjectId or generation reference set
    holder class
    start/expiry/cancellation state
```

Lease timing is operational/noncanonical. GC eligibility treats active/unknown lease conservatively.

## 23. GC decision/report

```text
ObjectGcDecision
    ObjectId
    retained generation reference count
    active pointer reference
    lease state
    quarantine/corruption state
    eligible: yes | no | unknown
    reasons[]
```

```text
GarbageCollectionReport
    report_id
    reference snapshot identity
    candidates/scanned/deleted/skipped/quarantined IDs[]
    budgets/cancellation
    canonical semantic digest excluding run time/order
```

Only `eligible=yes` deletes.

## 24. Project store (deferred model)

```text
ProjectStoreGeneration
    project-store generation ID
    selected ProjectGenerationId
    schema/migration/runtime profile IDs
    WAL/checkpoint state
    publication transaction ID
```

No implementation in E1-A. See `PROJECT_STORE.md`.

## 25. Store open result

```text
StoreOpenResult
    store/generation/manifest IDs
    open mode/profile
    validation report
    read view/transaction factory identity
    capability state
```

Reference open rejects any write capability or sidecar generation.

## 26. Failure/quarantine record

```text
StoreFailureRecord
    candidate/operation/store IDs
    error code
    schema/migration/file/object/integrity refs
    previous active/last-known-good IDs
    quarantine/temp cleanup state
    recovery class
```

No raw private path or SQL text in public form.

## 27. Durability level

```text
DurabilityLevel
    process_atomic
    crash_atomic_same_volume
    power_loss_best_effort
    power_loss_tested
```

Exact vocabulary may evolve, but record must distinguish rename atomicity from flushed durable persistence. Never promote level without platform test evidence.

## 28. Budget model

```text
StoreBudgetSpec/Usage
    max schema objects/migrations/operations
    max DB/page/file/object sizes
    max object count/bytes
    max read/write rows/result bytes
    max validation/GC work units
    cancellation checkpoints
```

Wall-clock timing is supplemental; deterministic work/size counts are canonical.

## 29. Canonical ordering

```text
schema bundles by namespace/version/ID
migration edges by namespace/from/to/ID
operations by catalog/operation ID
integrity checks by kind/ID
object manifests by ObjectId
payloads by codec/version/payload ID
references by generation/ObjectId/owner key
publication/retention records by store/generation/state/ID
```

Physical row insertion/page order is not semantic ordering.

## 30. Fixture IDs

Closed E1-A case sets:

```text
wow-store-e1-runtime-profile-v1
wow-store-e1-schema-migrations-v1
wow-store-e1-reference-publication-v1
wow-store-e1-object-store-v1
wow-store-e1-security-integrity-v1
```

Exact library/profile/schema/migration/store/object/publication IDs/digests freeze before implementation.
