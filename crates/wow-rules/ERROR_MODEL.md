# `wow-rules` E0-E error model

**Status:** normative typed provider/context/output failure vocabulary.

Errors are distinct from `NotEvaluated`. `NotEvaluated` means required evidence/capability/semantic support is unavailable. `Failed` means the provider/registry/output contract failed despite a coherent attempted execution.

## 1. Error shape

```text
RuleError
    code
    operation
    rule_id: optional
    rule_version: optional
    registry_id: optional
    execution_context_id: optional
    generation_context: optional
    reference_view_id: optional
    project_snapshot_id: optional
    analyzer_snapshot_id: optional
    source_handle_ids[]
    fact_ids[]
    lookup_or_facet_ids[]
    capability_ids[]
    coverage_ids[]
    conflict_ids[]
    message_arguments
    recovery_class
```

`recovery_class`:

```text
never
after_context_fix
after_capability_or_input_fix
after_provider_fix
after_contract_revision
retry_same_context_if_cancelled
```

## 2. Registry/descriptor errors

### `rule_descriptor_invalid`

Missing/contradictory ID/version/category/severity/rollout/capabilities/budget/fixture/remediation policy.

### `rule_descriptor_duplicate`

Duplicate RuleId/version.

### `rule_descriptor_version_conflict`

Multiple incompatible active versions for one RuleId.

### `rule_registry_invalid`

Registry references, enabled rules, digest, or milestone state invalid.

### `rule_inactive_for_milestone`

Attempt to enable a planned non-E0 rule.

### `rule_dependency_inactive`

Descriptor/provider requires inactive graph/store/search/other dependency.

### `rule_rollout_policy_invalid`

Technical severity/rollout mapping missing or provider silently changed policy.

### `rule_fixture_policy_invalid`

Fixture policy mismatched profile/reference/rule version or missing required semantics.

## 3. Execution context errors

### `rule_execution_context_invalid`

Required views/identities/budgets/records missing or structurally invalid.

### `rule_profile_mismatch`

ProfileIdentity differs across request/reference/project/fixture policy.

### `rule_reference_generation_mismatch`

ReferenceView/GenerationContext mismatch.

### `rule_project_generation_mismatch`

ProjectView/facts/findings/source handles mismatch.

### `rule_analyzer_snapshot_mismatch`

Analyzer facts/findings do not match project binding/context.

### `rule_source_handle_invalid`

Project/reference source handle stale, wrong origin/role/content/generation.

### `rule_registry_context_mismatch`

Execution context references another registry/policy identity.

### `rule_last_known_good_substitution_forbidden`

Old project snapshot substituted for requested target generation.

### `rule_context_retry_other_generation_forbidden`

Provider/executor attempted to recover by silently selecting another snapshot/profile.

Context errors are not downgraded to normal `NotEvaluated` when identities are contradictory.

## 4. Capability/input assembly errors

### `rule_capability_requirement_invalid`

Malformed or unresolved capability/partition selector.

### `rule_required_capability_unavailable`

Normally represented as `NotEvaluated`; error if executor/provider tries to run anyway.

### `rule_required_coverage_unavailable`

Partial/Failed/Unknown/NotEvaluated exact partition blocks rule.

### `rule_conflict_blocker_ignored`

Provider/executor attempted to ignore relevant conflict.

### `rule_truncation_blocker_ignored`

Provider/executor treated truncated input/scope as complete.

### `rule_stale_input_forbidden`

Fact/lookup/facet/coverage/source from stale context.

### `rule_fact_kind_invalid`

Missing/wrong-kind/ambiguous/dynamic fact for E0 input.

### `rule_fact_reference_graph_invalid`

Fact IDs/relations cross scope/snapshot or do not resolve.

### `rule_lookup_outcome_invalid`

Reference lookup outcome malformed/incompatible with requested query/profile.

### `rule_authority_bypass_forbidden`

Provider inferred absence from empty/unresolved/summary-complete data instead of structured authority decision.

### `rule_project_source_as_platform_evidence_forbidden`

