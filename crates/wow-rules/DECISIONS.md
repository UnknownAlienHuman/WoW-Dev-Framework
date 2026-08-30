# `wow-rules` E0-E decisions

**Status:** normative for the E0-E two-rule vertical slice.

## RULES-001 — Only two rules activate in E0

**Decision:** E0-E implements `wow.api.exists@1` and `wow.secret.local_operation@1` only.

**Consequence:** planned event/widget/load/overlay/framework rules remain documentation-only until their prerequisite facts and corpora exist.

## RULES-002 — Providers are pure over immutable context

**Decision:** rule evaluation has no IO, mutation, background work, editor/client access, or hidden global state.

**Consequence:** equivalent structured inputs and rule versions yield deterministic outcomes.

## RULES-003 — Service assembles context; rules validate it

**Decision:** `wow-rules` does not select profiles, references, project snapshots, or analyzer generations.

**Consequence:** context mismatch is rejected before evaluation rather than silently corrected.

## RULES-004 — Rule outcomes distinguish clean, findings, and unavailable

**Decision:** providers return `Findings`, `EvaluatedClean`, `NotEvaluated`, `Failed`, or `Cancelled`.

**Consequence:** an empty findings vector cannot masquerade as a clean evaluation.

## RULES-005 — Capability requirements are exact and declarative

**Decision:** each descriptor names the precise capabilities/partitions/fact kinds/lookups required.

**Consequence:** partial, failed, unknown, conflicted, stale, or truncated inputs block the dependent rule with a structured `NotEvaluated` record.

## RULES-006 — No private negative-authority shortcut

**Decision:** API absence uses the structured E0-B exact lookup and `wow-core` authority decision.

**Consequence:** `all coverage == Complete`, empty maps, or analyzer unresolved status cannot prove platform absence locally.

## RULES-007 — Project and reference evidence remain independent

**Decision:** findings retain project source/analyzer facts separately from reference facts/coverage/authority inputs.

**Consequence:** addon source location never becomes Blizzard platform evidence, and absent reference entities receive no fabricated source handle.

## RULES-008 — Analyzer unresolved is not WoW absence

**Decision:** an unresolved member/call fact is only a project-side candidate for exact reference verification.

**Consequence:** `wow.api.exists` emits nothing until the ReferenceView returns `authoritative_absent`.

## RULES-009 — API rule uses exact lookup only

**Decision:** no alias, fuzzy, prefix, FTS, semantic, lineage, deprecation, external, or replacement lane runs in E0.

**Consequence:** a missing API finding cannot include an unproven replacement or autofix.

## RULES-010 — API finding is per exact use site

**Decision:** E0 emits one finding per canonical unresolved direct member/call use location, deduplicated only by structured equivalent identity.

**Consequence:** users receive actionable project spans while repeated identical observations at one span do not duplicate.

## RULES-011 — `found` can be clean only for the declared API-existence scope

**Decision:** exact reference presence under usable lookup capability yields `EvaluatedClean` for `wow.api.exists`, not a blanket statement about signature, deprecation, restrictions, or runtime safety.

**Consequence:** later rule families retain separate responsibilities.

## RULES-012 — Non-authoritative API miss is `NotEvaluated`

**Decision:** partial/failed/conflicted/stale exact lookup cannot emit an API-not-found finding.

**Consequence:** missing coverage is visible rather than converted into a false positive or clean result.

## RULES-013 — API E0 remediation is `plan_only`

**Decision:** the finding instructs consumers to verify the selected profile/current contract; it proposes no replacement/edit.

**Consequence:** similarity can never mutate source.

## RULES-014 — Secret facet is first-class reference evidence

**Decision:** the Secret-local rule requires an exact unconflicted selected-profile restriction facet for the producer/return slot.

**Consequence:** annotations/type names alone cannot establish Secret status.

## RULES-015 — E0 guard semantics are fixture-bound

**Decision:** the closed E0 fixture accepts normalized `access_single` guard facts for `canaccessvalue` under a dedicated fixture policy.

**Consequence:** this proves the seam without claiming a permanent release/profile guard registry. Production guard semantics must come from the selected reference/dialect contract.

## RULES-016 — Guard must target the exact value

**Decision:** a guard applies only to the same binding/value identity established by analyzer facts.

