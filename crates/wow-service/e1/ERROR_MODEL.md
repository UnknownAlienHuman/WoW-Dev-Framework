# E1-D error model

**Status:** normative typed pack orchestration, assembly, validation, determinism, recovery, and application error vocabulary.

## Shape

```text
ReferencePackServiceError
    code
    operation/stage
    request/build/candidate/pack IDs
    profile/reference/component/layout IDs
    member/path/store/annotation/map/loss/parity/consumer/gate IDs
    budget/cancellation state
    structured message arguments
    recovery class
```

Human text is not the contract and must not expose raw source, private roots, tokens, credentials, or unbounded component output.

## Request and prerequisite errors

- `pack_build_request_invalid`
- `pack_validation_request_invalid`
- `pack_rebuild_request_invalid`
- `pack_implicit_current_or_profile_forbidden`
- `pack_component_contract_or_implementation_unfrozen`
- `pack_component_incompatible`
- `pack_profile_or_generation_mismatch`
- `pack_required_capability_unavailable`
- `pack_layout_profile_invalid`
- `pack_budget_invalid`

## Source and staging errors

- `pack_source_snapshot_invalid`
- `pack_source_root_escape_or_manifest_mismatch`
- `pack_staging_policy_invalid`
- `pack_staging_root_invalid_or_overlap`
- `pack_destination_conflict`
- `pack_unsafe_path_or_symlink`
- `pack_materialization_entry_invalid`
- `pack_materialization_failed`
- `pack_materialized_digest_mismatch`
- `pack_atomic_finalization_failed`

## Component orchestration errors

- `pack_reference_build_failed`
- `pack_reference_data_blocked`
- `pack_reference_store_validation_failed`
- `pack_reference_view_validation_failed`
- `pack_annotation_build_failed`
- `pack_annotation_artifact_blocked`
- `pack_annotation_source_map_or_loss_invalid`
- `pack_parity_or_consumer_gate_failed`
- `pack_component_result_contract_violation`
- `pack_component_fallback_or_substitution_forbidden`

## Assembly and manifest errors

- `pack_required_member_missing`
- `pack_undeclared_member_present`
- `pack_member_duplicate_or_path_collision`
- `pack_member_kind_or_profile_invalid`
- `pack_member_checksum_mismatch`
- `pack_manifest_invalid`
- `pack_manifest_identity_cycle`
- `pack_checksum_manifest_invalid`
- `pack_license_or_provenance_incomplete`
- `pack_redistribution_forbidden_or_unknown`
- `pack_eligibility_claim_invalid`
- `pack_partial_candidate_exposed_as_validated`

## Validation errors

- `pack_validation_root_invalid`
- `pack_validation_nonrepairing_contract_violated`
- `pack_store_readonly_or_integrity_invalid`
- `pack_reference_golden_query_mismatch`
- `pack_annotation_file_or_syntax_invalid`
- `pack_generated_source_map_invalid`
- `pack_projection_loss_or_coverage_invalid`
- `pack_oracle_or_consumer_result_invalid`
- `pack_mandatory_check_not_evaluated`

## Determinism errors

- `pack_semantic_rebuild_mismatch`
- `pack_canonical_bytes_mismatch`
- `pack_annotation_bytes_mismatch`
- `pack_object_identity_mismatch`
- `pack_store_logical_mismatch`
- `pack_sqlite_physical_determinism_overclaimed`
- `pack_archive_determinism_not_profiled`
- `pack_nondeterministic_member_order_or_path`
- `pack_volatile_host_state_in_identity`

## Security errors

- `pack_source_or_generated_code_execution_forbidden`
- `pack_filesystem_network_shell_editor_access_forbidden`
- `pack_private_path_or_credential_leak`
- `pack_unbounded_input_output_or_decompression`
- `pack_malicious_component_or_probe_output`
- `pack_publication_signing_or_upload_forbidden_e1`

## Lifecycle errors

- `pack_build_stage_invalid`
- `pack_build_cancelled`
- `pack_late_work_after_cancel_forbidden`
- `pack_prior_destination_modified_on_failure`
- `pack_partial_cleanup_or_quarantine_failed`
- `pack_recovery_identity_insufficient`
- `pack_operation_not_implemented_for_milestone`

## Recovery classes

```text
never
after-request-fix
after-component-freeze-or-compatibility-fix
after-source-snapshot-fix
after-reference-rebuild
after-annotation-rebuild
after-layout-or-license-policy-fix
after-candidate-rematerialization
after-validator-or-implementation-fix
safe-cleanup-or-quarantine-only
retry-exact-inputs
```

## Fatal versus scoped

Fatal:

- profile/component/source identity mismatch;
- security/path violation;
- component contract violation;
- manifest/checksum/store/source-map closure failure;
- prior destination mutation;
- mandatory determinism failure.

Scoped blocker:

- optional member unavailable;
- advisory projection loss;
- nonmandatory consumer/oracle profile failure;
- candidate-only capability.

Scoped blockers remain visible and cannot satisfy a stronger target.
