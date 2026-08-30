# `wow-reference` E0-B operations

**Status:** normative pure-operation inventory for the future E0-B implementation.

Concrete Rust names may change only when semantic ownership, [`CONTRACT.json`](CONTRACT.json), fixtures, and consumer documentation are updated in the same change.

## 1. General operation rules

Every E0-B operation must be:

- deterministic for equivalent logical inputs;
- explicit about profile and reference generation;
- free of filesystem, network, process, clock, random, editor, and client side effects;
- bounded by declared evaluator/model limits;
- typed on failure;
- conservative about coverage and conflict;
- unable to create project-side evidence or diagnostic findings.

The implementation may load already-read fixture bytes in a test/application adapter. The domain operations consume supplied bytes/records, not arbitrary host paths.

## 2. Profile operations

### `validate_fixture_profile`

**Input**

```text
FixtureProfileDeclaration
```

**Output**

```text
validated FixtureProfileDeclaration
```

**Checks**

- profile ID is canonical;
- `profile_kind = fixture`;
- `release_eligible = false`;
- flavor, Interface, build, revision, and content digest are present;
- no floating ref such as `main`, `latest`, or unqualified `live` is used as durable identity;
- reference generation agrees with canonical fixture inputs;
- fixture schema version is supported.

**Errors**

```text
fixture_profile_invalid
fixture_profile_release_masquerade
fixture_profile_floating_identity
reference_generation_mismatch
schema_version_unsupported
```

### `derive_fixture_reference_generation`

Derive the reference generation from the canonical profile declaration, input inventory digest, evaluator policy version, and fixture variant identity. Do not include timestamps or local paths.

### `require_fixture_profile`

Reject a fixture model/view when a caller requests a release-grade profile. This is not a warning or fallback.

## 3. Input inventory operations

### `inventory_fixture_inputs`

**Input**

```text
declared inventory metadata
supplied input descriptors and content identities
```

**Output**

```text
ReferenceInputInventory
inventory diagnostics
partition impact records
```

**Behavior**

1. normalize repository-relative paths;
2. reject absolute/traversal/device paths;
3. verify declared order and unique input IDs;
4. verify byte length and SHA-256 content digest;
5. classify duplicate path/input identities;
6. report missing/undeclared inputs;
7. calculate canonical inventory digest;
8. attach affected partition IDs to every gap.

### `validate_reference_input`

Validate one input descriptor and supplied content identity without evaluating its contents.

### `canonicalize_input_inventory`

Serialize inventory in declared semantic order while canonicalizing non-semantic map/set ordering.

## 4. Raw-value operations

### `parse_raw_value`

Parse only the canonical fixture data representation into `RawValue`.

This operation is not a general Lua parser. When implemented over Emmy CST facts, the adapter first proves that the source expression belongs to the allow-listed declarative subset.

### `validate_raw_value`

Enforce:

- allowed value variants;
- object key uniqueness;
- depth/entry/byte limits;
- canonical number constraints;
- valid strings;
- no runtime-only value kinds.

### `canonicalize_raw_value`

Produce byte-stable canonical JSON/value bytes for hashing and fixtures.

## 5. Restricted evaluator operations

### `restricted_evaluate_fixture`

**Input**

```text
validated profile
validated inventory
EvaluatorPolicy
normalized syntax facts or closed fixture registration records
```

**Output**

```text
RestrictedEvaluationResult
    accepted RawCanonicalRecord[]
    rejected EvaluationDiagnostic[]
    exact partition impacts
```

**Allowed E0 forms**

- literals;
- table constructors;
- local bindings to supported values;
- field access to known local constants;
- one explicit documentation-registration shape;
- bounded expressions named in the fixture contract.

**Rejected forms**

- arbitrary calls;
- loops/recursion;
- function execution;
- IO/environment/network;
- module loading;
- metatables or debug behavior;
- unknown registration target;
- unsupported dynamic expressions.

### `evaluate_registration_record`

Evaluate one proven allow-listed registration call into a raw canonical record. Preserve its source span and registration index.

### `classify_unsupported_construct`

Return a typed diagnostic and exact affected capabilities/partitions. Do not throw away the surrounding file/model when isolation is possible.

## 6. Unknown-field operations

### `preserve_unknown_fields`

Compare a raw record with the current typed-lowering field registry and emit `UnknownField` values for every unmatched field/path.

### `classify_unknown_field_impact`

Classify an unknown field as:

```text
preserved_uninterpreted
preserved_known_safe_projection_gap
preserved_capability_blocking
invalid_field_shape
```

The classifier must identify exact affected capability IDs. A default “ignore” classification is prohibited.

### `round_trip_unknown_fields`

Verify that canonical raw serialization after typed lowering still contains every unknown field/value exactly.

## 7. Lowering operations

### `lower_system_record`

Lower one supported system registration into `NormalizedSystem`.

Requirements:

- canonical namespace/entity identity;
- profile/reference generation match;
- raw record and source handle retained;
- unknown fields retained;
- no child function synthesized without source record;
- duplicate observations returned for classification.

### `lower_function_record`

Lower one supported function record into `NormalizedFunction`.

Requirements:

- canonical `System.Member` identity;
- stable argument/return positions;
- explicit nullable/optional state;
- availability bound to selected profile;
- raw record/source/evidence retained;
- unknown fields retained;
- no inferred alias or replacement.

### `lower_restriction_facets`

Extract normalized restriction facets while preserving raw source metadata.

E0 supports only the fixture `secret.return` shape. Unknown facet/target/predicate forms remain raw and block only the dependent facet capability.

### `validate_lowered_fact`

