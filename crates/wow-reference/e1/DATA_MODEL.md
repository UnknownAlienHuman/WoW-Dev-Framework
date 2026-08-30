# `wow-reference` E1-B data model

**Status:** normative semantic model for exact source snapshots, raw observations, normalized reference facts, corrections, coverage, persistent schema/build plans, and ReferenceView results.

Concrete Rust/SQLite layout may differ. Identity, provenance, profile isolation, raw preservation, correction, coverage, and query semantics may not.

## 1. Object graph

```text
SourceSnapshotManifest
└── ReferenceProfile
    └── ReferenceBuildRequest
        ├── InputPartitionManifest[]
        │   └── SourceFileRecord[]
        │       └── ParsedSourceRecord
        │           └── EvaluationRecord[]
        │               └── RawObservation[]
        ├── NormalizedReferenceFact[]
        ├── UnsupportedConstructRecord[]
        ├── UnknownFieldRecord[]
        ├── CuratedCorrectionSet
        │   └── CorrectionApplicationRecord[]
        ├── ReferenceConflictRecord[]
        ├── CoverageRecord[] / CapabilitySummary[]
        ├── ReferenceStoreSchemaBundle
        ├── ReferenceStoreBuildPlan
        └── ReferenceDataBuildReport / ReferenceDataManifest

ReferenceView
├── exact fact/raw/source/capability operations
└── NegativeAuthorityDecision
```

## 2. Source snapshot

```text
SourceSnapshotManifest
    snapshot_id
    provider_provenance[]
    materialized_root_identity (private operational root, not canonical absolute path)
    source revision/label
    logical content manifest/digest
    flavor/edition
    Interface number
    client build/version
    source timestamp/release label: supplemental
    declared input partition records[]
    file manifest[]
    licenses/notices[]
    acquisition report identity
    snapshot schema/version
    canonical digest
```

Provider can be mirror/local official export/interface export/etc. Multiple provider records may point to the same logical content digest.

## 3. Reference profile

```text
ReferenceProfile
    ProfileIdentity
    eligibility: fixture | candidate | release
    SourceSnapshotManifest ID
    parser/evaluator/normalizer versions
    schema registry/bundle versions
    correction-set ID/digest
    input-partition policy ID
    capability expectation profile
    selected optional resource inputs
    profile schema version
```

Release eligibility requires all mandatory identity/source/license/input conditions. Fixture profiles explicitly remain incomplete/nonrelease.

## 4. Input partition

```text
InputPartitionManifest
    partition_id
    partition kind
    capability families produced
    declared file/source records in semantic order
    required/optional
    dependencies on other partitions
    completeness criteria
    parser/evaluator/normalizer budgets
    expected source digest/count
```

Initial kinds:

```text
apidoc.systems
apidoc.generated
apidoc.deprecated
apidoc.events
apidoc.tables_structures
apidoc.widgets_script_objects
resource.enums
resource.cvars
resource.predicates_restrictions
transition.explicit
```

Exact physical grouping depends on the pinned snapshot. One file may feed several logical capability partitions through explicit mapping.

## 5. Source file record

```text
SourceFileRecord
    source_file_id
    snapshot/profile/partition IDs
    normalized snapshot-relative path
    content digest/byte length/encoding/line ending facts
    declared semantic order
    file kind/dialect
    license/provenance refs
    source handle root
```

No absolute host path in public identity.

## 6. Parsed source record

```text
ParsedSourceRecord
    parsed_source_id
    source file/content/parser IDs
    syntax tree/root identity
    parse diagnostics[]
    source span map identity
    node/count/depth/byte budgets
    parse status/coverage
    canonical digest of relevant syntax facts
```

Parser tree/internal handles do not escape as public API unless converted into stable source spans/facts.

## 7. Canonical raw value

```text
RawCanonicalValue
    Null
    Boolean
    Integer or canonical numeric representation
    String (exact bytes/text encoding contract)
    Array/ordered table entries
    Map/field entries with canonical key/value observation order
    Reference to known constant/raw binding when preserving reference form matters
    Unsupported value record reference
```

Lua table semantics require explicit treatment of array/map order, duplicate keys, computed keys, nil entries, and numeric normalization. No opaque JSON string shortcut.

## 8. Evaluation record

```text
EvaluationRecord
    evaluation_id
    parsed source/node/span
    evaluator version/policy
    environment/input bindings identity
    operation kind
    resulting RawCanonicalValue or UnsupportedConstructRecord
    step/depth/table/string budgets
    status
    canonical digest
```

Evaluation environment contains only allow-listed canonical bindings/constants/registration functions.

## 9. Registration observation

```text
RegistrationObservation
    observation_id
    registration kind/function/system
    declared source order
    raw arguments/value table IDs
    source handle/span
    parser/evaluator/producer IDs
    profile/reference build candidate context
```

Duplicates/conflicts remain separate until exact normalization policy resolves them.

## 10. Raw observation

