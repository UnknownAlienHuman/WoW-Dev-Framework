# `wow-project` E0-D error model

**Status:** normative typed project-generation/publication failure vocabulary.

Messages are projections. Callers/tests inspect structured codes, project/generation/configuration/file/analyzer context, capability/partition blockers, and recovery class.

## 1. Error shape

```text
ProjectError
    code
    operation
    project_id: optional
    current_project_generation: optional
    target_project_generation: optional
    project_snapshot_digest: optional
    profile_id: optional
    reference_generation: optional
    analyzer_pin_id: optional
    analyzer_snapshot_id: optional
    workspace_id: optional
    source_origin_id: optional
    file_ids[]
    capability_ids[]
    partition_ids[]
    message_arguments
    recovery_class
```

`recovery_class`:

```text
never
after_request_fix
after_input_fix
after_analyzer_recovery
after_configuration_change
rebuild_candidate
retry_same_expected_generation_if_cancelled
```

## 2. Configuration errors

### `project_configuration_invalid`

Required fields missing, contradictory, unsupported, or noncanonical.

### `project_fixture_release_masquerade`

Fixture project used/marked as release repository project.

### `project_profile_floating_identity`

Project configuration uses floating current/latest/unpinned profile/reference.

### `project_reference_generation_mismatch`

Selected ProfileIdentity/reference generation disagree with supplied context.

### `project_analyzer_binding_invalid`

Pin/probe/config/fixture contract IDs missing, unaccepted, or contradictory.

### `project_capability_policy_invalid`

Mandatory/degradable/deferred sets overlap illegally or omit required publication capabilities.

### `project_budget_policy_invalid`

Invalid/negative/unsupported bounds or output-affecting budget excluded from generation identity.

### `project_deferred_dependency_forbidden`

E0-D attempts to activate `wow-store`, `wow-graph`, or `wow-recognizers`.

### `project_deferred_capability_fake_success`

TOC/XML/load/graph capability returned complete/empty success in E0-D.

## 3. Input inventory/source errors

### `project_input_inventory_invalid`

Duplicate/missing file IDs, inconsistent workspace/origin, or unresolved declarations.

### `project_input_missing`

Declared project file missing.

### `project_input_undeclared`

Supplied file not in closed inventory or authorized update.

### `project_file_id_invalid`

File ID/path/origin/workspace grammar invalid.

### `project_file_path_invalid`

Absolute/traversal/device/UNC/tokenized/invalid relative path.

### `project_file_case_collision`

Two paths collide under declared cross-platform case policy.

### `project_file_role_invalid`

Library/external/runtime/unknown file attempts to enter first-party Main registry.

### `project_file_language_invalid`

Unsupported non-Lua file in E0-D.

### `project_file_not_utf8`

Source bytes invalid UTF-8.

### `project_file_digest_mismatch`

Supplied content differs from declared/expected digest.

### `project_file_length_mismatch`

Supplied byte length mismatch.

### `project_source_origin_invalid`

Origin/root/project/workspace identity invalid or leaks host-specific path.

### `project_source_registry_invalid`

Registry references, mappings, role, generation, or digest invariants fail.

### `project_source_handle_invalid`

Source handle does not map to current registered file/content/generation.

### `project_source_handle_role_mismatch`

Library/reference/external handle claims project Main origin/role.

### `project_source_payload_leak_forbidden`

Default error/output attempts to expose excessive/private source/path/token data.

## 4. Generation errors

### `project_generation_derivation_invalid`

Canonical derivation inputs incomplete, contradictory, volatile, or unsupported.

### `expected_project_generation_mismatch`

Update/request expected current generation differs from published current state.

### `project_generation_candidate_invalid`

Candidate derivation/input references/digests fail validation.

### `project_generation_volatile_input_forbidden`

Timestamp/temp path/process/thread/memory/random/log text included in identity.

### `project_generation_same_state_mismatch`

Equivalent final logical state derives different generation IDs.

### `project_generation_different_state_collision`

Different semantic inputs derive same generation identity.

### `project_generation_cross_context_forbidden`

Facts/findings/source handles/config from another generation mixed into candidate/snapshot.

## 5. Update errors

### `project_update_request_invalid`

Malformed project/update/configuration request.

### `project_update_conflicting_operations`

Multiple ambiguous/order-dependent operations for one file.

### `project_update_add_existing_file`

Add target already exists.

### `project_update_target_missing`

Update/remove target absent.

### `project_update_expected_digest_mismatch`

Current file digest differs from update/remove precondition.

### `project_update_path_scope_escape`

Add/move target outside registered root or role/type policy.

### `project_update_budget_exceeded`

Operation/final file/source/output budget exceeded.

### `project_update_no_change`

Structured NoChange outcome when final canonical state equals current; normally not an exceptional failure.

### `project_update_cancelled`

Cancelled before project publication.

### `project_update_analyzer_batch_invalid`

Constructed `wow-emmy` batch does not represent exact target project state/generation.

## 6. Analyzer binding errors

### `project_analyzer_update_failed`

Analyzer update/index/snapshot operation failed.

### `analyzer_snapshot_project_generation_mismatch`

Returned snapshot belongs to another project generation.

