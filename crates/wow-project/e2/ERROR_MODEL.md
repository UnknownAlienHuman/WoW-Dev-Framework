# E2-C project indexing error model

**Status:** normative typed failure vocabulary.

```text
ProjectIndexError
    code
    operation/stage
    request/project/source snapshot/generation/candidate IDs
    universe/root/package/TOC/XML/file/Lua unit/partition IDs
    analyzer/recognizer/graph profile and result IDs
    capability/coverage/conflict/truncation/budget/cancellation state
    structured safe message arguments
    recovery class
```

## Request/source/root/universe

- `project_e2_index_request_invalid`
- `project_e2_source_snapshot_invalid`
- `project_e2_source_snapshot_digest_or_revision_mismatch`
- `project_e2_root_or_universe_invalid`
- `project_e2_path_escape_absolute_device_uri_or_collision`
- `project_e2_symlink_reparse_submodule_not_allowed`
- `project_e2_unexpected_or_unreadable_source_affects_completeness`
- `project_e2_first_party_dependency_library_universe_mix_forbidden`
- `project_e2_floating_repository_profile_or_variant_forbidden`
- `project_e2_private_path_token_or_source_payload_leak_forbidden`

## TOC

- `project_e2_toc_profile_invalid`
- `project_e2_toc_document_invalid`
- `project_e2_toc_variant_selection_none_or_ambiguous`
- `project_e2_toc_cross_variant_merge_forbidden`
- `project_e2_toc_directive_or_file_record_invalid`
- `project_e2_toc_unknown_or_unsupported_semantics`
- `project_e2_toc_file_missing_duplicate_or_path_invalid`
- `project_e2_toc_dependency_invalid_or_ambiguous`
- `project_e2_required_dependency_unresolved`
- `project_e2_toc_savedvariable_declaration_invalid_or_conflicting`
- `project_e2_toc_lod_or_bootstrap_runtime_claim_forbidden`
- `project_e2_toc_budget_exceeded`

## XML

- `project_e2_xml_profile_invalid`
- `project_e2_xml_document_invalid`
- `project_e2_xml_external_entity_dtd_xinclude_or_network_forbidden`
- `project_e2_xml_entity_depth_node_text_or_attribute_budget_exceeded`
- `project_e2_xml_include_invalid_missing_or_cycle`
- `project_e2_xml_template_object_or_inheritance_invalid`
- `project_e2_xml_parent_or_template_ambiguous`
- `project_e2_xml_script_record_invalid`
- `project_e2_xml_embedded_lua_extraction_or_source_map_invalid`
- `project_e2_xml_unknown_or_unsupported_semantics`
- `project_e2_xml_or_script_execution_forbidden`

## Load model

- `project_e2_load_profile_invalid`
- `project_e2_package_dependency_cycle_or_conflict`
- `project_e2_load_unit_or_order_invalid`
- `project_e2_load_include_expansion_invalid`
- `project_e2_load_reachability_unknown_or_conflicted`
- `project_e2_load_transitive_edge_materialization_forbidden`
- `project_e2_static_load_as_runtime_success_forbidden`
- `project_e2_frame_or_lifecycle_readiness_claim_forbidden`
- `project_e2_load_model_budget_exceeded`

## Analyzer/Lua units

- `project_e2_lua_unit_invalid_or_duplicate`
- `project_e2_second_lua_parser_forbidden`
- `project_e2_analyzer_plan_invalid`
- `project_e2_analyzer_snapshot_generation_profile_or_manifest_mismatch`
- `project_e2_analyzer_virtual_source_map_mismatch`
- `project_e2_analyzer_removed_unit_fact_retained`
- `project_e2_analyzer_capability_unavailable`
- `project_e2_analyzer_fact_or_finding_rewrite_forbidden`

## Fact adapter/recognizers

