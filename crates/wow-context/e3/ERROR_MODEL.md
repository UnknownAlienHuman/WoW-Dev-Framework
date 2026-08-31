# E3-A context error model

**Status:** normative structured failure vocabulary.

## Error shape

```text
ContextError
    code
    operation/stage
    context request/plan/artifact/bundle IDs
    publication/store/project/graph/reference generation IDs
    context/project-map/skeleton/expansion/source/budget/tokenizer/security/evaluation profile IDs
    root/entity/relation/path/source/evidence/loss/omission/frontier/cursor/metric IDs
    structured bounded arguments
    budget/cancellation state
    recovery class
```

Public errors omit raw source beyond explicitly safe excerpt handles, private paths, credentials, cursor bytes, analyzer/store internals, and runtime-sensitive payloads.

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

## Configuration and dependency

- `context_configuration_invalid`
- `context_profile_invalid_or_incompatible`
- `context_project_map_profile_invalid`
- `context_skeleton_profile_invalid`
- `context_expansion_profile_invalid`
- `context_source_excerpt_profile_invalid`
- `context_budget_profile_invalid`
- `context_tokenizer_profile_invalid_or_unpinned`
- `context_security_profile_invalid`
- `context_evaluation_profile_invalid`
- `context_dependency_boundary_violation`
- `context_search_or_analyzer_internal_dependency_forbidden`
- `context_model_correctness_dependency_forbidden`
- `context_renderer_semantic_mutation_forbidden`

## Input snapshot and roots

- `context_request_invalid`
- `context_input_snapshot_invalid`
- `context_publication_project_graph_generation_mismatch`
- `context_reference_profile_or_generation_mismatch`
- `context_store_image_or_publication_set_mismatch`
- `context_current_or_latest_selector_forbidden_after_acquisition`
- `context_root_invalid_or_unresolved`
- `context_root_universe_or_generation_mismatch`
- `context_exact_root_required_search_deferred`
- `context_required_query_bundle_or_capability_unavailable`
- `context_stale_or_mixed_input_record`
- `context_candidate_or_external_universe_merge_forbidden`

## Planning and Project Map

- `context_plan_invalid`
- `context_plan_nondeterministic`
- `context_project_map_invalid`
- `context_project_map_section_invalid`
- `context_project_map_principal_root_unproven`
- `context_project_map_variant_or_universe_merge_forbidden`
- `context_project_map_static_runtime_claim_forbidden`
- `context_project_map_signal_system_collapse_forbidden`
- `context_project_map_savedvariables_content_forbidden`
- `context_project_map_missing_mandatory_blocker_or_route`
- `context_project_map_bulk_export_forbidden`
- `context_project_map_repository_name_condition_forbidden`

## Skeleton model

- `context_skeleton_invalid`
- `context_skeleton_subject_kind_unsupported`
- `context_skeleton_identity_nondeterministic`
- `context_skeleton_field_without_input_or_derivation`
- `context_skeleton_member_invalid_or_orphaned`
- `context_skeleton_signature_semantics_invalid`
- `context_skeleton_source_node_invalid`
- `context_skeleton_analyzer_internal_type_leak_forbidden`
- `context_skeleton_generated_code_or_source_reconstruction_forbidden`
- `context_skeleton_documentation_derived_semantics_forbidden`
- `context_skeleton_direct_edge_from_path_forbidden`
- `context_skeleton_generic_parent_or_related_collapse_forbidden`
- `context_skeleton_confidence_or_coverage_upgrade_forbidden`
- `context_skeleton_omission_or_truncation_unreported`
- `context_skeleton_deduplication_evidence_loss`
- `context_skeleton_nondeterministic`

## Detail and expansion

- `context_expansion_request_invalid`
- `context_expansion_frontier_invalid`
- `context_expansion_lane_or_direction_invalid`
- `context_expansion_scope_broadening_forbidden`
- `context_expansion_generation_switch_forbidden`
- `context_expansion_query_snapshot_or_result_invalid`
- `context_expansion_query_capability_unavailable`
- `context_expansion_cycle_or_path_state_invalid`
- `context_expansion_possible_or_candidate_policy_violation`
- `context_expansion_duplicate_classification_invalid`
- `context_expansion_evidence_loss_during_merge`
- `context_expansion_optional_branch_starved_mandatory_budget`
- `context_expansion_unbounded_traversal_or_export_forbidden`
- `context_expansion_model_priority_forbidden`
- `context_expansion_step_nondeterministic`
- `context_no_new_evidence_misclassified_as_absence`

## Evidence, coverage, loss, and omission

- `context_evidence_link_invalid`
- `context_material_claim_without_evidence_or_derivation`
- `context_evidence_generation_or_source_mismatch`
- `context_provenance_upgrade_or_misclassification_forbidden`
- `context_confidence_upgrade_forbidden`
- `context_coverage_axis_collapse_forbidden`
- `context_projection_status_invalid`
- `context_coverage_record_invalid`
- `context_loss_record_invalid_or_missing`
- `context_omission_record_invalid_or_missing`
- `context_stopping_record_invalid_or_missing`
- `context_conflict_silently_resolved_forbidden`
- `context_partial_or_not_evaluated_hidden_forbidden`
- `context_empty_or_omitted_as_authoritative_absence_forbidden`
- `context_loss_report_truncated_as_complete_forbidden`
- `context_artifact_eligibility_invalid`

## Source excerpts and security