Ensure the lowered fact refers to existing raw/source/evidence records and one generation.

## 8. Duplicate/conflict operations

### `classify_duplicate_registration`

**Input**

```text
RegistrationObservation[] for one normalized subject
```

**Output**

```text
unique | equivalent_duplicate | incompatible_duplicate
```

Classification uses canonical contract digests, not input order or message text.

### `merge_equivalent_duplicate_provenance`

Create one normalized fact retaining all equivalent evidence/source observations. It must not erase duplicate provenance.

### `build_registration_conflict`

Create a `wow-core ConflictRecord` plus domain metadata for incompatible observations. No normalized winner is emitted for the conflicted contract dimension.

## 9. Coverage operations

### `build_reference_coverage_records`

Emit exact `wow-core CoverageRecord` values for each E0 producer/capability/partition.

Inputs include:

- inventory state;
- evaluator diagnostics;
- lowering results;
- unknown-field impact;
- conflict records;
- variant declarations.

This operation does not produce a whole-operation summary.

### `coverage_for_exact_symbol_lookup`

Select exact coverage records required to decide presence/absence in the queried system/domain.

### `coverage_for_restriction_lookup`

Select exact coverage records required to decide whether restriction facets are present, absent, or unavailable for one subject.

### `evaluate_lookup_negative_authority`

Invoke the `wow-core` negative-authority operation with exact coverage, conflict, generation, and truncation inputs.

Local replacement with `all(status == Complete)` is prohibited.

## 10. Model assembly operations

### `assemble_fixture_reference_model`

**Input**

```text
validated profile/inventory/policy
evaluation result
lowered systems/functions/facets
conflicts
coverage records
source handles/evidence
```

**Output**

```text
FixtureReferenceModel
```

**Checks**

- one profile/reference generation;
- all IDs resolve exactly once;
- no project-source origin/handle;
- raw/lowered links complete;
- conflict dimensions not silently materialized;
- coverage producer/partition identities valid;
- canonical order independent of discovery order;
- canonical model digest correct.

### `validate_reference_model`

Validate an already assembled model. Never repair it silently.

### `canonicalize_fixture_model`

Produce canonical bytes/digest according to [`DATA_MODEL.md`](DATA_MODEL.md) and `wow-core` rules.

### `apply_fixture_variant`

Apply one declared closed variant overlay to the base fixture bundle. Undeclared mutation is rejected.

## 11. Reference view operations

### `open_reference_view`

Create an immutable view only from a validated model.

The view binds:

```text
ProfileIdentity
ReferenceGenerationId
FixtureVariantId
model digest
```

### `reference_view_identity`

Return the exact view/profile/generation/model identity without exposing internal indexes.

### `lookup_symbol_exact`

**Input**

```text
ReferenceView
GenerationContext or expected profile/reference generation
canonical EntityKey
```

**Outcome**

```text
found
authoritative_absent
absent_without_authority
conflict
profile_mismatch
capability_unavailable
```

**Algorithm**

1. verify requested profile/reference generation;
2. validate canonical key/entity kind;
3. select exact symbol-domain coverage records;
4. inspect exact normalized-key index only;
5. inspect conflicts affecting the key/domain;
6. if found and unconflicted for required dimensions, return fact/evidence/coverage;
7. if absent, invoke core negative-authority evaluation;
8. return authoritative or non-authoritative absence accordingly;
9. never search aliases/fuzzy/lineage/external candidates.

### `lookup_restriction_facets`

**Algorithm**

1. verify view/query context;
2. require subject symbol identity;
3. select exact restriction-facet coverage;
4. return normalized facets and raw evidence when unconflicted;
5. return conflict when incompatible observations affect requested facet dimensions;
6. return authoritative none only under complete, unblocked facet coverage;
7. otherwise return unavailable.

### `resolve_reference_source_handle`

Resolve an entity/raw/facet/evidence reference to a registered fixture/reference `SourceHandle`. No filesystem read and no project path.

## 12. Lookup-case operations

### `validate_lookup_case`

Validate expected outcome, entity/facet lists, coverage status, authority state, and conflict codes.

### `execute_lookup_case`

Run a fixture lookup case against the declared variant and compare the entire structured outcome, not only a message/string.

### `canonicalize_lookup_cases`

Sort by case ID and canonicalize all expected values for checksum generation.

## 13. Fixture-bundle operations

### `validate_fixture_bundle`

Validate profile, inventory, policy, records, variants, lookup cases, and checksums as one closed unit.

### `verify_fixture_checksums`

Verify every normative file/member checksum. Drift is a test failure, not an automatic rewrite.

### `fixture_bundle_digest`

Derive the domain-separated SHA-256 identity of canonical fixture content.

## 14. Explicitly unsupported E0 operations

The E0-B public surface must not pretend to implement:

```text
download_reference_source
build_full_reference_pack
open_reference_sqlite
apply_curated_corrections
generate_annotations
build_framexml_graph
lookup_alias
lookup_replacement
lookup_lineage
search_reference_text
query_current_profile
execute_ketho_or_numy
import_runtime_spell_secrecy
```

If exposed by a higher-level long-term interface, they return a typed `operation_not_implemented_for_milestone` state rather than empty success.

## 15. Deterministic operation order

Recommended E0-B pipeline:

```text
validate fixture profile
-> inventory/verify inputs
-> restricted evaluate
-> preserve unknown fields
-> lower systems/functions/facets
-> classify duplicates/conflicts
-> build exact coverage records
-> assemble/validate model
-> open immutable view
-> execute lookup cases
-> canonicalize/checksum
```

Do not interleave lookup with partially assembled model mutation.