```text
RawObservation
    raw_observation_id
    profile/build candidate context
    entity candidate key/kind
    raw field path/name
    RawCanonicalValue ID
    registration/source/partition IDs
    source handle/span
    declared order
    known/unknown field classification
    parser/evaluator versions
    canonical digest
```

Raw observations are immutable and survive corrections/projection changes.

## 11. Unknown field record

```text
UnknownFieldRecord
    unknown_field_id
    raw observation ID
    entity/system/kind context
    field path/name/value kind
    affected capability families
    quarantine classification
    review status
    source evidence
```

Unknown does not imply ignored. Dependent coverage is Partial/Unknown/NotEvaluated according to policy.

## 12. Unsupported construct record

```text
UnsupportedConstructRecord
    record_id
    source/node/span
    construct/operator/call/control-flow kind
    evaluator rule/failure code
    affected registration/entity/field/partition/capabilities
    bounded raw syntax summary/digest
    recoverability/review status
```

No arbitrary source dump in default result.

## 13. Normalized entity

```text
ReferenceEntity
    EntityKey / reference entity ID
    ProfileId / ReferenceGenerationId
    kind
    canonical name
    namespace/system/owner keys
    signature/type identity where relevant
    source/raw observation IDs[]
    lifecycle/applicability/build fields
    provenance/confidence
    canonical digest
```

Initial kinds:

```text
api_system
api_function
api_method
api_table
api_structure
api_field
event
event_payload_field
enum
enum_value
cvar
widget
script_object
predicate
deprecation
explicit_transition
restriction_facet
source_artifact
raw_metadata_entity
```

## 14. Callable fact

```text
ApiCallableFact
    entity ID
    callable kind/system/name/receiver
    parameters[]
    returns[]
    throws/availability/applicability metadata where exact
    deprecation/transition refs
    restriction facet refs
    raw observation/evidence IDs
    coverage partition refs
```

Parameter/return records preserve name/type/nilability/enum/table/secret/predicate/source metadata as supported; unknown fields remain raw.

## 15. Table/structure fact

```text
ApiTableFact
    entity ID
    table/structure kind
    fields[]
    inheritance/extension only when explicitly represented
    documentation metadata
    restriction/predicate refs
    raw/evidence/coverage refs
```

## 16. Event fact

```text
EventFact
    entity ID
    system/name
    payload fields/order
    applicability/deprecation/restrictions
    raw/evidence/coverage refs
```

An event declaration does not prove readable runtime payload in every context; restriction metadata and downstream runtime gaps remain explicit.

## 17. Enum/CVar/widget facts

```text
EnumFact / EnumValueFact
CVarFact
WidgetFact / WidgetMethodFact
```

Each retains exact source/projection/raw/coverage. E1 supports only fields present in the selected input contract; no inference from community code.

## 18. Predicate and restriction facts

```text
PredicateFact
    predicate ID/name/system
    target API/entity/argument/return/field scope
    predicate semantics/form as exact source data
    applicability/build/profile
    raw observations/evidence
```

```text
RestrictionFacetFact
    facet ID/kind/version
    target entity/member/argument/return/field
    canonical known payload
    raw payload/value IDs
    applicability/predicate refs
    source/provenance/coverage
```

Known facet registry remains open. Unknown restriction fields/facets are raw-preserved and block dependent safety claims.

## 19. Deprecation/transition fact

```text
DeprecationFact
    target entity
    status/message/version/build applicability
    explicit replacement/moved-to entity only when source states it
    raw/evidence/coverage
```

No replacement from text/name similarity. Historical lineage beyond explicit source facts belongs later.

## 20. Correction set

```text
CuratedCorrectionSet
    correction_set_id
    version
    profile/build applicability
    correction records[]
    reviewer/evidence policy
    canonical digest
```

```text
CuratedCorrection
    correction_id/version
    target exact entity/raw observation/field path
    expected source/value digest
    replacement RawCanonicalValue or normalized field value
    evidence/source handles[]
    rationale/reviewer/review record
    applicability
    dependency/conflict policy
```

## 21. Correction application

```text
CorrectionApplicationRecord
    application_id
    correction ID/set
    target/raw value/source digest observed
    status: Applied | Expired | Rejected | Conflict | NotApplicable
    normalized fact field before/after
    evidence/provenance
    affected coverage/capability refs
    canonical digest
```

Raw observation is unchanged.

## 22. Reference conflict

```text
ReferenceConflictRecord
    conflict_id
    profile/build candidate context
    subject entity/field/partition
    competing raw/fact/correction/evidence IDs[]
    conflict kind
    affected capabilities
    resolution state
```

Conflicts are context-bound and can block authority independently of ingestion completeness.

## 23. Coverage and capability

Use `wow-core` records:

```text
CoverageRecord
CapabilitySummary
NotEvaluatedRecord
ConflictRecord / mapped ReferenceConflictRecord
```

Reference-specific partitions include:

```text
snapshot/file/partition
system/namespace/entity kind
raw metadata family
signature/event payload/table fields
restriction facets/predicates
corrections
persistent schema/write/read validation
```

