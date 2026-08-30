# `wow-store` E1-A error model

**Status:** normative typed storage/schema/transaction/publication/object/integrity/security failure vocabulary.

Human messages are projections. Callers/tests inspect structured codes, store/build/generation/schema/migration/transaction/object/publication/integrity IDs, exact state, and recovery class.

## 1. Error shape

```text
StoreError
    code
    operation
    store kind/ID: optional
    build/candidate/staging/sealed/generation/pointer IDs: optional
    schema registry/bundle/migration/operation IDs: optional
    SQLite runtime profile/probe ID: optional
    transaction ID: optional
    ObjectId/payload/reference/lease IDs: optional
    manifest/integrity/publication/retention report IDs: optional
    previous active/last-known-good/failed target IDs: optional
    budget/cancellation state
    message arguments
    recovery class
```

`recovery_class`:

```text
never
after_configuration_fix
after_runtime_profile_or_capability_fix
after_schema_or_migration_fix
after_rebuild_candidate
after_reacquire_or_restore_generation
after_release_references_or_leases
after_contract_or_implementation_fix
retry_idempotent_same_identity
```

Public errors exclude raw SQL, private paths, source/object bytes, credentials, and memory handles.

## 2. Configuration/root errors

### `store_configuration_invalid`

Missing/contradictory roots/runtime/schema/publication/object/retention/budget policy.

### `store_root_invalid`

Configured root missing/incompatible/not permitted/wrong volume.

### `store_path_invalid`

Malformed/absolute/traversal/device/reserved/overlong/invalid-encoding component.

### `store_path_escape_forbidden`

Resolved/opened path escapes configured root through normalization/link/reparse/race.

### `store_publication_volume_mismatch`

Staging/final/pointer paths cannot satisfy selected atomic publication contract.

### `store_platform_adapter_invalid`

Adapter identity/capability/behavior inconsistent with configuration.

### `store_budget_policy_invalid`

Invalid/contradictory resource/cancellation bounds.

## 3. SQLite runtime/profile errors

### `sqlite_runtime_unprobed`

No accepted exact SQLite/binding capability report.

### `sqlite_runtime_profile_invalid`

Version/compile options/open flags/PRAGMA/limits/platform mismatch.

### `sqlite_required_capability_unavailable`

Required URI/read-only/foreign-key/transaction/integrity/defensive/virtual-table/etc. capability absent.

### `sqlite_effective_pragma_mismatch`

Set/read effective value differs from contract.

### `sqlite_extension_loading_forbidden`

Extension loading requested/enabled/possible through exposed path.

### `sqlite_attach_forbidden`

ATTACH/DETACH or nonowned DB path attempted.

### `sqlite_foreign_keys_disabled`

Relevant connection does not enforce foreign keys.

### `sqlite_open_mode_invalid`

Staging/reference/project open flags/state do not match store kind/state.

### `sqlite_reference_write_capability_forbidden`

Sealed/published ReferenceStore opened/mutated with write capability.

### `sqlite_reference_sidecar_forbidden`

Journal/WAL/SHM/temp sidecar present/created for sealed artifact.

### `sqlite_busy_or_lock_unexpected`

Lock/busy state violates one-writer/read-only lifecycle or exceeds policy.

### `sqlite_runtime_limit_unavailable`

Required runtime bound cannot be enforced and no accepted outer-layer bound exists.

### `sqlite_error_normalization_failed`

Binding/raw error cannot be classified safely/deterministically.

## 4. Schema/registry errors

### `schema_bundle_invalid`

Malformed namespace/version/objects/capabilities/migrations/operations/checks/digest.

### `schema_bundle_duplicate_or_conflict`

Namespace/version/object/operation collision.

### `schema_bundle_digest_mismatch`

Canonical recomputation mismatch/tampering.

### `schema_registry_invalid`

Bundle set/dependency/identity/digest invalid.

### `schema_object_forbidden_or_unexpected`

Unexpected/missing/reserved/temp/attached/unsupported schema object.

### `schema_required_sqlite_capability_unavailable`

Bundle requires unsupported feature.

### `prepared_operation_catalog_invalid`

Static operation/parameter/result/store-state/transaction/cardinality/digest invalid.

### `prepared_operation_unknown_or_forbidden`

Unknown operation or wrong store kind/state/catalog.

### `raw_sql_surface_forbidden`

Unregistered/dynamic SQL from user/source/consumer/transport attempted.

### `domain_semantics_in_store_forbidden`

Store implementation imports/interprets domain entities/relations/restrictions/findings/project logic.

## 5. Migration errors

### `migration_graph_invalid`

Cycle/missing endpoint/duplicate/ambiguous unsupported path.

### `migration_plan_invalid`

Source/target/edge order/capabilities/preconditions/expected digest inconsistent.

### `migration_source_state_unknown`

Current schema/ledger cannot be established exactly.

### `migration_unknown_or_tampered_edge`

Ledger/catalog edge missing/unknown/digest mismatch.

### `migration_skip_forbidden`

