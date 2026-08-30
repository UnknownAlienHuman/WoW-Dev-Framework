# Root-cause and presentation folding

**Status:** normative E0-F diagnostic presentation projection.

Folding produces a concise deterministic presentation graph while preserving every raw finding/outcome record. It is not destructive deduplication and does not change evidence, severity, rollout, or finding identity.

## 1. Inputs

```text
raw generic findings
raw WoW rule findings
rule clean records
rule NotEvaluated records
rule failures
component failures/warnings
rule CausalRelationHints
component dependency/blocker records
exact core duplicate identities
one ServiceContextLease
```

All inputs belong to the same exact context.

## 2. Outputs

```text
FindingPresentationGraph
    all presentation nodes
    all raw record references
    display root IDs
    primary parent edges
    secondary/competing relation edges
    independent/orphan roots
    deterministic ordering
    canonical digest
```

Raw records remain separately present in the result envelope.

## 3. Node kinds

```text
finding
not_evaluated
component_failure
warning
```

Clean evaluation records are normally summarized outside the diagnostic root graph because they are not problems/blockers. They remain in the envelope.

## 4. Relation kinds

### `causes_or_explains`

A proven root finding explains a downstream symptom.

E0 example:

```text
wow.api.exists finding
    -> same-source/same-fact generic unresolved-member symptom
```

Requires a valid `wow-rules` causal hint or an equivalent lower-layer structured relation.

### `blocked_by`

A component/capability failure prevents a rule/scope from evaluation.

Examples:

```text
annotation library failure
    -> API/Secret NotEvaluated records requiring resolution/facts

reference facet conflict
    -> Secret rule NotEvaluated
```

### `exact_duplicate_of`

Two records are structurally equivalent under exact canonical duplicate policy.

This relation affects presentation only. Raw IDs remain.

### `related_competing_cause`

A child/symptom has another valid evidence-backed possible/secondary cause. E0 retains the relation but selects one primary parent deterministically.

## 5. Relation admission

Every edge requires:

```text
same GenerationContext / ServiceContextLease
valid source/fact/finding/outcome IDs
relation-specific exact evidence IDs
confidence allowed by relation contract
no stale/mismatched record
no message-text-only rationale
```

Invalid/unproven hints are rejected or reported as service folding failures; they are not silently used.

## 6. API causal relation

A `wow.api.exists` finding may explain a generic analyzer symptom only when all hold:

```text
same ProjectGenerationId / AnalyzerSnapshotId
same Main project file/content
same exact member/reference fact or exact compatible span
compatible generic semantic category frozen by E0-C
API reference absence authoritative
valid wow-rules CausalRelationHint
```

The API finding becomes a display root; generic symptom becomes child.

No relation when:

- only message wording/name matches;
- different span/file/generation;
- reference miss nonauthoritative;
- generic family unclassified;
- hint absent/invalid.

## 7. Component blocker relation

A component failure may block rule NotEvaluated records when the rule requirements identify that capability/partition.

Example:

```text
component failure node:
    annotation library health failed

children:
    API rule NotEvaluated for missing reference facts
    Secret rule NotEvaluated for missing producer/local facts
```

Do not attach unrelated generic diagnostics or rules whose required capabilities remain usable.

## 8. Exact duplicates

Exact duplicate policy uses core finding identity/equivalence, not message text.

Rules:

- identical raw finding ID cannot appear twice in canonical raw set;
- distinct finding IDs may be exact duplicates only if structured identity/evidence contract explicitly permits;
- choose canonical representative by core total order;
- add `exact_duplicate_of` edge;
- raw duplicate records remain accessible;
- distinct source spans never exact duplicates in E0.

## 9. Primary parent selection

E0 allows at most one primary presentation parent per child.

Candidate parent precedence:

1. exact deterministic `blocked_by` mandatory component/root failure;
2. proven `causes_or_explains` root finding;
3. exact duplicate canonical representative;
4. otherwise no primary parent.

When multiple same-precedence valid parents exist:

- compare relation confidence/evidence authority according to contract;
- then canonical parent record order;
- select one primary parent;
- retain others as `related_competing_cause`/secondary edges;
- record selection rule.

Never choose first-returned or shortest message.

## 10. Acyclicity

Before publication:

- no self edge;
- no directed cycle;
- primary-parent graph forms a forest;
- secondary relations cannot create an invalid causal cycle according to graph contract;
- every child/root references a known raw/problem record;
- every non-child problem record appears as root/independent node.

Cycle detection failure rejects the envelope.

## 11. Root determination

A problem node is a display root when:

- no valid primary parent;
- parent record missing/invalid;
- relation intentionally secondary only;
- independent finding/failure/warning.

Root order uses referenced record canonical order, not severity/message length alone.

## 12. Raw preservation

Result envelope contains:

```text
raw_findings[]
not_evaluated_records[]
component_failures/warnings[]
presentation_graph
```

Folding cannot:

- delete a raw finding;
- mutate finding source/evidence/severity/rollout/remediation;
- rewrite NotEvaluated blockers;
- replace generic finding ID with root finding ID;
- count presentation roots as raw finding count.

Counts report both raw and display-root/child values.

## 13. Semantic status interaction

Status uses raw outcomes/completeness, not number of display roots.

Examples:

- 2 raw findings folded into 1 root+1 child -> status still `findings`;
- 1 finding + 2 blocked NotEvaluated children -> `partial`;
- all blocked, no findings -> `partial`, not clean;
- graph validation failure -> `failed`.

## 14. Warnings

Warnings can be roots or children only when structured relation exists. Informational warnings unrelated to analysis completeness may remain separate warning list rather than diagnostic graph.

## 15. Canonical identity

Graph identity includes:

```text
GenerationContext / ServiceContextLease ID
referenced raw record IDs
nodes and kinds
edges/relation kinds/evidence/confidence/primary flag
root order
primary-parent selection records
folding contract version
```

Exclude rendered labels, indentation, UI expansion state, message prose, timestamps, and input return order.

## 16. Required operations

```text
build_presentation_nodes
validate_causal_hint
build_component_blocker_relations
build_exact_duplicate_relations
select_primary_presentation_parents
retain_competing_relations
validate_presentation_graph_references
detect_presentation_cycles
derive_display_roots
canonicalize_presentation_graph
derive_presentation_graph_id
```

## 17. Required tests

- API authoritative root + exact generic symptom child;
- no hint for same text/different span;
- library failure blocks only dependent rule outcomes;
- independent generic type/Secret findings remain roots;
- exact duplicate relation and canonical representative;
- distinct source spans not duplicates;
- two valid parents deterministic primary + retained secondary;
- invalid relation evidence rejected;
- cross-generation relation rejected;
- self/cycle rejected;
- orphan problem becomes root;
- raw counts/IDs unchanged after folding;
- status based on raw outcomes/blockers, not root count;
- shuffled input/edge order -> identical graph bytes/digest;
- message wording/UI ordering -> no identity change.

## 18. Hard stops

- no message-text grouping;
- no raw finding deletion;
- no severity/remediation mutation;
- no first-returned parent selection;
- no multiple primary parents;
- no cycles/self edges;
- no cross-generation edge;
- no root-count-based clean/findings status;
- no rule-level final suppression duplicated here;
- no UI-specific mutable tree state in canonical graph.
