# E2-D ProjectStore error model

**Status:** normative typed vocabulary.

```text
ProjectStoreError
    code
    operation/stage
    operation ID/request digest/attempt ID when applicable
    StoreId/EpochId/StoreGenerationId
    base/target CurrentPublicationRecord IDs
    ProjectGeneration/ProjectSnapshot IDs
    GraphGeneration/GraphSnapshot IDs
    publication set/partition/object/schema/catalog/query/cursor IDs
    budget/cancellation/durability state
    structured arguments
    recovery class
```

Public errors omit SQL text, private paths, tokens, raw source, object payloads, and runtime secret-capable values.

## Configuration/profile

- `project_store_configuration_invalid`
- `project_store_physical_profile_invalid`
- `project_store_runtime_profile_unprobed_or_mismatch`
- `project_store_effective_pragma_mismatch`
- `project_store_platform_or_filesystem_profile_invalid`
- `project_store_external_reader_or_multiple_writer_unsupported`
- `project_store_epoch_incompatible`
- `project_store_epoch_registry_invalid`
- `project_store_epoch_upgrade_requires_rebuild`

## Schema/catalog

- `project_store_schema_set_invalid`
- `project_store_schema_bundle_missing_or_mismatch`
- `project_store_operation_catalog_missing_or_mismatch`
- `project_store_validation_catalog_missing_or_mismatch`
- `project_store_unregistered_operation`
- `project_store_raw_sql_or_identifier_input_forbidden`
- `project_store_schema_or_catalog_owner_violation`
- `project_store_schema_integrity_failed`

## Operation/idempotency

- `project_store_operation_record_invalid`
- `project_store_operation_state_invalid`
- `project_store_idempotency_key_conflict`
- `project_store_existing_operation_target_mismatch`
- `project_store_response_loss_reconciliation_failed`
- `project_store_retry_requires_exact_revalidation`
- `project_store_quarantine_required`

## Writer/base/transaction

- `project_store_writer_unavailable_or_busy`
- `project_store_second_writer_forbidden`
- `project_store_stale_base_current_record`
- `project_store_silent_rebase_or_merge_forbidden`
- `project_store_write_transaction_failed`
- `project_store_transaction_state_invalid`
- `project_store_partial_commit_or_autocommit_forbidden`
- `project_store_cancelled`
- `project_store_late_background_work_forbidden`

## Partitions/membership

- `project_store_partition_key_invalid`
- `project_store_partition_version_invalid`
- `project_store_partition_existing_content_mismatch`
- `project_store_partition_write_after_seal`
- `project_store_partition_manifest_or_row_mismatch`
- `project_store_partition_reference_closure_failed`
- `project_store_generation_membership_incomplete`
- `project_store_generation_membership_duplicate_or_conflicting`
- `project_store_recursive_delta_chain_forbidden`
- `project_store_cross_generation_row_leakage`

## Publication/binding

- `project_store_publication_set_invalid`
- `project_store_project_graph_analyzer_binding_mismatch`
- `project_store_generation_identity_invalid`
- `project_store_inactive_generation_build_failed`
- `project_store_inactive_generation_not_found_or_changed`
- `project_store_inactive_validation_failed`
- `project_store_activation_without_validation_forbidden`
- `project_store_activation_cas_failed`
- `project_store_current_record_invalid`
- `project_store_current_record_points_to_invalid_generation`
- `project_store_mixed_generation_view_forbidden`
- `project_store_last_known_good_relabel_forbidden`
- `project_store_rollback_requires_explicit_validated_cas`
- `project_store_already_current`
- `project_store_existing_generation_collision`

## Reader/query

- `project_store_read_snapshot_failed`
- `project_store_read_snapshot_generation_mismatch`
- `project_store_read_generation_not_retained`
- `project_store_reader_switched_generation_forbidden`
- `project_store_generation_lease_invalid`
- `project_store_lease_admission_or_gc_race`
- `project_store_query_catalog_or_ordering_mismatch`
- `project_store_continuation_invalid_or_stale`
- `project_store_raw_connection_or_cursor_leak_forbidden`
- `project_store_registered_read_budget_exceeded`
- `project_store_domain_negative_authority_forbidden`

## WAL/checkpoint/locking

- `project_store_wal_mode_or_sidecar_invalid`
- `project_store_wal_unbounded`
- `project_store_checkpoint_busy`
- `project_store_checkpoint_failed`
- `project_store_checkpoint_changed_logical_state`
- `project_store_busy_retry_unbounded_forbidden`
- `project_store_durability_claim_unproven`
- `project_store_windows_sharing_violation`

## Integrity/recovery/backup

- `project_store_sqlite_integrity_failed`
- `project_store_foreign_key_or_reference_integrity_failed`
- `project_store_object_reference_integrity_failed`
- `project_store_recovery_state_ambiguous`
- `project_store_recovery_budget_exceeded`
- `project_store_current_corrupt`
- `project_store_inactive_recovery_ineligible`
- `project_store_automatic_repair_forbidden`
- `project_store_backup_incomplete_or_invalid`
- `project_store_main_db_copy_without_wal_forbidden`
- `project_store_restore_identity_mismatch`
- `project_store_rebuild_required`

## Retention/GC

- `project_store_retention_snapshot_invalid`
- `project_store_current_or_leased_generation_gc_forbidden`
- `project_store_partition_still_reachable`
- `project_store_object_still_referenced`
- `project_store_gc_plan_invalid`
- `project_store_gc_plan_stale`
- `project_store_gc_partial_or_integrity_failed`
- `project_store_age_only_gc_forbidden`
- `project_store_epoch_gc_forbidden`

## Security/budgets

- `project_store_path_escape_or_collision`
- `project_store_untrusted_database_or_extension_forbidden`
- `project_store_filesystem_network_process_editor_client_access_forbidden`
- `project_store_private_data_or_source_payload_leak_forbidden`
- `project_store_runtime_secret_or_savedvariables_content_forbidden`
- `project_store_input_output_or_resource_budget_exceeded`

## Benchmark/freeze

- `project_store_benchmark_profile_invalid`
- `project_store_benchmark_gate_failed`
- `project_store_profile_changed_without_contract_revision`
- `project_store_fixture_or_checksum_not_frozen`
- `project_store_logical_determinism_failed`
- `project_store_physical_determinism_overclaimed`
- `project_store_superseded_physical_model_forbidden`

## Recovery classes

```text
never
after-request-or-profile-fix
after-schema-or-catalog-fix
retry-same-operation-after-durable-state-read
retry-exact-base
retry-after-reader-lock-or-sharing-release
revalidate-existing-inactive-generation
explicit-rollback-to-retained-validated-generation
rebuild-new-generation
rebuild-new-epoch
restore-validated-backup
quarantine-and-investigate
run-with-smaller-explicit-budget
```