### `analyzer_snapshot_profile_or_reference_mismatch`

Profile/reference generation mismatch.

### `analyzer_snapshot_pin_or_configuration_mismatch`

Pin/probe/config identity mismatch.

### `analyzer_workspace_mismatch`

Main/Library workspace declaration differs from project/analyzer binding contract.

### `analyzer_file_manifest_mismatch`

Extra/missing/path/digest/length/role difference between analyzer Main files and project manifest.

### `analyzer_source_mapping_mismatch`

Analyzer project handles/files cannot map one-to-one to project source registry.

### `analyzer_capability_coverage_missing`

Required analyzer capability lacks exact coverage record.

### `analyzer_removed_file_state_retained`

Current analyzer snapshot still contains facts/findings/file for project-removed file.

### `analyzer_snapshot_invalid_for_publication`

Snapshot is structurally valid to analyzer but fails project binding/publication policy.

## 7. Snapshot/publication errors

### `project_snapshot_candidate_invalid`

Candidate project snapshot references/invariants/capabilities/digests invalid.

### `project_mandatory_capability_unavailable`

Mandatory publication capability not Complete/usable according to policy.

### `project_degraded_capability_unreported`

A partial/failed per-file analyzer capability is hidden or treated as clean.

### `project_snapshot_invalid`

Published-shape validation failure.

### `project_snapshot_digest_mismatch`

Declared canonical snapshot digest differs from recomputation.

### `project_publication_aborted`

Target publication transaction aborted after candidate derivation.

### `project_partial_publication_forbidden`

File/analyzer/coverage/current-pointer state exposed before atomic completion.

### `project_current_pointer_invalid`

Current pointer references candidate/failed/mismatched snapshot.

### `project_published_snapshot_mutation_forbidden`

Attempt to mutate an immutable published snapshot.

### `project_last_known_good_relabel_forbidden`

Prior snapshot relabeled as target/new generation.

### `project_last_known_good_unavailable`

No valid prior snapshot retained where degradation/status requested one.

## 8. View/query errors

### `project_view_generation_mismatch`

View/request generation mismatch.

### `project_file_not_present_in_generation`

Exact file absent from selected published generation.

### `project_file_path_noncanonical`

Path lookup requires guessing/fuzzy/case correction.

### `project_analyzer_fact_capability_unavailable`

Requested analyzer facts not available for file/capability.

### `project_generic_findings_capability_unavailable`

Generic diagnostics unavailable/partial; empty list cannot be clean.

### `project_raw_analyzer_handle_exposure_forbidden`

Public project API attempts to expose mutable/raw upstream analyzer state.

### `project_platform_authority_forbidden`

Project layer attempts to assert API existence/restriction/replacement/runtime truth.

### `project_diagnostic_algorithm_forbidden`

Project layer attempts to create/modify generic or WoW findings.

## 9. Security/policy errors

### `project_repository_scan_forbidden_e0`

Filesystem/repository scan/watch/discovery attempted.

### `project_installed_addon_universe_forbidden_e0`

Installed addon/client/SavedVariables/log universe requested.

### `project_source_execution_forbidden`

Attempt to execute Lua/build/test/generator/hook content.

### `project_process_or_shell_escape_forbidden`

Arbitrary process/shell invocation attempted.

### `project_editor_mutation_forbidden`

User/workspace editor settings mutation attempted.

### `project_untrusted_instruction_ignored`

Source/comment documentation has no policy effect; normally a test observation/notice.

## 10. Deferred operation error

### `operation_not_implemented_for_milestone`

Requested TOC/XML/load graph/state/event/hook/graph/persistence/scan/runtime operation is outside E0-D.

Must not return empty/default success.

## 11. Fatal versus degradable

### Fatal for target publication

```text
project_configuration_invalid
project_source_registry_invalid
project_generation_derivation_invalid
expected_project_generation_mismatch
project_update_conflicting_operations
project_analyzer_update_failed
any analyzer snapshot identity/manifest mismatch
project_mandatory_capability_unavailable
project_snapshot_invalid
project_snapshot_digest_mismatch
project_partial_publication_forbidden
project_source_execution_forbidden
project_editor_mutation_forbidden
```

### Degradable only under explicit policy

```text
per-file analyzer parse/diagnostic/fact capability partial/failed
nonmandatory analyzer capability unavailable
explicitly deferred E2 capability NotEvaluated
bounded optional output partial
```

Degradation still requires one coherent analyzer/project snapshot and exact coverage records.

### NoChange

`project_update_no_change` is a structured nonpublication outcome, not an error masquerading as a new generation.

## 12. Recovery rules

- stale generation/digest -> caller must reread current snapshot and construct a fresh request;
- input/path/digest -> fix explicit inputs;
- analyzer failure/corruption -> follow E0-C recovery/recreate session; do not publish target;
- configuration/profile/pin change -> derive a new explicit candidate;
- deferred capability -> wait for milestone/contract activation;
- security/policy violation -> never blind retry.

## 13. Error testing

Every E0-used code needs:

- exact triggering case;
- expected current/target generation assertions;
- file/analyzer/capability/partition context;
- no private path/source leak;
- deterministic canonical serialization;
- recovery class;
- mutation proving the check prevents incorrect publication.
