# Persistent reference schema and operation bundles

**Status:** normative E1-B domain schema, prepared-operation, validation, and `wow-store` seam contract.

`wow-reference` owns the logical schema and typed record adapters. `wow-store` owns SQLite/schema execution, transactions, file/object lifecycle, integrity, sealing, publication, and read-only connection behavior.

## 1. No raw SQL seam

The cross-crate seam contains:

```text
ReferenceStoreSchemaBundle
ReferenceStoreWriteOperationCatalog
ReferenceStoreReadOperationCatalog
ReferenceStoreValidationCatalog
ReferenceStoreBuildPlan
encoded typed operation parameters/results
```

It does not contain application/source/user-provided SQL, raw connection handles, or callbacks that can execute arbitrary storage logic.

Static SQL/structured schema declarations are repository-owned implementation artifacts inside the registered bundle and digest-bound by `wow-store`.

## 2. Schema namespace

Conceptual domain namespace:

```text
wow_reference_e1
```

Exact physical table/index names freeze before code. The schema must separate normalized reference facts from raw observations and operational/evidence state.

## 3. Logical record families

### Identity/profile/build

```text
reference_profile
source_snapshot
source_provider_provenance
source_partition
source_file
reference_generation
reference_data_manifest
reference_build_report
```

### Raw ingestion

```text
parsed_source
raw_value
raw_value_edge_or_entry
registration_observation
raw_observation
unknown_field
unsupported_construct
```

### Normalized entities/facts

```text
reference_entity
entity_name_owner_system_index
api_callable
callable_parameter
callable_return
api_table_or_structure
api_table_field
event
event_payload_field
enum
enum_value
cvar
widget_or_script_object
widget_method
predicate
restriction_facet
deprecation
explicit_transition
exact_reference_link
```

### Evidence/correction/coverage

```text
source_handle_or_source_map_ref
evidence_reference
curated_correction_set
curated_correction
correction_application
reference_conflict
coverage_record
capability_summary
not_evaluated_record
```

### Objects/manifests

Large raw metadata/value/source-map/report artifacts may use `wow-store` ObjectStore with domain object references. Schema records exact ObjectId/type/owner/reference links.

## 4. Normalization and redundancy policy

- One canonical profile/reference generation foreign-key prefix or store-generation metadata guards every domain record.
- Stable domain IDs stored explicitly and validated; database rowid is not public identity.
- Ordered members/fields carry semantic ordinal.
- Names/system/kind/type keys indexed for exact query.
- Raw canonical value tree can use normalized nodes/edges plus optional object payload for large values; logical round-trip mandatory.
- JSON blobs may store bounded extension attributes only when schema/version/validation/unknown-field preservation contract allows; not as sole representation for critical identity/facts/coverage.
- Duplicate source occurrences use occurrence/evidence tables rather than overwriting normalized fact.
- Conflicts/corrections/coverage remain first-class, not flags hidden in entity rows.

## 5. Required keys and constraints

Conceptually:

```text
all public domain IDs unique
all generation/profile IDs equal store manifest context
all raw observations reference valid file/registration/value/source records
all normalized facts reference valid entity/raw/evidence records
ordered member ordinals unique per owner/sequence as contract requires
restriction facets target valid exact entity/member/position
correction applications reference valid correction/raw/before/after facts
conflicts reference >=2 competing records where required
coverage references known capability/partition/producer/generation
object refs point to verified ObjectIds in generation reference set
```

Foreign keys/checks/unique indexes enforce physical invariants; registered validation checks enforce cross-table semantic closure.

## 6. Schema bundle identity

```text
ReferenceStoreSchemaBundle
    bundle ID = null until freeze
    namespace/version
    owner contract ID/version
    parent version(s)
    schema declarations/digests
    required SQLite capabilities
    migration edges
    write/read operation catalog IDs
    validation catalog ID
    expected normalized schema digest
```

E1 initial path is empty -> `wow-store` metadata v1 -> reference domain schema v1. No released in-place migration.

## 7. Write operation catalog

Operations are narrow and batchable. Conceptual groups:

### Build identity

```text
insert_reference_profile
insert_source_snapshot_and_providers
insert_source_partitions_and_files
insert_reference_generation_candidate_metadata
```

