# `wow.secret.local_operation@1`

**Status:** normative E0-E rule algorithm.

## 1. Purpose

Report one supported direct function-local operation on a value produced by an exact selected-profile `secret.return` contract when no accepted guard for the exact value provably dominates that operation.

E0 proves the evidence join and local-flow/guard contract. It does not solve general interprocedural Secret flow, dynamic runtime secrecy, containers, managed objects, combat/protected behavior, or arbitrary sinks.

## 2. Descriptor

```text
rule_id: wow.secret.local_operation
version: 1
semantic_category: wow.secret.unsafe_local_operation
technical_severity: error
rollout_policy: advisory
remediation_tiers: plan_only
source_scope: one function-local producer -> binding -> direct operation
supported operation: concatenation
supported facet: secret.return, return_position 1, unconditional_fixture
supported profile: fixture-retail-120100-e0-v1
```

## 3. Required producer facts

```text
ReferenceFact:
    member C_E0Fixture.SecretText
    resolution_status = resolved
    resolved_symbol_key = function:C_E0Fixture.SecretText

CallFact:
    direct member call
    result_use_kind = assigned_local

LocalBindingFact:
    exact initializer call fact
    stable binding/value key
    containing function/scope
```

The rule does not infer the producer from spelling alone when semantic resolution is absent/ambiguous/dynamic.

## 4. Required reference facet

Exact E0-B restriction lookup:

```text
entity: function:C_E0Fixture.SecretText
facet kind: secret.return
target: return_position:1
applicability: unconditional_fixture
outcome: found
coverage: Complete
conflict: none
selected profile/reference generation coherent
```

The facet's raw/reference source, evidence, producer, coverage, and generation IDs are retained.

Annotations, Lua types, comments, or function name do not substitute for this facet.

## 5. Required local use/operation facts

```text
LocalUseFact
    exact binding/value key
    exact use span

OperationFact
    operation_kind = concatenation
    exact operand includes the same value key
    exact operation span
    same containing function
    control-flow region ID
```

E0 operation example:

```lua
local text = C_E0Fixture.SecretText()
return text .. "!"
```

## 6. E0 fixture guard policy

```text
policy_id: wow-rules-e0-fixture-policy/1
recognized structural guard callee: canaccessvalue
guard_kind: access_single
accepted target arity: one exact value
required relation: guard/accepted branch dominates the operation
```

This is synthetic fixture semantics. It proves rule plumbing only.

Production guard/predicate semantics must later come from selected reference/dialect contracts and current KB/runtime guidance. Do not generalize the fixture policy globally.

## 7. Guard applicability

A guard protects an operation only when all hold:

1. guard fact is recognized by the active fixture/profile rule policy;
2. guarded value key is exactly the producer-derived value used by the operation;
3. guard arguments/resolution are exact, not ambiguous/dynamic;
4. control-flow relation proves the accepted branch/region dominates the operation;
5. guard and operation belong to the same supported function-local flow slice;
6. no intervening unsupported flow invalidates identity/guard relation;
7. source/generation/capability records are current.

Variable-name equality alone is insufficient.

## 8. Decision table

| Facet/input | Guard/control flow | Rule outcome |
|---|---|---|
| exact unconditional `secret.return`; direct concat | no guard | finding |
| exact facet; direct concat | exact-value accepted guard dominates | `EvaluatedClean` |
| exact facet; direct concat | guard after operation | finding |
| exact facet; direct concat | guard on different value/binding | finding |
| exact facet; direct concat | guard in unrelated/non-dominating branch | finding |
| exact facet; copy/conversion then concat, exact flow retained | no dominating guard for resulting use | finding when E0 facts support flow |
| exact facet | operation kind unsupported | `NotEvaluated` or nonapplicable according to scope |
| facet partial/conflict/unavailable | any | `NotEvaluated` |
| producer/binding/flow/operation/control-flow fact unavailable | any | `NotEvaluated` |
| conditional/runtime-dependent facet unsupported by E0 | any | `NotEvaluated` |
| ordinary producer with authoritative no matching facet | supported scope fully evaluated | `EvaluatedClean` only for this rule condition |
| source/profile/generation mismatch | any | context failure |

## 9. Finding primary source

Primary span: exact operation expression (`text .. "!"`) or exact value use inside the operation when the contract selects it consistently. E0 fixture freezes one policy before implementation.

Related project evidence:

- producer member/call span;
- binding declaration/initializer;
- local use;
- operation;
- guard/control-flow facts when present.

Related reference evidence:

- restriction facet and target slot;
- raw/source evidence;
- facet coverage/producer/generation.

## 10. Finding arguments

```text
producer_entity_key
producer_return_position
facet_kind = secret.return
facet_applicability = unconditional_fixture
operation_kind = concatenation
guard_state:
    absent
    after_use
    different_value
    non_dominating
    unsupported_flow (normally NotEvaluated, not finding)
scope_kind = function_local
selected_profile_id
```

Non-normative message example:

