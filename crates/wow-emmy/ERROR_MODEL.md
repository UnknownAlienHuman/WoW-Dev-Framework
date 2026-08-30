# `wow-emmy` E0-C error model

**Status:** normative typed failure vocabulary.

Messages are projections. Callers/tests inspect code, operation, pin/session/snapshot/context, file/capability/partition IDs, and recovery class.

## 1. Error shape

```text
EmmyAdapterError
    code
    operation
    upstream_pin_id: optional
    analyzer_session_id: optional
    analyzer_snapshot_id: optional
    generation_context: optional
    workspace_id: optional
    file_id: optional
    source_handle_id: optional
    capability_ids[]
    partition_ids[]
    upstream_code: optional
    message_arguments
    recovery_class
```

`recovery_class`:

```text
never
after_request_fix
after_fixture_fix
after_adapter_fix
after_upstream_pin_change
recreate_session
retry_same_snapshot_if_cancelled
```

Blind retries against a changed generation are prohibited.

## 2. Pin/probe errors

### `upstream_pin_invalid`

Missing/floating commit, invalid crate/version/feature record, or unsupported repository identity.

### `upstream_license_unverified`

License/SPDX/notices not established.

### `upstream_public_api_missing`

Mandatory public operation cannot be implemented without private/internal access or fork.

### `upstream_private_api_dependency_forbidden`

Candidate requires a private/internal module contract unsuitable for stable adapter use.

### `compatibility_probe_failed`

One or more mandatory probe cases failed.

### `compatibility_capability_missing`

Specific mandatory capability absent.

### `compatibility_diagnostic_unclassified`

Candidate introduces/changes a diagnostic family without classification.

### `compatibility_coordinate_failure`

Source range conversion is not exact/reliable.

### `compatibility_nondeterministic`

Equivalent inputs produce differing canonical outputs.

### `last_known_good_unavailable`

No accepted rollback pin exists when required.

## 3. Configuration/workspace errors

### `analyzer_configuration_invalid`

Configuration lacks required explicit settings, exceeds policy, or is contradictory.

### `editor_configuration_mutation_forbidden`

Adapter attempted to modify user/workspace editor settings.

### `workspace_role_invalid`

Unknown/illegal Main/Library role assignment.

### `workspace_root_invalid`

Logical root/path invalid, overlapping without policy, or escapes configured scope.

### `workspace_duplicate_file`

Same logical file registered multiple times or under incompatible roles.

### `full_blizzard_workspace_forbidden_e0`

E0 request attempts to load the full Blizzard UI implementation tree.

### `library_workspace_missing`

Required annotation fixture workspace absent.

### `library_health_failed`

Annotation library failed parse/index/resolution health.

## 4. File/source errors

### `source_not_utf8`

Input bytes are not valid UTF-8.

### `source_path_invalid`

Absolute, traversal, device, UNC, credential-bearing, or otherwise invalid path.

### `source_file_unregistered`

Upstream diagnostic/fact references a file not in the snapshot registry.

### `source_content_digest_mismatch`

Supplied/registered/current content identity mismatch.

### `source_span_out_of_bounds`

Converted span outside exact content bytes.

### `source_span_not_codepoint_boundary`

Text span starts/ends inside UTF-8 code point.

### `source_span_end_semantics_ambiguous`

Adapter cannot prove inclusive/exclusive conversion.

### `source_coordinate_conversion_failed`

Upstream range/position cannot convert exactly.

### `source_snapshot_mismatch`

Fact/finding/span belongs to another analyzer snapshot/project generation.

### `source_uri_leaks_host_path`

Public result would expose absolute/temp/user path or credential-bearing URI.

## 5. Session lifecycle errors

### `analyzer_session_state_invalid`

Operation not valid for current state.

### `analyzer_session_failed`

Session entered fatal failed state.

### `analyzer_session_closed`

Operation attempted after close.

### `analyzer_session_corrupted`

Upstream panic/poisoned state/invariant violation makes state untrustworthy.

### `analyzer_session_reentrant_mutation`

Mutation attempted while a conflicting mutation/read publication is active.

### `analyzer_snapshot_stale`

Caller expected another current snapshot.

### `analyzer_snapshot_invalid`

Snapshot references/capabilities/digests violate contract.

### `project_generation_mismatch`

Supplied target/project generation differs from session/update/snapshot inputs.

### `update_expected_digest_mismatch`

File update/remove expected old digest does not match.

### `update_batch_invalid`

Duplicate/conflicting operations or invalid target generation.

### `update_batch_failed`

Upstream update/index failed before coherent publication.

### `index_refresh_failed`

Required upstream index/compilation refresh failed.

### `request_cancelled`

Operation cancelled without publishing partial state.

### `request_budget_exceeded`

