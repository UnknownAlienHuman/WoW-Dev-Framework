# `wow-service` E0-F error model

**Status:** normative typed service/status/check/context/result failure vocabulary.

Messages are projections. Callers/tests inspect structured codes, operation/request/context/component/generation/capability/partition/conflict identities, semantic impact, and recovery class.

## 1. Error shape

```text
ServiceError
    code
    operation
    request_id: optional
    service_configuration_id: optional
    service_context_lease_id: optional
    project_id: optional
    requested_generation_selector: optional
    resolved_project_generation: optional
    ProfileId / ReferenceGenerationId: optional
    ProjectSnapshot / AnalyzerSnapshot / RuleRegistry IDs: optional
    component_ids[]
    source/finding/outcome IDs[]
    capability/coverage/conflict IDs[]
    failed_target_ids[]
    last_known_good_ids[]
    message_arguments
    recovery_class
```

`recovery_class`:

```text
never
after_request_fix
after_configuration_fix
after_component_recovery
after_reacquire_exact_context
after_contract_or_implementation_fix
retry_same_request_if_cancelled_and-context-still-current
```

## 2. Configuration and registry errors

### `service_configuration_invalid`

Missing/contradictory project/profile/reference/analyzer/rule/output/budget/deferred configuration.

### `service_component_registry_invalid`

Component records/contract/implementation/health/snapshot/deferred references invalid.

### `service_component_dependency_inactive`

E0 operation attempts to activate store/annotations/graph/recognizers/search/CBM/context or another inactive component.

### `service_output_contract_invalid`

Envelope schema/canonicalization/tool contract missing or incompatible.

### `service_budget_policy_invalid`

Invalid stage/total/output/cancellation policy.

### `service_deferred_registry_invalid`

Deferred operation/capability records missing, contradictory, or marked available.

### `service_fixture_identity_unfrozen`

Implementation starts while prerequisite/result/checksum IDs remain null.

## 3. Request errors

### `service_status_request_invalid`

Malformed project/detail/budget request.

### `service_check_request_invalid`

Malformed project/generation/scope/rules/rollout/presentation/budget request.

### `service_project_unknown`

Requested project ID not configured/published.

### `service_generation_selector_invalid`

Missing/unsupported/unscoped latest/current selector.

### `service_project_generation_unavailable`

Exact generation not published/available.

### `service_scope_invalid`

Unknown/stale/foreign/Library/path-glob/fuzzy/over-budget scope.

### `service_rule_selection_invalid`

Unknown/inactive/non-E0 rule or invalid rollout selector.

### `service_operation_not_supported_e0`

Alias for typed deferred operation invocation; canonical external code is `operation_not_implemented_for_milestone`.

## 4. Context acquisition errors

### `service_context_acquisition_failed`

Could not build any coherent immutable context.

### `service_context_invalid`

Lease structurally invalid or unresolved references.

### `service_profile_mismatch`

ProfileIdentity differs across configuration/reference/project/rules.

### `service_reference_generation_mismatch`

ReferenceView/project/config mismatch.

### `service_project_generation_mismatch`

Selected snapshot/view/analyzer/facts differ.

### `service_analyzer_binding_mismatch`

Analyzer snapshot/pin/probe/config/main-manifest binding invalid.

### `service_rule_registry_mismatch`

Rule registry/fixture policy/profile/reference/version mismatch.

### `service_component_snapshot_unavailable`

Required immutable view/snapshot absent.

### `service_context_source_or_fact_stale`

Scope/source/fact/finding/coverage record belongs to another content/generation.

### `service_context_switched_mid_request_forbidden`

Executor reacquired/reread current and mixed/switched snapshot.

### `service_context_retry_fallback_forbidden`

Silent retry/fallback to another profile/reference/project generation.

### `service_last_known_good_substitution_forbidden`

Old snapshot used to satisfy another required target.

### `service_partial_context_exposure_forbidden`

Operation started/published with incomplete mandatory lease.

## 5. Status errors

### `status_component_record_invalid`

Component state/identity/health/capabilities invalid.

### `status_false_validation_claim_forbidden`

Status emits check/test/runtime/clean/safe/working/release claim without identified run evidence.

### `status_component_ready_interpreted_as_clean_forbidden`

