# Findings, clean records, root causes, and remediation

**Status:** normative E0-E output contract.

## 1. Output separation

Rule evaluation can produce:

```text
Finding
CleanEvaluationRecord
NotEvaluatedRecord
RuleFailure
CausalRelationHint
RuleExecutionCoverageReport
```

These records are independent. A scope cannot simultaneously be clean and have a finding/NotEvaluated outcome for the same rule evaluation.

## 2. Finding construction

Use the `wow-core Finding` contract. A rule-specific builder supplies:

```text
RuleId/version
semantic category
technical severity
rollout policy
GenerationContext
primary project SourceHandle
structured message arguments
project evidence IDs[]
reference evidence/authority IDs[]
rule derivation evidence IDs[]
coverage/conflict IDs[]
root-cause key
remediation
related source/evidence records
```

Validation occurs before publication to service.

## 3. Primary source policy

### API existence

Primary source is the exact unresolved member/reference use in Main project source.

Preferred span:

```text
member name
then full member reference
never whole file when exact span exists
```

### Secret local operation

Primary source is the exact operation/value-use expression in Main project source.

Producer call/binding/facet/guard sources are related evidence.

## 4. Evidence layers

### Project evidence

```text
project SourceHandle
analyzer ReferenceFact/CallFact
binding/use/operation/guard/control-flow facts
project/analyzer producer IDs and coverage
```

### Reference evidence/authority

```text
ReferenceView lookup/result ID
restriction facet and raw/source evidence
coverage and conflict records
negative-authority decision
selected profile/reference generation
```

### Rule derivation

```text
provider ID/version
rule descriptor/fixture policy
canonical decisive input IDs
rule evaluation ID
```

Never collapse these into one vague “source-confirmed” flag.

## 5. Absent entity evidence

For an authoritative absence finding, no source handle exists for the absent entity.

Evidence is:

```text
exact query key and ReferenceView identity
complete exact partition coverage
negative-authority decision
absence lookup result
producer/generation/schema identity
```

Fabricating a reference source location is prohibited.

## 6. Finding identity

```text
FindingIdentityInput
    rule ID/version
    GenerationContext ID
    primary SourceHandle/content/span
    canonical subject/entity key
    rule-specific operation/use kind
    decisive project fact IDs
    decisive reference/facet/authority IDs
    structured arguments
    provider version
```

Exclude:

```text
message prose
local temp paths
timestamps
worker order
memory addresses
external popularity/model inference
causal symptom IDs unless identity contract explicitly requires them (E0 does not)
```

## 7. Technical severity versus rollout

```text
TechnicalSeverity
    error | warning | information | hint

RolloutPolicy
    shadow | advisory | blocking
```

E0:

```text
wow.api.exists@1: severity error, rollout advisory
wow.secret.local_operation@1: severity error, rollout advisory
```

Service/app decides presentation/exit policy. Rule cannot silently promote itself to blocking.

## 8. Root-cause key

A root-cause key is a deterministic structured identity, not a rendered message.

E0 API key includes exact missing entity/use/authority inputs.

E0 Secret key includes producer return facet, exact value/operation/guard classification.

Multiple findings may share a higher-level cause but remain distinct use-site findings. Service may group them later while retaining raw records.

## 9. Causal relation hints

```text
CausalRelationHint
    cause root/finding key
    symptom finding ID
    relation kind
    exact evidence fact IDs
    confidence
```

Allowed E0 example:

```text
API authoritative absence explains same-source generic unresolved-member symptom
```

Not allowed:

- grouping by similar message;
- assuming all unknown globals are downstream;
- suppressing generic finding inside `wow-rules`;
- relation across generation/file/span mismatch.

## 10. Clean evaluation record

A clean record proves:

```text
the provider executed over the declared exact scope
all required capabilities/coverage were usable
all decisive facts/lookups were examined
no finding condition matched
budget completed
```

It carries:

```text
rule ID/version/context
evaluated scope
fact/query/facet IDs
coverage/conflict IDs
budget usage
narrow clean claim
canonical digest
```

Narrow claims:

```text
api_exists_for_exact_use
secret_fixture_operation_guarded_for_exact_value_and_scope
```

No general project/runtime safety claim.

## 11. NotEvaluated record

Must identify:

```text
rule ID/version/context
attempted scope
missing capabilities
blocking coverage/conflicts
stale/mismatch/unsupported semantic details
budget/truncation blockers
structured next evidence requirements
```

It does not include a source edit or speculative finding.

## 12. Remediation tiers

```text
exact_edit
    mechanically proven edit with generation/source preconditions

validated_recipe
    structured transformation requiring post-check

plan_only
    evidence-backed steps with no automatic mutation

candidate_only
    investigation options without proof
```

E0 allows only `plan_only`.

## 13. API remediation

```text
code: verify-current-api-contract
steps:
    confirm selected profile/reference generation
    inspect exact authoritative reference/current Blizzard contract
    establish explicit replacement/migration only from proven evidence
    change project code under the correct owner
    rerun checks/tests
required_post_checks:
    wow check
    project tests when available
runtime_scenarios:
    only if final replacement behavior is runtime-dependent
```

No replacement candidate or edit in E0.

## 14. Secret-local remediation

```text
code: restructure-secret-capable-value-use
steps:
    confirm exact producer facet/value flow
    choose documented access predicate/native sink/project architecture for real profile
    gate before all Lua evaluation using exact value and proven control-flow coverage
    avoid conversion/copy/serialization/pcall bypass assumptions
    rerun checks
    execute required client scenarios for production behavior
required_post_checks:
    wow check
    project tests
runtime_scenarios:
    combat/restriction/context scenarios only when real implementation requires them
```

No automatic guard insertion because semantics/return/control flow may change and E0 guard policy is fixture-only.

## 15. Related evidence limits

- cap related evidence count/bytes;
- order deterministically;
- retain only minimal source handles/IDs by default, not full source bodies;
- redact private/local paths;
- no hidden source value rendering for Secret-related finding;
- truncation prevents a clean/full evidence claim unless exact decisive evidence remains complete and report says partial.

## 16. Finding validation

Required operations:

```text
build_rule_finding_input
validate_rule_finding_input
build_core_finding
validate_core_finding
build_rule_root_cause_key
build_causal_relation_hint
build_plan_only_remediation
validate_rule_remediation
build_clean_evaluation_record
validate_clean_evaluation_record
build_rule_not_evaluated
validate_rule_output_exclusivity
```

## 17. Output exclusivity

For one canonical evaluation scope/rule/version/context exactly one primary outcome:

```text
Findings (one or more)
EvaluatedClean
NotEvaluated
Failed
Cancelled
```

Causal hints/coverage report are supplemental.

## 18. Deterministic ordering

Findings sort by:

```text
rule ID/version
primary file path/source handle
byte start/end
semantic category
canonical subject/entity key
operation/use kind
finding ID
```

Related evidence/remediation steps also have stable declared order.

## 19. Privacy/security

Default finding/remediation output excludes:

- raw Secret-capable value;
- large source snippets;
- local absolute path;
- credentials/private URL;
- memory address/debug object;
- arbitrary source-comment instruction;
- runtime claim without evidence.

## 20. E0 mutation tests

- remove reference evidence but still emit API finding -> fail;
- use project source as platform evidence -> fail;
- fabricate absent source handle -> fail;
- message-only finding identity -> fail;
- severity automatically becomes blocking -> fail;
- clean without scope/capability IDs -> fail;
- NotEvaluated plus empty-clean marker -> fail;
- Secret edit auto-inserted -> fail;
- fuzzy replacement included in API remediation -> fail;
- causal hint across wrong span/generation -> fail;
- raw value/local path leaked -> fail.
