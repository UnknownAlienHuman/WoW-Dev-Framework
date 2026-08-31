# E3-B error model

**Status:** normative.

```text
ContextError
    code
    operation and stage
    universe-set/request/profile IDs
    project/graph/reference/source/skeleton/map/pack/artifact IDs
    root/item/evidence/coverage/conflict/omission IDs
    budget/tokenizer/privacy/license/boundary/cache/cursor state
    structured message arguments
    recovery class
```

Errors carry stable identifiers and bounded structured arguments. They do not include unrestricted source, private paths, credentials, arbitrary request text, SQL, or raw owner handles.

## Milestone and schema

- `context_milestone_alias_conflict`
- `context_legacy_and_current_type_both_activated`
- `context_legacy_and_current_operation_both_activated`
- `context_contract_or_schema_version_unsupported`
- `context_profile_schema_invalid_or_unknown`
- `context_profile_breaking_change_unversioned`
- `context_unknown_field_or_enum_value`
- `context_identity_dag_cycle_or_forward_reference`
- `context_canonicalization_profile_mismatch`

## Universe binding and inputs

- `context_universe_set_invalid`
- `context_primary_project_or_graph_missing`
- `context_optional_platform_universe_required_but_missing`
- `context_project_graph_source_generation_mismatch`
- `context_reference_profile_incompatible`
- `context_platform_source_profile_incompatible`
- `context_cross_universe_identity_collapse_forbidden`
- `context_same_name_or_path_join_forbidden`
- `context_floating_current_or_latest_forbidden`
- `context_raw_store_analyzer_or_parser_handle_forbidden`
- `context_input_capability_missing_or_failed`
- `context_input_coverage_partial_or_conflicted`
- `context_owner_record_wrong_universe_or_generation`
- `context_source_coordinate_or_digest_mismatch`
- `context_store_image_identity_forbidden`

## Request and roots

- `context_request_invalid`
- `context_root_selector_unresolved_or_fuzzy`
- `context_root_not_in_bound_universe`
- `context_root_kind_not_allowed_for_intent`
- `context_requested_facet_axis_or_relation_unknown`
- `context_confidence_or_authority_upgrade_forbidden`
- `context_executable_selector_or_callback_forbidden`
- `context_search_or_ranking_request_out_of_scope`
- `context_request_budget_invalid`
- `context_renderer_or_tokenizer_profile_incompatible`

## Project Map

- `context_project_map_profile_invalid`
- `context_project_map_root_or_node_unresolved`
- `context_project_map_edge_not_backed_by_graph`
- `context_project_map_path_materialized_as_direct_edge`
- `context_project_map_group_rule_forbidden`
- `context_project_map_group_membership_or_count_mismatch`
- `context_project_map_cross_universe_merge_forbidden`
- `context_project_map_mandatory_content_missing`
- `context_project_map_budget_or_continuation_invalid`
- `context_project_map_validation_failed`

## L0 and L1 skeletons

- `context_l0_scope_invalid_or_unsupported`
- `context_l0_body_or_unbounded_members_forbidden`
- `context_l0_role_or_importance_guess_forbidden`
- `context_l0_member_count_or_page_mismatch`
- `context_l1_root_invalid_or_unsupported`
- `context_l1_signature_type_or_span_unresolved`
- `context_l1_relation_or_path_invalid`
- `context_l1_event_hook_state_class_collapse_forbidden`
- `context_l1_possible_or_ambiguous_relation_upgraded`
- `context_l1_control_effect_fact_unsupported`
- `context_second_parser_cfg_ssa_or_dataflow_forbidden`
- `context_skeleton_mandatory_closure_missing`
- `context_skeleton_validation_failed`

## Expansion, selection, and continuation

- `context_expansion_profile_or_stage_invalid`
- `context_expansion_prerequisite_missing`
- `context_candidate_dependency_cycle`
- `context_candidate_origin_or_cost_invalid`
- `context_selection_tier_or_tie_key_invalid`
- `context_mandatory_candidate_omitted`
- `context_duplicate_merge_lost_origin_or_confidence`
- `context_hidden_root_universe_relation_or_confidence_broadening`
- `context_no_new_evidence_claim_invalid`
- `context_stop_state_invalid`
- `context_continuation_cursor_invalid_or_stale`
- `context_continuation_changed_snapshot_request_or_profile`
- `context_continuation_total_budget_reset_forbidden`
- `context_expansion_cancelled`
- `context_background_continuation_forbidden`

## Coverage, conflicts, authority, and omissions