Project source/fact used as reference platform proof.

### `rule_absent_entity_source_handle_forbidden`

Provider fabricated source handle for absent API.

## 5. API rule errors

### `api_exists_scope_invalid`

Input is not the supported direct Main unresolved member/call use.

### `api_exists_query_invalid`

Canonical exact EntityKey/entity kind cannot be built/validated.

### `api_exists_nonexact_lookup_forbidden`

Alias/fuzzy/prefix/FTS/semantic/lineage/external lookup attempted.

### `api_exists_unresolved_fact_upgraded_forbidden`

Analyzer unresolved fact treated as platform absence without ReferenceView authority.

### `api_exists_finding_without_authority`

Finding built from partial/conflict/unavailable miss.

### `api_exists_replacement_inference_forbidden`

Replacement/candidate/edit inferred in E0.

### `api_exists_clean_claim_too_broad`

Clean record claims signature/deprecation/security/runtime correctness beyond exact existence.

### `api_exists_generic_finding_suppression_forbidden`

Rule crate attempted final suppression/folding.

### `api_exists_causal_hint_unproven`

Generic symptom relation lacks exact same source/fact/context proof.

## 6. Secret-local rule errors

### `secret_local_scope_invalid`

Input outside supported function-local E0 scope.

### `secret_local_producer_invalid`

Producer/member/call/return-slot relation missing or ambiguous.

### `secret_local_facet_invalid`

Facet kind/subject/slot/applicability/profile/generation malformed.

### `secret_local_facet_unavailable`

Normally `NotEvaluated`; error if provider tries to evaluate finding/clean anyway.

### `secret_local_operation_invalid`

Operation unsupported, stale, wrong binding/value, or malformed.

### `secret_local_binding_identity_invalid`

Binding/use/flow identity unresolved/shadowed/cross-function.

### `secret_local_guard_policy_invalid`

Guard fact/callee/kind incompatible with fixture policy.

### `secret_local_guard_value_mismatch`

Provider accepted guard for another value/binding.

### `secret_local_guard_after_use_accepted_forbidden`

Provider treated later guard as protective.

### `secret_local_guard_dominance_unproven`

Provider accepted unknown/non-dominating flow.

### `secret_local_copy_or_conversion_declassification_forbidden`

Provider treated copy/conversion/format/serialization/`pcall` as declassification.

### `secret_local_annotation_only_authority_forbidden`

Secret status inferred from annotations/type/name/comments without exact facet.

### `secret_local_runtime_generalization_forbidden`

Fixture/static result generalized to runtime/global/combat safety.

### `secret_local_spell_whitelist_forbidden`

Static permanent spell/value whitelist introduced.

### `secret_local_autoguard_edit_forbidden`

Automatic guard insertion/edit emitted in E0.

### `secret_local_clean_claim_too_broad`

Clean record claims general Secret safety rather than exact fixture guarded operation.

## 7. Outcome/finding errors

### `rule_outcome_invalid`

Malformed or multiple primary statuses for one evaluation.

### `rule_empty_findings_not_clean`

Provider returned empty findings without explicit validated clean/NotEvaluated outcome.

### `rule_outcome_exclusivity_violation`

Findings/Clean/NotEvaluated/Failed/Cancelled overlap for same scope.

### `rule_finding_input_invalid`

Missing/contradictory project/reference/derivation/source/coverage fields.

### `rule_finding_context_invalid`

Finding does not belong to execution context.

### `rule_finding_primary_source_invalid`

Wrong role/origin/generation/span; whole-file fallback despite exact span.

### `rule_finding_evidence_invalid`

Evidence IDs missing/wrong provenance/cross-context.

### `rule_finding_identity_invalid`

Noncanonical/volatile/message-based identity.

### `rule_finding_duplicate`

Equivalent canonical duplicate within execution output.

### `rule_clean_record_invalid`

Missing scope/inputs/capabilities/budget or hidden blocker.

### `rule_not_evaluated_record_invalid`

No exact blocker or speculative output included.