## 24. ReferenceData generation

```text
ReferenceDataGeneration
    ReferenceGenerationId
    ProfileIdentity
    SourceSnapshotManifest ID/digest
    parser/evaluator/normalizer IDs
    correction set ID/digest
    schema/operation/validation bundle IDs
    normalized fact manifest digest/counts
    raw/unknown/unsupported/conflict manifest digests/counts
    coverage/capability manifest digest
    object/reference store build plan ID
    build tool/version/schema IDs
```

ReferenceGenerationId construction is noncyclic/domain-separated and excludes output fields containing itself.

## 25. Persistent schema bundle

```text
ReferenceStoreSchemaBundle
    exact `wow-store` SchemaBundle shape
    owner contract = wow-reference/e1-persistent-schema-and-build-plan
    normalized/raw/evidence/coverage/conflict/correction/source tables/indexes
    prepared write/read operation catalog
    validation catalog
    expected schema digest
```

Exact SQL/schema names freeze before implementation. Semantics are defined in `STORE_SCHEMA_AND_OPERATIONS.md`.

## 26. Build plan

```text
ReferenceStoreBuildPlan
    plan_id
    ReferenceDataGeneration candidate
    StoreConfiguration/schema/runtime profile IDs
    deterministic registered operation invocations[]
    object write plan[]
    object reference plan[]
    expected counts/digests
    validation invocation plan[]
    budgets/cancellation
    requested durability
    canonical digest
```

No arbitrary SQL/callbacks/hidden state.

## 27. Build report and manifest

```text
ReferenceDataBuildReport
    report_id
    input/profile/parser/evaluator/normalizer/correction IDs
    per partition/file parse/evaluate/normalize outcomes
    raw/unknown/unsupported/fact/correction/conflict/coverage counts
    store build/publication/open results
    validation/parity readiness
    warnings/errors/NotEvaluated/deferred capabilities
    canonical digest
```

```text
ReferenceDataManifest
    manifest schema/version
    ProfileIdentity / ReferenceGenerationId
    SourceSnapshotManifest ID
    correction/schema/parser/tool identities
    ReferenceStore generation/manifest ID
    raw metadata/source-map object refs
    capability/coverage/conflict manifests
    licenses/provenance refs
    build report/checksums
    release eligibility
    canonical digest
```

Annotation artifact is not included/owned here; final pack assembler links it later.

## 28. ReferenceView

```text
ReferenceView
    view_id
    exact ProfileIdentity / ReferenceGenerationId
    ReferenceStore open result/manifest/schema IDs
    capability/coverage/conflict registries
    typed read operation catalog adapter
    budgets/cancellation
```

No raw SQLite handle.

## 29. Lookup request/result

```text
ReferenceLookupRequest
    view/profile/generation
    exact entity kind
    exact canonical name/namespace/system/receiver/member key
    optional exact signature/type discriminator
    requested detail fields
    budgets
```

```text
ReferenceLookupResult
    Found(entity/facts/raw/evidence/coverage/source refs)
    AbsentAuthoritative(NegativeAuthorityDecision)
    NotFoundPartial
    Conflict
    NotEvaluated
    InvalidRequest
```

No fuzzy candidate lane.

## 30. Negative authority decision

```text
NegativeAuthorityDecision
    decision_id
    exact query/profile/generation
    relevant capability/partition IDs
    coverage/conflict/truncation/staleness/runtime-gap checks
    authoritative: yes | no
    blocker reasons[]
    canonical digest
```

## 31. Raw metadata read

```text
RawMetadataRequest
    exact entity/raw observation/field path IDs
    detail depth/count/byte budget
```

```text
RawMetadataResult
    canonical value tree/observation refs
    source/evaluator context
    unknown/unsupported/correction relations
    truncation state
```

No whole-store/source dump.

## 32. Source handle resolution

Reference source handles use snapshot/profile/reference generation, normalized path, exact spans/digest/entity/raw IDs. Resolution is bounded and may return exact source span only through a separate source artifact/object contract; default query can return handle without source body.

## 33. Canonical ordering

```text
partitions by declared semantic order then ID
files by declared order then normalized path/ID
raw observations by entity/field/source order/ID
entities by kind/namespace/name/owner/signature/ID
facts/members/arguments/returns by explicit semantic order then ID
unknown/unsupported/conflicts/corrections by subject/path/source/ID
coverage by capability/partition/ID
build operations by phase/table/entity/operation/ID
query result facts/evidence/source refs by canonical IDs/order
```

## 34. Fixture IDs

Closed E1-B case sets:

```text
wow-reference-e1-source-profile-v1
wow-reference-e1-apidoc-evaluator-v1
wow-reference-e1-normalization-v1
wow-reference-e1-corrections-v1
wow-reference-e1-build-plan-v1
wow-reference-e1-view-v1
```

Exact parser/source/schema/fact/store/query IDs/digests freeze before code.
