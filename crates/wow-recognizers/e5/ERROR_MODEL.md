# E5-A calibration error model

**Status:** normative typed failure vocabulary.

```text
CalibrationError
    code
    operation/stage
    candidate-source/corpus/example/label/split/pack/run/case/artifact IDs
    repository revision/tree/publication/fact/provenance-group IDs when safe
    rule/mutation/partition/graph-validation/profile IDs
    capability/coverage/conflict/budget/cancellation state
    structured safe message arguments
    recovery class
```

Errors do not include raw source bodies, private host paths, credentials, sealed-holdout labels/results, unrestricted reviewer notes, or arbitrary pack payloads.

## Configuration and dependency boundary

- `calibration_profile_invalid_or_incompatible`
- `calibration_profile_unknown_or_unfrozen`
- `calibration_prerequisite_implementation_or_fixture_unfrozen`
- `calibration_direct_dependency_boundary_violation`
- `calibration_symbolic_current_or_latest_forbidden`
- `calibration_cross_generation_profile_or_universe_mismatch`
- `calibration_operation_not_implemented_for_milestone`

## Candidate source and materialization

- `calibration_candidate_source_invalid`
- `calibration_repository_revision_or_tree_unresolved`
- `calibration_floating_repository_ref_forbidden`
- `calibration_source_inventory_incomplete_or_inconsistent`
- `calibration_source_root_or_exclusion_invalid`
- `calibration_source_materialization_execution_forbidden`
- `calibration_source_publication_or_fact_binding_missing`
- `calibration_source_generation_or_digest_mismatch`
- `calibration_source_handle_or_evidence_unresolved`
- `calibration_candidate_source_not_admission_ready`

## Corpus, provenance, license, and privacy

- `calibration_corpus_manifest_invalid`
- `calibration_corpus_identity_cycle_forbidden`
- `calibration_corpus_member_not_admitted`
- `calibration_corpus_generalization_scope_unsupported`
- `calibration_provenance_group_graph_invalid`
- `calibration_upstream_fork_copy_lineage_unknown`
- `calibration_correlated_examples_counted_as_independent`
- `calibration_license_or_notice_unresolved`
- `calibration_redistribution_class_not_permitted`
- `calibration_privacy_profile_violation`
- `calibration_quarantine_member_used_as_negative`

## Labels and review evidence

- `calibration_label_set_invalid`
- `calibration_label_output_or_graph_type_invalid`
- `calibration_label_evidence_or_decisive_clause_missing`
- `calibration_negative_label_without_complete_closed_scope`
- `calibration_unknown_possible_not_evaluated_conflict_conflation`
- `calibration_label_conflict_unresolved`
- `calibration_label_reviewer_independence_unsatisfied`
- `calibration_label_visibility_policy_violated`
- `calibration_label_copied_from_candidate_output_forbidden`
- `calibration_label_changed_after_observation_without_new_version`
- `calibration_review_authorization_not_implemented_e5_a`

## Split, leakage, and holdout

- `calibration_split_manifest_invalid`
- `calibration_provenance_component_crosses_split`
- `calibration_mutation_family_crosses_split`
- `calibration_repository_fork_copy_vendor_leakage_detected`
- `calibration_chronological_or_generated_template_leakage_detected`
- `calibration_label_or_expected_output_leakage_detected`
- `calibration_recognizer_output_or_model_leakage_detected`
- `calibration_holdout_not_sealed_before_candidate_freeze`
- `calibration_holdout_access_before_freeze_forbidden`
- `calibration_consumed_holdout_reported_as_untouched`
- `calibration_split_changed_without_new_identity`
- `calibration_leakage_analysis_incomplete_or_truncated`

## Pack schema and universal semantics

- `calibration_pack_candidate_invalid`
- `calibration_pack_trust_class_or_rollout_invalid`
- `calibration_pack_e2_schema_validation_failed`
- `calibration_named_metadata_reaches_match_semantics`
- `calibration_repository_addon_owner_path_condition_forbidden`
- `calibration_popularity_split_label_reviewer_condition_forbidden`
- `calibration_model_search_or_external_candidate_condition_forbidden`
- `calibration_convention_literal_unjustified_or_unmutated`
- `calibration_output_not_universal_or_graph_registered`
- `calibration_output_confidence_upgrade_forbidden`
- `calibration_negative_clause_runtime_coverage_bypass_forbidden`
- `calibration_generalization_claim_exceeds_evidence`
- `calibration_core_or_default_rollout_field_forbidden`
- `calibration_pack_identity_or_version_inconsistent`