- `project_e2_fact_adapter_profile_invalid`
- `project_e2_fact_adapter_loss_unreported`
- `project_e2_recognizer_bundle_invalid_or_scope_leak`
- `project_e2_recognizer_profile_or_generation_mismatch`
- `project_e2_recognizer_output_partition_invalid`
- `project_e2_recognizer_partial_as_complete_forbidden`
- `project_e2_project_specific_rule_mutation_forbidden`
- `project_e2_native_custom_cvar_signal_conflation_forbidden`
- `project_e2_hook_or_event_safety_claim_forbidden`
- `project_e2_savedvariable_root_without_toc_forbidden`

## Graph proposals

- `project_e2_graph_profile_or_registry_mismatch`
- `project_e2_graph_proposal_request_invalid`
- `project_e2_graph_proposal_rejected`
- `project_e2_graph_rejection_or_conflict_hidden`
- `project_e2_graph_final_identity_or_publication_forbidden`
- `project_e2_direct_project_proposal_semantics_not_owned`

## Invalidation/reuse

- `project_e2_update_stale_base`
- `project_e2_invalidation_graph_invalid_or_cyclic`
- `project_e2_invalidation_dependency_missing`
- `project_e2_partition_reuse_unproven`
- `project_e2_unknown_impact_not_widened`
- `project_e2_stale_source_fact_match_or_proposal_retained`
- `project_e2_removed_input_output_closure_failed`
- `project_e2_old_generation_relabel_or_mix_forbidden`
- `project_e2_no_change_misclassified`
- `project_e2_invalidation_nondeterministic`

## Candidate/publication boundary

- `project_e2_generation_derivation_invalid`
- `project_e2_candidate_invalid`
- `project_e2_candidate_mandatory_capability_unavailable`
- `project_e2_candidate_partiality_hidden`
- `project_e2_candidate_digest_mismatch`
- `project_e2_candidate_mutation_after_validation`
- `project_e2_persistent_store_or_current_pointer_forbidden`
- `project_e2_final_graph_generation_forbidden`
- `project_e2_publication_bundle_invalid`
- `project_e2_projectstore_publication_deferred_to_e2_d`

## Security/side effects

- `project_e2_filesystem_discovery_outside_materializer_forbidden`
- `project_e2_repository_hook_build_test_generator_execution_forbidden`
- `project_e2_source_lua_xml_toc_execution_forbidden`
- `project_e2_dependency_download_or_network_forbidden`
- `project_e2_process_shell_editor_client_access_forbidden`
- `project_e2_prompt_or_source_instruction_ignored`
- `project_e2_unbounded_input_output_forbidden`
- `project_e2_cancelled`
- `project_e2_late_work_after_cancel_forbidden`

## Fixture/deferred

- `project_e2_fixture_or_checksum_not_frozen`
- `project_e2_real_addon_fixture_not_pinned`
- `project_e2_projectstore_not_implemented_e2_c`
- `project_e2_runtime_data_not_implemented_e2_c`
- `project_e2_full_dependency_source_universe_not_implemented_e2_c`
- `project_e2_operation_not_implemented_for_milestone`

## Recovery classes

```text
never
after-request-source-root-or-profile-fix
after-toc-or-xml-source-fix
after-dependency-or-load-policy-fix
after-analyzer-or-virtual-unit-fix
after-adapter-recognizer-or-graph-contract-fix
after-conservative-reindex
after-budget-or-scope-reduction
retry-exact-base-and-target
explicit-partial-or-NotEvaluated-only
deferred-to-E2-D-or-later
```

## Rules

- expected unsupported/dynamic cases become structured coverage/outcomes where contract allows, not fatal errors by default;
- fatal context/security/generation/candidate errors expose no complete target candidate;
- errors do not include arbitrary source, private paths, credentials, SavedVariables contents, runtime event payloads, or huge hostile metadata;
- message text is not identity;
- cancellation is neither clean nor failed publication;
- error serialization is deterministic.
