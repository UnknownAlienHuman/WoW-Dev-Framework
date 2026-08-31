# E3-B Blizzard UI source error model

**Status:** normative.

```text
BlizzardUiSourceError
    code
    operation/stage
    provider/source-profile/materialized-snapshot/source-generation IDs
    root/package/file/source-handle/analyzer/partition/entity/relation/bridge IDs
    reference/graph/publication/store/current-record IDs
    license/build-binding/coverage/conflict/budget/cancellation state
    structured bounded message arguments
    recovery class
```

Errors never echo full source, provider credentials, private checkout paths or unbounded comments/filenames.

## Source profile and materialization

- `ui_source_profile_invalid_or_unsupported`
- `ui_source_provider_or_revision_invalid`
- `ui_source_floating_or_unqualified_source_forbidden`
- `ui_source_materialized_snapshot_invalid`
- `ui_source_snapshot_not_sealed_or_immutable`
- `ui_source_root_definition_invalid`
- `ui_source_root_or_path_escape_forbidden`
- `ui_source_path_normalization_or_case_collision`
- `ui_source_file_manifest_or_content_digest_mismatch`
- `ui_source_file_kind_encoding_or_size_unsupported`
- `ui_source_symlink_reparse_submodule_lfs_or_archive_forbidden`
- `ui_source_materialization_partial_failed_or_cancelled`
- `ui_source_repository_hook_workflow_or_generated_code_execution_forbidden`
- `ui_source_source_instruction_or_prompt_escape_forbidden`

## Build and profile binding

- `ui_source_build_binding_invalid`
- `ui_source_build_binding_unverified`
- `ui_source_build_binding_mismatch`
- `ui_source_provider_claim_upgraded_without_evidence`
- `ui_source_reference_profile_incompatible`
- `ui_source_nearest_latest_or_cross_profile_fallback_forbidden`
- `ui_source_current_selector_invalid_or_ambiguous`
- `ui_source_publication_ineligible_for_selector`

## Package, TOC, XML and load

- `ui_source_package_or_global_unit_invalid`
- `ui_source_package_boundary_ambiguous`
- `ui_source_toc_variant_invalid_or_mixed`
- `ui_source_toc_parse_or_resolution_failed`
- `ui_source_xml_parse_include_or_source_map_failed`
- `ui_source_xml_external_entity_network_or_execution_forbidden`
- `ui_source_load_model_invalid_or_conflicted`
- `ui_source_static_load_runtime_claim_forbidden`
- `ui_source_savedvariables_contents_access_forbidden`
- `ui_source_unknown_record_or_file_silently_dropped`

## Analyzer and facts

- `ui_source_analyzer_profile_or_snapshot_mismatch`
- `ui_source_second_lua_parser_or_raw_source_fallback_forbidden`
- `ui_source_analyzer_unit_or_source_map_missing`
- `ui_source_fact_bundle_invalid`
- `ui_source_fact_adapter_loss_unreported`
- `ui_source_cross_universe_fact_collapse_forbidden`
- `ui_source_analyzer_partial_failed_or_cancelled`

## Recognizers and graph proposals

- `ui_source_recognizer_pack_or_partition_invalid`
- `ui_source_repository_path_or_product_specific_rule_forbidden`
- `ui_source_recognizer_confidence_upgrade_forbidden`
- `ui_source_graph_proposal_partition_invalid`
- `ui_source_graph_registry_or_generation_mismatch`
- `ui_source_graph_entity_or_relation_identity_invalid`
- `ui_source_graph_cross_universe_merge_forbidden`
- `ui_source_graph_proposal_rejected_or_conflicted`
- `ui_source_graph_transitive_edge_materialization_forbidden`
- `ui_source_graph_partial_publication_forbidden`

## Reference/source bridges