Version advanced without exact edge path.

### `migration_target_metadata_advanced_early_forbidden`

Target metadata/ledger committed before corresponding schema/data operation.

### `migration_nontransactional_operation_unprobed`

Implicit/explicit auto-commit or unsupported operation lacks staged interruption contract.

### `migration_apply_failed`

Registered edge operation failed; candidate transaction rolled back.

### `migration_ledger_invalid`

Missing/duplicate/out-of-order/digest/schema mismatch.

### `migration_target_schema_mismatch`

Normalized target schema differs from expected digest/object set.

### `migration_released_reference_in_place_forbidden`

Attempt to alter sealed/published ReferenceStore.

### `migration_force_or_repair_forbidden`

Caller tries to skip/force/repair unknown state.

## 6. Transaction/build errors

### `store_build_request_invalid`

Profile/reference/schema/domain operation/object/budget identities invalid.

### `store_candidate_state_invalid`

State transition/identity inconsistent.

### `staging_store_create_failed`

Private staging DB/directory cannot be safely created.

### `store_write_transaction_invalid`

Wrong candidate/state/base/catalog/owner/lifecycle.

### `store_operation_batch_invalid`

Unknown/duplicate/out-of-order/over-budget operation/parameters/cardinality.

### `store_transaction_commit_failed`

Commit failed/uncertain; candidate cannot advance.

### `store_transaction_rollback_failed`

Rollback state uncertain; quarantine candidate.

### `store_cancelled`

Cancelled before publication; no active pointer change.

### `store_late_work_after_cancel_forbidden`

Background/late mutation/publication after cancellation.

### `store_write_after_seal_forbidden`

Any DB/object/manifest mutation after seal.

### `store_partial_candidate_exposure_forbidden`

Staging/candidate visible through reader/active path.

## 7. Validation/integrity errors

### `store_validation_report_invalid`

Mandatory check set/result/reference/digest inconsistent.

### `store_mandatory_check_unavailable`

Required integrity/foreign-key/domain/file/object check skipped/unavailable.

### `store_schema_integrity_failed`

Registry/schema object/migration ledger mismatch.

### `store_foreign_key_integrity_failed`

FK violation/check failure.

### `store_database_integrity_failed`

SQLite quick/integrity check failure/corruption.

### `store_domain_validation_failed`

Registered consumer validation check failed.

### `store_manifest_invalid`

Missing/contradictory store/profile/schema/file/object/publication fields/references.

### `store_manifest_digest_mismatch`

Canonical manifest digest mismatch.

### `store_file_digest_or_length_mismatch`

SQLite file changed/corrupt/mismatched.

### `store_reference_closure_invalid`

Manifest/schema/object/report IDs unresolved/cross-generation.

### `store_corruption_auto_repair_forbidden`

Implementation attempts silent mutation/repair into trusted state.

## 8. Seal/publication errors

### `store_seal_preconditions_failed`

Active transaction/sidecar/missing validation/object/manifest/budget/cancellation blocker.

### `store_seal_failed`

Cannot establish immutable complete sealed generation.

### `store_generation_id_invalid`

Identity construction/self-reference/noncanonical input mismatch.

### `store_generation_publish_failed`

Final-path generation publication failed; prior active unchanged.

### `store_generation_existing_mismatch`

Same generation path/ID exists with different manifest/file/object content.

### `store_generation_overwrite_forbidden`

Attempt to overwrite distinct published generation.

### `store_final_path_revalidation_failed`

Published path cannot reopen/validate; no pointer update.

### `store_active_pointer_invalid`

Malformed/cross-store/missing/mismatched generation/path/manifest/digest.

### `store_active_pointer_publish_failed`

Pointer replacement/flush/reopen failed; previous pointer remains authoritative or state classified exactly.

### `store_pointer_to_missing_or_partial_generation_forbidden`

Pointer target not fully published/validated.

### `store_publication_state_transition_invalid`

State skipped/reordered/repeated incompatibly.

### `store_durability_claim_unproven`

Recorded/requested level exceeds tested adapter behavior.

### `store_last_known_good_relabel_forbidden`

Old generation represented as failed/requested target.

## 9. ReferenceStore errors

### `reference_store_identity_mismatch`

Store/profile/reference/schema/runtime/object identities disagree.

### `reference_store_not_sealed`

Attempt to publish/open candidate/unsealed state.

### `reference_store_open_validation_failed`

Read-only open/manifest/schema/file/object/integrity validation failure.

### `reference_store_mutation_forbidden`

Write/DDL/pragma persistent mutation attempted.

### `reference_store_in_place_update_forbidden`

Domain rows/schema/metadata changed after seal/release.

### `reference_store_reader_generation_switch_forbidden`

Reader switches when active pointer changes.

### `reference_store_absence_as_authority_forbidden`

Store infers domain negative authority from missing row/table.

## 10. Object errors

### `object_id_invalid`

Malformed/noncanonical ObjectId.

### `object_logical_digest_or_length_mismatch`

Written/read decoded bytes do not match ObjectId/length.

