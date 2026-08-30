# `wow-reference` E0-B data model

**Status:** normative E0-B semantic model; no Rust layout is mandated.

The types below define ownership and invariants. Concrete Rust names may vary only if [`CONTRACT.json`](CONTRACT.json), this document, and executable fixtures are updated together.

## 1. Object graph

```text
FixtureBundle
├── FixtureProfileDeclaration
├── ReferenceInputInventory
│   └── ReferenceInput[]
├── EvaluatorPolicy
├── RawCanonicalRecord[]
├── NormalizedSystem[]
├── NormalizedFunction[]
├── RestrictionFacet[]
├── RegistrationConflict[]
├── ReferenceCoverageSet
└── ReferenceLookupCase[]

Validated FixtureBundle
    -> FixtureReferenceModel
    -> immutable ReferenceView
```

All identity/evidence/coverage primitives use `wow-core` contracts.

## 2. Fixture profile declaration

```text
FixtureProfileDeclaration
    profile_identity: ProfileIdentity
    profile_kind: fixture
    fixture_schema_version: SchemaVersion
    fixture_catalog_id: String
    reference_generation: ReferenceGenerationId
    source_context: FixtureSourceContext
    release_eligible: false
```

`FixtureSourceContext` contains:

```text
flavor
Interface number
client build
source repository identity
source revision
source verification date
fixture derivation note
fixture content digest
```

The source context is provenance. Fixture records remain project-owned minimized test data.

### Invariants

- `profile_kind` is exactly `fixture`.
- `release_eligible` is always false.
- profile ID, Interface, build, revision, and content digest are non-floating.
- reference generation is derived from canonical fixture inputs, not wall-clock time.
- no `latest`, `current`, branch-only, or URL-only identity is accepted.

## 3. Reference input inventory

```text
ReferenceInputInventory
    profile_id: ProfileId
    reference_generation: ReferenceGenerationId
    declared_order: ReferenceInputId[]
    inputs: ReferenceInput[]
    inventory_digest: ContentDigest
```

```text
ReferenceInput
    input_id: ReferenceInputId
    kind: ReferenceInputKind
    provider: String
    revision: String
    normalized_path: String
    content_digest: ContentDigest
    byte_length: u64
    license_class: String
    declared_partition_ids: CoveragePartitionId[]
    order_index: u32
```

E0 `ReferenceInputKind` values:

```text
fixture_registration_source
fixture_inventory
fixture_variant_overlay
```

### Invariants

- normalized paths are repository-relative and slash-normalized;
- every input appears exactly once in `declared_order`;
- `order_index` agrees with `declared_order`;
- duplicate path/digest identities are classified explicitly;
- digest/byte length are verified before evaluation;
- undeclared input is rejected;
- missing declared input invalidates or degrades only its declared partitions according to the variant contract.

## 4. Evaluator policy

```text
EvaluatorPolicy
    policy_id: String
    policy_version: SchemaVersion
    max_input_bytes: u64
    max_records: u32
    max_table_depth: u16
    max_table_entries: u32
    max_expression_steps: u32
    allowed_value_forms: String[]
    allowed_registration_shapes: String[]
    rejected_capabilities: String[]
```

The policy is explicit fixture input, not hidden implementation configuration.

## 5. Canonical raw values

```text
RawValue
    Null
    Boolean(bool)
    Integer(i64)
    Decimal(CanonicalDecimal)
    String(String)
    Array(RawValue[])
    Object(sorted String -> RawValue)
```

E0 prefers integer-safe fixture values. Decimal support may remain declared-but-unused if `wow-core` canonical numeric rules are not implementation-ready.

### Canonical object rules

- keys are Unicode scalar strings validated by the fixture grammar;
- object keys serialize in lexical byte order;
- array order is semantic;
- duplicate object keys are invalid;
- no functions, userdata, threads, metatables, or opaque runtime values;
- no NaN, infinity, negative zero ambiguity, or locale-dependent number text.

## 6. Raw canonical records

```text
RawCanonicalRecord
    raw_record_id: RawRecordId
    profile_id: ProfileId
    reference_generation: ReferenceGenerationId
    input_id: ReferenceInputId
    registration_index: u32
    registration_kind: String
    raw_payload: RawValue
    unknown_fields: UnknownField[]
    source_handle: SourceHandle
    evidence_id: EvidenceId
```

