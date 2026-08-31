# Graph conflict, coverage, provenance, and confidence

**Status:** normative.

## Provenance

Assertions retain the original `wow-core` provenance classes. Graph producer adapters cannot relabel external/community/model evidence as platform/project source.

## Confidence

```text
Proven      direct explicit structural/contract evidence
Derived     deterministic rule over proven inputs
Possible    structure permits but cannot prove the relation
Candidate   discovery hypothesis only
```

Graph aggregation cannot upgrade confidence. A derived path is not a direct edge.

## Conflicts

Conflict classes include:

```text
exclusive-attribute-disagreement
relation-multiplicity-violation
forbidden-cycle
endpoint-kind-disagreement
cross-scope-generation-conflict
producer-identity-or-schema-conflict
evidence-or-source-handle-conflict
coverage-versus-assertion-conflict
```

Every conflict links exact assertions and affected capabilities/axes. Resolution may select a reviewed correction/producer policy later, but rejected assertions remain traceable according to retention policy.

## Coverage axes

Keep separate:

- producer input coverage;
- recognizer/rule coverage;
- graph partition publication coverage;
- graph registry/schema coverage;
- query traversal coverage/budget;
- store/read validation coverage.

A complete graph write does not make an incomplete source partition complete.

## Negative authority

Graph can report authoritative relation/entity absence only when:

- exact scope/snapshot/kind/relation known;
- all relevant producer partitions declare Complete coverage for the capability;
- graph publication/store/read validation complete;
- no unresolved conflict or truncation affects the subject;
- query was not candidate-inclusive or budget-truncated.

Otherwise return partial/conflict/NotEvaluated.

## View aggregation

- preserve all supporting assertion IDs;
- retain competing assertions/conflicts;
- field merge policy comes from registry;
- summaries cite exact coverage records;
- no hidden majority vote, last write, popularity, or model judgment.

## Producer disable/update

Replacing/removing a producer partition:

- removes only its assertions;
- recomputes views/conflicts;
- downgrades coverage where applicable;
- cannot mutate other assertions;
- yields a new graph generation.