- `ui_source_bridge_profile_invalid_or_unsupported`
- `ui_source_bridge_input_generation_or_profile_mismatch`
- `ui_source_bridge_source_endpoint_unresolved`
- `ui_source_bridge_reference_endpoint_unresolved`
- `ui_source_bridge_ambiguous_or_conflicted`
- `ui_source_bridge_string_or_nearest_name_resolution_forbidden`
- `ui_source_bridge_relation_or_direction_invalid`
- `ui_source_bridge_confidence_or_authority_upgrade_forbidden`
- `ui_source_bridge_negative_authority_unavailable`
- `ui_source_project_bridge_requires_exact_project_snapshot`
- `ui_source_runtime_bridge_not_implemented_for_milestone`

## Authority and coverage

- `ui_source_implementation_source_as_api_authority_forbidden`
- `ui_source_source_absence_as_api_absence_forbidden`
- `ui_source_source_usage_as_public_contract_forbidden`
- `ui_source_runtime_secret_taint_protected_or_combat_claim_forbidden`
- `ui_source_coverage_incomplete_or_inconsistent`
- `ui_source_conflict_record_invalid_or_unresolved`
- `ui_source_negative_authority_unavailable`
- `ui_source_candidate_partial_or_truncated_marked_complete`
- `ui_source_last_known_good_relabel_forbidden`

## License and redistribution

- `ui_source_license_record_missing_or_invalid`
- `ui_source_license_or_redistribution_conflict`
- `ui_source_redistribution_decision_missing_unknown_or_expired`
- `ui_source_raw_source_or_excerpt_redistribution_forbidden`
- `ui_source_derived_artifact_redistribution_not_evaluated`
- `ui_source_required_notice_or_attribution_missing`
- `ui_source_pack_or_database_release_ineligible`
- `ui_source_private_path_credential_or_user_data_forbidden`

## Candidate and invalidation

- `ui_source_candidate_invalid`
- `ui_source_candidate_publication_state_invalid`
- `ui_source_invalidation_plan_invalid_or_incomplete`
- `ui_source_reuse_proof_invalid`
- `ui_source_removed_input_stale_derived_state_retained`
- `ui_source_nochange_claim_invalid`
- `ui_source_update_order_or_worker_determinism_mismatch`
- `ui_source_cross_build_lineage_inference_not_owned_e3b`

## Store and publication

- `ui_source_publication_bundle_invalid`
- `ui_source_store_namespace_or_epoch_mismatch`
- `ui_source_user_project_current_mutation_forbidden`
- `ui_source_store_stale_base`
- `ui_source_inactive_generation_build_failed`
- `ui_source_inactive_readback_validation_failed`
- `ui_source_current_activation_cas_failed`
- `ui_source_current_advanced_before_validation_forbidden`
- `ui_source_store_source_graph_bridge_identity_mismatch`
- `ui_source_source_or_other_universe_leakage_detected`
- `ui_source_recovery_backup_retention_or_gc_invalid`

## Security and budgets

- `ui_source_filesystem_network_process_editor_or_client_access_forbidden`
- `ui_source_raw_sql_extension_attach_or_store_handle_forbidden`
- `ui_source_source_lua_xml_toc_or_binary_execution_forbidden`
- `ui_source_unbounded_input_output_or_graph_traversal_forbidden`
- `ui_source_budget_invalid_or_exceeded`
- `ui_source_cancelled`
- `ui_source_private_data_leak_forbidden`
- `ui_source_operation_not_implemented_for_milestone`

## Recovery classes

```text
never
after-source-profile-or-contract-fix
after-rematerialization-and-reseal
after-build-binding-evidence-fix
after-license-or-redistribution-review
after-parser-analyzer-recognizer-or-graph-fix
after-compatible-reference-source-selection
after-source-or-bridge-conflict-resolution
rebuild-affected-source-partitions
retry-exact-base-and-inputs
retry-with-smaller-explicit-budget
safe-local-analysis-only
safe-handle-or-fact-only-output
recover-or-revalidate-inactive-generation
explicit-last-known-good-with-mismatch-disclosure
future-project-search-lineage-runtime-or-release-milestone-required
```

## Partial outputs

Only operations whose contract explicitly permits a partial candidate/query may return one. Partial, conflict, truncation, cancellation, missing license/build binding or failed read-back is never silently complete/current/releasable.