- `context_claim_origin_or_evidence_incomplete`
- `context_authority_provenance_or_confidence_upgrade_forbidden`
- `context_conflict_hidden_or_unresolved`
- `context_coverage_axes_collapsed`
- `context_negative_authority_unavailable`
- `context_empty_or_omitted_section_used_as_absence`
- `context_omission_record_missing_or_invalid`
- `context_selected_omitted_unenumerated_count_mismatch`
- `context_partial_pack_reported_complete`
- `context_projection_loss_hidden_or_invalid`
- `context_existing_finding_generation_mismatch`

## Source, privacy, license, and boundaries

- `context_source_excerpt_request_invalid`
- `context_source_handle_digest_or_range_mismatch`
- `context_source_path_fallback_forbidden`
- `context_source_privacy_denied`
- `context_source_license_or_redistribution_denied`
- `context_source_consumer_trust_insufficient`
- `context_source_encoding_unsupported`
- `context_source_transformation_or_redaction_invalid`
- `context_source_excerpt_budget_exceeded`
- `context_source_truncation_or_continuation_invalid`
- `context_source_boundary_profile_invalid`
- `context_source_boundary_escape_or_roundtrip_failed`
- `context_source_text_controlled_metadata_forbidden`
- `context_private_path_credential_or_sensitive_data_leak`
- `context_arbitrary_filesystem_network_process_editor_or_client_access_forbidden`

## Budgets and tokenization

- `context_minimum_required_content_exceeds_budget`
- `context_semantic_budget_exceeded`
- `context_render_budget_exceeded`
- `context_item_cost_or_budget_accounting_mismatch`
- `context_partial_structured_item_pruning_forbidden`
- `context_mandatory_identity_evidence_or_boundary_pruning_forbidden`
- `context_tokenizer_profile_invalid`
- `context_exact_tokenizer_unavailable_or_digest_mismatch`
- `context_exact_token_claim_without_exact_profile`
- `context_token_estimate_or_upper_bound_invalid`
- `context_renderer_overhead_prediction_mismatch`
- `context_unbounded_input_output_or_resource_budget_forbidden`

## Semantic pack and rendering

- `context_semantic_pack_invalid`
- `context_semantic_pack_dangling_reference`
- `context_semantic_pack_mixed_generation`
- `context_semantic_pack_alias_duplication`
- `context_semantic_pack_canonical_digest_mismatch`
- `context_renderer_created_or_changed_semantic_fact`
- `context_renderer_required_item_missing`
- `context_renderer_item_range_mapping_invalid`
- `context_rendering_loss_hidden_or_disallowed`
- `context_rendered_artifact_digest_or_encoding_mismatch`
- `context_json_roundtrip_failed`
- `context_markdown_boundary_or_template_validation_failed`
- `context_metrics_or_evaluation_entered_semantic_identity`
- `context_validation_attempted_repair`

## Cache and determinism

- `context_cache_key_invalid_or_incomplete`
- `context_cache_artifact_schema_digest_or_profile_mismatch`
- `context_cache_cross_generation_relabel_forbidden`
- `context_cache_cross_privacy_or_consumer_reuse_forbidden`
- `context_cache_partial_or_cancelled_used_as_complete`
- `context_cache_corrupted_or_unresolvable`
- `context_physical_cache_storage_out_of_scope`
- `context_nondeterministic_order_selection_or_bytes`
- `context_worker_storage_cache_or_host_dependent_output`
- `context_rebuild_comparison_incomparable`

## Security and cancellation

- `context_source_or_request_code_execution_forbidden`
- `context_model_embedding_or_external_tool_call_forbidden`
- `context_sql_expression_regex_program_or_plugin_forbidden`
- `context_resource_exhaustion_guard_triggered`
- `context_malformed_or_tampered_artifact`
- `context_cancelled`
- `context_output_confidentiality_violation`
- `context_downstream_authorization_claim_forbidden`

## Prerequisites and freeze

- `context_prerequisite_implementation_or_fixture_missing`
- `context_platform_source_e3_a_prerequisite_missing`
- `context_profile_or_fixture_pin_missing`
- `context_checksum_manifest_incomplete`
- `context_first_rust_commit_before_freeze_forbidden`
- `context_executable_probe_or_benchmark_missing`
- `context_required_evaluation_gate_not_executed`
- `context_documentation_only_state_violation`

## Recovery classes

```text
never
after-contract-or-profile-fix
after-exact-universe-or-generation-rebind
after-owner-capability-or-coverage-fix
after-root-resolution-by-higher-layer
after-budget-or-renderer-profile-change
after-privacy-license-or-consumer-policy-change
after-source-handle-or-range-fix
after-cache-eviction-and-exact-rebuild
after-prerequisite-implementation-and-freeze
resume-with-same-exact-continuation
return-explicit-partial-or-not-evaluated-only
```

A retry that changes universe, request, profile, privacy, tokenizer, renderer, or total budget creates a new request and artifact identity.