Ready state converted into a check result.

### `status_last_known_good_relabel_forbidden`

Retained snapshot represented as failed target/current-new identity.

### `status_deferred_operation_marked_available`

Inactive operation/capability shown as available.

### `status_capability_summary_invalid`

Important partial/failed/conflict/truncation state omitted or misrepresented.

### `status_output_budget_partial`

Optional detailed status truncated; must return explicit partial status, not silent omission.

### `status_result_invalid`

Status references/status/identity/digest violation.

## 6. Generic finding collection errors

### `service_generic_finding_set_invalid`

Generic set context/source/finding/coverage/budget references invalid.

### `service_generic_capability_unavailable`

Normally contributes partial/NotEvaluated blocker; error if treated complete/empty clean.

### `service_generic_finding_mutation_forbidden`

Service changes category/severity/message/evidence/source/root/remediation.

### `service_generic_finding_recomputed_forbidden`

Service reruns/normalizes analyzer diagnostics instead of consuming ProjectView.

### `service_generic_empty_clean_without_coverage_forbidden`

Empty generic findings under unavailable/partial capability treated complete.

## 7. Rule execution aggregation errors

### `service_rule_execution_report_invalid`

Report context/outcome/reference/budget/digest invalid.

### `service_rule_execution_context_mismatch`

Rule report belongs to another lease/generation/profile.

### `service_rule_weaker_retry_forbidden`

Service reruns blocked rule with weaker capability/coverage policy.

### `service_rule_outcome_dropped_forbidden`

Finding/clean/NotEvaluated/failure/cancelled outcome omitted.

### `service_rule_failure_partial_policy_invalid`

Provider contract failure improperly downgraded to coherent partial.

### `service_rule_algorithm_reimplementation_forbidden`

Service locally evaluates API/Secret rule rather than invoking rules crate.

## 8. Raw check data errors

### `service_raw_check_data_invalid`

Raw generic/rule/outcome/capability/budget aggregation invalid.

### `service_raw_finding_context_invalid`

Finding from another context/source generation.

### `service_raw_finding_duplicate_invalid`

Equivalent ID duplicated or non-equivalent findings incorrectly collapsed.

### `service_raw_finding_deleted_by_folding_forbidden`

Presentation folding removes raw finding.

### `service_clean_record_invalid`

Clean record lacks complete scope/capability inputs or hidden blocker.

### `service_not_evaluated_record_invalid`

Blockers absent/dropped/mismatched.

### `service_empty_output_interpreted_clean_forbidden`

No findings/outcomes under incomplete/unproven scope classified clean.

## 9. Presentation graph errors

### `service_presentation_graph_invalid`

Node/edge/root/reference/digest invariant failure.

### `service_presentation_relation_unproven`

No valid structured causal/blocker/duplicate evidence.

### `service_message_based_folding_forbidden`

Message similarity used for parent/dedup/root relation.

### `service_presentation_cross_context_edge_forbidden`

Relation spans contexts/generations.

### `service_presentation_self_edge_forbidden`

Node relates to itself.

### `service_presentation_cycle_forbidden`

Directed cycle detected.

### `service_presentation_multiple_primary_parents_forbidden`

Child has more than one primary parent.

### `service_presentation_parent_selection_invalid`

First-returned/message-based/nondeterministic parent selected or competing relation dropped.

### `service_presentation_root_coverage_invalid`

Problem record absent from root/child graph.

### `service_root_count_as_raw_count_forbidden`

Presentation root count substituted for raw finding/problem counts/status.

### `service_final_finding_mutation_forbidden`

Folding changes finding severity/source/evidence/remediation/identity.

## 10. Semantic status errors

### `service_semantic_status_invalid`

Status contradicts raw outcomes/completeness/precedence.

### `service_clean_with_findings_forbidden`

Clean with nonempty raw findings.

### `service_clean_with_not_evaluated_forbidden`

Clean with requested-scope blockers.

### `service_clean_with_truncation_forbidden`

Clean under incomplete budget/scope.

### `service_findings_with_blockers_must_be_partial`

Findings plus `NotEvaluated`/degradable failure/truncation classified findings.

### `service_advisory_findings_interpreted_clean_forbidden`

Rollout advisory used to hide semantic findings.

### `service_failed_as_partial_forbidden`

