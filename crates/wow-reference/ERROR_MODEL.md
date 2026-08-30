# `wow-reference` E0-B error model

**Status:** normative typed failure vocabulary.

Human-readable messages are projections. Callers and tests must inspect structured codes, profile/generation context, affected capability/partition IDs, and source/evidence references.

## 1. Error shape

```text
ReferenceError
    code: ReferenceErrorCode
    operation: String
    profile_id: optional ProfileId
    reference_generation: optional ReferenceGenerationId
    input_id: optional ReferenceInputId
    entity_key: optional EntityKey
    source_handle_id: optional StableHandleId
    capability_ids: CapabilityId[]
    partition_ids: CoveragePartitionId[]
    conflict_ids: ConflictId[]
    message_arguments: ordered map
    retry_class: never | after_contract_fix | after_input_fix | after_profile_selection
```

Errors must not embed full untrusted source, credentials, absolute host paths, or opaque model-generated prose as contract data.

## 2. Profile errors

### `fixture_profile_invalid`

The fixture declaration is missing a required field, uses invalid canonical identity, or contains contradictory profile data.

### `fixture_profile_release_masquerade`

A fixture profile is marked/used as release-grade.

### `fixture_profile_floating_identity`

A durable identity uses `latest`, `current`, an unpinned branch, or equivalent floating selector.

### `reference_generation_mismatch`

Two records/view/request inputs refer to different reference generations.

### `reference_view_profile_mismatch`

A lookup request profile/generation differs from the opened view.

### `schema_version_unsupported`

Fixture/model schema version is not supported.

## 3. Inventory errors

### `input_inventory_invalid`

The inventory has duplicate/missing IDs, inconsistent declared order, invalid paths, or unresolved references.

### `missing_declared_input`

A declared input is absent. The error/gap includes affected partitions.

### `undeclared_input`

Supplied material is not part of the closed inventory.

### `input_digest_mismatch`

Supplied content identity differs from the declared digest.

### `input_length_mismatch`

Supplied canonical byte length differs from the declaration.

### `input_path_invalid`

Path is absolute, traverses outside the fixture root, uses a device/UNC path, or violates normalization rules.

### `input_order_invalid`

Declared semantic order is duplicate, incomplete, or inconsistent.

## 4. Raw-value and evaluator errors

### `raw_value_invalid`

Canonical value violates type, depth, duplicate-key, number, string, or size constraints.

### `unsupported_declarative_construct`

The source/fixture contains a construct outside the allow-list. The record includes exact capability/partition impact.

### `evaluator_budget_exceeded`

Declared byte/record/depth/entry/expression budget is exceeded. Truncation/partial coverage is explicit.

### `registration_shape_invalid`

A supposed documentation registration does not match an allow-listed shape.

### `registration_target_unknown`

Registration target/system is not declared by the fixture policy.

### `dynamic_execution_forbidden`

An adapter attempted or requested arbitrary execution, IO, module loading, metatable behavior, or another prohibited runtime capability.

This is a hard security-contract failure.

## 5. Raw/typed projection errors

### `unknown_field_preserved`

An unknown field was retained and classified. This is normally a structured notice/gap, not fatal by itself.

### `unknown_field_blocks_capability`

An unknown field prevents a dependent typed capability from being complete.

### `invalid_unknown_field_classification`

Unknown field lacks exact affected capabilities/partitions or uses an unsupported classification.

### `lowering_contract_invalid`

Raw input cannot be lowered into the requested typed system/function/facet contract.

### `lowered_fact_reference_invalid`

A lowered fact points to missing/mismatched raw/source/evidence/context records.

### `restriction_facet_shape_unsupported`

Facet kind/target/applicability/payload is outside E0 support. Raw data remains preserved.

## 6. Duplicate/conflict errors

### `duplicate_registration_equivalent`

Equivalent duplicate observations were found. This is usually non-fatal and retains all provenance.

### `duplicate_registration_conflict`

Observations for one normalized subject disagree on a contract dimension.

### `conflict_record_invalid`

