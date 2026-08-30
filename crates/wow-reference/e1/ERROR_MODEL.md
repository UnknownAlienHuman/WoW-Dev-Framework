# `wow-reference` E1-B error model

**Status:** normative typed snapshot/profile/parser/evaluator/raw/normalization/correction/coverage/store/build/view failure vocabulary.

Human prose is a projection. Tests/callers inspect exact codes, profile/reference/snapshot/partition/file/entity/raw/correction/capability/store/query IDs, stage, and recovery class.

## 1. Error shape

```text
ReferenceError
    code
    stage/operation
    build request/candidate/report IDs
    ProfileId / ReferenceGenerationId
    source snapshot/partition/file/parser/evaluator IDs
    entity/raw observation/field path/fact IDs
    correction/set/application/conflict IDs
    capability/coverage/NotEvaluated IDs
    schema/operation/build plan/store generation IDs
    ReferenceView/query/result IDs
    budget/cancellation state
    message arguments
    recovery class
```

Recovery classes:

```text
never
after_request_or_profile_fix
after_source_snapshot_or_manifest_fix
after_parser_evaluator_or_normalizer_fix
after_correction_review
after_schema_or_store_fix
after_rebuild_reference_generation
after_capability_or_runtime_evidence_available
after_contract_or_implementation_fix
retry_exact_same_inputs
```

Public errors exclude absolute roots, raw SQL, full source/raw payloads, tokens/private URLs, runtime Secret values.

## 2. Snapshot/profile errors

### `reference_build_request_invalid`

Missing/contradictory snapshot/profile/policy/parser/correction/schema/store/budget inputs.

### `source_snapshot_manifest_invalid`

Malformed file/provider/content/license/partition manifest.

### `source_snapshot_root_invalid`

Configured root missing/unsafe/inaccessible/incompatible.

### `source_snapshot_path_escape_forbidden`

Absolute/traversal/device/unsafe link/reparse/case collision/root escape.

### `source_file_missing_or_unexpected`

Required file missing or unexpected/extra file violates policy.

### `source_file_digest_or_length_mismatch`

Actual bytes disagree with manifest.

### `source_file_encoding_invalid`

Unsupported/invalid BOM/encoding/line-ending contract.

### `source_input_order_invalid`

Declared semantic order absent/contradictory/nondeterministic.

### `provider_provenance_invalid`

Malformed/inconsistent provider/acquisition/license evidence.

### `reference_profile_invalid`

Flavor/Interface/build/source/parser/schema/correction/eligibility contradiction.

### `reference_profile_floating_or_implicit_forbidden`

Current/latest/provider HEAD/local installation/last-used implicit selection.

### `reference_profile_mismatch`

Input/store/view/query profile/generation mismatch.

### `reference_fixture_release_masquerade_forbidden`

Fixture/incomplete profile labeled release.

### `reference_cross_profile_generation_mix_forbidden`

Record/link/store/query combines another profile/reference generation.

## 3. Parser/evaluator errors

### `reference_parser_unpinned_or_unprobed`

No accepted exact parser compatibility report.

### `reference_parser_profile_invalid`

Parser version/features/dialect/span/numeric/string/recovery behavior mismatch.

### `reference_parse_failed`

Fatal file parse failure.

### `reference_parse_recovery_node_unsupported`

Recovered syntax occurs in value/registration and cannot safely emit fact.

### `reference_evaluator_environment_invalid`

Unknown/malformed binding/constant/registration/helper bundle.

### `reference_unsupported_statement`
### `reference_unsupported_expression`
### `reference_unknown_global`
### `reference_unknown_call`
### `reference_unsupported_helper`
### `reference_unsupported_operator`
### `reference_unsupported_table_key`
### `reference_binding_use_before_definition`
### `reference_binding_cycle`
### `reference_binding_invalidated`

Produce `UnsupportedConstructRecord` and coverage impact, not guessed value.

### `reference_registration_call_invalid`

Unknown/wrong receiver/callee/arity/value shape/order for registration descriptor.

### `reference_numeric_value_unrepresentable`

Cannot losslessly lower under canonical numeric policy.

### `reference_string_value_invalid`

Invalid escape/encoding/length/representation.

### `reference_evaluator_budget_exceeded`

Steps/depth/table/string/binding/registration/output limit.

### `reference_source_execution_forbidden`

Any attempt to execute Lua/helper/global/source/repository/client/editor/network/process side effect.

### `reference_evaluator_nondeterministic`

Equivalent source/environment/policy produces different values/observations/unsupported records.

## 4. Raw/normalization errors

### `raw_canonical_value_invalid`

Malformed/cyclic/disallowed/unbounded raw value tree.

### `raw_observation_invalid`

Missing source/registration/value/field path/profile/producer/context.

### `raw_observation_loss_forbidden`

Source field/value/duplicate/unknown/unsupported observation dropped or overwritten.