### `rule_root_cause_key_invalid`

Message-based/volatile/cross-context root identity.

### `rule_causal_hint_invalid`

Unproven cause/symptom relation.

### `rule_final_folding_forbidden`

Rule crate suppressed/reordered global stream.

## 8. Remediation errors

### `rule_remediation_invalid`

Tier/code/steps/checks inconsistent with descriptor/outcome.

### `rule_remediation_tier_forbidden_e0`

Tier other than `plan_only` emitted.

### `rule_exact_edit_without_proof_forbidden`

Automatic edit generated without exact contract/preconditions (all E0 edits forbidden).

### `rule_similarity_replacement_forbidden`

Fuzzy/semantic/name/external candidate included as proven remediation.

### `rule_runtime_claim_without_evidence_forbidden`

Remediation/finding claims client behavior without runtime record.

### `rule_source_mutation_forbidden`

Provider/executor attempts to apply edits.

## 9. Execution/budget/cancellation errors

### `rule_execution_budget_exceeded`

Bound exceeded before complete scope outcome.

### `rule_output_budget_exceeded`

Finding/evidence/report output exceeds declared budget.

### `rule_cancelled`

Execution cancelled without late/background output.

### `rule_late_result_after_cancel_forbidden`

Provider publishes after cancellation.

### `rule_nondeterministic_output`

Equivalent structured inputs produce differing canonical outcome/report.

### `rule_hidden_global_state_forbidden`

Cross-request mutable cache/state affects semantics.

## 10. Security/IO errors

### `rule_io_forbidden`

File/database/network IO attempted.

### `rule_process_or_shell_forbidden`

Process/shell execution attempted.

### `rule_editor_or_client_access_forbidden`

Editor/WoW client/runtime access attempted.

### `rule_source_execution_forbidden`

Analyzed Lua/repository code executed.

### `rule_source_or_secret_payload_leak_forbidden`

Default output leaks raw value/large source/local path/token/private data.

### `rule_untrusted_instruction_ignored`

Source comment/text has no policy effect; normally a test observation.

## 11. Deferred operation error

### `operation_not_implemented_for_milestone`

Requested later rule family, graph-dependent evaluation, autofix application, runtime verification, search/replacement, persistence, or transport behavior.

Must not return default/empty success.

## 12. NotEvaluated blocker vocabulary

These are structured blocker reasons, not provider failures:

```text
missing_capability
partial_coverage
failed_coverage
unknown_coverage
upstream_not_evaluated
relevant_conflict
truncated_input
budget_incomplete
unsupported_fact_kind
unsupported_operation_kind
unsupported_control_flow
unsupported_conditional_or_runtime_semantics
reference_absence_non_authoritative
annotation_library_unavailable
fact_resolution_ambiguous_or_dynamic
```

## 13. Fatal versus scope-local

### Fatal request/context

```text
registry invalid
generation/profile/reference/analyzer mismatch
stale/invalid source handle
invalid context/fixture policy
hidden inactive dependency
security/IO/mutation violation
```

### Scope-local NotEvaluated

```text
exact required capability/coverage/fact/facet/control-flow unavailable
non-authoritative reference miss
supported rule semantics unavailable
budget/truncation for selected scope
```

### Provider Failed

```text
provider violated input/output contract
unexpected internal invariant/error under runnable context
nondeterministic output
invalid finding/remediation/root cause
```

## 14. Recovery rules

- context mismatch/staleness -> caller/service reacquires exact coherent context;
- capability/input unavailable -> build/fix required producer partition or mark scope NotEvaluated;
- provider/output bug -> fix provider and rerun same frozen fixtures;
- unsupported semantics/rule family -> contract/milestone revision, not heuristic fallback;
- security/policy violation -> never blind retry.

## 15. Error tests

Every E0-used error requires:

- a dedicated trigger/mutation;
- exact rule/context/source/fact/query/capability IDs;
- no private/raw value leak;
- deterministic serialization;
- recovery class;
- assertion that no finding/clean/edit/late output escaped when forbidden.