```text
This concatenation uses a value produced by a `secret.return` contract without an accepted dominating access guard.
```

No hidden value or source payload is rendered.

## 11. Finding identity

Canonical fingerprint includes:

```text
rule ID/version
GenerationContext ID
primary operation SourceHandle/span/content digest
producer entity key + return position
producer CallFact ID
binding/value key
LocalUseFact/OperationFact IDs
facet lookup/facet/evidence IDs
guard classification and decisive guard/control-flow fact IDs
fixture policy ID
provider version
```

Excludes rendered prose, timestamps, discovery order, temp paths, and runtime guesses.

## 12. Guard classifications

### `dominating_exact_value`

Accepted guard targets exact value and proven relation dominates operation. Clean branch.

### `absent`

No accepted guard fact for exact value in supported flow. Finding.

### `after_use`

Guard exists but operation precedes it/no dominance. Finding.

### `different_value`

Guard targets another binding/value. Finding.

### `non_dominating`

Guard exists in unrelated/conditional branch without proven dominance. Finding.

### `unknown_or_unsupported`

Facts/control flow insufficient. `NotEvaluated`, not a speculative finding/clean.

## 13. Copy/conversion rules

The analyzer may report:

```text
copy LocalFlowEdge
conversion_call LocalFlowEdge/OperationFact
```

Rule semantics:

- copy preserves tracked relation;
- conversion/formatting/serialization/`pcall` is not declassification;
- if exact flow to supported operation remains proven and no accepted dominating guard applies, finding;
- if exact flow is not supported/proven, `NotEvaluated`;
- never assume conversion yields an ordinary value.

## 14. Ordinary/no-facet clean outcome

A fully evaluated exact producer whose restriction lookup authoritatively contains no matching `secret.return` facet may yield clean for this rule's trigger condition.

This does not imply the value is always safe/nonsecret in every runtime context unless the selected reference contract explicitly proves that. E0 fixture uses narrow synthetic semantics.

When facet absence itself lacks authority, return `NotEvaluated`.

## 15. NotEvaluated cases

- producer/member/call unresolved/ambiguous/dynamic;
- return-slot relation unavailable;
- binding/value identity unavailable or shadowing unresolved;
- local flow crosses unsupported dynamic/interprocedural boundary;
- operation kind unsupported;
- operation/source span stale/invalid;
- guard facts unavailable/ambiguous;
- dominance/control-flow capability partial/failed;
- facet lookup partial/conflict/unavailable/profile mismatch;
- facet conditional/runtime-dependent beyond fixture support;
- rule fixture policy mismatch/missing;
- budget/truncation prevents complete selected-scope evaluation;
- requested project generation not current/matching.

## 16. Clean outcome

Guarded fixture clean record contains:

- exact producer/facet/binding/use/operation IDs;
- accepted guard fact/value key;
- proven dominance relation;
- all capability/coverage IDs;
- fixture policy ID;
- budget usage;
- narrow clean claim:

```text
this exact supported fixture operation is under an accepted dominating guard for the same value
```

It does not certify runtime/general Secret safety.

## 17. Remediation

Tier: `plan_only`.

Structured plan:

1. confirm the selected profile/reference facet and exact value flow;
2. choose a documented access predicate/native sink/architecture appropriate to the real project/profile;
3. ensure any guard occurs before and dominates every Lua evaluation of the exact value;
4. avoid conversion/copy/serialization/`pcall` as a bypass;
5. rerun static checks;
6. perform required in-client scenario tests for production/runtime-dependent behavior.

E0 emits no edit because inserting a guard may alter control flow/return semantics and fixture policy is not global production authority.

## 18. Required operations

```text
is_secret_local_scope_applicable
resolve_secret_producer_and_return_slot
evaluate_secret_local_capabilities
classify_secret_facet_lookup
trace_supported_local_value_flow
classify_secret_operation
classify_applicable_guard
classify_guard_dominance
build_secret_local_finding
build_secret_local_clean_record
build_secret_local_not_evaluated
validate_secret_local_outcome
```

## 19. Fixture cases

```text
secret.unsafe-concat
secret.guarded-concat
secret.guard-after-use
secret.different-value-guard
secret.unrelated-branch-guard
secret.copy-then-concat
secret.conversion-then-concat
secret.facet-conflict
secret.facet-partial
secret.control-flow-unavailable
secret.dynamic-producer
secret.shadowed-binding
secret.policy-mismatch
secret.ordinary-authoritative-no-facet
secret.no-facet-without-authority
secret.budget-truncation
```

## 20. Hard stops

- no annotation/name/type-only Secret inference;
- no guard by variable name alone;
- no guard after use accepted;
- no different-value guard accepted;
- no guessed dominance;
- no conversion/copy/serialization/`pcall` declassification;
- no permanent spell whitelist;
- no runtime/general safety claim;
- no finding/clean under facet/control-flow conflict or partial coverage;
- no automatic guard insertion/edit;
- no broad interprocedural flow in E0.