```text
UnknownField
    canonical_field_path: String
    raw_value: RawValue
    classification: UnknownFieldClassification
    affected_capability_ids: CapabilityId[]
```

`UnknownFieldClassification`:

```text
preserved_uninterpreted
preserved_known_safe_projection_gap
preserved_capability_blocking
invalid_field_shape
```

A blocking unknown field does not disappear after lowering.

## 7. Normalized system

```text
NormalizedSystem
    entity_key: EntityKey
    canonical_name: String
    namespace_kind: String
    documentation: optional String
    raw_record_id: RawRecordId
    source_handle: SourceHandle
    evidence_ids: EvidenceId[]
    unknown_fields: UnknownField[]
```

E0 defines one system:

```text
C_E0Fixture
```

## 8. Normalized function

```text
NormalizedFunction
    entity_key: EntityKey
    system_entity_key: EntityKey
    canonical_name: String
    qualified_name: String
    arguments: FunctionArgument[]
    returns: FunctionReturn[]
    documentation: optional String
    availability: AvailabilityContract
    raw_record_id: RawRecordId
    source_handle: SourceHandle
    evidence_ids: EvidenceId[]
    unknown_fields: UnknownField[]
```

```text
FunctionArgument
    position: u16
    name: String
    type_name: String
    nullable: bool
    optional: bool
```

```text
FunctionReturn
    position: u16
    name: optional String
    type_name: String
    nullable: bool
```

```text
AvailabilityContract
    profile_id: ProfileId
    present: true
    build_applicability: String
```

E0 functions:

```text
C_E0Fixture.KnownApi
C_E0Fixture.SecretText
```

`C_E0Fixture.RemovedApi` is not a stored entity; it is an exact query key in lookup cases.

## 9. Restriction facet

```text
RestrictionFacet
    facet_id: RestrictionFacetId
    facet_kind: String
    subject_entity_key: EntityKey
    target_slot: RestrictionTargetSlot
    applicability: RestrictionApplicability
    normalized_payload: RawValue
    raw_field_path: String
    source_handle: SourceHandle
    evidence_ids: EvidenceId[]
    coverage_partition_id: CoveragePartitionId
```

```text
RestrictionTargetSlot
    return_position(u16)
    argument_position(u16)
    entity
```

```text
RestrictionApplicability
    unconditional_fixture
    predicate_bound(String)
    unknown
```

E0 stores:

```text
facet_kind = secret.return
target = return_position(1)
subject = C_E0Fixture.SecretText
applicability = unconditional_fixture
```

This is a test contract, not a runtime wrapper claim.

## 10. Duplicate and conflict records

```text
RegistrationObservation
    normalized_subject_key: String
    raw_record_id: RawRecordId
    evidence_id: EvidenceId
    canonical_contract_digest: ContentDigest
```

```text
RegistrationClassification
    unique
    equivalent_duplicate
    incompatible_duplicate
```

```text
RegistrationConflict
    conflict_record: ConflictRecord
    subject_key: String
    observation_ids: RawRecordId[]
    affected_capability_ids: CapabilityId[]
    affected_partition_ids: CoveragePartitionId[]
```

Equivalent duplicates retain all provenance but produce one normalized fact. Incompatible duplicates produce no arbitrary winner.

## 11. Coverage partitions

E0 partition IDs:

```text
reference.fixture.inventory
reference.fixture.apidoc.system:C_E0Fixture
reference.fixture.restriction:C_E0Fixture.SecretText
```

E0 capability IDs:

```text
reference.fixture.profile.valid
reference.fixture.inputs.complete
reference.symbol.exact_lookup
reference.restriction.facets
reference.source_handle.resolve
```

`ReferenceCoverageSet` is a collection of exact `wow-core CoverageRecord` values. It is not a free-form summary object.

## 12. Fixture variants

