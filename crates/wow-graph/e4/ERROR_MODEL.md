# E4-B lineage error model

**Status:** normative.

```text
LineageError
    code
    operation/stage
    universe/comparison/profile/snapshot IDs
    optional producer/partition/entity/proposal/component/assertion/change/migration/impact IDs
    structured safe arguments
    capability/coverage/conflict/budget/cancellation state
    recovery class
```

Errors never expose raw SQL, source bodies, unrestricted review/search text, private paths, credentials or mutable storage handles.

## Profile and universe errors

- `lineage_profile_set_invalid`
- `lineage_profile_unknown_or_incompatible`
- `lineage_relation_registry_invalid`
- `lineage_producer_schema_invalid`
- `lineage_proof_ceiling_profile_invalid`
- `lineage_review_profile_invalid`
- `lineage_change_profile_invalid`
- `lineage_migration_profile_invalid`
- `lineage_impact_profile_invalid`
- `lineage_universe_set_invalid`
- `lineage_universe_class_mismatch`
- `lineage_before_after_generation_invalid`
- `lineage_generation_or_snapshot_mismatch`
- `lineage_comparison_profile_incompatible`
- `lineage_current_or_latest_selector_forbidden`
- `lineage_cross_universe_relation_forbidden`
- `lineage_generation_local_identity_mutation_forbidden`

## Producer/input errors

- `lineage_input_partition_invalid`
- `lineage_input_partition_stale_base`
- `lineage_input_partition_generation_mismatch`
- `lineage_input_partition_incomplete`
- `lineage_input_entity_unresolved`
- `lineage_input_evidence_or_source_unresolved`
- `lineage_input_coverage_or_conflict_invalid`
- `lineage_input_producer_class_forbidden`
- `lineage_input_proof_ceiling_exceeded`
- `lineage_fingerprint_profile_or_feature_invalid`
- `lineage_fingerprint_candidate_authority_upgrade_forbidden`
- `lineage_search_candidate_generation_mismatch`
- `lineage_search_candidate_authority_upgrade_forbidden`
- `lineage_reference_transition_scope_invalid`
- `lineage_reference_transition_conflicted_or_incomplete`

## Candidate and ambiguity errors

- `lineage_blocking_profile_invalid`
- `lineage_blocking_scope_or_key_invalid`
- `lineage_blocking_bucket_budget_exceeded`
- `lineage_all_pairs_generation_forbidden`
- `lineage_candidate_pair_invalid`
- `lineage_candidate_pair_budget_exceeded`
- `lineage_proposal_invalid`
- `lineage_proposal_relation_or_endpoint_invalid`
- `lineage_proposal_evidence_or_ceiling_invalid`
- `lineage_candidate_component_invalid`
- `lineage_candidate_component_budget_exceeded`
- `lineage_ambiguity_hidden_or_collapsed`
- `lineage_greedy_or_score_only_assignment_forbidden`
- `lineage_unique_candidate_auto_promotion_forbidden`
- `lineage_one_to_many_or_many_to_one_forced_bijection`
- `lineage_copy_move_classification_conflict`
- `lineage_split_merge_evidence_insufficient`
- `lineage_candidate_generation_truncated`

## Review and conflict errors

- `lineage_review_decision_invalid`
- `lineage_review_target_mismatch`
- `lineage_review_attestation_or_authority_invalid`
- `lineage_review_proof_ceiling_exceeded`
- `lineage_review_candidate_only_promotion_forbidden`
- `lineage_review_missing_required_independent_evidence`
- `lineage_review_note_or_payload_forbidden`
- `lineage_review_predecessor_or_supersession_invalid`
- `lineage_review_decision_conflict`
- `lineage_conflict_record_invalid`
- `lineage_stable_identity_collision`
- `lineage_multiplicity_or_exclusivity_conflict`
- `lineage_last_write_majority_or_popularity_resolution_forbidden`
- `lineage_rejected_or_deferred_proposal_lost`

## Assertion/publication errors

- `lineage_assertion_invalid`
- `lineage_assertion_relation_or_scope_invalid`
- `lineage_assertion_proposal_or_evidence_closure_failed`
- `lineage_assertion_proof_ceiling_violation`
- `lineage_assertion_ambiguity_or_conflict_blocked`
- `lineage_assertion_cross_generation_entity_merge_forbidden`
- `lineage_publication_request_invalid`
- `lineage_publication_stale_base`
- `lineage_publication_plan_invalid`
- `lineage_publication_partial_commit_forbidden`
- `lineage_snapshot_manifest_invalid`
- `lineage_snapshot_publication_failed`
- `lineage_snapshot_post_open_validation_failed`
- `lineage_snapshot_not_immutable_or_read_only`
- `lineage_prior_snapshot_modified_or_relabelled`
- `lineage_store_logical_integrity_failed`
- `lineage_store_index_or_reference_closure_failed`

## Change and absence errors

- `lineage_change_pair_not_accepted`
- `lineage_change_record_invalid`
- `lineage_change_field_or_relation_profile_invalid`
- `lineage_change_value_state_collapsed`
- `lineage_change_before_after_origin_unresolved`
- `lineage_change_coverage_incomplete`
- `lineage_change_relation_path_as_direct_edge_forbidden`
- `lineage_change_authority_class_collapsed`
- `lineage_removal_authority_unavailable`
- `lineage_introduction_authority_unavailable`
- `lineage_removal_or_introduction_conflict`
- `lineage_unmatched_promoted_to_absence_forbidden`
- `lineage_empty_search_or_graph_as_absence_forbidden`
- `lineage_no_new_evidence_as_absence_forbidden`

