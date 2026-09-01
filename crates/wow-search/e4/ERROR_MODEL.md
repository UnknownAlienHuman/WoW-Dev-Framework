# E4-A search error model

**Status:** normative.

```text
SearchError
    code
    operation/stage
    search shard/universe/request/query/lane/result IDs
    owner generation/profile/partition/entity/document/field IDs
    structured message arguments
    capability/coverage/conflict/budget/cancellation state
    recovery class
```

Errors do not include raw source bodies, private paths, credentials, unrestricted query text, SQL, or physical handles.

## Profile and source binding

- `search_profile_invalid_or_incompatible`
- `search_profile_unknown_or_unfrozen`
- `search_owner_binding_invalid`
- `search_owner_generation_or_profile_mismatch`
- `search_required_owner_capability_unavailable`
- `search_universe_set_invalid_or_incompatible`
- `search_required_shard_missing`
- `search_optional_universe_omitted`
- `search_floating_current_or_latest_forbidden`

## Documents and fields

- `search_document_partition_invalid`
- `search_document_owner_entity_unresolved`
- `search_document_cross_generation_or_universe_mismatch`
- `search_document_kind_or_field_not_allowed`
- `search_field_value_invalid_or_over_budget`
- `search_field_origin_or_evidence_unresolved`
- `search_field_coverage_or_conflict_inconsistent`
- `search_explicit_alias_record_invalid`
- `search_forbidden_source_body_or_private_field`
- `search_normalization_or_projection_loss_unrecorded`

## Shard build and publication

- `search_shard_build_request_invalid`
- `search_shard_stale_or_incompatible_base`
- `search_shard_build_plan_invalid`
- `search_shard_partition_membership_incomplete`
- `search_shard_stale_document_or_index_entry`
- `search_shard_physical_materialization_failed`
- `search_shard_validation_failed`
- `search_shard_read_only_reopen_failed`
- `search_shard_unvalidated_or_mutable_open_forbidden`
- `search_shard_corrupt_or_quarantined`
- `search_shard_no_change_misclassified`

## Query and normalization

- `search_request_invalid`
- `search_request_unknown_field_or_profile`
- `search_query_class_or_root_invalid`
- `search_query_text_or_identifier_over_budget`
- `search_query_normalization_failed`
- `search_query_raw_fts_sql_regex_expression_forbidden`
- `search_query_ast_invalid_or_over_budget`
- `search_query_required_lane_unavailable`
- `search_query_plan_invalid`
- `search_query_privacy_or_license_denied`

## Lane execution

- `search_exact_lane_failed_or_ambiguous`
- `search_alias_lane_conflict_or_incomplete`
- `search_prefix_expansion_budget_exceeded`
- `search_text_lane_unavailable_or_failed`
- `search_fts_runtime_or_profile_mismatch`
- `search_fts_integrity_or_mapping_failed`
- `search_identifier_similarity_failed_or_over_budget`
- `search_shape_lane_unsupported_or_partial`
- `search_graph_lane_snapshot_or_profile_mismatch`
- `search_graph_lane_truncated_or_failed`
- `search_lane_cancelled`
- `search_lane_partial_state_hidden_forbidden`

## Ranking and explanation

- `search_candidate_signal_invalid`
- `search_candidate_cross_universe_merge_forbidden`
- `search_candidate_authority_upgrade_forbidden`
- `search_raw_cross_shard_fts_comparison_forbidden`
- `search_ranking_profile_or_feature_invalid`
- `search_ranking_overflow_or_nondeterminism`
- `search_ranking_tie_key_invalid`
- `search_candidate_explanation_incomplete_or_inconsistent`
- `search_lineage_replacement_migration_impact_claim_forbidden`

## Miss, result, and continuation

- `search_miss_classification_invalid`
- `search_exact_negative_authority_unavailable`
- `search_approximate_empty_as_absence_forbidden`
- `search_result_set_manifest_invalid`
- `search_result_candidate_cap_or_page_state_invalid`
- `search_result_validation_failed`
- `search_result_minimum_explanation_exceeds_budget`
- `search_continuation_invalid_or_tampered`
- `search_continuation_generation_or_profile_mismatch`
- `search_continuation_result_set_unavailable`
- `search_continuation_budget_reset_forbidden`
- `search_continuation_cancelled`

## Store and security

- `search_store_schema_or_operation_invalid`
- `search_raw_storage_sql_connection_or_extension_forbidden`
- `search_external_database_or_path_forbidden`
- `search_source_or_executable_input_forbidden`
- `search_network_process_editor_client_model_access_forbidden`
- `search_private_data_or_source_leak_forbidden`
- `search_input_output_resource_limit_exceeded`
- `search_physical_artifact_integrity_failed`

## Evaluation and freeze

- `search_evaluation_corpus_or_label_invalid`
- `search_evaluation_hard_gate_failed`
- `search_evaluation_threshold_not_frozen`
- `search_false_authority_detected`
- `search_determinism_or_golden_mismatch`
- `search_prerequisite_implementation_or_fixture_unfrozen`
- `search_sqlite_fts_tokenizer_probe_missing`
- `search_checksum_or_member_freeze_incomplete`

## Recovery classes

```text
never
after-profile-contract-fix
after-owner-generation-or-coverage-fix
after-document-projection-fix
after-shard-rebuild
after-store-or-physical-profile-fix
after-query-reduction
after-privacy-or-license-policy-change
retry-exact-same-shards-and-request
continue-from-exact-retained-result-set
after-evaluation-or-freeze-completion
safe-quarantine-or-discard-derived-shard
```
