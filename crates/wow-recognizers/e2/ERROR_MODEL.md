# E2-B recognizer error model

**Status:** normative typed failure vocabulary.

```text
RecognizerError
    code
    operation/stage
    pack/rule/profile IDs
    input bundle/generation/scope/partition IDs
    fact/clause/capture/match/ambiguity/proposal IDs
    graph registry/output partition IDs
    capability/coverage/budget/cancellation state
    structured safe message arguments
    recovery class
```

## Pack/schema

- `recognizer_pack_invalid`
- `recognizer_pack_schema_unsupported`
- `recognizer_pack_trust_class_not_activatable`
- `recognizer_pack_executable_or_external_content_forbidden`
- `recognizer_pack_duplicate_or_incompatible_id`
- `recognizer_pack_repository_identity_condition_forbidden`
- `recognizer_pack_literal_convention_undeclared`
- `recognizer_rule_invalid`
- `recognizer_rule_clause_graph_invalid_or_cyclic`
- `recognizer_rule_operator_unsupported`
- `recognizer_rule_unbounded_plan`
- `recognizer_rule_missing_fixture_or_evaluation_gate`
- `recognizer_rule_rollout_invalid`

## Fact input

- `recognizer_fact_bundle_invalid`
- `recognizer_fact_schema_incompatible`
- `recognizer_fact_generation_or_scope_mismatch`
- `recognizer_fact_source_or_evidence_invalid`
- `recognizer_fact_capability_unavailable`
- `recognizer_fact_adapter_loss_unreported`
- `recognizer_fact_cross_partition_join_forbidden`
- `recognizer_fact_stale_or_digest_mismatch`
- `recognizer_raw_source_or_second_parser_forbidden`

## Compile/match

- `recognizer_compile_failed`
- `recognizer_capture_schema_or_binding_invalid`
- `recognizer_join_type_or_cardinality_invalid`
- `recognizer_negative_clause_without_complete_coverage`
- `recognizer_order_or_control_flow_relation_unavailable`
- `recognizer_match_invariant_invalid`
- `recognizer_match_confidence_upgrade_forbidden`
- `recognizer_first_or_last_match_resolution_forbidden`
- `recognizer_ambiguity_record_invalid`
- `recognizer_match_budget_exceeded`
- `recognizer_output_amplification_limit_exceeded`
- `recognizer_match_nondeterministic`
- `recognizer_cancelled`
- `recognizer_late_work_after_cancel_forbidden`

## Rule-specific structural boundary

- `recognizer_custom_callback_producer_unconfirmed`
- `recognizer_native_custom_signal_conflation_forbidden`
- `recognizer_hook_safety_or_runtime_claim_forbidden`
- `recognizer_saved_variable_root_not_declared`
- `recognizer_state_path_dynamic_or_ambiguous`
- `recognizer_frame_parent_or_template_unresolved`
- `recognizer_library_identity_or_embed_unresolved`
- `recognizer_framework_lifecycle_heuristic_deferred`

These normally map to Possible/NotEvaluated/Partial outcomes rather than fatal pack failure when the rule contract permits.

## Graph proposal/handoff

- `recognizer_graph_registry_incompatible`
- `recognizer_graph_kind_relation_or_attribute_undeclared`
- `recognizer_graph_endpoint_or_direction_invalid`
- `recognizer_graph_semantic_key_ingredients_invalid`
- `recognizer_graph_proposal_invalid`
- `recognizer_graph_proposal_rejected`
- `recognizer_graph_final_identity_or_publication_forbidden`
- `recognizer_output_partition_invalid`
- `recognizer_output_partition_partial_as_complete_forbidden`

## Version/partition

- `recognizer_pack_or_rule_version_mismatch`
- `recognizer_output_partition_stale_base`
- `recognizer_stale_producer_output_retained`
- `recognizer_foreign_producer_output_mutation_forbidden`
- `recognizer_disabled_pack_coverage_not_downgraded`
- `recognizer_last_known_good_relabel_forbidden`

## Evaluation/mutation

- `recognizer_corpus_manifest_invalid`
- `recognizer_corpus_label_unverified_or_conflicting`
- `recognizer_precision_report_invalid`
- `recognizer_unknown_or_not_evaluated_hidden_from_metrics`
- `recognizer_repository_or_path_overfit_detected`
- `recognizer_decisive_literal_mutation_not_detected`
- `recognizer_near_negative_false_positive`
- `recognizer_positive_false_negative`
- `recognizer_graph_validation_rejection_in_corpus`
- `recognizer_promotion_gate_failed`
- `recognizer_fixture_or_checksum_not_frozen`

## Security/side effects

- `recognizer_filesystem_network_process_editor_access_forbidden`
- `recognizer_source_or_generated_code_execution_forbidden`
- `recognizer_dynamic_plugin_callback_regex_or_expression_forbidden`
- `recognizer_private_path_token_or_source_payload_leak_forbidden`
- `recognizer_unbounded_input_output_forbidden`
- `recognizer_cross_universe_identity_collision_forbidden`
- `recognizer_model_inference_correctness_path_forbidden`

## Deferred capability

- `recognizer_calibration_pack_not_implemented_e2_b`
- `recognizer_framework_specific_rule_not_implemented_e2_b`
- `recognizer_secret_guard_sink_rule_not_implemented_e2_b`
- `recognizer_runtime_fact_not_implemented_e2_b`
- `recognizer_search_or_semantic_candidate_not_implemented_e2_b`
- `recognizer_operation_not_implemented_for_milestone`

## Recovery classes

```text
never
after-pack-or-rule-contract-fix
after-fact-adapter-or-capability-fix
after-graph-registry-or-output-fix
after-budget-or-scope-reduction
after-corpus-label-or-fixture-review
after-versioned-rule-update
retry-exact-same-inputs
explicit-Possible-or-NotEvaluated-only
deferred-to-E2-C-E5-or-later
```

## Error rules

- human messages are projections, not identity;
- errors do not include raw source, private paths, credentials, SavedVariables contents, or arbitrary pack payloads;
- per-rule unsupported/dynamic cases remain structured outcomes when expected;
- fatal pack/input/graph-contract errors publish no complete output partition;
- cancellation is not failure or clean completion;
- errors serialize deterministically.