### `raw_field_path_invalid`

Ambiguous/malformed/unbounded field path.

### `unknown_as_absent_or_default_forbidden`

Unknown/unsupported coerced to missing/null/default.

### `reference_field_registry_invalid`

Known field/entity path/value/normalizer/capability/default mapping malformed/conflicting.

### `reference_entity_candidate_invalid`

Cannot classify exact supported entity shape.

### `reference_entity_identity_invalid`

Missing/contradictory kind/name/system/owner/signature/profile identity.

### `reference_entity_identity_collision`

Distinct normalized facts produce same unique key without exact duplicate/conflict policy.

### `reference_normalized_fact_invalid`

Fact/member/type/restriction/predicate/deprecation/transition invariant failure.

### `reference_member_order_invalid`

Ordinal/order/owner/cardinality inconsistency.

### `reference_type_reference_unresolved`

Exact type/entity target unavailable; result remains unresolved/partial rather than silent `any`.

### `reference_link_ambiguous_or_conflicted`

Multiple exact candidates/competing sources.

### `reference_fuzzy_identity_or_replacement_forbidden`

Name/text/similarity used for identity/alias/replacement.

### `reference_restriction_projection_invalid`

Facet target/payload/version/predicate/applicability malformed.

### `reference_runtime_security_claim_forbidden`

Static source fact generalized to current runtime/spell safety/whitelist.

### `reference_raw_projection_closure_invalid`

Normalized fact lacks raw/source/evidence/derivation closure.

### `reference_normalization_nondeterministic`

Equivalent raw input/registry/policy yields different facts/manifests.

## 5. Correction errors

### `correction_record_invalid`

Missing target/field/expected digest/replacement/evidence/reviewer/applicability/version.

### `correction_set_invalid`

Malformed set/dependency/profile/digest/order.

### `correction_dependency_cycle`

Cycle in correction graph.

### `correction_target_not_found`

Exact target/raw observation/field path unavailable.

### `correction_expected_digest_mismatch`

Source/value/shape/normalizer digest changed; status Expired.

### `correction_profile_not_applicable`

Wrong profile/build; status NotApplicable.

### `correction_operation_unsupported`

Wildcard/fuzzy/code/SQL/runtime whitelist/raw delete/etc.; Rejected.

### `correction_replacement_invalid`

Replacement violates field/entity/type/link/restriction identity.

### `correction_conflict`

Competing corrections/source evidence/identity collision.

### `correction_raw_mutation_forbidden`

Raw source/observation deleted/rewritten.

### `correction_best_effort_or_auto_digest_update_forbidden`

Mismatch automatically widened/updated/applied.

### `correction_application_record_invalid`

Before/after/status/evidence/coverage/application closure invalid.

## 6. Coverage/authority errors

### `reference_capability_registry_invalid`
### `reference_coverage_partition_invalid`
### `reference_coverage_dependency_invalid`
### `reference_coverage_record_invalid`
### `reference_coverage_summary_invalid`

Malformed/missing/nonconservative capability/partition/status/producer/generation relations.

### `reference_unknown_or_unsupported_blocker_missing`

Dependent capability marked Complete without required blocker relation.

### `reference_conflict_record_invalid`

Competing record set/subject/capability/resolution invalid.

### `reference_complete_overrides_conflict_forbidden`

Complete ingestion used to ignore unresolved conflict.

### `reference_negative_authority_unavailable`

Relevant conditions not all satisfied; absence cannot be asserted.

### `reference_empty_result_as_absence_forbidden`

Null/empty lookup/list interpreted as authoritative without decision.

### `reference_negative_authority_scope_invalid`

Claim broader/ambiguous/wrong kind/system/profile/runtime semantics.

### `reference_not_evaluated_as_clean_forbidden`

Blocked/unavailable operation treated as pass/absence/safety.

### `reference_truncation_as_complete_forbidden`

Omitted inputs/results under budget treated complete.

### `reference_runtime_or_hotfix_gap`

Requested current behavior requires runtime/freshness evidence absent from static generation.

### `reference_release_eligibility_invalid`

Declared release capabilities/gates do not match actual identity/coverage/conflict/correction/store state.

## 7. Schema/store/build errors

### `reference_schema_bundle_invalid`
### `reference_operation_catalog_invalid`
### `reference_validation_catalog_invalid`

Domain schema/operations/validation incompatible/malformed/digest mismatch.

### `reference_raw_sql_or_store_connection_bypass_forbidden`

Raw SQL/connection/store internals exposed or used outside registered seam.

### `reference_store_plan_invalid`

Operation/object/validation order/records/digests/dependencies/budgets/context invalid.

### `reference_store_plan_nondeterministic`

Equivalent facts produce different plan/order/digest.

### `reference_store_build_failed`

`wow-store` candidate/migration/write/validation/seal/publication failed.

