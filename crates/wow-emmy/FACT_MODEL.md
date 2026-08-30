# `wow-emmy` E0-C normalized fact model

**Status:** normative framework-owned analyzer fact contract.

These facts describe source/analyzer observations. They do not contain World of Warcraft platform authority, restriction verdicts, load ownership, or rule conclusions.

## 1. Common fact envelope

```text
AnalyzerFactHeader
    fact_id
    fact_kind
    producer_id
    producer_version
    analyzer_snapshot_id
    generation_context
    file_id
    source_handle_id
    source_span
    confidence
    coverage_id
```

E0 facts are normally `Proven` when directly established by syntax/semantic analysis in the selected snapshot, or `Possible` when resolution is incomplete/dynamic. The adapter cannot mark platform claims as `Proven`.

## 2. File fact

```text
FileFact
    header
    workspace_role: Main | Library
    normalized_relative_path
    content_digest
    byte_length
    parse_state
    semantic_state
```

A Library file remains distinguishable from a Main file in every public fact.

## 3. Syntax node identity

```text
SyntaxNodeRef
    file_id
    node_kind
    byte_range
    syntax_path_or_stable_local_key
```

This is a snapshot-local source identity. It is not a durable cross-edit entity key unless content/range identity remains valid.

## 4. Reference fact

```text
ReferenceFact
    header
    reference_kind
    spelling
    receiver_spelling: optional
    member_spelling: optional
    resolution_status
    resolved_symbol_key: optional String
    upstream_resolution_detail: optional normalized data
```

`reference_kind` E0 values:

```text
global
member
local
annotation_type
```

`resolution_status`:

```text
resolved
unresolved
ambiguous
dynamic
not_evaluated
```

### Examples

```text
C_E0Fixture
    global reference, resolved from library

C_E0Fixture.KnownApi
    member reference, resolved

C_E0Fixture.RemovedApi
    member reference, unresolved
```

An unresolved member fact means only that the selected analyzer snapshot did not resolve it. It does not prove absence from the selected WoW profile.

## 5. Call fact

```text
CallFact
    header
    callee_reference_fact_id
    call_kind
    argument_spans[]
    result_use_kind
    resolved_callable_key: optional String
```

`call_kind`:

```text
direct_global
direct_member
direct_local
dynamic
```

`result_use_kind`:

```text
discarded
assigned_local
returned
passed_argument
nested_expression
unknown
```

The fact records source call structure; it does not validate WoW argument contracts.

## 6. Local binding fact

```text
LocalBindingFact
    header
    binding_key
    name
    declaration_span
    initializer_expression_span: optional
    initializer_call_fact_id: optional
    scope_span
```

`binding_key` is stable within one analyzer snapshot/function scope. It is not a global entity identity.

E0 required example:

```lua
local text = C_E0Fixture.SecretText()
```

The binding links `text` to the producer call structurally. It does not mark `text` Secret.

## 7. Local value-flow edge

```text
LocalFlowEdge
    header
    from_value_key
    to_value_key
    flow_kind
    source_expression_span
```

E0 `flow_kind` values:

```text
initializer
copy
argument_pass
return_flow
conversion_call
unknown
```

The adapter may report a copy/conversion flow. It must not label conversion as declassification.

## 8. Local use fact

```text
LocalUseFact
    header
    binding_key
    use_span
    operation_fact_id: optional
    control_flow_region_id
```

Every use is tied to one binding/value identity where resolution proves it. Name equality alone is insufficient across shadowed scopes.

## 9. Operation fact

```text
OperationFact
    header
    operation_kind
    operand_value_keys[]
    operand_spans[]
    result_value_key: optional
    containing_function_key
    control_flow_region_id
```

E0 `operation_kind` values:

```text
concatenation
comparison
arithmetic
branch_condition
format_or_logging_call
table_key_or_index
conversion_call
copy_assignment
```

Only operation kinds required by E0 fixtures must be implemented. The enum may be open/versioned in later milestones.

An operation fact states that the operation exists. It does not state that the operation is permitted for Secret values.

## 10. Guard fact

```text
GuardFact
    header
    guard_kind
    callee_reference_fact_id
    guarded_value_keys[]
    predicate_result_value_key: optional
    branch_or_region_id
```

E0 recognized structural guard call:

```text
canaccessvalue(value)
```

`guard_kind` values:

```text
access_single
access_tuple
secret_test
unknown_guard_call
```

The selected profile/reference contract determines whether a guard kind is accepted for a rule. `wow-emmy` only reports the call/arguments/control-flow relationship.

## 11. Control-flow region

```text
ControlFlowRegion
    region_id
    containing_function_key
    parent_region_id: optional
    region_kind
    byte_range
```

