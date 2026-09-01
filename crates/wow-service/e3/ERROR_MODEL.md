# E3-C service error model

**Status:** normative.

```text
ContextServiceError
    code
    operation/stage/owner
    request/configuration/selector/resolution IDs
    safely available store/project/graph/reference/context/artifact/profile IDs
    capability/coverage/conflict/omission/budget/continuation IDs
    bounded structured arguments
    recovery class
```

Errors exclude source bodies, private paths, credentials, raw owner handles, SQL, arbitrary request text, and terminal/process state.

## Configuration and contracts

- `service_context_configuration_invalid`
- `service_context_contract_or_schema_unsupported`
- `service_context_owner_port_catalog_invalid`
- `service_context_profile_alias_registry_invalid`
- `service_context_default_profile_invalid`
- `service_context_operation_profile_invalid`
- `service_context_dependency_inactive_or_forbidden`
- `service_context_prerequisite_implementation_missing`
- `service_context_fixture_or_checksum_unfrozen`

## Request, selectors, roots, profiles

- `service_context_request_invalid`
- `service_context_operation_unknown_or_deferred`
- `service_context_selector_invalid`
- `service_context_current_selector_unscoped`
- `service_context_exact_generation_or_publication_unavailable`
- `service_context_expected_current_guard_mismatch`
- `service_context_selector_owner_or_project_mismatch`
- `service_context_second_current_read_forbidden`
- `service_context_hidden_retry_or_fallback_forbidden`
- `service_context_last_known_good_substitution_forbidden`
- `service_context_root_unresolved_or_nonexact`
- `service_context_root_wrong_universe_or_generation`
- `service_context_search_or_natural_language_root_forbidden`
- `service_context_profile_alias_unknown_or_ambiguous`
- `service_context_profile_or_renderer_incompatible`
- `service_context_budget_override_invalid`
- `service_context_privacy_or_license_profile_upgrade_forbidden`

## Acquisition and compatibility

- `service_context_primary_acquisition_failed`
- `service_context_platform_acquisition_failed`
- `service_context_reference_acquisition_failed`
- `service_context_owner_record_invalid`
- `service_context_owner_record_wrong_generation`
- `service_context_selection_incompatible`
- `service_context_reference_binding_mismatch`
- `service_context_platform_profile_mismatch`
- `service_context_skeleton_input_view_mismatch`
- `service_context_required_capability_missing_or_failed`
- `service_context_capability_partial_or_conflicted`
- `service_context_universe_binding_failed`
- `service_context_universe_set_invalid`
- `service_context_partial_lease_exposure_forbidden`
- `service_context_distributed_atomicity_claim_forbidden`

## Operations

- `service_context_status_failed`
- `service_context_map_failed`
- `service_context_inspect_failed`
- `service_context_build_failed`
- `service_context_continue_failed`
- `service_context_validate_failed`
- `service_context_render_failed`
- `service_context_owner_operation_unexpected`
- `service_context_operation_extra_lane_forbidden`
- `service_context_domain_algorithm_reimplementation_forbidden`
- `service_context_result_payload_invalid`

## Context artifacts and status

- `service_context_artifact_invalid_or_mixed_generation`
- `service_context_artifact_owner_closure_unavailable`
- `service_context_artifact_validation_attempted_repair`
- `service_context_renderer_changed_semantic_pack`
- `service_context_renderer_or_tokenizer_result_invalid`
- `service_context_invalid_artifact_misclassified_as_service_failure`
- `service_context_partial_or_truncated_reported_complete`
- `service_context_not_evaluated_status_invalid`
- `service_context_status_precedence_violation`
- `service_context_empty_output_interpreted_complete`
- `service_context_authority_or_confidence_upgrade_forbidden`
- `service_context_omission_conflict_or_coverage_hidden`

## Continuation and retention

- `service_context_continuation_invalid_or_too_large`
- `service_context_continuation_generation_unavailable`
- `service_context_continuation_receipt_missing_or_invalid`
- `service_context_continuation_changed_universe_request_profile`
- `service_context_continuation_current_resolution_forbidden`
- `service_context_continuation_total_budget_reset_forbidden`
- `service_context_continuation_retention_admission_failed`
- `service_context_continuation_advertised_without_retention`
- `service_context_continuation_release_failed`
- `service_context_background_continuation_forbidden`

## Lifecycle and closure

- `service_context_acquisition_order_violation`
- `service_context_release_order_violation`
- `service_context_resource_close_failed`
- `service_context_resource_leak_detected`
- `service_context_success_before_close_forbidden`
- `service_context_close_failure_hidden_as_warning`
- `service_context_cancelled_during_close`
- `service_context_late_owner_result_after_cancel`
- `service_context_multiple_public_results_forbidden`
- `service_context_panic_unwind_cleanup_failed`

## Envelope and determinism

- `service_context_envelope_invalid`
- `service_context_result_reference_closure_invalid`
- `service_context_result_digest_mismatch`
- `service_context_result_nondeterministic`
- `service_context_volatile_identity_forbidden`
- `service_context_result_order_invalid`
- `service_context_resource_closure_report_invalid`
- `service_context_source_or_private_data_leak`
- `service_context_cli_or_transport_state_in_semantic_result`

## Security

- `service_context_raw_store_analyzer_parser_or_source_handle_forbidden`
- `service_context_filesystem_network_process_editor_client_access_forbidden`
- `service_context_source_or_repository_execution_forbidden`
- `service_context_sql_plugin_callback_or_expression_forbidden`
- `service_context_model_embedding_search_or_cbm_forbidden`
- `service_context_artifact_or_continuation_malformed`
- `service_context_cross_store_or_project_access_forbidden`
- `service_context_resource_budget_exceeded`
- `service_context_output_confidentiality_violation`
- `service_context_downstream_tool_authorization_forbidden`

## Cancellation and application seam

- `service_context_cancelled`
- `service_context_late_success_after_cancel_forbidden`
- `service_context_application_lower_crate_bypass_forbidden`
- `service_context_application_resolved_current_forbidden`
- `service_context_application_artifact_path_leak_forbidden`
- `service_context_application_exit_or_terminal_policy_in_service_forbidden`

## Recovery classes

```text
never
after-configuration-or-contract-fix
after-exact-selector-or-guard-update
after-owner-publication-or-capability-fix
after-profile-budget-privacy-or-renderer-change
after-exact-retained-generation-restoration
after-continuation-retention-fix
after-artifact-input-fix
after-resource-lifecycle-fix
new-request-required
explicit-partial-or-not-evaluated-only
```

Changing selectors, roots, profiles, privacy, renderer, artifact bytes, or total budget produces a new request identity. A retry cannot silently reuse the failed request ID with different semantics.