### `object_codec_unregistered_or_unsupported`

Unknown/inactive codec/profile.

### `object_payload_digest_or_length_mismatch`

Encoded payload corrupt/mismatched.

### `object_expansion_or_resource_limit_exceeded`

Decode/compression/object budget exceeded.

### `object_path_invalid_or_escape_forbidden`

Derived/temp/final path violates root policy.

### `object_write_failed`

Temp/encode/flush/verify/publication failed.

### `object_existing_mismatch`

Same ObjectId/payload path exists but content/manifest mismatch.

### `object_overwrite_forbidden`

Attempt to replace mismatch/collision/corrupt existing payload.

### `object_manifest_invalid`

Logical/payload/type/codec/path/reference fields/digest invalid.

### `object_reference_set_invalid`

Missing/duplicate/cross-generation/undigested references.

### `object_reference_before_publication_forbidden`

Generation references unverified/unpublished object.

### `object_missing_or_corrupt`

Required referenced object unavailable/invalid.

### `object_gc_reference_state_incomplete`

Cannot prove complete retained-generation/reference scan.

### `object_gc_lease_or_retention_blocked`

Active/last-known-good/configured/lease protection.

### `object_gc_delete_forbidden`

Eligibility not `yes`, path/revalidation invalid, or referenced object targeted.

### `object_gc_age_only_decision_forbidden`

Age/name used as authority without complete references/leases.

### `object_gc_failed`

Bounded delete/report operation failed; completed deletions retained in report.

## 11. Retention/reader errors

### `store_generation_retention_state_invalid`

Active/last-known-good/published/lease/configured state inconsistent.

### `store_generation_delete_forbidden`

Active/retained/leased/uncertain generation targeted.

### `store_reader_lease_invalid`

Lease/context/generation lifecycle invalid.

### `store_cross_process_lease_unproven`

Process-local lease assumed to protect another process.

## 12. ProjectStore deferred errors

### `project_store_operation_not_implemented_e1`

Any ProjectStore open/write/WAL/read snapshot/checkpoint/backup/GC operation in E1-A.

### `project_store_physical_model_unselected`

Attempt to encode file-per-generation/versioned-row/hybrid choice before E2 contract.

### `project_store_wal_policy_in_reference_forbidden`

WAL behavior applied to ReferenceStore.

Canonical external deferred error can be `operation_not_implemented_for_milestone` with these structured details.

## 13. Security errors

### `store_extension_loading_forbidden`
### `store_attach_or_external_db_write_forbidden`
### `store_dynamic_or_untrusted_sql_forbidden`
### `store_source_or_repository_code_execution_forbidden`
### `store_network_process_shell_editor_client_access_forbidden`
### `store_path_link_race_forbidden`
### `store_private_path_or_payload_leak_forbidden`
### `store_untrusted_instruction_ignored`

Any security/policy violation blocks publication/open and is never blind-retried.

## 14. Determinism errors

### `store_nondeterministic_schema_or_migration`
### `store_nondeterministic_logical_manifest`
### `store_nondeterministic_object_identity`
### `store_nondeterministic_publication_record`
### `store_physical_byte_reproducibility_unproven`

The last code is not necessarily failure if the contract claims only logical determinism; it becomes error when release/build claims byte identity without proof.

## 15. Deferred operation error

### `operation_not_implemented_for_milestone`

E1-A requests:

```text
ProjectStore/WAL/mutable generation APIs
ExternalStore
FTS/search-specific store behavior
release signing/distribution
network/download/import generic external DB
```

No default/empty success.

## 16. Fatal versus candidate-local

### Fatal for build/open/publication

```text
configuration/runtime/schema/migration mismatch
transaction/rollback uncertainty
mandatory integrity/file/object failure
seal/publication/pointer validation failure
path/security violation
resource limit preventing complete artifact
```

### Candidate-local recoverable

Candidate failure before publication can be cleaned/quarantined/rebuilt while old active remains. Still no successful publication result.

### Reader-local

Read operation parameter/budget/cancellation failure does not mutate store/generation; open/store corruption may invalidate generation for further use.

## 17. Recovery rules

- configuration/runtime -> fix/reprobe;
- schema/migration -> fix bundle/plan/rebuild candidate;
- candidate write/integrity -> abort/quarantine/rebuild;
- generation published/pointer failed -> revalidate/retry pointer with same identity;
- active corruption -> stop using, retain evidence, select prior generation/rebuild explicitly;
- object mismatch -> quarantine/no overwrite/rebuild/reacquire;
- retention/lease block -> defer deletion;
- ProjectStore request -> wait for E2;
- security violation -> never blind retry.

## 18. Error tests

Every E1-used code requires:

- dedicated fixture/mutation/crash point;
- exact store/build/schema/migration/object/publication IDs and state;
- previous active/last-known-good preservation assertion;
- deterministic serialization;
- no private/raw SQL/path/payload leak;
- recovery class;
- assertion that no generation/pointer/reference/object deletion escaped when forbidden.
