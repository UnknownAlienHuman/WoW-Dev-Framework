# E2-D integrated publication error model

**Status:** normative.

```text
ProjectPublicationError
    code
    operation/phase
    request/candidate/base/head IDs
    profile/reference/project/analyzer/recognizer/graph/store IDs
    bundle/plan/snapshot/coherence/report IDs
    capability/conflict/coverage blockers
    budget/cancellation state
    recovery class
```

## Request and candidate

- `project_e2d_publication_request_invalid`
- `project_e2d_candidate_invalid_or_not_e2c`
- `project_e2d_candidate_generation_or_digest_mismatch`
- `project_e2d_candidate_not_publication_eligible`
- `project_e2d_profile_or_reference_mismatch`
- `project_e2d_floating_current_latest_or_fallback_forbidden`
- `project_e2d_stale_expected_head_or_base`
- `project_e2d_required_capability_unavailable`

## Analyzer/recognizer

- `project_e2d_analyzer_snapshot_or_manifest_mismatch`
- `project_e2d_analyzer_source_handle_closure_failed`
- `project_e2d_recognizer_pack_rule_or_result_mismatch`
- `project_e2d_recognizer_partition_partial_or_stale`
- `project_e2d_recognizer_output_rewrite_forbidden`

## Graph

- `project_e2d_graph_registry_or_base_mismatch`
- `project_e2d_graph_proposal_validation_missing_or_stale`
- `project_e2d_graph_replacement_plan_invalid`
- `project_e2d_graph_rejected_proposal_hidden_forbidden`
- `project_e2d_graph_conflict_or_coverage_blocks_publication`
- `project_e2d_graph_snapshot_or_golden_query_mismatch`
- `project_e2d_graph_semantics_rewrite_forbidden`

## Bundle and store handoff

- `project_e2d_logical_write_plan_invalid`
- `project_e2d_publication_bundle_invalid`
- `project_e2d_registered_operation_collision_or_order_invalid`
- `project_e2d_raw_sql_store_handle_or_path_forbidden`
- `project_e2d_store_generation_build_failed`
- `project_e2d_store_generation_not_sealed_or_open_validated`
- `project_e2d_store_generation_or_artifact_mismatch`

## Post-open and snapshots

- `project_e2d_project_post_open_validation_failed`
- `project_e2d_removed_or_stale_record_present`
- `project_e2d_project_snapshot_manifest_invalid`
- `project_e2d_graph_snapshot_manifest_invalid`
- `project_e2d_coherence_manifest_invalid`
- `project_e2d_snapshot_identity_cycle`
- `project_e2d_cross_generation_mix_forbidden`
- `project_e2d_physical_success_overrides_domain_failure_forbidden`

## Head and publication

- `project_e2d_publication_head_invalid`
- `project_e2d_separate_project_graph_heads_forbidden`
- `project_e2d_head_compare_and_swap_conflict`
- `project_e2d_head_cas_result_ambiguous`
- `project_e2d_headed_generation_not_coherent`
- `project_e2d_already_published`
- `project_e2d_last_known_good_relabel_forbidden`

## Recovery/read

- `project_e2d_inactive_generation_not_adoptable`
- `project_e2d_recovery_revalidation_failed`
- `project_e2d_current_generation_corrupt`
- `project_e2d_published_view_coherence_failed`
- `project_e2d_exact_head_or_generation_not_found`
- `project_e2d_generation_lease_or_close_failed`

## Policy/security

- `project_e2d_partial_publication_policy_invalid`
- `project_e2d_cancelled`
- `project_e2d_budget_exceeded`
- `project_e2d_late_work_after_cancel_forbidden`
- `project_e2d_source_analyzer_recognizer_late_read_forbidden`
- `project_e2d_source_execution_network_process_editor_client_forbidden`
- `project_e2d_runtime_or_client_validation_claim_forbidden`
- `project_e2d_ci_or_release_automation_forbidden`

## Recovery classes

```text
never
after-request-or-candidate-fix
after-component-generation-or-manifest-fix
after-graph-plan-or-conflict-resolution
after-store-profile-or-build-fix
retry-fresh-request-against-current-head
revalidate-sealed-inactive
retain-last-known-good-explicitly
quarantine-and-investigate
manual-current-registry-recovery
smaller-explicit-budget
```
