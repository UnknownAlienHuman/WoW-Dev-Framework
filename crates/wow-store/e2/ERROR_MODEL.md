# E2-D store error model

**Status:** normative.

```text
ProjectStoreError
    code
    operation/stage
    Store/Profile/Bundle/Generation/Artifact/Head/Lease/GC IDs
    invocation/schema/object/member/check IDs
    expected and observed structured values
    budget/cancellation/fault state
    recovery class
```

## Profile and root

- `project_store_profile_invalid`
- `project_store_runtime_or_compile_options_incompatible`
- `project_store_filesystem_capability_unavailable`
- `project_store_root_invalid_or_not_owned`
- `project_store_path_escape_link_device_uri_or_collision`
- `project_store_private_path_or_payload_leak_forbidden`

## Bundles and operations

- `project_store_registered_bundle_invalid_or_incompatible`
- `project_store_schema_or_migration_bundle_invalid`
- `project_store_operation_catalog_invalid`
- `project_store_operation_unregistered`
- `project_store_operation_payload_invalid_or_oversized`
- `project_store_operation_plan_cycle_or_order_invalid`
- `project_store_raw_sql_connection_callback_or_pragma_forbidden`
- `project_store_expected_effect_manifest_mismatch`

## Transaction and staging

- `project_store_stale_base_or_head`
- `project_store_writer_conflict`
- `project_store_staging_creation_failed`
- `project_store_transaction_begin_failed`
- `project_store_registered_operation_failed`
- `project_store_transaction_validation_failed`
- `project_store_transaction_commit_failed`
- `project_store_transaction_rollback_failed`
- `project_store_checkpoint_or_close_failed`
- `project_store_cancelled`
- `project_store_budget_exceeded`
- `project_store_late_work_after_cancel_forbidden`

## Objects and manifests

- `project_store_object_plan_invalid`
- `project_store_object_digest_collision_or_substitution`
- `project_store_object_write_or_materialization_failed`
- `project_store_object_reference_closure_failed`
- `project_store_generation_manifest_invalid`
- `project_store_artifact_manifest_or_checksum_invalid`
- `project_store_logical_count_or_digest_mismatch`
- `project_store_manifest_identity_cycle`

## Seal/open

- `project_store_seal_precondition_failed`
- `project_store_atomic_materialization_failed`
- `project_store_mutable_wal_or_sidecar_after_seal`
- `project_store_sealed_generation_modified`
- `project_store_read_only_open_failed`
- `project_store_open_generation_mismatch`
- `project_store_sqlite_or_domain_integrity_failed`
- `project_store_registered_golden_read_failed`
- `project_store_physical_determinism_unproven`

## Head and leases

- `project_store_head_record_invalid`
- `project_store_head_target_not_sealed_or_validated`
- `project_store_head_compare_and_swap_conflict`
- `project_store_mixed_generation_head_forbidden`
- `project_store_generation_lease_invalid`
- `project_store_generation_lease_limit`
- `project_store_generation_collected_while_leased_forbidden`
- `project_store_read_handle_generation_mismatch`
- `project_store_exact_read_not_found_without_fallback`

## Recovery

- `project_store_recovery_inventory_invalid`
- `project_store_staging_state_ambiguous`
- `project_store_sealed_inactive_revalidation_failed`
- `project_store_inactive_adoption_precondition_failed`
- `project_store_current_generation_corrupt`
- `project_store_last_known_good_relabel_forbidden`
- `project_store_manual_recovery_required`
- `project_store_in_place_repair_or_migration_forbidden`

## Retention and GC

- `project_store_retention_root_invalid`
- `project_store_gc_plan_invalid_or_stale`
- `project_store_gc_reachability_incomplete`
- `project_store_gc_current_pinned_leased_or_evidence_root_forbidden`
- `project_store_gc_object_still_referenced`
- `project_store_age_only_gc_forbidden`
- `project_store_gc_execution_failed`
- `project_store_gc_cancelled`

## Recovery classes

```text
never
after-profile-or-root-fix
after-bundle-or-plan-fix
retry-exact-base-and-head
retry-same-staging-before-seal-if-safe
rebuild-new-generation
revalidate-sealed-inactive
quarantine-and-investigate
retain-conservatively
manual-registry-recovery
smaller-explicit-budget
```

Errors never contain raw SQL, full database pages, private paths, credentials, source bodies, or arbitrary repository text.