### `reference_store_context_mismatch`

Returned StoreGeneration/manifest/profile/reference/schema/runtime does not match build.

### `reference_store_persisted_closure_invalid`

Raw/fact/member/correction/conflict/coverage/object/count/digest rows incomplete/orphan/cross-generation.

### `reference_store_in_place_mutation_forbidden`

Attempt to update/migrate/correct sealed ReferenceStore.

### `reference_store_absence_as_authority_forbidden`

Missing row used directly as authority.

### `reference_store_open_validation_failed`

Published read-only store/view cannot validate.

### `reference_build_stage_invalid`

State skipped/reordered/inconsistent.

### `reference_build_cancelled`

No completed ReferenceData publication/manifest; no background continuation.

### `reference_build_late_work_after_cancel_forbidden`

Late parse/write/publication/report after cancellation.

### `reference_build_report_invalid`
### `reference_data_manifest_invalid`
### `reference_data_manifest_digest_mismatch`

Input/stage/count/coverage/store/object/license/checksum/reference closure invalid.

### `reference_physical_store_reproducibility_overclaim`

Raw SQLite bytes claimed deterministic without store proof.

## 8. ReferenceView/query errors

### `reference_view_open_request_invalid`
### `reference_view_invalid`
### `reference_view_profile_or_generation_mismatch`
### `reference_view_reader_generation_switch_forbidden`

Exact immutable context/open validation failure.

### `reference_lookup_request_invalid`

Malformed/ambiguous/string-only wrong entity/member key/detail/budget.

### `reference_lookup_cardinality_conflict`

Multiple nonidentical rows for unique exact key.

### `reference_lookup_result_invalid`

Result variant/context/fact/evidence/coverage/conflict/truncation/reference closure invalid.

### `reference_lookup_hidden_fallback_forbidden`

Other kind/name/system/profile/generation/external/fuzzy fallback.

### `reference_lookup_fuzzy_or_search_forbidden`

FTS/trigram/semantic/similarity lane.

### `reference_raw_metadata_request_invalid`
### `reference_raw_metadata_budget_exceeded`

Invalid/unbounded raw detail request.

### `reference_source_detail_unavailable_or_forbidden`

Source object/license/budget/security does not permit body; stable handle can remain.

### `reference_query_budget_exceeded`
### `reference_query_cancelled`

No misleading absence/complete result.

### `reference_query_nondeterministic`

Equivalent view/request yields different result/order/digest.

## 9. Security/privacy errors

### `reference_network_or_source_acquisition_forbidden`
### `reference_filesystem_process_shell_editor_client_access_forbidden`
### `reference_untrusted_instruction_ignored`
### `reference_input_path_escape_forbidden`
### `reference_private_path_or_payload_leak_forbidden`
### `reference_unbounded_input_or_output_forbidden`
### `reference_corrupt_or_untrusted_store_or_object`

Security violations block build/open/query as applicable; never blind retry.

## 10. Deferred capability errors

### `reference_annotations_not_implemented_e1_b`
### `reference_full_ui_graph_not_implemented_e1_b`
### `reference_search_or_lineage_not_implemented_e1_b`
### `reference_runtime_probe_not_implemented_e1_b`
### `reference_release_assembly_not_implemented_e1_b`

Externally can map to `operation_not_implemented_for_milestone` with structured details. No empty success.

## 11. Fatal versus partition-local

### Fatal build/open

```text
snapshot/profile/root/parser/evaluator policy invalid
raw/identity/schema/store/manifest/reference closure invalid
security violation
store publication/open mismatch
```

### Partition-local candidate degradation

```text
one optional/independent file/field/helper/unknown/conflict/correction issue
```

Retain useful facts with exact Partial/Failed/NotEvaluated coverage; release eligibility/authority scoped accordingly.

### Query-local

Invalid key/detail/budget/cancellation; immutable view unaffected.

## 12. Recovery

- snapshot/profile -> fix materialized input/manifest/request;
- parser/evaluator/normalizer -> fix/probe/upgrade and rebuild new generation;
- unknown/unsupported -> add reviewed support/new version/rebuild;
- correction -> review/update/remove/new correction set/rebuild;
- conflict -> source/correction/normalizer resolution/new generation;
- store -> fix schema/plan/store layer and rebuild candidate; old active unchanged;
- view -> open exact valid generation;
- runtime gap -> later exact runtime evidence/probe, never static guess;
- security -> never blind retry.

## 13. Error tests

Every E1-used code requires:

- exact stage/profile/source/entity/capability/store/query IDs;
- dedicated fixture/mutation;
- coverage/release/authority impact assertion;
- raw source preservation where applicable;
- no completed/active manifest after fatal/cancel;
- deterministic serialization;
- no private/raw payload/path/SQL leak;
- recovery class;
- no annotation/search/runtime/source execution escape.