## Replacement and migration errors

- `lineage_replacement_relation_invalid`
- `lineage_replacement_evidence_insufficient`
- `lineage_replacement_inferred_from_lineage_forbidden`
- `lineage_replacement_inferred_from_similarity_forbidden`
- `lineage_deprecation_scope_or_target_invalid`
- `lineage_migration_candidate_invalid`
- `lineage_migration_candidate_promoted_to_recipe_forbidden`
- `lineage_migration_recipe_invalid`
- `lineage_migration_source_or_target_scope_invalid`
- `lineage_migration_precondition_incomplete`
- `lineage_migration_transformation_step_invalid`
- `lineage_migration_executable_payload_forbidden`
- `lineage_migration_postcondition_or_validation_incomplete`
- `lineage_migration_proof_ceiling_or_coverage_invalid`
- `lineage_migration_runtime_or_edit_success_claim_forbidden`

## Static-impact errors

- `lineage_impact_request_invalid`
- `lineage_impact_root_invalid_or_unresolved`
- `lineage_impact_snapshot_or_generation_mismatch`
- `lineage_impact_relation_axis_or_direction_forbidden`
- `lineage_impact_path_invalid`
- `lineage_impact_path_as_direct_edge_forbidden`
- `lineage_impact_confidence_or_proof_upgrade_forbidden`
- `lineage_impact_cross_universe_bridge_invalid`
- `lineage_impact_budget_exceeded`
- `lineage_impact_truncated`
- `lineage_impact_coverage_or_conflict_blocked`
- `lineage_impact_runtime_breakage_severity_safety_claim_forbidden`
- `lineage_impact_unbounded_traversal_forbidden`

## Query and continuation errors

- `lineage_query_invalid`
- `lineage_query_snapshot_or_generation_mismatch`
- `lineage_query_relation_or_change_filter_invalid`
- `lineage_query_fuzzy_or_natural_language_root_forbidden`
- `lineage_query_budget_invalid_or_exceeded`
- `lineage_query_cancelled`
- `lineage_query_not_evaluated`
- `lineage_query_unbounded_export_forbidden`
- `lineage_continuation_invalid`
- `lineage_continuation_integrity_failed`
- `lineage_continuation_snapshot_or_generation_mismatch`
- `lineage_continuation_request_or_profile_mismatch`
- `lineage_continuation_result_manifest_mismatch`
- `lineage_continuation_budget_reset_forbidden`
- `lineage_continuation_retained_input_unavailable`
- `lineage_continuation_background_work_forbidden`

## Security/privacy/store errors

- `lineage_source_or_generated_code_execution_forbidden`
- `lineage_filesystem_network_process_editor_client_access_forbidden`
- `lineage_raw_sql_store_connection_extension_or_vfs_forbidden`
- `lineage_source_controlled_schema_profile_or_proof_rule_forbidden`
- `lineage_model_embedding_or_external_authority_forbidden`
- `lineage_prompt_or_instruction_text_used_as_control_forbidden`
- `lineage_private_path_credential_or_source_leak_forbidden`
- `lineage_privacy_or_license_scope_invalid`
- `lineage_cross_consumer_private_artifact_reuse_forbidden`
- `lineage_input_resource_limit_exceeded`
- `lineage_output_resource_limit_exceeded`
- `lineage_artifact_or_cursor_corrupt`

## Evaluation and freeze errors

- `lineage_evaluation_corpus_missing_or_invalid`
- `lineage_evaluation_ground_truth_incomplete`
- `lineage_evaluation_false_authority_detected`
- `lineage_evaluation_threshold_not_met`
- `lineage_evaluation_unknown_or_partial_as_pass_forbidden`
- `lineage_benchmark_missing_or_failed`
- `lineage_prerequisite_implementation_or_fixture_missing`
- `lineage_checksum_manifest_incomplete`
- `lineage_fixture_digest_mismatch`
- `lineage_first_rust_commit_before_freeze_forbidden`

## Cancellation/internal errors

- `lineage_operation_cancelled`
- `lineage_operation_deadline_or_budget_exceeded`
- `lineage_internal_invariant_failed`
- `lineage_serialization_or_canonicalization_failed`
- `lineage_operation_unsupported`

## Recovery classes

```text
never
after-request-or-profile-correction
after-owner-generation-or-coverage-fix
after-producer-partition-fix
after-reference-transition-fix
after-review-or-conflict-resolution
after-lineage-store-rebuild
retry-same-exact-inputs-with-smaller-explicit-budget
continue-with-exact-cursor
reopen-exact-retained-generations
downgrade-to-candidate-or-partial-only
quarantine-and-rebuild
implementation-or-freeze-required
```

## Payload states versus errors

Valid fully executed payload states include:

```text
Complete
Partial
CandidateOnly
ConflictBlocked
NotEvaluated
Truncated
NoChange
InvalidValidationResult
Cancelled
Failed
```

A validator proving a snapshot/recipe invalid returns a typed `Invalid` payload. Failure to execute validation is an error. No payload status hides a failed mandatory producer, unresolved conflict, truncated candidate component or unavailable negative authority.