### Raw records

```text
insert_parsed_source_records
insert_raw_values_and_entries
insert_registration_observations
insert_raw_observations
insert_unknown_fields
insert_unsupported_constructs
```

### Normalized records

```text
insert_reference_entities
insert_api_callable_records
insert_callable_parameters_returns
insert_table_structure_records_and_fields
insert_event_records_and_payloads
insert_enum_cvar_widget_records
insert_predicate_restriction_records
insert_deprecation_transition_records
insert_exact_reference_links
```

### Evidence/corrections/coverage

```text
insert_source_evidence_records
insert_correction_sets_records_and_applications
insert_reference_conflicts
insert_coverage_records_and_capability_summaries
insert_not_evaluated_records
```

### Final manifests

```text
insert_reference_build_report_and_manifest
finalize_reference_generation_record
```

Each operation descriptor defines exact parameter/result schema, store state, transaction requirements, cardinality/budget, and static statement digest.

## 8. Build operation order

Conceptual phase order:

```text
profile/snapshot/partitions/files
-> parsed/raw values/registrations/observations
-> entities/normalized facts/members/links
-> corrections/applications
-> conflicts/coverage/capabilities/NotEvaluated
-> objects/source maps/raw metadata refs
-> build report/manifest/final generation metadata
```

Within independent record sets, canonical ID order. Foreign-key/dependency order explicit. No hidden auto-generated DB IDs used to determine semantic ordering.

## 9. Read operation catalog

Exact read groups:

```text
read_profile_generation_manifest
read_capability_coverage_partition
lookup_entity_by_exact_key
lookup_callable_by_exact_system_name_kind_signature
lookup_member_by_exact_owner_name_or_ordinal
lookup_event_by_exact_system_name
lookup_table_structure_enum_cvar_widget
lookup_predicate_and_restriction_facets
lookup_explicit_deprecation_transition
read_raw_observation_or_field
read_unknown_unsupported_conflict_correction_relations
resolve_reference_source_handle
list_exact_scope_entities_bounded
read_build_report_manifest_counts
```

No fuzzy `LIKE`/trigram/FTS semantic fallback in E1 ReferenceView. Prefix/list operations only under exact bounded documented semantics and never masquerade as exact entity lookup.

## 10. Result schemas

Read operations return encoded domain rows with:

```text
stable domain IDs
exact profile/reference/store generation
ordered fact/member/raw/evidence/coverage/conflict/correction refs
operation result cardinality/truncation/budget state
```

`wow-reference` decodes/validates into ReferenceView domain results. Raw SQLite row types do not escape.

## 11. Validation catalog

Mandatory checks before seal/publication/open include:

### Identity closure

- all records belong to exact ProfileId/ReferenceGenerationId;
- profile/snapshot/parser/evaluator/normalizer/correction/schema/build identities match manifest;
- no cross-profile/generation links.

### Raw closure

- raw values/entries form valid acyclic/allowed canonical value graphs;
- every observation resolves file/registration/source/value;
- unknown/unsupported records preserve exact raw/source refs;
- no orphan large object references.

### Normalized closure

- entity keys unique/valid per kind;
- every fact/member resolves owner/entity/raw/evidence;
- ordered members/positions contiguous or explicitly sparse under contract;
- type/reference/restriction/predicate/transition links valid;
- exact duplicates/conflicts represented consistently.

### Correction closure

- set/records/applications/digests/status valid;
- Applied has valid before/after/raw/evidence;
- Expired/Conflict blockers linked to coverage;
- raw source unchanged.

### Coverage/authority closure

- every declared capability/partition has required coverage records;
- summaries conservatively derive from records;
- unknown/unsupported/conflict/truncation blockers linked;
- no authoritative-negative precomputed flag contradicts records;
- store success does not erase partial ingestion.

### Count/digest/manifests

- declared fact/raw/unknown/unsupported/conflict/correction/coverage counts match rows/objects;
- logical manifests/digests match canonical domain records;
- ReferenceGeneration/ReferenceDataManifest/store generation references close;
- build report records every declared input outcome.

## 12. Object plan

Large logical artifacts can include:

