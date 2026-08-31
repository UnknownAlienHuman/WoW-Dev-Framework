# E2-B bounded deterministic match engine

**Status:** normative execution semantics.

## Compilation

`compile_recognizer_pack`:

1. validates pack/fact/graph schema compatibility;
2. validates clause/capture/output DAGs;
3. derives deterministic rule and join order from schema-defined cost classes and stable IDs;
4. builds indexes over declared typed fields only;
5. computes worst-case fact/capture/output bounds;
6. rejects a plan that cannot satisfy the selected resource profile;
7. emits a canonical semantic plan digest plus noncanonical physical index plan.

Runtime statistics cannot alter semantic ordering or output identity.

## Partition execution

```text
validate exact bundle/generation/capabilities
-> choose applicable rules by declared fact partition/scope
-> preindex allowed fact fields
-> execute clauses under deterministic plan
-> bind typed captures
-> validate match confidence/ambiguity
-> construct proposed outputs/explanation
-> produce rule outcomes and partition manifest
```

One run does not mutate another rule or shared graph state.

## Join semantics

- Equality joins compare canonical typed values/IDs.
- Duplicate exact facts do not multiply identical match identity; all evidence refs remain retained.
- Many-to-many joins are allowed only under explicit cardinality/output bounds.
- Join order cannot change the logical result set.
- A fact from another scope/partition is invisible unless the bundle explicitly declares it as a dependency.

## Existence clauses

### Positive existence

`exists` returns all bounded compatible matches or a boolean witness according to rule schema. The witness facts remain in explanation/evidence closure.

### Negative existence

`not_exists` requires:

```text
closed declared search scope
complete relevant fact capability/coverage
no truncation or conflict affecting the searched relation
bounded evaluation completed
```

Otherwise outcome is `NotEvaluated` or `Partial`; it is never treated as a successful negative condition.

## Ordering and control flow

- Source order uses exact normalized ordinals/spans.
- Dominance/reachability uses supplied `wow-emmy` relations only.
- Matcher does not build a parser, symbol resolver, call graph, CFG, TOC order, XML tree, or state model.
- Unknown ordering/control-flow relation remains unavailable/Possible.

## Match confidence

### `Derived`

Allowed when all required structural predicates and endpoint identities are exact, all decisive inputs are `Proven` or accepted exact deterministic facts, coverage is complete for the required rule scope, and no ambiguity/conflict affects the output.

### `Possible`

Required when target/receiver/parent/producer/path resolution is dynamic, ambiguous, partial, or supported only by a Possible input.

`Proven` and `Candidate` are not E2 recognizer output classes.

## Ambiguity

When multiple bindings satisfy a rule:

- emit distinct matches/proposals if they represent distinct semantic possibilities;
- group them in one ambiguity record when mutually competing;
- do not choose by source order, name length, repository popularity, or first discovery;
- output remains `Possible` unless the graph schema explicitly permits independent simultaneous relations.

## Deduplication

Match key includes:

```text
pack/rule/version
input bundle and producer partition
exact generation/scope
canonical capture bindings
matched decisive fact IDs
output semantic declaration ID
```

Evidence-only duplicate facts may merge supporting refs into one match when the rule profile declares them semantically identical.

## Output amplification

Bound:

```text
matches per rule/partition
captures per match
proposals per match
join expansions
ambiguity group size
explanation steps/bytes
partition total proposals/bytes
```

Exceeding a bound stops the affected rule/partition deterministically and marks it `Partial`/truncated. A truncated partition cannot publish as complete.

## Cancellation

Check at:

- pack/bundle validation;
- index building;
- each bounded join batch;
- capture expansion;
- output/explanation construction;
- partition canonicalization.

Cancellation publishes no complete output partition and schedules no background continuation.

## Determinism

Equivalent logical inputs and profiles produce identical:

```text
compiled semantic plan digest
rule applicability/outcome
matches/captures/ambiguities
proposed assertions
coverage and explanation records
output partition digest
```

Independent of hash iteration, input serialization order, worker count, filesystem order, timing, row IDs, or process identity.

## Parallelism

Rules or independent partitions may execute in parallel only when:

- inputs are immutable;
- output buffers are isolated;
- cancellation/budget accounting is deterministic;
- final merge is canonical;
- 1/2/N workers pass byte-equality fixtures.

No rule observes another rule's unpublished output in E2-B. Multi-stage rule dependency requires a future explicit acyclic derived-fact profile, not hidden execution order.
