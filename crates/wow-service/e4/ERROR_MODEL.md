# E4-C service error model

**Status:** normative.

```text
E4ServiceError
    code
    operation/stage
    public and normalized request IDs
    exact selector/acquisition/owner/shard/lineage/context IDs available
    owner error refs
    authorization/privacy/license/budget/cancellation state
    durable artifact/idempotency/retention receipt refs
    resource closure state
    recovery class
    bounded structured arguments
```

## Configuration and request

- `e4_service_configuration_invalid`
- `e4_service_profile_alias_unknown_or_ambiguous`
- `e4_service_request_schema_invalid`
- `e4_service_operation_unsupported`
- `e4_service_budget_invalid_or_unbounded`
- `e4_service_root_or_candidate_selector_invalid`
- `e4_service_unknown_field_or_profile`
- `e4_service_cross_operation_field_forbidden`

## Selector and acquisition

- `e4_selector_generation_or_profile_mismatch`
- `e4_selector_current_guard_failed`
- `e4_selector_latest_or_fallback_forbidden`
- `e4_acquisition_unstable_current`
- `e4_acquisition_order_violation`
- `e4_acquisition_required_owner_unavailable`
- `e4_acquisition_partial_view_set_forbidden`
- `e4_acquisition_cross_universe_or_comparison_invalid`
- `e4_acquisition_compatibility_failed`
- `e4_catalog_none_for_exact_binding`
- `e4_catalog_multiple_for_exact_binding`
- `e4_retained_generation_or_artifact_unavailable`

## Search shard and query

- `e4_search_shard_unavailable`
- `e4_search_shard_not_validated_or_not_sealed`
- `e4_search_shard_owner_binding_mismatch`
- `e4_search_shard_profile_mismatch`
- `e4_search_implicit_build_forbidden`
- `e4_search_owner_operation_failed`
- `e4_search_lane_state_hidden_or_rewritten`
- `e4_search_result_validation_failed`
- `e4_search_authoritative_miss_upgrade_forbidden`
- `e4_search_candidate_authority_upgrade_forbidden`
- `e4_search_raw_query_or_storage_surface_forbidden`
- `e4_search_continuation_invalid_or_stale`
- `e4_search_result_or_candidate_not_found`
- `e4_search_candidate_result_guard_mismatch`
- `e4_search_rank_only_selection_forbidden`
- `e4_search_automatic_candidate_selection_forbidden`

## Search-to-context

- `e4_search_context_selection_receipt_invalid`
- `e4_search_context_owner_generation_mismatch`
- `e4_search_context_root_not_exact`
- `e4_search_context_rank_to_authority_forbidden`
- `e4_search_context_profile_or_privacy_mismatch`
- `e4_search_context_owner_operation_failed`
- `e4_search_context_continuation_selection_changed`

## Lineage producers and publication

- `e4_lineage_comparison_invalid`
- `e4_lineage_producer_partition_unavailable`
- `e4_lineage_producer_partition_validation_failed`
- `e4_lineage_search_candidate_ceiling_violation`
- `e4_lineage_component_or_proposal_invalid`
- `e4_lineage_ambiguity_hidden_or_collapsed`
- `e4_lineage_proof_ceiling_escalation_forbidden`
- `e4_lineage_graph_build_or_publication_failed`
- `e4_lineage_graph_snapshot_validation_failed`
- `e4_lineage_snapshot_unavailable_or_mismatch`
- `e4_lineage_in_place_mutation_forbidden`
- `e4_lineage_current_or_latest_snapshot_forbidden`
- `e4_lineage_query_failed`
- `e4_lineage_path_flattening_forbidden`
- `e4_lineage_absence_authority_unavailable`

## Review authorization and decisions

- `e4_review_envelope_schema_or_digest_invalid`
- `e4_review_target_or_profile_mismatch`
- `e4_review_authorization_port_unavailable`
- `e4_review_principal_unauthorized`
- `e4_review_attestation_or_signature_invalid`
- `e4_review_scope_mismatch`
- `e4_review_expired_revoked_or_replayed`
- `e4_review_semantic_validation_failed`
- `e4_review_requested_confidence_exceeds_ceiling`
- `e4_review_plain_prose_authority_forbidden`
- `e4_review_operator_identity_inference_forbidden`
- `e4_review_apply_stale_base`
- `e4_review_apply_publication_failed`
- `e4_review_prior_decision_overwrite_forbidden`

## Migration

- `e4_migration_candidate_request_invalid`
- `e4_migration_governing_assertion_unavailable`
- `e4_migration_replacement_inference_forbidden`
- `e4_migration_recipe_schema_or_digest_invalid`
- `e4_migration_recipe_precondition_or_step_invalid`
- `e4_migration_recipe_validation_failed`
- `e4_migration_application_or_source_edit_forbidden`
- `e4_migration_runtime_success_claim_forbidden`
- `e4_migration_privacy_or_license_failed`

## Static impact

- `e4_impact_root_or_plan_invalid`
- `e4_impact_graph_generation_mismatch`
- `e4_impact_relation_or_confidence_profile_invalid`
- `e4_impact_owner_operation_failed`
- `e4_impact_path_or_evidence_invalid`
- `e4_impact_runtime_or_severity_upgrade_forbidden`
- `e4_impact_budget_exceeded_or_truncated`
- `e4_impact_continuation_invalid_or_stale`
- `e4_impact_path_flattening_forbidden`
- `e4_impact_negative_authority_unavailable`

## Lifecycle, idempotency, and retention

- `e4_operation_id_request_digest_conflict`
- `e4_operation_durable_state_invalid`
- `e4_operation_response_loss_recovery_failed`
- `e4_operation_duplicate_artifact_or_catalog_write`
- `e4_retention_admission_failed`
- `e4_continuation_advertised_without_retention`
- `e4_continuation_budget_reset_forbidden`
- `e4_resource_close_failed`
- `e4_public_success_before_close_forbidden`
- `e4_background_work_forbidden`
- `e4_operation_cancelled`

## Security, privacy, and output

- `e4_executable_or_raw_query_input_forbidden`
- `e4_filesystem_network_process_editor_client_access_forbidden`
- `e4_model_embedding_cbm_call_forbidden`
- `e4_private_source_or_credential_disclosure_forbidden`
- `e4_privacy_license_consumer_scope_mismatch`
- `e4_untrusted_text_control_injection_forbidden`
- `e4_artifact_schema_digest_or_size_invalid`
- `e4_output_serialization_failed`
- `e4_error_or_log_redaction_failed`
- `e4_application_bypassed_service_boundary`

## Recovery classes

```text
never
after-request-or-profile-fix
after-exact-selector-or-retention-fix
after-search-shard-build-and-validation
after-owner-capability-or-coverage-fix
after-review-authorization-or-attestation-fix
after-review-or-proof-semantics-fix
after-lineage-rebuild-or-validation
after-privacy-license-consumer-policy-fix
retry-same-operation-id-and-request-digest
retry-same-exact-read-inputs
retry-current-acquisition-within-finite-profile
continue-with-exact-retained-artifacts
safe-cleanup-or-quarantine
```

## Error privacy

Default errors expose stable IDs, counts, stages and bounded enum arguments only. Raw query/source/review/migration text, private paths, credentials, keys, signatures, database handles and stack dumps are excluded unless a separate explicit local-debug profile safely permits them.
