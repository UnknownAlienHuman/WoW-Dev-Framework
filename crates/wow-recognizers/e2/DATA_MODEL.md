# E2-B recognizer data model

**Status:** normative.

## Pack

```text
RecognizerPack
    pack_id/version
    trust_class = core | calibration | experimental
    fact_schema_profile_id
    graph_registry_bundle_id
    applicability constraints
    rule descriptors[]
    evaluation profile ID
    budgets
    canonical digest
```

Only `core` activates in E2-B.

## Rule descriptor

```text
RecognizerRuleDescriptor
    rule_id/version
    pack_id/version
    purpose and universal role contract
    required capabilities/fact kinds
    accepted universe/scope/partition kinds
    clause graph
    capture schema
    output declarations
    confidence/ambiguity/coverage policy
    budgets/cancellation checkpoints
    fixture/evaluation IDs
    rollout state = disabled | shadow | default
    canonical digest
```

## Fact bundle

```text
RecognizerFactBundle
    bundle_id
    exact GenerationContext
    analyzer/project input snapshot IDs
    fact schema profile ID
    universe/scope/partition manifest
    normalized facts[]
    capability and coverage records
    source/evidence registries
    budget/cancellation state
    canonical digest
```

## Fact header

```text
RecognizerFactHeader
    recognizer_fact_id
    source_fact_kind
    source_fact_id
    producer ID/version
    exact generation/context
    universe/scope/partition
    source handle/evidence IDs
    confidence
    coverage IDs
```

The recognizer fact ID is an adapter identity. It never replaces the source `wow-emmy`/project fact ID.

## Normalized fact variants

```text
LuaReferenceFact
LuaCallFact
LuaAssignmentFact
LuaTableFieldFact
LuaFunctionFact
LuaOperationFact
LuaControlFlowFact
TocPackageFact
TocFileFact
TocDependencyFact
TocLoadOnDemandFact
TocSavedVariableFact
XmlTemplateFact
XmlObjectFact
XmlInheritanceFact
XmlScriptFact
ProjectOwnershipFact
```

Exact schemas live in `FACT_INPUT_MODEL.md` and the frozen profile.

## Clauses

```text
FactSelectorClause
JoinClause
FieldPredicateClause
ScopePredicateClause
ExistsClause
NotExistsClause
OrderedRelationClause
ControlFlowRelationClause
AllOfClause
AnyOfClause
```

The clause graph is acyclic and bounded. No arbitrary recursion or user function.

## Capture

```text
RecognizerCapture
    capture_id/name
    declared type/domain
    bound fact/entity/value/reference IDs
    canonical value
    source/evidence refs
```

Captures bind exact structured fields, not arbitrary source snippets.

## Compiled pack

```text
CompiledRecognizerPack
    pack ID/version/digest
    fact/graph profile IDs
    validated rule plans
    deterministic indexes/join order
    resource plan
    compatibility report
```

Physical matcher indexes are noncanonical; compiled semantic plan/digest is canonical.

## Match

```text
RecognizerMatch
    match_id
    rule/pack/version
    exact input bundle/partition/generation
    capture bindings
    matched clause/fact IDs
    derivation/explanation steps
    confidence = Derived | Possible
    ambiguity group ID: optional
    coverage/input blockers
    proposed output IDs
    canonical digest
```

## Ambiguity

```text
RecognizerAmbiguityRecord
    ambiguity_id
    rule/input scope
    competing match/output IDs
    reason class
    unresolved captures/targets
    affected capabilities/graph outputs
    canonical digest
```

## Proposed outputs

```text
ProposedEntityAssertion
    proposal_id
    graph entity kind
    semantic key ingredients
    typed attributes
    rule/match/capture/input fact refs
    source/evidence/coverage
    Derived/Possible confidence
```

```text
ProposedRelationAssertion
    proposal_id
    graph relation kind/direction
    source/target semantic key ingredients
    semantic qualifiers/attributes
    rule/match/capture/input fact refs
    source/evidence/coverage
    Derived/Possible confidence
```

Final graph keys/assertion IDs are constructed/validated by `wow-graph`.

## Rule outcome

```text
RecognizerRuleOutcome
    rule/partition IDs
    status = Matched | EvaluatedNoMatch | NotApplicable | NotEvaluated | Partial | Failed | Cancelled
    match/proposal/ambiguity IDs
    exact capability/coverage/blocker refs
    budget usage/truncation
    canonical digest
```

`EvaluatedNoMatch` is valid only when every required fact partition is Complete and the rule's closed scope was fully evaluated.

## Output partition

```text
RecognizerOutputPartition
    output_partition_id
    producer identity = pack/rule/version
    exact input bundle/scope/partition/generation
    registry profile
    outcomes/matches/proposals/ambiguities/coverage
    counts/digests
    complete/partial state
    canonical digest
```

## Evaluation

```text
RecognizerCorpusManifest
    corpus ID/version
    pinned fixture/project/repository provenance
    labeled expected roles/relations/nonmatches/unknowns
    mutation definitions
    license/security state
```

```text
RecognizerEvaluationReport
    pack/rule/corpus IDs
    TP/FP/FN/TN where labelable
    Unknown/NotEvaluated/Partial/Truncated counts
    precision/recall and denominators
    repository/path/name mutation results
    determinism and budget results
    promotion blockers
    canonical digest
```