Declared file/diagnostic/fact/time/output budget exceeded.

## 6. Diagnostic normalization errors

### `upstream_diagnostic_unknown`

Diagnostic family not in accepted/shadow/ignored mapping.

### `upstream_diagnostic_mapping_invalid`

Mapping lacks category/severity/code/version/span/arguments policy.

### `generic_diagnostic_span_invalid`

Diagnostic span cannot be validated.

### `generic_diagnostic_source_role_invalid`

Project finding would use Library source as primary.

### `generic_finding_context_invalid`

Finding context/snapshot/project generation mismatch.

### `diagnostic_message_identity_forbidden`

Implementation attempted to use rendered prose as identity/dedup key.

### `upstream_blocking_policy_inheritance_forbidden`

Implementation adopted upstream severity as blocking policy without explicit mapping.

### `diagnostic_budget_partial`

Diagnostic result truncated/partial with exact count/budget; never clean success.

## 7. Fact extraction errors

### `fact_capability_unavailable`

Required syntax/semantic/control-flow capability unavailable.

### `fact_source_span_invalid`

Fact span/content/snapshot invalid.

### `fact_resolution_invalid`

Resolution status/key/upstream detail inconsistent.

### `fact_reference_graph_invalid`

Fact IDs/edges point to missing/wrong-kind/cross-snapshot facts.

### `fact_binding_scope_invalid`

Binding/use crosses invalid/shadowed scope without proof.

### `fact_control_flow_relation_unproven`

Adapter attempted to emit `dominates`/other relation without proof.

### `fact_wow_authority_forbidden`

Fact attempts to state API availability, Secret status, hook safety, replacement, or runtime truth.

### `fact_upstream_type_leak_forbidden`

Public contract exposes raw upstream CST/semantic/range/URI type.

### `fact_set_invalid`

Fact set context, capability, references, ordering, or digest invalid.

### `fact_budget_partial`

Fact extraction truncated/partial; exact coverage/budget state required.

## 8. Incremental errors

### `incremental_invalidation_unknown`

Adapter cannot prove affected/unaffected sets.

### `stale_fact_reuse_forbidden`

Old-snapshot fact would be published as current without proof.

### `removed_file_fact_retained`

Current snapshot still contains fact/finding for removed file.

### `library_dependency_not_invalidated`

Library change failed to invalidate dependent reference facts.

### `update_order_nondeterministic`

Same final contents yield differing canonical snapshot/facts/findings.

## 9. Security errors

### `analyzed_code_execution_forbidden`

Attempt to execute Lua/repository code.

### `process_or_shell_escape_forbidden`

Library adapter attempts arbitrary process/shell execution.

### `untrusted_comment_instruction_ignored`

Normally a test observation/notice: source instructions have no policy effect.

### `path_scope_escape_forbidden`

Resolved path escapes configured root.

### `source_payload_leak_forbidden`

Default error/finding attempts to embed excessive/private source.

## 10. Deferred-operation errors

### `operation_not_implemented_for_milestone`

Requested E1+/E7 operation such as LSP/MCP server, external diagnostic plugin registration, full-project architecture graph, or editor bootstrap.

It must not return empty/default success.

## 11. Fatal versus degradable

### Fatal session/pin

```text
upstream_pin_invalid
upstream_public_api_missing
compatibility_probe_failed
compatibility_coordinate_failure
compatibility_nondeterministic
analyzer_configuration_invalid
analyzer_session_corrupted
analyzer_snapshot_invalid
editor_configuration_mutation_forbidden
analyzed_code_execution_forbidden
process_or_shell_escape_forbidden
```

### File/capability degradable when isolated

```text
source_not_utf8
source_span conversion failure for one observation
library_health_failed (blocks dependent capabilities)
file parse failure
fact_capability_unavailable
diagnostic_budget_partial
fact_budget_partial
```

Degradation requires a coherent validated snapshot and exact coverage records. It cannot fabricate clean absence.

### Request rejection

```text
project_generation_mismatch
analyzer_snapshot_stale
update_expected_digest_mismatch
source_snapshot_mismatch
request_cancelled
request_budget_exceeded
```

## 12. Root-cause codes

Analyzer-owned causal roots may include:

```text
root:upstream-pin-incompatible
root:configuration-invalid
root:annotation-library-failed
root:file-parse-failed:<file-id>
root:source-coordinate-failed:<file-id>
root:session-corrupted
```

Final folding remains `wow-service` responsibility.

## 13. Error tests

Every E0-used code requires:

- a dedicated trigger case;
- exact operation/session/snapshot/context assertions;
- no local path/private source leak;
- deterministic canonical serialization;
- recovery-class assertion;
- deliberate mutation proving the check is effective.

Future unused codes are not exported merely to appear complete.
