# E3-A context error model

**Status:** normative structured failure vocabulary.

```text
ContextError
    code
    operation/stage
    request/plan/artifact/bundle/envelope IDs
    epoch/store/publication/project/graph/analyzer/reference generation IDs
    profile/root/entity/relation/path/source/evidence/loss/frontier/cursor IDs
    bounded structured arguments
    budget/cancellation state
    recovery class
```

Public errors omit raw source beyond safe handles, private paths, credentials, cursor bytes, store/analyzer internals, and runtime-sensitive payloads.

## Recovery classes

```text
never
after-request-or-profile-fix
after-exact-root-or-generation-fix
after-input-capability-or-conflict-fix
after-source-license-security-fix
start-new-request-for-new-publication
after-budget-increase-in-new-request
continue-with-valid-cursor
query-with-smaller-explicit-scope
after-implementation-or-contract-fix
```

## Configuration/dependencies

- `context_configuration_invalid`
- `context_profile_invalid_or_incompatible`
- `context_project_map_profile_invalid`
- `context_skeleton_or_control_effect_profile_invalid`
- `context_expansion_or_continuation_profile_invalid`
- `context_source_excerpt_profile_invalid`
- `context_budget_or_tokenizer_profile_invalid`
- `context_security_renderer_or_evaluation_profile_invalid`
- `context_dependency_boundary_violation`
- `context_search_store_or_analyzer_internal_dependency_forbidden`
- `context_model_correctness_dependency_forbidden`

## Input snapshot/roots/universes

- `context_request_invalid`
- `context_input_snapshot_invalid`
- `context_store_epoch_generation_or_publication_set_mismatch`
- `context_publication_project_graph_analyzer_generation_mismatch`
- `context_reference_profile_or_generation_mismatch`
- `context_store_image_identity_forbidden`
- `context_current_or_latest_selector_forbidden_after_acquisition`
- `context_root_invalid_or_unresolved`
- `context_root_universe_or_generation_mismatch`
- `context_exact_root_required_search_deferred`
- `context_required_query_catalog_or_capability_unavailable`
- `context_stale_or_mixed_input_record`
- `context_candidate_external_or_runtime_universe_merge_forbidden`
- `context_cross_universe_name_or_path_join_forbidden`
- `context_platform_source_producer_or_manifest_unavailable`

## Project Map

- `context_plan_or_project_map_invalid`
- `context_project_map_section_invalid`
- `context_project_map_principal_root_unproven`
- `context_project_map_variant_or_universe_merge_forbidden`
- `context_project_map_static_runtime_claim_forbidden`
- `context_project_map_signal_system_collapse_forbidden`
- `context_project_map_savedvariables_content_forbidden`
- `context_project_map_missing_mandatory_blocker_or_route`
- `context_project_map_bulk_export_forbidden`
- `context_project_map_repository_name_or_model_condition_forbidden`

## Skeleton/control/effects

- `context_skeleton_invalid_or_unsupported`
- `context_skeleton_identity_nondeterministic`
- `context_skeleton_field_without_input_or_derivation`
- `context_skeleton_member_or_source_node_invalid`
- `context_skeleton_signature_semantics_invalid`
- `context_control_effect_node_invalid`
- `context_control_tree_unproven`
- `context_second_parser_cfg_ssa_or_dataflow_forbidden`
- `context_analyzer_internal_type_leak_forbidden`
- `context_generated_code_or_source_reconstruction_forbidden`
- `context_documentation_derived_semantics_forbidden`
- `context_direct_edge_from_path_forbidden`
- `context_generic_parent_or_related_collapse_forbidden`
- `context_confidence_coverage_or_runtime_safety_upgrade_forbidden`
- `context_unknown_collapsed_or_omitted_region_unreported`
- `context_deduplication_evidence_or_occurrence_loss`

## Expansion/stopping/continuation

- `context_expansion_request_or_frontier_invalid`
- `context_expansion_lane_or_direction_invalid`
- `context_expansion_scope_confidence_or_generation_broadening_forbidden`
- `context_expansion_query_snapshot_or_result_invalid`
- `context_expansion_cycle_or_path_state_invalid`
- `context_expansion_possible_or_candidate_policy_violation`
- `context_expansion_optional_branch_starved_mandatory_budget`
- `context_expansion_unbounded_traversal_or_export_forbidden`
- `context_expansion_model_priority_forbidden`
- `context_no_new_evidence_misclassified_as_absence`
- `context_stopping_reason_or_scope_invalid`
- `context_requested_complete_unproven`
- `context_continuation_invalid_or_tampered`
- `context_continuation_snapshot_request_profile_or_ordering_mismatch`
- `context_continuation_frontier_or_visited_digest_mismatch`
- `context_continuation_budget_reset_or_scope_change_forbidden`
- `context_continuation_expired_input_unavailable`
- `context_continuation_after_complete_forbidden`
- `context_cancelled`
- `context_late_work_or_background_continuation_forbidden`

## Evidence/coverage/loss

