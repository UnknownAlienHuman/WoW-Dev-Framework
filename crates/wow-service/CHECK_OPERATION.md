# `check` operation

**Status:** normative E0-F end-to-end orchestration contract.

`check` combines generic analyzer findings and the two E0 WoW rule providers under one exact reference/project/analyzer generation, preserves every raw outcome, and returns one deterministic service result.

## 1. Request

```text
CheckRequest
    request_id
    service_id
    project_id
    project_generation_selector
    scope
    requested_rule_ids or E0 default
    include_generic_findings = true
    rollout_filter
    presentation_mode = raw_and_roots
    operation_budget
    cancellation
```

E0 default rule set:

```text
wow.api.exists@1
wow.secret.local_operation@1
```

## 2. Scope

Supported:

```text
AllProjectFiles
FileIds(ProjectFileId[])
ExplicitScopes(valid file/function/fact/use IDs)
```

Rules:

- scope resolves through immutable ProjectView;
- no filesystem glob/search;
- no Library source as first-party scope;
- stale/foreign IDs rejected;
- canonical file/scope ordering;
- full-scope fixture includes all four Main files/functions.

## 3. Operation sequence

```text
validate CheckRequest and budgets
-> acquire/validate one ServiceContextLease
-> resolve/canonicalize scope
-> collect GenericFindingSet through ProjectView
-> validate generic findings/context/coverage
-> build RuleExecutionRequest from the same lease/scope
-> execute E0 RuleRegistry
-> validate RuleExecutionReport/outcomes
-> assemble RawCheckData
-> validate/deduplicate exact raw finding identities only
-> build FindingPresentationGraph from structured causal/blocker/duplicate hints
-> derive ServiceSemanticStatus
-> assemble CheckResultEnvelope
-> validate all references/status/budgets/canonical ordering
-> canonicalize and publish once
```

Any mandatory failure before coherent RawCheckData/result validation returns a `ServiceFailureResult`, not a malformed partial check envelope.

## 4. Generic finding collection

Service calls ProjectView only:

```text
analyzer_generic_findings(scope)
analyzer/project capability records for those files
```

It does not:

- rerun or normalize Emmy diagnostics;
- read source directly;
- alter severity/category/message/evidence/root identity;
- infer platform causes;
- treat unavailable diagnostics capability as empty clean.

Generic collection yields:

```text
complete set
partial/failed capability with explicit blockers
context failure
cancelled
```

## 5. Rule execution

Service passes:

- exact lease context;
- exact ReferenceView/ProjectView identities;
- selected scope;
- active rule IDs/rollout filter;
- budgets/cancellation.

It receives and preserves:

```text
rule findings
clean evaluation records
NotEvaluated records
rule failures
cancellation
causal hints
rule execution coverage/report
```

Service cannot rerun a blocked rule with a weaker capability policy.

## 6. Raw check assembly

```text
raw_findings = exact canonical union(
    generic_findings,
    rule_findings
)
```

Deduplicate only when `wow-core` canonical finding identity is exactly equal. Same message or same missing API at another source span remains distinct.

Also retain:

```text
clean evaluation records
NotEvaluated records
rule failures
component warnings/failures
capability/coverage/conflict IDs
rule/generic budget usage
```

## 7. E0 full-scope expected semantic records

After prerequisite freeze, baseline full scope should contain at least:

```text
generic:
    one accepted E0-C generic fixture finding

wow.api.exists:
    one finding for C_E0Fixture.RemovedApi authoritative absence

wow.secret.local_operation:
    finding: unsafe_concat
    clean: guarded_concat
    finding: guard_after_use
    finding: different_value_guard
```

Expected fixed rule counts:

```text
API findings = 1
Secret findings = 3
Secret clean evaluations = 1
```

Accepted generic count/families are frozen by E0-C. A generic unresolved-member symptom for `RemovedApi` is optional until the upstream probe freezes it; service never invents it.

With complete capabilities and no blockers, semantic status = `findings`.

## 8. Clean check scope

The clean fixture scope selects only source uses that produce no generic or WoW finding and whose rules can prove narrow clean outcomes, for example:

```text
KnownApi exact found use
guarded_concat exact-value dominating guard
```

Requirements:

- generic diagnostic capabilities complete and no findings in selected scope;
- API existence clean record for exact found use or scope nonapplicability documented;
- Secret guarded clean record;
- no `NotEvaluated`, failures, truncation;
- complete scope/budget proof.

Status = `clean`.

## 9. Partial check

A coherent partial result may include findings and blockers.

Example broken annotation library:

```text
component/root failure: annotation library failed
API/Secret resolution-dependent scopes: NotEvaluated
independent generic diagnostics/facts may remain when valid
raw findings retained
presentation graph blocks dependent outcomes under component root
status = partial
```

Other partial causes:

- per-file control-flow fact capability failed;
- reference exact partition partial/conflicted;
- rule execution budget/truncation;
- one degradable component capability failed.

A partial result must identify completed and blocked scopes. Empty findings under blockers are not clean.

## 10. Failed check

Return `ServiceFailureResult` when:

- request/configuration invalid;
- exact project generation unavailable/mismatched;
- profile/reference/project/analyzer/rule context incoherent;
- mandatory ReferenceView/ProjectSnapshot/RuleRegistry invalid;
- rule execution report structurally invalid;
- result/presentation graph/envelope validation fails;
- security/policy violation;
- mandatory output cannot be constructed coherently.

No check envelope with semantic `failed` is required if the common service failure contract is cleaner; the E0 fixtures freeze one representation before implementation. In either case, no clean/findings/partial claim.

## 11. Cancellation

Cancellation phases:

```text
before context acquisition
during scope/generic collection
before/during rule execution
during presentation/envelope construction
before publication
```

Rules:

- no late/background result;
- no partially published envelope;
- no component mutation to roll back (service reads immutable views/rules are pure);
- exact cancellation phase/context identity recorded safely;
- CLI maps to cancellation exit code.

## 12. Root-cause presentation

Service consumes structured hints/blockers, then builds a separate graph.

E0 examples:

- authoritative API finding -> primary parent of same-source generic unresolved-member symptom when exact hint exists;
- annotation-library component failure -> primary blocker parent for resolution-dependent API/Secret `NotEvaluated` records;
- unsafe/after-use/different-value Secret findings remain independent roots;
- generic type fixture error remains independent root;
- exact duplicates may be presented once with duplicate relation while all raw IDs remain.

See [`ROOT_CAUSE_FOLDING.md`](ROOT_CAUSE_FOLDING.md).

## 13. Semantic status derivation

```text
if mandatory failure/context/result invalid:
    failed
else if cancelled before publication:
    cancelled
else if any requested-scope NotEvaluated, degradable failure, or truncation:
    partial
else if raw_findings not empty:
    findings
else if every requested scope/lane completed and clean/nonapplicable explicitly:
    clean
else:
    failed (status contract violation)
```

Warnings that do not affect requested analysis completeness do not automatically cause partial; warning policy is explicit.

## 14. Rollout and severity

Service preserves technical severity and rollout on findings.

- E0 rule findings are advisory rollout but still raw findings.
- Semantic status remains `findings`, not clean.
- Service does not choose command exit/fail policy from rollout.
- CLI maps the service status explicitly.

## 15. Deferred operations and lanes

E0 check does not invoke:

```text
search/replacement
project graph/load/TOC/XML
CBM/external repositories
skeleton/context builder
runtime probes
source edits
```

Requests requiring these return typed unavailable/NotEvaluated only when the check contract supports that exact scope; otherwise fail as deferred operation.

## 16. Budgets

Stages:

```text
context acquisition
scope resolution
generic finding collection
rule execution
raw aggregation
presentation graph
serialization/output
```

Budget exhaustion:

- before coherent result -> failure/cancelled;
- after coherent useful subset only when explicit partial policy exists -> partial with exact truncation;
- never clean;
- no silent record dropping;
- raw/presentation relation/evidence counts and bytes reported.

## 17. Required operations

```text
validate_check_request
resolve_check_scope
collect_generic_finding_set
validate_generic_finding_set
build_rule_execution_request
validate_rule_execution_report_for_context
assemble_raw_check_data
validate_raw_check_data
build_finding_presentation_graph
derive_service_semantic_status
assemble_check_result_envelope
validate_check_result_envelope
canonicalize_check_result_envelope
```

## 18. Required tests

- exact/current-published full-scope check;
- clean scope;
- findings scope with exact fixed rule counts;
- partial broken-library/reference/control-flow cases;
- findings plus NotEvaluated -> partial;
- context/profile/generation mismatch -> failed;
- generic capability failed -> not empty clean;
- raw findings preserved through folding;
- exact API generic causal hint and no message-based hint;
- independent roots retained;
- graph cycle/multiple-parent/dedup mutations rejected;
- advisory findings -> semantic findings;
- empty complete scope -> clean only with explicit clean/nonapplicable outcomes;
- cancellation every phase no late output;
- budget/truncation explicit;
- deferred lanes never invoked;
- randomized lower-layer return/order/temp/message changes -> identical canonical envelope.

## 19. Hard stops

- no source re-read/parse;
- no second/later snapshot acquisition;
- no weaker rule retry;
- no raw finding deletion;
- no message-based dedup/folding;
- no clean from empty outputs;
- no advisory finding called clean;
- no last-known-good substitution;
- no search/replacement/autofix/runtime/client behavior;
- no partial or cancelled late publication;
- no transport/exit-code logic in service.
