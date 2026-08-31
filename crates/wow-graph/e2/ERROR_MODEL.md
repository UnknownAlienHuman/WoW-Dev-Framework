# E2-A graph error model

**Status:** normative.

```text
GraphError
    code
    operation/stage
    graph/registry/snapshot/partition/producer IDs
    entity/relation/assertion/conflict/coverage/query IDs
    structured message arguments
    budget/cancellation state
    recovery class
```

## Registry

- `graph_registry_bundle_invalid`
- `graph_entity_kind_invalid_or_duplicate`
- `graph_relation_kind_invalid_or_duplicate`
- `graph_attribute_definition_invalid`
- `graph_axis_definition_invalid`
- `graph_registry_breaking_change_unversioned`
- `graph_generic_parent_semantics_forbidden`

## Identity/assertions

- `graph_entity_key_invalid`
- `graph_relation_key_invalid`
- `graph_assertion_invalid`
- `graph_assertion_endpoint_unresolved`
- `graph_assertion_cross_scope_forbidden`
- `graph_assertion_confidence_or_provenance_invalid`
- `graph_assertion_evidence_or_coverage_invalid`
- `graph_assertion_derivation_cycle`
- `graph_attribute_schema_or_budget_invalid`

## Partitions/publication

- `graph_partition_batch_invalid`
- `graph_partition_stale_base`
- `graph_partition_registry_or_generation_mismatch`
- `graph_partition_replacement_plan_invalid`
- `graph_partition_partial_publication_forbidden`
- `graph_snapshot_manifest_invalid`
- `graph_snapshot_publication_failed`
- `graph_snapshot_post_open_validation_failed`
- `graph_prior_snapshot_modified_or_relabelled`

## Conflicts/coverage

- `graph_conflict_record_invalid`
- `graph_forbidden_cycle_or_multiplicity`
- `graph_coverage_incomplete_or_inconsistent`
- `graph_negative_authority_unavailable`
- `graph_candidate_authority_upgrade_forbidden`
- `graph_assertion_overwrite_or_last_write_wins_forbidden`

## Queries

- `graph_query_invalid`
- `graph_query_snapshot_mismatch`
- `graph_query_relation_or_axis_invalid`
- `graph_query_budget_invalid_or_exceeded`
- `graph_query_truncated`
- `graph_query_cancelled`
- `graph_query_cursor_invalid_or_stale`
- `graph_query_unbounded_export_forbidden`
- `graph_query_not_evaluated`

## Store/security

- `graph_store_operation_or_schema_invalid`
- `graph_store_generation_mismatch`
- `graph_store_logical_integrity_failed`
- `graph_raw_sql_or_storage_handle_forbidden`
- `graph_source_or_executable_input_forbidden`
- `graph_filesystem_network_process_editor_access_forbidden`
- `graph_private_data_leak_forbidden`
- `graph_unbounded_input_output_forbidden`

## Recovery classes

```text
never
after-registry-contract-fix
after-producer-batch-fix
after-evidence-or-coverage-fix
after-store-or-publication-fix
retry-exact-base-and-inputs
query-with-smaller-explicit-budget
explicit-partial-or-candidate-query-only
safe-cleanup-or-quarantine
```