## Shadow run and graph validation

- `calibration_shadow_run_request_invalid`
- `calibration_fact_snapshot_invalid_or_incompatible`
- `calibration_hidden_holdout_label_used_during_match`
- `calibration_shadow_partition_identity_or_membership_invalid`
- `calibration_shadow_output_marked_core_or_default_forbidden`
- `calibration_match_or_case_result_invalid`
- `calibration_first_last_or_greedy_match_resolution_forbidden`
- `calibration_partial_truncated_cancelled_as_complete_forbidden`
- `calibration_graph_validation_failed_or_missing`
- `calibration_graph_invalid_proposal_hidden_from_report`
- `calibration_run_cancelled`
- `calibration_late_work_after_cancel_forbidden`

## Mutation and anti-overfitting

- `calibration_mutation_suite_invalid`
- `calibration_repository_owner_addon_path_invariance_failed`
- `calibration_irrelevant_local_identifier_invariance_failed`
- `calibration_nonsemantic_prose_or_order_invariance_failed`
- `calibration_decisive_literal_sensitivity_failed`
- `calibration_structural_or_join_sensitivity_failed`
- `calibration_resolution_or_coverage_sensitivity_failed`
- `calibration_near_miss_family_missing_or_false_positive`
- `calibration_named_condition_static_audit_failed`
- `calibration_duplicate_or_metadata_amplification_detected`
- `calibration_adversarial_resource_or_security_gate_failed`

## Metrics, evaluation, and gates

- `calibration_case_result_manifest_invalid`
- `calibration_metric_denominator_or_exclusion_invalid`
- `calibration_per_case_result_hidden_or_missing`
- `calibration_mandatory_failure_hidden_by_weighting`
- `calibration_false_positive_or_false_negative_unresolved`
- `calibration_authority_upgrade_detected`
- `calibration_hard_gate_failed`
- `calibration_threshold_profile_missing_or_unfrozen`
- `calibration_metric_threshold_failed`
- `calibration_baseline_or_run_comparison_incompatible`
- `calibration_test_or_holdout_contamination_unrecorded`
- `calibration_promotion_eligibility_misclassified`
- `calibration_candidate_artifact_incomplete_or_invalid`

## Partition, supersession, and deactivation

- `calibration_shadow_partition_stale_or_incompatible`
- `calibration_foreign_or_core_partition_mutation_forbidden`
- `calibration_stale_candidate_output_retained`
- `calibration_supersession_record_invalid`
- `calibration_deactivation_plan_invalid`
- `calibration_deactivation_reference_closure_incomplete`
- `calibration_deactivation_coverage_downgrade_invalid`
- `calibration_historical_evidence_destroyed_forbidden`

## Security, resources, determinism, and freeze

- `calibration_filesystem_network_process_editor_client_access_forbidden`
- `calibration_source_or_generated_code_execution_forbidden`
- `calibration_executable_plugin_regex_expression_or_template_forbidden`
- `calibration_private_source_path_token_or_note_leak_forbidden`
- `calibration_holdout_secret_or_credential_in_fixture_forbidden`
- `calibration_input_output_resource_limit_exceeded`
- `calibration_provenance_or_mutation_expansion_limit_exceeded`
- `calibration_output_or_metric_amplification_limit_exceeded`
- `calibration_nondeterministic_semantic_output`
- `calibration_checksum_or_member_freeze_incomplete`
- `calibration_benchmark_or_threshold_evidence_missing`
- `calibration_no_rust_or_ci_during_documentation_phase`

## Recovery classes

```text
never
after-profile-or-contract-fix
after-prerequisite-implementation-and-freeze
after-source-materialization-or-publication-fix
after-provenance-license-or-privacy-review
after-label-review-or-versioned-correction
after-split-or-holdout-regeneration
after-pack-or-rule-version-update
after-graph-registry-or-proposal-fix
after-mutation-or-corpus-expansion
after-threshold-or-benchmark-freeze
after-budget-or-scope-reduction
retry-exact-same-immutable-inputs
explicit-Partial-Conflict-or-NotEvaluated-only
quarantine-candidate-or-corpus-member
deferred-to-E5-B-or-E5-C
```

## Error rules

- messages are projections, not identity;
- fatal schema/identity/security errors publish no complete run or candidate artifact;
- expected dynamic/partial/unknown cases remain structured case outcomes when the contract permits;
- cancellation is not failure or completion;
- all errors serialize deterministically with bounded safe arguments.