**Consequence:** same variable name, copied name, or different binding does not satisfy the rule.

## RULES-017 — Guard must dominate the operation

**Decision:** the rule treats the operation as guarded only when the accepted guard/control-flow relation proves dominance for the exact value/use.

**Consequence:** guard after use, unrelated branch, or uncertain flow cannot become clean.

## RULES-018 — Copy/conversion does not declassify

**Decision:** copy assignment, `tostring`, `tonumber`, serialization, formatting, or `pcall` flows retain the original value relationship unless an explicit selected-profile contract says otherwise.

**Consequence:** E0 fixture mutations cannot bypass the rule through superficial transformation.

## RULES-019 — Secret-local E0 is function-local and direct

**Decision:** E0 covers one producer call, local binding/flow, direct concatenation use, and bounded guard/control-flow facts in one function.

**Consequence:** dynamic callbacks, broad interprocedural summaries, runtime predicates, containers, and arbitrary sinks remain outside scope/`NotEvaluated`.

## RULES-020 — Conditional/runtime-dependent secrecy outside fixture is unavailable

**Decision:** E0's synthetic facet is unconditional fixture data; unknown/conditional production semantics are not guessed.

**Consequence:** no static global spell whitelist or runtime generalization.

## RULES-021 — Secret-local E0 remediation is `plan_only`

**Decision:** the rule does not auto-insert a guard or rewrite the expression.

**Consequence:** semantic behavior and native sink/runtime requirements remain subject to project/runtime review.

## RULES-022 — Technical severity and rollout policy are independent

**Decision:** both E0 rules have technical severity `error` but rollout `advisory` during the E0 evaluation phase.

**Consequence:** implementation can measure false blocking before enabling blocking policy.

## RULES-023 — Root-cause keys are structured

**Decision:** root-cause/causal relations derive from rule/version/context/source/fact/lookup IDs, not message text.

**Consequence:** service can fold a generic unresolved symptom under a proven API rule finding deterministically.

## RULES-024 — Final folding remains service-owned

**Decision:** providers may emit causal hints but do not suppress/reorder the global stream.

**Consequence:** generic findings remain inspectable and independent findings are not hidden.

## RULES-025 — No finding under unavailable capability

**Decision:** a rule blocked by missing/partial/conflict inputs returns one exact `NotEvaluated` outcome for its scope rather than speculative findings.

**Consequence:** absence of a finding is never misinterpreted as success.

## RULES-026 — `EvaluatedClean` records examined scope

**Decision:** clean outcome includes rule/version/context, scope/fact/query IDs, capability/coverage IDs, and budget usage.

**Consequence:** tests can prove the provider ran and did not simply skip the path.

## RULES-027 — Findings use exact project primary spans

**Decision:** API finding primary span targets the member/reference use; Secret finding primary span targets the operation. Producer/reference evidence is related evidence.

**Consequence:** diagnostics point to the actionable source operation without conflating evidence ownership.

## RULES-028 — Message prose is not identity

**Decision:** finding/evaluation/root-cause identities use structured fields.

**Consequence:** wording/localization changes do not create duplicate/different findings.

## RULES-029 — No source edits in E0

**Decision:** E0 remediation tier is `plan_only`; no edit/recipe application is emitted.

**Consequence:** source generation mismatch and unproven transformations cannot corrupt code.

## RULES-030 — No graph dependency in E0

**Decision:** the permitted future `wow-graph` edge remains inactive.

**Consequence:** E0 rules cannot quietly depend on load/call/state graph data not yet implemented.

## RULES-031 — No per-rule IO or caches across generations

**Decision:** all inputs arrive in the execution context, and any memoization is request-local/identity-keyed without changing semantics.

**Consequence:** stale facts/reference results cannot leak into another generation.

## RULES-032 — Fixture identities freeze before implementation

**Decision:** rule input/output IDs and checksums remain null while documentation-only, then freeze after E0-A–D implementations and before first Rust code.

**Consequence:** code cannot start against mutable or invented evidence/finding vectors.

## RULES-033 — Current KB remains external

**Decision:** live Secret/event/hook/API guidance is linked rather than copied into the stable two-rule contract.

**Consequence:** current platform guidance can evolve without silently changing rule algorithms.