```text
FixtureVariant
    variant_id: complete | partial | conflict
    base_bundle_digest: ContentDigest
    overlay_records: RawCanonicalRecord[]
    omitted_input_ids: ReferenceInputId[]
    forced_unknown_fields: UnknownField[]
    expected_conflicts: RegistrationConflict[]
    expected_coverage_records: CoverageRecord[]
```

### `complete`

All declared E0 partitions are complete and unconflicted.

### `partial`

The exact-symbol partition is partial because one declared source capability is unavailable. Existing known symbols may still resolve; an absent query lacks negative authority.

### `conflict`

Input reading is complete, but `SecretText` has incompatible restriction observations. Symbol presence can remain known while restriction-facet lookup is conflicted.

## 13. Reference model

```text
FixtureReferenceModel
    profile: FixtureProfileDeclaration
    inventory: ReferenceInputInventory
    evaluator_policy: EvaluatorPolicy
    raw_records: RawCanonicalRecord[]
    systems: NormalizedSystem[]
    functions: NormalizedFunction[]
    restriction_facets: RestrictionFacet[]
    conflicts: RegistrationConflict[]
    coverage_records: CoverageRecord[]
    source_handles: SourceHandle[]
    evidence_records: EvidenceRecord[]
    canonical_model_digest: ContentDigest
```

### Invariants

- all records share one profile/reference generation;
- all referenced IDs resolve exactly once;
- source handles are reference/fixture-side only;
- evidence provenance is eligible for the registered origin;
- facts with incompatible duplicates are not silently materialized;
- raw records remain accessible after lowering;
- unknown fields and conflicts participate in model validation;
- canonical digest excludes volatile metadata.

## 14. Reference view

`ReferenceView` is an immutable read capability over one validated model/variant.

Conceptual methods:

```text
profile_identity() -> ProfileIdentity
reference_generation() -> ReferenceGenerationId
coverage_records(capability_ids, partition_ids) -> CoverageRecord[]
lookup_symbol_exact(entity_key) -> ExactSymbolLookup
lookup_restriction_facets(entity_key) -> RestrictionFacetLookup
resolve_reference_source_handle(subject_id) -> SourceHandleResolution
```

It exposes no mutation, persistence, SQL, evaluator, or source-acquisition API.

## 15. Exact lookup result

```text
ExactSymbolLookup
    query: ExactSymbolQuery
    context: GenerationContext
    outcome: ExactLookupOutcome
    entity: optional NormalizedFunction/System
    evidence_ids: EvidenceId[]
    conflict_ids: ConflictId[]
    coverage_ids: CoverageId[]
    negative_authority: NegativeAuthorityDecision
```

```text
ExactLookupOutcome
    found
    authoritative_absent
    absent_without_authority
    conflict
    profile_mismatch
    capability_unavailable
```

A found result still reports coverage and conflicts relevant to the returned fact.

## 16. Restriction lookup result

```text
RestrictionFacetLookup
    subject_entity_key: EntityKey
    context: GenerationContext
    outcome: RestrictionLookupOutcome
    facets: RestrictionFacet[]
    evidence_ids: EvidenceId[]
    conflict_ids: ConflictId[]
    coverage_ids: CoverageId[]
```

```text
RestrictionLookupOutcome
    found
    none_authoritatively
    unavailable
    conflict
    profile_mismatch
```

`none_authoritatively` requires complete restriction-facet coverage for the subject domain, not merely symbol coverage.

## 17. Lookup case record

```text
ReferenceLookupCase
    case_id: String
    variant_id: String
    query_kind: symbol_exact | restriction_facets
    query_key: String
    expected_outcome: String
    expected_entity_keys: String[]
    expected_facet_kinds: String[]
    expected_coverage_status: String
    expected_negative_authority: String
    expected_conflict_codes: String[]
```

These records drive implementation tests and are not production search history.

## 18. Canonicalization boundary

Canonical fixture/model serialization follows `wow-core` JSON rules plus these domain rules:

- systems/functions sort by canonical entity key;
- arguments/returns sort by numeric position;
- facets sort by subject, kind, target slot, evidence ID;
- raw records preserve declared registration index;
- unknown fields sort by canonical field path;
- coverage records sort by capability, partition, producer, coverage ID;
- lookup cases sort by `case_id`;
- no timestamps or local paths in canonical identity.