Conflict record has fewer than two competing evidence records, mismatched context, unresolved IDs, or no affected capability/partition.

### `conflict_winner_forbidden`

An implementation attempted to select a first/last/popular observation rather than returning conflict.

## 7. Coverage/authority errors

### `coverage_record_invalid`

Coverage record has invalid producer/capability/partition/context or unsupported status/reason combination.

### `lookup_capability_unavailable`

Required capability has no usable coverage for this view/query.

### `lookup_conflict`

An unresolved conflict blocks the requested contract dimension.

### `negative_authority_unavailable`

An absent exact key cannot be declared absent authoritatively. Reasons are explicit and generally represented in a successful `absent_without_authority` lookup outcome rather than thrown as an exceptional error.

### `coverage_partition_selection_invalid`

Lookup selected a broader, unrelated, or incomplete partition set incapable of proving the requested result.

### `authority_bypass_forbidden`

An implementation attempted to infer absence without the `wow-core` authority decision.

## 8. Model/view errors

### `reference_model_invalid`

The assembled fixture model violates ID resolution, context, raw/lowered, evidence, conflict, coverage, or canonical digest invariants.

### `reference_model_digest_mismatch`

Declared canonical model digest differs from recomputed bytes.

### `reference_view_invalid`

A view was opened from an unvalidated/mutable model or lacks required identity.

### `reference_source_handle_invalid`

A source handle uses an unregistered origin, project location, invalid path/span, or mismatched digest/context.

### `reference_subject_not_resolvable`

Requested entity/raw/facet/evidence cannot resolve to a registered source handle.

## 9. Lookup query errors

### `exact_query_invalid`

Entity key/kind/profile/generation is malformed or unsupported.

### `exact_query_noncanonical`

Query requires guessing/case correction/namespace repair rather than exact canonical parsing.

### `lookup_fallback_forbidden`

Exact lookup attempted alias, fuzzy, lineage, semantic, external, or replacement fallback.

### `operation_not_implemented_for_milestone`

A caller requested a deferred E1+ operation. This must not return an empty successful result.

## 10. Fixture errors

### `fixture_variant_invalid`

Variant overlay is undeclared, inconsistent with base digest, or changes data outside its allowed scope.

### `fixture_lookup_case_invalid`

Lookup case references unknown variant/query/outcome/expected IDs.

### `fixture_checksum_mismatch`

Normative fixture/member digest differs from declaration.

### `fixture_bundle_invalid`

Closed bundle fails profile, inventory, model, lookup-case, or checksum validation.

## 11. Fatal versus degradable

### Fatal for model/view creation

```text
fixture_profile_invalid
fixture_profile_release_masquerade
reference_generation_mismatch
input_inventory_invalid
input_digest_mismatch
input_path_invalid
dynamic_execution_forbidden
reference_model_invalid
reference_model_digest_mismatch
fixture_bundle_invalid
```

### Usually degradable to partition coverage/gap

```text
missing_declared_input
unsupported_declarative_construct
evaluator_budget_exceeded
unknown_field_blocks_capability
restriction_facet_shape_unsupported
duplicate_registration_conflict
```

Degradation is allowed only when the model can isolate the affected partition and retain one coherent generation.

### Lookup outcomes rather than exceptions

```text
found
authoritative_absent
absent_without_authority
conflict
capability_unavailable
```

Profile/generation mismatch remains an error/rejection rather than an ordinary miss.

## 12. Retry classes

```text
never
    deterministic invalid request or prohibited behavior

after_contract_fix
    implementation/schema contract must change

after_input_fix
    fixture/source/inventory must be corrected

after_profile_selection
    caller must select matching explicit profile/generation
```

Blind retry loops are prohibited.

## 13. Error testing

Each code used by E0-B must have:

- one positive triggering fixture/case;
- exact operation/context assertions;
- no leaked absolute path/source payload;
- deterministic canonical serialization;
- a mutation proving the check can fail.

Unused future error codes must not be exported merely to appear comprehensive.