E0 `region_kind`:

```text
function_body
if_then
if_else
early_return_tail
block
```

## 12. Control-flow relation

```text
ControlFlowRelation
    header
    relation_kind
    from_region_or_fact_id
    to_region_or_fact_id
```

E0 `relation_kind`:

```text
dominates
post_dominates
same_region
precedes_without_dominance
conditional_reachability
```

The adapter emits `dominates` only when established by the implemented control-flow model. Unknown/dynamic cases remain absent or explicit `Possible`, never guessed.

## 13. Function scope fact

```text
FunctionScopeFact
    header
    function_key
    declaration_or_chunk_span
    parameter_binding_keys[]
    root_control_flow_region_id
```

Chunk-level code may use a synthetic snapshot-local function key.

## 14. Generic diagnostic observation

Before conversion to a core `Finding`, retain an internal normalized observation:

```text
GenericDiagnosticObservation
    observation_id
    upstream_code
    upstream_severity
    upstream_message_template_or_category
    normalized_category
    source_span
    structured_arguments
    related_spans[]
    analyzer_snapshot_id
    coverage_id
```

See [`DIAGNOSTIC_NORMALIZATION.md`](DIAGNOSTIC_NORMALIZATION.md).

## 15. Fact set

```text
AnalyzerFactSet
    fact_set_id
    analyzer_snapshot_id
    generation_context
    file_id
    capability_ids[]
    coverage_ids[]
    facts[]
    canonical_digest
```

Rules:

- all facts belong to one file/snapshot/context;
- facts sort by kind, byte range, stable structured key, and fact ID;
- all references resolve within the snapshot/global normalized registries;
- canonical digest excludes upstream object addresses and discovery order.

## 16. E0-C required source cases

### `clean.lua`

Required facts:

```text
resolved global C_E0Fixture
resolved member KnownApi
resolved direct member call
ordinary local assignment/use as fixture requires
```

No generic diagnostic for selected category.

### `generic-error.lua`

Required:

- exact built-in diagnostic observation/finding;
- valid source span;
- parse/semantic capabilities according to selected diagnostic case;
- no fabricated unrelated facts.

### `missing-api.lua`

Required facts:

```text
resolved global C_E0Fixture
unresolved member RemovedApi
call fact tied to unresolved member
```

Prohibited:

```text
API absent from selected WoW profile
replacement candidate
WoW diagnostic finding
```

### `secret-local.lua`

Unsafe variant required facts:

```text
resolved member/call SecretText
local binding initialized from producer call
local use tied to exact binding
selected operation fact (concatenation in E0 fixture)
no dominating access guard
```

Guarded variant required facts:

```text
same producer/binding
resolved canaccessvalue call or configured known-global reference
GuardFact for exact binding
if_then region
proven dominates relation from accepted branch/guard region to operation use
```

Guard-after-use variant:

```text
guard call present
operation precedes guard without dominance
```

Different-value guard variant:

```text
guard targets another binding/value key
no guard relation for producer binding
```

Again, no fact says the producer result is Secret.

## 17. Fact confidence rules

### `Proven`

Use when the adapter can point to exact syntax/semantic source and deterministic resolution in the snapshot.

### `Possible`

Use for dynamic/ambiguous resolution or bounded control-flow possibility.

### `Candidate`

Normally not emitted by `wow-emmy`; semantic/external candidates belong to later discovery layers.

No model inference upgrades a fact.

## 18. Coverage requirements

Each fact set names exact capabilities, e.g.:

```text
emmy.file.parsed
emmy.fact.references
emmy.fact.calls
emmy.fact.local_bindings
emmy.fact.local_flow
emmy.fact.operations
emmy.fact.guards
emmy.fact.control_flow
```

If parsing/resolution/control-flow capability is unavailable, dependent facts are absent and the coverage record explains why. An empty list alone is not a clean negative result.

## 19. Public seam

Consumers receive read-only normalized fact sets or narrow queries such as:

```text
facts_for_file(file_id, required_capabilities)
references_in_span(file_id, byte_range)
resolved_call_at(file_id, byte_offset)
local_flow_slice(function_or_binding_key)
guard_relations_for(binding_key, operation_fact_id)
```

Concrete query names may differ, but no raw upstream semantic model/CST reference escapes.

## 20. Invalidation

A fact is current only when:

- its file content digest matches snapshot manifest;
- configuration/library dependencies used for the fact match snapshot identity;
- its project generation matches;
- its capability coverage is current;
- source span validates.

An update invalidates all dependent fact IDs/digests. Reused unchanged facts retain semantic equality but belong to the newly validated snapshot/context only through explicit publication.