Mandatory context/result-contract failure downgraded.

### `service_status_precedence_violation`

Failed/cancelled/partial/findings/clean precedence violated.

## 11. Envelope errors

### `service_result_envelope_invalid`

General envelope structure/reference/schema/version violation.

### `service_result_reference_closure_invalid`

Unresolved/cross-context IDs.

### `service_result_digest_mismatch`

Canonical recomputation mismatch.

### `service_result_nondeterministic`

Equivalent semantic inputs yield different canonical bytes/ID/digest.

### `service_result_volatile_identity_forbidden`

Timestamp/temp path/process/thread/message/UI state enters identity.

### `service_result_order_invalid`

Output order depends on lower-layer return/discovery order.

### `service_failure_embeds_malformed_check_forbidden`

Failure returned as incomplete check envelope with misleading semantic status.

### `service_cancelled_check_published_forbidden`

Check envelope published after cancellation.

### `service_result_payload_leak_forbidden`

Raw Secret-capable value, excessive/private source, local path, token/private URL leaked.

## 12. Budgets/cancellation errors

### `service_context_budget_exceeded`

Cannot acquire/validate required context within budget.

### `service_scope_budget_exceeded`

Requested scope too large.

### `service_generic_budget_exceeded`

Generic collection incomplete.

### `service_rule_budget_exceeded`

Rule report incomplete/over budget.

### `service_presentation_budget_exceeded`

Cannot build full required graph; partial/failure policy applies.

### `service_output_budget_exceeded`

Canonical result cannot fit configured output budget.

### `service_cancelled`

Operation cancelled before publication.

### `service_late_result_after_cancel_forbidden`

Late/background publication.

## 13. Security and architecture errors

### `service_source_execution_forbidden`

Analyzed/repository code executed.

### `service_io_or_process_escape_forbidden`

Arbitrary filesystem/database/network/process/shell access outside declared lower component seams.

### `service_editor_or_client_mutation_forbidden`

Editor/WoW client/runtime state accessed/mutated.

### `service_source_mutation_forbidden`

Service applies edit/remediation.

### `service_lower_crate_bypass_forbidden`

Application/service imports internals/raw upstream handles or reconstructs a lower algorithm.

### `service_search_or_replacement_forbidden_e0`

E0 invokes search/alias/fuzzy/replacement lane.

### `service_runtime_claim_without_evidence_forbidden`

Service output claims client/runtime behavior without runtime record.

### `service_untrusted_instruction_ignored`

Source/comment text has no policy effect; normally a test observation.

## 14. Deferred operation error

### `operation_not_implemented_for_milestone`

Requested:

```text
lookup
search
tree
skeleton
plan
patch_impact
index_repo
runtime_review
LSP/MCP
release/pack publication
strict/autofix/source-mutation operation
```

Must not return empty/default success.

## 15. Fatal versus partial

### Fatal/no coherent check result

```text
invalid request/configuration/context
mandatory ReferenceView/ProjectSnapshot/AnalyzerSnapshot/RuleRegistry failure
cross-generation/profile mismatch
invalid generic/rule report
invalid presentation graph
envelope/reference/digest/status violation
security/architecture violation
mandatory output budget failure
```

### Partial coherent result

```text
requested-scope rule NotEvaluated
nonmandatory per-file analyzer/reference/project capability failure
explicit result truncation with coherent completed subset and policy
findings plus any above blocker
```

Provider implementation contract failure is fatal by default in E0, not partial.

## 16. Recovery rules

- request/scope -> caller fixes exact request;
- context mismatch -> reacquire exact coherent snapshot; no silent fallback;
- component failure -> recover owning component/project publication;
- partial capability -> expose blocker; do not weaken rule;
- presentation/envelope/provider bug -> fix implementation and rerun frozen fixtures;
- deferred operation -> wait for contract/milestone;
- security violation -> never blind retry;
- cancellation -> retry only with same explicit selector/current semantics and fresh acquisition.

## 17. Error testing

Every E0-used code needs:

- dedicated fixture/mutation;
- exact request/context/component/generation/source/finding/capability IDs;
- no private/raw payload leak;
- deterministic canonical failure serialization;
- recovery class;
- assertion that no misleading status/check envelope/late result escaped.
