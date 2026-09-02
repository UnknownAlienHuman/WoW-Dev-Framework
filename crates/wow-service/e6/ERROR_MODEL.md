# E6-B service error model

**Status:** normative.

```text
E6BServiceError
    code
    operation/stage
    public/normalized request and OperationId
    provider/descriptor/session/external-state IDs when known
    result/candidate/artifact/locator/mapping/selection/context IDs when known
    owner/adapter/store/context error refs
    coverage/conflict/privacy/license/retention/audit/closure state
    effect/reconciliation/recovery state
    bounded structured arguments
```

## Configuration and requests

- `e6b_configuration_invalid`
- `e6b_request_schema_invalid`
- `e6b_operation_unsupported`
- `e6b_unknown_field_or_profile`
- `e6b_budget_invalid_or_unbounded`
- `e6b_cross_operation_field_forbidden`
- `e6b_current_latest_best_selector_forbidden`

## Provider descriptor, credentials, and session

- `e6b_provider_not_configured`
- `e6b_provider_descriptor_invalid_or_mismatch`
- `e6b_provider_adapter_unavailable_or_invalid`
- `e6b_provider_capability_missing_or_changed`
- `e6b_provider_authorization_port_unavailable`
- `e6b_provider_unauthorized`
- `e6b_provider_authorization_expired_revoked_or_replayed`
- `e6b_provider_authorization_scope_mismatch`
- `e6b_provider_session_acquisition_failed`
- `e6b_provider_session_receipt_invalid_or_substituted`
- `e6b_provider_session_close_failed`
- `e6b_raw_credential_or_session_handle_forbidden`

## External state

- `e6b_external_state_invalid_or_mismatch`
- `e6b_external_state_class_upgrade_forbidden`
- `e6b_stable_generation_not_verified`
- `e6b_observed_mutable_state_receipt_stale_or_substituted`
- `e6b_opaque_state_reproducibility_claim_forbidden`
- `e6b_external_state_refreshed_mid_operation`

## Durable operations and response loss

- `e6b_operation_id_request_digest_conflict`
- `e6b_operation_state_invalid`
- `e6b_effect_before_registration_forbidden`
- `e6b_duplicate_provider_or_store_effect_forbidden`
- `e6b_provider_effect_outcome_unknown`
- `e6b_effect_reconciliation_failed_or_unsupported`
- `e6b_response_loss_recovery_failed`
- `e6b_nochange_without_exact_proof`
- `e6b_run_or_retry_identity_changed`
- `e6b_operation_cancelled`
- `e6b_background_work_forbidden`

## E6-A owner operations and artifacts

- `e6b_cbm_owner_port_unavailable`
- `e6b_cbm_owner_operation_failed`
- `e6b_cbm_result_validation_failed`
- `e6b_external_result_or_artifact_unavailable`
- `e6b_external_result_schema_digest_or_profile_mismatch`
- `e6b_candidate_not_in_exact_result_set`
- `e6b_candidate_authority_upgrade_forbidden`
- `e6b_zero_result_negative_authority_forbidden`
- `e6b_provider_score_or_rank_cross_boundary_forbidden`
- `e6b_arbitrary_provider_tool_or_mcp_call_forbidden`

## Mapping

- `e6b_mapping_request_invalid`
- `e6b_mapping_owner_port_unavailable`
- `e6b_mapping_owner_generation_or_profile_mismatch`
- `e6b_mapping_locator_invalid_or_substituted`
- `e6b_mapping_result_invalid`
- `e6b_mapping_multiple_or_ambiguous`
- `e6b_mapping_partial_or_not_evaluated`
- `e6b_mapping_conflict`
- `e6b_mapping_exact_claim_without_owner_evidence`
- `e6b_mapping_provider_claim_upgrade_forbidden`
- `e6b_mapping_name_path_snippet_heuristic_forbidden`
- `e6b_mapping_cross_universe_or_generation_substitution`
- `e6b_mapping_negative_authority_unavailable`

## Selection

- `e6b_selection_request_invalid`
- `e6b_selection_requires_exact_mapping`
- `e6b_selection_candidate_mapping_root_mismatch`
- `e6b_selection_not_explicit`
- `e6b_top_first_best_highest_score_or_sole_selection_forbidden`
- `e6b_selection_authority_or_permission_upgrade_forbidden`
- `e6b_selection_receipt_invalid_or_supersession_failed`
- `e6b_selection_record_publication_failed`

## Context handoff

- `e6b_context_request_invalid`
- `e6b_context_owner_unavailable`
- `e6b_context_generation_or_root_mismatch`
- `e6b_context_provider_metadata_injected_as_framework_fact`
- `e6b_context_provider_summary_fallback_forbidden`
- `e6b_context_result_invalid_or_partial`
- `e6b_context_effect_outcome_unknown`
- `e6b_external_attachment_privacy_or_license_blocked`

## Catalog, cache, retention, and continuation

- `e6b_catalog_none_for_exact_binding`
- `e6b_catalog_multiple_for_exact_binding`
- `e6b_catalog_snapshot_or_continuation_stale`
- `e6b_cache_entry_invalid_or_mismatch`
- `e6b_cache_freshness_or_authority_upgrade_forbidden`
- `e6b_cache_privacy_or_license_scope_mismatch`
- `e6b_continuation_invalid_or_stale`
- `e6b_continuation_provider_state_or_budget_reset_forbidden`
- `e6b_retention_admission_failed`
- `e6b_gc_race_or_artifact_disappeared`
- `e6b_public_success_before_close_forbidden`
- `e6b_resource_close_failed`

## Security, privacy, and output

- `e6b_provider_install_start_configure_index_or_delete_forbidden`
- `e6b_provider_database_or_raw_storage_access_forbidden`
- `e6b_filesystem_network_process_editor_client_access_forbidden`
- `e6b_raw_sql_script_plugin_shell_model_or_tool_execution_forbidden`
- `e6b_provider_path_or_url_follow_forbidden`
- `e6b_private_endpoint_credential_cursor_or_handle_disclosure_forbidden`
- `e6b_privacy_license_consumer_scope_mismatch`
- `e6b_untrusted_provider_or_source_text_control_injection_forbidden`
- `e6b_audit_record_invalid_or_incomplete`
- `e6b_error_or_log_redaction_failed`
- `e6b_output_serialization_failed`
- `e6b_application_bypassed_service_boundary`
- `e6b_hidden_provider_or_cache_fallback_forbidden`

## Recovery classes

```text
never
after-request-profile-or-selector-fix
after-provider-configuration-or-capability-fix
after-provider-authorization-or-session-fix
after-external-state-fix
after-owner-artifact-or-retention-fix
after-project-or-reference-mapping-fix
after-explicit-selection-fix
after-context-or-privacy-license-fix
retry-same-operation-id-and-request-digest
reconcile-exact-provider-store-or-context-effect
continue-with-exact-retained-state-and-budget
safe-cleanup-or-quarantine
```

## Error privacy

Default errors expose stable IDs, states, counts, and reason codes only. They exclude credentials, private endpoints, session handles, provider cursors, private paths/URIs/snippets, source bodies, owner/store internals, and unrestricted stack data.
