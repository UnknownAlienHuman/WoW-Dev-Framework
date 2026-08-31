# E3-A error model

**Status:** normative.

```text
BlizzardUiIndexError
    code
    operation/stage
    source/profile/project/analyzer/recognizer/graph/store/publication IDs
    package/file/unit/entity/partition/source-handle IDs
    structured message arguments
    coverage/conflict/budget/cancellation state
    recovery class
```

Errors must not expose private absolute roots, credentials, tokens, or unrestricted source text.

## Source/profile/materialization

- `blizzard_ui_source_profile_invalid`
- `blizzard_ui_source_snapshot_invalid_or_incomplete`
- `blizzard_ui_source_revision_or_content_mismatch`
- `blizzard_ui_source_root_or_path_escape`
- `blizzard_ui_source_case_or_identity_collision`
- `blizzard_ui_source_external_reference_not_materialized`
- `blizzard_ui_source_encoding_or_byte_policy_unsupported`
- `blizzard_ui_source_build_profile_incompatible`
- `blizzard_ui_source_license_or_provenance_unresolved`
- `blizzard_ui_source_floating_input_forbidden`

## Package/TOC/load

- `blizzard_ui_package_discovery_invalid`
- `blizzard_ui_toc_variant_missing_ambiguous_or_incompatible`
- `blizzard_ui_cross_flavor_merge_forbidden`
- `blizzard_ui_toc_parse_or_resolution_failed`
- `blizzard_ui_dependency_unresolved_or_conflicted`
- `blizzard_ui_load_model_invalid`
- `blizzard_ui_load_cycle_or_order_conflict`
- `blizzard_ui_runtime_readiness_claim_forbidden`

## XML/Lua/analyzer

- `blizzard_ui_xml_parse_or_expansion_failed`
- `blizzard_ui_xml_external_entity_or_execution_forbidden`
- `blizzard_ui_xml_include_or_script_cycle_budget_exceeded`
- `blizzard_ui_virtual_lua_unit_or_source_map_invalid`
- `blizzard_ui_second_lua_parser_forbidden`
- `blizzard_ui_analyzer_plan_or_snapshot_mismatch`
- `blizzard_ui_analyzer_fact_adapter_loss_or_failure`
- `blizzard_ui_silent_any_or_global_injection_forbidden`

## Recognizer/graph

- `blizzard_ui_recognizer_profile_or_input_mismatch`
- `blizzard_ui_named_source_semantic_condition_forbidden`
- `blizzard_ui_recognizer_partition_partial_or_failed`
- `blizzard_ui_graph_registry_or_scope_mismatch`
- `blizzard_ui_graph_proposal_rejected_or_conflicted`
- `blizzard_ui_graph_partition_replacement_invalid`
- `blizzard_ui_cross_universe_identity_merge_forbidden`
- `blizzard_ui_confidence_or_authority_upgrade_forbidden`

## Candidate/update/publication

- `blizzard_ui_index_candidate_invalid`
- `blizzard_ui_index_candidate_partial_not_allowed`
- `blizzard_ui_incremental_dependency_or_reuse_proof_invalid`
- `blizzard_ui_removed_input_stale_closure_failed`
- `blizzard_ui_nochange_claim_invalid`
- `blizzard_ui_publication_bundle_invalid`
- `blizzard_ui_store_graph_project_generation_mismatch`
- `blizzard_ui_inactive_publication_or_readback_failed`
- `blizzard_ui_current_activation_stale_or_failed`
- `blizzard_ui_prior_generation_modified_or_relabelled`

## Fingerprint/skeleton input

- `blizzard_ui_fingerprint_profile_or_record_invalid`
- `blizzard_ui_fingerprint_used_as_lineage_authority_forbidden`
- `blizzard_ui_skeleton_input_request_invalid`
- `blizzard_ui_skeleton_input_snapshot_mismatch`
- `blizzard_ui_skeleton_input_source_slice_invalid_or_denied`
- `blizzard_ui_skeleton_input_budget_exceeded`
- `blizzard_ui_skeleton_input_cursor_invalid_or_stale`
- `blizzard_ui_context_rendering_in_project_crate_forbidden`
- `blizzard_ui_unbounded_source_or_graph_export_forbidden`

## Security/resource/cancellation

- `blizzard_ui_source_or_generated_code_execution_forbidden`
- `blizzard_ui_filesystem_network_process_editor_client_access_forbidden`
- `blizzard_ui_prompt_or_source_instruction_injection_forbidden`
- `blizzard_ui_private_data_or_source_leak_forbidden`
- `blizzard_ui_budget_invalid_or_exceeded`
- `blizzard_ui_operation_cancelled`
- `blizzard_ui_truncated_result_cannot_be_complete`

## Recovery classes

```text
never
after-source-materialization-fix
after-profile-or-compatibility-fix
after-license-provenance-review
after-package-toc-xml-source-fix
after-analyzer-or-annotation-fix
after-recognizer-or-graph-contract-fix
after-update-reuse-or-removal-fix
after-store-publication-or-readback-fix
retry-exact-base-and-unchanged-inputs
request-smaller-explicit-bounds
explicit-partial-view-only
safe-quarantine-or-rebuild
```
