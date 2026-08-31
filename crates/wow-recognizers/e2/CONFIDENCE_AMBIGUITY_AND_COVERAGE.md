# Recognizer confidence, ambiguity, coverage, and no-match semantics

**Status:** normative.

## Confidence

### Derived

A match/output is `Derived` only when:

- rule/pack/fact/graph profiles are exact and compatible;
- every decisive input fact and identity is exact for the selected generation;
- all required clauses evaluate completely;
- relevant capabilities/partitions are Complete;
- no dynamic/ambiguous/conflicting capture affects the output;
- the rule derivation is deterministic and frozen.

### Possible

Required when:

- receiver/callee/parent/template/target/path is dynamic or ambiguous;
- input confidence is Possible;
- optional cross-partition dependency is incomplete;
- multiple mutually exclusive capture bindings remain;
- exact producer/owner/endpoint cannot be proven.

Recognizer outputs are never `Proven` or `Candidate` in E2-B.

## Ambiguity reasons

```text
multiple-resolved-targets
dynamic-receiver-or-callee
multiple-custom-event-producers
unresolved-parent-or-template
state-path-dynamic-segment
conflicting-project-or-analyzer-facts
cross-partition-coverage-gap
output-key-collision-before-graph-validation
```

Ambiguity records retain every competing match/proposal and the exact fields/facts causing uncertainty.

## Coverage layers

Keep separate:

```text
source/analyzer/TOC/XML fact coverage
fact-adapter coverage
rule applicability coverage
rule clause/match coverage
output/proposal construction coverage
graph proposal validation coverage
corpus label/evaluation coverage
```

A complete matcher run cannot upgrade partial source fact coverage.

## Rule outcomes

### `Matched`

At least one complete nontruncated match/output exists. It may contain Possible/ambiguity records.

### `EvaluatedNoMatch`

Allowed only when:

- all required capabilities Complete;
- declared closed scope fully scanned;
- no conflict/truncation/cancellation;
- negative clauses, if any, satisfied under their complete scopes.

It is a rule-local result, not proof that a semantic role cannot exist through unsupported/dynamic patterns outside the declared rule scope.

### `NotApplicable`

Fact/partition/scope does not meet declared applicability; not a missing capability.

### `NotEvaluated`

Required capability/profile/registry/input unavailable or conflicting before reliable evaluation.

### `Partial`

Useful matches exist or part of scope evaluated, but fact/matcher/output budget, unsupported fact shape, dependency gap, or truncation prevents complete evaluation.

### `Failed`

Contract/invariant/internal validation error. No complete output partition.

### `Cancelled`

Cancellation before publication; no background continuation.

## Negative authority

Recognizers do not generally own project-wide negative authority. They may state only:

```text
this exact rule found no match in this exact closed fully covered partition
```

They cannot state:

```text
this architecture/role/event/hook/state path does not exist anywhere
```

unless a future higher-level contract combines all relevant complete producer capabilities.

## Custom event example

```text
RegisterCallback("X") found
TriggerEvent("X") found in closed complete scope
    -> Derived custom subscription relation

RegisterCallback("X") found
producer scope partial or dynamic
    -> Possible unresolved subscription + blocker

RegisterCallback("PLAYER_LOGIN") found
no TriggerEvent producer under complete local scope
    -> evaluated structural registration remains, but not a custom-event/native-event proof
```

## Hook example

A `hooksecurefunc` match can be Derived as a structural hook relation when target/callback resolve. The following remain unknown:

```text
target safety/accessibility
protected/managed/forbidden status
taint consequences
combat legality
call frequency/performance
runtime success
```

Those are separate reference/rule/runtime capabilities.

## State example

TOC-declared root plus exact literal chain can be Derived. Dynamic key suffix is Possible. A same-named undeclared global is not a state root.

## Corpus metrics

Reports preserve denominators:

```text
true positive
false positive
false negative
true negative where meaningful
unknown/unlabeled
NotEvaluated
Partial
truncated
incompatible profile
```

Precision/recall exclude no item silently; report exactly which categories are included in each metric.

## Promotion

A rule moves from shadow to default only when:

- all contract fixtures pass;
- mutation invariance proves no repository/path/name overfitting;
- structurally similar negatives are present;
- dynamic/partial cases stay honest;
- graph output validates;
- frozen evaluation profile thresholds pass;
- unresolved labels and coverage are reported;
- no named repository branch condition exists.

E2 documentation does not freeze arbitrary numeric thresholds. The implementation/evaluation profile must freeze them before promotion.