```text
raw canonical metadata partitions
source-map/span bundles
bounded source evidence snippets only if policy permits
build/evaluation/correction/coverage reports
raw parsed input snapshots only if redistribution/license policy permits
```

Each object request specifies logical type/canonicalization/bytes/expected digest/codec profile and opaque domain owner key. Store creates/verifies/publishes objects and reference set.

No source path/name in object filename. Reference manifest links exact ObjectIds.

## 13. Incremental/reuse policy

E1 first implementation may rebuild complete persistent ReferenceStore deterministically. Reuse of unchanged objects is allowed through ObjectId dedup.

Row-level incremental reuse/copy from an old store is deferred unless an exact source/profile/schema/correction equality and validation contract is accepted. Never reuse facts because file path/name matches.

## 14. Read-only store rule

After `wow-store` seals/publishes:

- ReferenceView opens exact immutable generation read-only;
- no write operation catalog available;
- no migration/update/correction application against published file;
- no journal/WAL/SHM sidecar;
- store manifest/schema/file/object identities validated;
- active pointer change does not switch existing view.

## 15. Query indexes

E1 exact indexes should support measured operations, conceptually:

```text
entity(profile/reference, kind, system/namespace, canonical name, owner/signature discriminator)
member(owner entity, kind/name/ordinal)
source/raw(entity/file/field path)
restriction/predicate(target entity/member/facet kind)
deprecation/transition(target entity)
coverage(capability, partition)
conflict(subject/entity/field)
correction(target/status)
```

Do not add FTS/vector/fuzzy/trigram indexes in E1. Index count/shape must be justified by exact query/performance fixtures.

## 16. Schema evolution

A new field/entity/query capability requires:

1. raw backward compatibility analysis;
2. normalized model/field registry change;
3. schema/operation/validation bundle version and migration/rebuild plan;
4. coverage dependency update;
5. ReferenceView contract/tests;
6. annotation consumer impact where relevant;
7. new ReferenceGeneration build; no in-place released mutation.

## 17. Security

- static repository-owned SQL only via `wow-store`;
- typed encoded parameters/results;
- no source/user field used as SQL identifier;
- names/strings stored as values;
- bounded batches/results;
- no extension/attach/trigger/function beyond registered reviewed bundle;
- public errors omit raw SQL/private paths/excessive source/raw values;
- corrupt/untrusted DB rejects through store validation.

## 18. Determinism

Equivalent logical domain records/bundle/profile produce equivalent:

```text
schema/operation/validation bundle IDs/digests
build operation plan/order/digest
logical domain manifest/count/digest
ReferenceGeneration/ReferenceDataManifest IDs
exact read result IDs/order
```

Independent of SQL rowid/insertion/page order, worker order, temp root/time.

## 19. Required operations

```text
build_reference_store_schema_bundle
validate_reference_store_schema_bundle
build_reference_write_operation_catalog
build_reference_read_operation_catalog
build_reference_validation_catalog
encode_reference_records_for_store
build_reference_store_operation_plan
build_reference_object_plan
validate_reference_store_build_plan
validate_persisted_reference_domain_closure
decode_reference_store_read_results
validate_reference_store_read_result_context
```

## 20. Required tests

- exact schema/operation/validation bundle shape/digest;
- no domain semantics inside `wow-store` and no store SQL inside service/application;
- write all record families and validate closure;
- missing/orphan/cross-generation raw/fact/member/restriction/correction/coverage/object rows rejected;
- count/digest/manifest mismatch;
- exact read operations and bounded list/raw reads;
- no fuzzy/FTS/raw SQL operation;
- sealed store has no write catalog/sidecars;
- schema evolution creates new bundle/generation and preserves raw data;
- randomized input/insertion/row/page order -> same logical plan/manifests/query order;
- object plan licensing/path/privacy constraints.

## 21. Hard stops

- no raw SQL/connection public seam;
- no domain interpretation in store;
- no JSON-blob-only critical facts;
- no rowid semantic identity;
- no missing coverage/conflict records;
- no fuzzy/FTS/vector index/query;
- no in-place sealed update;
- no cross-profile/generation row/link;
- no incomplete build manifest publication;
- no source/user string as SQL identifier.
