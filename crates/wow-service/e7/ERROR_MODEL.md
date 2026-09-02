# E7-A service and protocol error model

**Status:** normative.

```text
E7AServiceError
    stable code
    protocol/session/operation/stage
    exact SessionViewSet/OverlayGeneration/document/result IDs known
    owner/protocol/authorization/privacy/cancellation/effect/closure refs
    bounded structured arguments
    recovery class
```

## Protocol/profile/handshake

- `e7a_protocol_profile_invalid_or_unfrozen`
- `e7a_protocol_revision_or_transport_unsupported`
- `e7a_protocol_framing_or_json_invalid`
- `e7a_protocol_duplicate_key_or_unknown_field`
- `e7a_protocol_message_or_structure_limit_exceeded`
- `e7a_protocol_method_tool_or_resource_unknown`
- `e7a_protocol_capability_conflict`
- `e7a_protocol_initialization_order_invalid`
- `e7a_protocol_projection_or_conformance_failed`

## Authentication and authorization

- `e7a_transport_authentication_failed_or_unavailable`
- `e7a_operation_authorization_unavailable`
- `e7a_operation_unauthorized`
- `e7a_authorization_scope_expired_revoked_or_replayed`
- `e7a_client_identity_authorization_inference_forbidden`
- `e7a_protocol_capability_authority_upgrade_forbidden`
- `e7a_credential_token_key_or_secret_disclosure_forbidden`

## Session/workspace/generation

- `e7a_session_not_initialized_or_not_active`
- `e7a_session_state_transition_invalid`
- `e7a_session_request_id_conflict`
- `e7a_workspace_binding_invalid_or_unauthorized`
- `e7a_workspace_root_or_uri_invalid`
- `e7a_workspace_or_owner_mapping_ambiguous`
- `e7a_session_view_generation_mismatch`
- `e7a_session_view_capability_or_coverage_unavailable`
- `e7a_current_refresh_mid_request_forbidden`
- `e7a_session_rebind_expected_old_mismatch`
- `e7a_session_rebind_blocked_by_overlay_or_operation`
- `e7a_cross_session_handle_or_cancel_forbidden`
- `e7a_session_retention_or_close_failed`

## Documents and overlays

- `e7a_document_identity_or_language_invalid`
- `e7a_document_not_open_or_already_open`
- `e7a_document_version_stale_duplicate_or_conflicting`
- `e7a_document_change_range_invalid`
- `e7a_document_position_encoding_or_boundary_invalid`
- `e7a_document_change_or_content_limit_exceeded`
- `e7a_document_overlay_base_or_generation_mismatch`
- `e7a_document_overlay_analysis_unavailable_or_failed`
- `e7a_document_save_as_disk_or_publication_proof_forbidden`
- `e7a_document_overlay_write_to_source_forbidden`
- `e7a_document_overlay_rebase_or_merge_not_supported`
- `e7a_document_snapshot_validation_failed`

## Analysis/navigation/context/search

- `e7a_operation_request_invalid`
- `e7a_owner_capability_or_port_unavailable`
- `e7a_owner_operation_failed`
- `e7a_owner_result_identity_or_evidence_invalid`
- `e7a_diagnostic_previous_result_incompatible`
- `e7a_definition_or_reference_result_partial_or_ambiguous`
- `e7a_search_candidate_auto_selection_forbidden`
- `e7a_external_candidate_authority_upgrade_forbidden`
- `e7a_context_source_privacy_or_license_denied`
- `e7a_code_action_edit_or_command_forbidden`
- `e7a_unsupported_feature_fallback_forbidden`
- `e7a_negative_authority_unavailable`

## Cancellation/progress/backpressure

- `e7a_operation_cancel_target_invalid_or_stale`
- `e7a_operation_cancel_not_supported`
- `e7a_operation_cancelled`
- `e7a_effect_outcome_unknown`
- `e7a_progress_sequence_or_stage_invalid`
- `e7a_progress_or_partial_result_limit_exceeded`
- `e7a_partial_result_or_continuation_invalid`
- `e7a_continuation_generation_profile_or_budget_mismatch`
- `e7a_queue_busy_or_backpressure_limit_exceeded`
- `e7a_response_or_notification_dropped_forbidden`
- `e7a_background_work_after_terminal_state_forbidden`

## Security/privacy/source

- `e7a_filesystem_url_network_process_editor_client_access_forbidden`
- `e7a_raw_sql_store_owner_or_generic_rpc_access_forbidden`
- `e7a_source_script_plugin_shell_wasm_or_native_execution_forbidden`
- `e7a_model_sampling_embedding_or_reranker_forbidden`
- `e7a_dynamic_method_tool_or_resource_registration_forbidden`
- `e7a_source_or_prompt_control_injection_forbidden`
- `e7a_private_source_path_or_metadata_disclosure_forbidden`
- `e7a_cross_consumer_privacy_or_license_scope_mismatch`
- `e7a_mcp_arbitrary_file_url_or_provider_locator_forbidden`
- `e7a_lsp_workspace_edit_execute_command_or_settings_mutation_forbidden`
- `e7a_error_log_or_telemetry_redaction_failed`

## Output/closure

- `e7a_service_result_validation_failed`
- `e7a_protocol_projection_loss_invalid`
- `e7a_output_serialization_or_framing_failed`
- `e7a_broken_pipe_or_transport_closed`
- `e7a_public_success_before_retention_or_close_forbidden`
- `e7a_shutdown_incomplete_or_failed`
- `e7a_nochange_or_clean_without_exact_proof`

## Recovery classes

```text
never
after-protocol-profile-or-client-capability-fix
after-authentication-or-authorization-fix
after-workspace-owner-publication-or-retention-fix
rebind-explicit-exact-session-view
after-document-version-or-full-content-resync
after-owner-capability-or-evidence-fix
continue-with-exact-session-view-and-budget
retry-safe-read-same-request
reconcile-exact-effect-operation
after-privacy-license-or-consumer-profile-fix
close-session-and-reinitialize
safe-cleanup-or-quarantine
E7B-implementation-or-release-profile-required
```

## Protocol mapping

Adapters map stable service errors to exact protocol error/result shapes under their frozen profiles. They preserve the service error code in structured data when permitted. An adapter never converts a domain blocker/partial result to a generic transport success that hides the state, nor converts a malformed protocol request into a domain finding.

## Privacy

Default error fields contain stable IDs, enums, bounded counts, and stages. They exclude document/source/query text, private roots, credentials, signatures, hidden review/holdout data, provider private endpoints, raw owner handles, and unrestricted stack traces.