- `context_source_excerpt_request_invalid`
- `context_source_handle_invalid_or_stale`
- `context_source_generation_or_digest_mismatch`
- `context_source_span_invalid_or_out_of_bounds`
- `context_source_origin_or_role_forbidden`
- `context_source_license_or_redistribution_forbidden`
- `context_source_privacy_or_security_forbidden`
- `context_source_object_missing_or_unreferenced`
- `context_source_excerpt_unfaithful_or_reconstructed`
- `context_source_excerpt_budget_exceeded`
- `context_source_prompt_instruction_treated_as_policy_forbidden`
- `context_source_container_directive_or_terminal_injection_forbidden`
- `context_source_private_path_credential_or_payload_leak_forbidden`
- `context_savedvariables_log_client_memory_access_forbidden`
- `context_source_filesystem_network_process_shell_editor_client_access_forbidden`
- `context_source_mutation_or_code_execution_forbidden`

## Budgets and tokenizer

- `context_budget_spec_invalid`
- `context_budget_override_exceeds_profile`
- `context_mandatory_reserve_unavailable`
- `context_budget_accounting_mismatch`
- `context_budget_exceeded`
- `context_atomic_record_or_utf8_boundary_violation`
- `context_lane_fairness_or_priority_invalid`
- `context_report_budget_hides_blocker_forbidden`
- `context_tokenizer_exact_count_without_pin_forbidden`
- `context_tokenizer_input_digest_or_config_mismatch`
- `context_tokenizer_result_invalid_or_nondeterministic`
- `context_token_estimate_unlabeled_as_exact_forbidden`
- `context_token_budget_renderer_subject_mismatch`
- `context_continuation_budget_reset_forbidden`

## Continuation and stopping

- `context_continuation_invalid_or_tampered`
- `context_continuation_schema_or_profile_mismatch`
- `context_continuation_snapshot_stale_or_mismatch`
- `context_continuation_request_or_ordering_mismatch`
- `context_continuation_frontier_or_visited_digest_mismatch`
- `context_continuation_budget_state_invalid`
- `context_continuation_scope_or_confidence_change_forbidden`
- `context_continuation_expired_input_unavailable`
- `context_continuation_after_complete_forbidden`
- `context_stopping_reason_invalid`
- `context_requested_complete_unproven`
- `context_no_change_misclassified`
- `context_cancelled`
- `context_late_work_or_background_continuation_forbidden`

## Bundle, rendering, and canonicalization

- `context_bundle_invalid`
- `context_bundle_input_or_profile_closure_invalid`
- `context_bundle_record_reference_or_manifest_invalid`
- `context_bundle_status_invalid`
- `context_bundle_digest_mismatch`
- `context_bundle_private_or_unsafe_payload`
- `context_renderer_profile_invalid`
- `context_renderer_output_semantic_mismatch`
- `context_renderer_loss_unreported`
- `context_renderer_injection_or_container_escape`
- `context_canonical_order_invalid`
- `context_artifact_nondeterministic`
- `context_volatile_field_in_canonical_identity`

## Metrics and evaluation

- `context_metrics_invalid_or_inconsistent`
- `context_metric_subject_or_denominator_mismatch`
- `context_mandatory_recall_incomplete`
- `context_evidence_closure_incomplete`
- `context_forbidden_or_invented_record_present`
- `context_relevance_label_or_task_profile_invalid`
- `context_redundancy_equivalence_invalid`
- `context_false_deduplication_detected`
- `context_compression_claim_invalid_or_misleading`
- `context_budget_adherence_failed`
- `context_continuation_equivalence_failed`
- `context_source_faithfulness_or_security_evaluation_failed`
- `context_consumer_utility_protocol_invalid`
- `context_external_model_evaluation_unpinned_or_authoritative_forbidden`
- `context_evaluation_corpus_mutation_forbidden`
- `context_evaluation_hard_gate_failed`
- `context_performance_claim_unscoped_or_unverified`
- `context_evaluation_report_nondeterministic`

## Deferred capabilities

- `context_search_not_implemented_e3_a`
- `context_lineage_or_patch_impact_not_implemented_e3_a`
- `context_full_source_or_bulk_graph_export_not_implemented_e3_a`
- `context_runtime_or_client_probe_not_implemented_e3_a`
- `context_code_generation_or_autofix_not_implemented_e3_a`
- `context_lsp_mcp_or_network_transport_not_implemented_e3_a`
- `context_cache_or_persistence_not_implemented_e3_a`
- `context_operation_not_implemented_for_milestone`

## Fatal versus scoped

### Fatal request/bundle

```text
input generation/profile mismatch
invalid root/request/profile
material claim/evidence invariant failure
private/security/source injection violation
canonical/determinism failure
invalid continuation integrity
```

### Root/lane/field scoped

```text
unsupported detail
partial/conflicted capability
source unavailable/forbidden
budget/depth/cycle boundary
Possible/Candidate excluded by policy
```

Independent roots/lanes may remain useful with exact loss/stopping state.

### Renderer/tokenizer/evaluation scoped

A renderer/tokenizer/consumer evaluation failure can leave the semantic bundle valid, but cannot satisfy the failed declared artifact/profile gate.

## Error tests

Every used code requires:

- exact operation/stage/input/profile/subject IDs;
- direct fixture or mutation;
- coverage/loss/stopping/bundle impact;
- deterministic serialization/order;
- recovery class;
- no private/source/cursor payload leak;
- no authority/confidence/generation upgrade;
- no complete artifact after fatal/cancelled state.