- `context_evidence_link_invalid`
- `context_material_claim_without_evidence_or_derivation`
- `context_evidence_generation_or_source_mismatch`
- `context_provenance_or_confidence_upgrade_forbidden`
- `context_coverage_axis_collapse_forbidden`
- `context_projection_or_coverage_status_invalid`
- `context_loss_omission_or_stopping_record_invalid_or_missing`
- `context_conflict_silently_resolved_forbidden`
- `context_partial_not_evaluated_or_truncation_hidden_forbidden`
- `context_empty_or_omitted_as_authoritative_absence_forbidden`
- `context_loss_report_truncated_as_complete_forbidden`
- `context_artifact_eligibility_invalid`

## Source/security

- `context_source_excerpt_request_invalid`
- `context_source_handle_invalid_or_stale`
- `context_source_generation_digest_or_span_mismatch`
- `context_source_origin_role_license_privacy_or_security_forbidden`
- `context_source_object_missing_or_unreferenced`
- `context_source_excerpt_unfaithful_or_reconstructed`
- `context_source_excerpt_budget_exceeded`
- `context_source_prompt_instruction_treated_as_policy_forbidden`
- `context_source_container_directive_html_json_or_terminal_injection_forbidden`
- `context_source_private_path_credential_or_payload_leak_forbidden`
- `context_savedvariables_log_client_memory_or_runtime_payload_access_forbidden`
- `context_filesystem_network_process_shell_editor_client_access_forbidden`
- `context_source_mutation_or_code_execution_forbidden`

## Budget/tokenizer

- `context_budget_spec_or_override_invalid`
- `context_mandatory_reserve_unavailable`
- `context_budget_accounting_mismatch_or_exceeded`
- `context_atomic_record_utf8_or_source_boundary_violation`
- `context_lane_fairness_or_priority_invalid`
- `context_report_budget_hides_blocker_forbidden`
- `context_tokenizer_exact_count_without_pin_forbidden`
- `context_tokenizer_input_digest_config_or_result_mismatch`
- `context_token_estimate_unlabeled_as_exact_forbidden`
- `context_token_budget_renderer_subject_mismatch`

## Bundle/rendering/canonicalization

- `context_bundle_core_invalid`
- `context_bundle_input_profile_or_record_closure_invalid`
- `context_bundle_status_or_digest_invalid`
- `context_artifact_identity_cycle_forbidden`
- `context_renderer_profile_or_schema_invalid`
- `context_renderer_output_semantic_mismatch`
- `context_renderer_loss_or_sidecar_unreported`
- `context_renderer_injection_or_container_escape`
- `context_canonical_order_or_bytes_invalid`
- `context_volatile_field_in_canonical_identity`
- `context_artifact_nondeterministic`
- `context_envelope_or_metric_backreference_cycle_forbidden`

## Metrics/evaluation

- `context_metrics_invalid_or_inconsistent`
- `context_metric_subject_or_denominator_mismatch`
- `context_mandatory_recall_or_evidence_closure_incomplete`
- `context_forbidden_or_invented_record_present`
- `context_relevance_or_redundancy_policy_invalid`
- `context_false_deduplication_detected`
- `context_compression_claim_invalid_or_misleading`
- `context_budget_or_continuation_equivalence_failed`
- `context_source_faithfulness_or_security_evaluation_failed`
- `context_consumer_utility_protocol_invalid`
- `context_external_model_unpinned_or_authoritative_forbidden`
- `context_evaluation_corpus_mutation_forbidden`
- `context_evaluation_hard_gate_failed`
- `context_performance_claim_unscoped_or_unverified`
- `context_evaluation_report_nondeterministic`

## Deferred capabilities

- `context_search_not_implemented_e3_a`
- `context_blizzard_source_extraction_not_implemented_e3_a`
- `context_lineage_or_patch_impact_not_implemented_e3_a`
- `context_full_source_or_bulk_graph_export_not_implemented_e3_a`
- `context_runtime_or_client_probe_not_implemented_e3_a`
- `context_code_generation_or_autofix_not_implemented_e3_a`
- `context_lsp_mcp_network_cache_or_persistence_not_implemented_e3_a`
- `context_operation_not_implemented_for_milestone`

## Fatal versus scoped

Fatal request/bundle failures include mixed identities, invalid profiles/roots, evidence or identity DAG violations, private/security/injection breaches, canonical/determinism failure, and invalid cursor integrity.

Root/lane/field-scoped states include unsupported detail, partial/conflicted capability, source unavailable/forbidden, budget/depth/cycle boundary, and excluded Possible/Candidate records. Independent scopes may remain useful with exact loss/stopping state.

Renderer/tokenizer/evaluation failure can leave the semantic bundle core valid but cannot satisfy the failed declared artifact/profile gate.

## Error tests

Every used code requires a direct fixture or mutation, exact input/profile/subject IDs, deterministic bounded serialization, recovery class, artifact/status impact, no private/source/cursor payload leak, and no authority/generation upgrade.
