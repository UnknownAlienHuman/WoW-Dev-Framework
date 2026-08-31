# Type lowering contract

**Status:** normative E1-C ReferenceView type/fact to consumer-neutral and LuaCATS/Emmy projection contract.

Every input type/member/signature/restriction position receives an explicit lowering result. Rendering cannot silently widen, omit, reorder, or invent semantics.

## 1. Two-stage lowering

```text
Reference type/fact model
-> consumer-neutral AnnotationType graph
-> consumer-profile-specific rendered type syntax
```

The first stage preserves semantic identity independently of EmmyLua/LuaLS syntax. The second stage records consumer limitations and loss.

## 2. Lowering result

```text
TypeLoweringResult
    result_id
    input ReferenceEntity/Fact/Member/Type/Restriction IDs
    semantic AnnotationType ID(s)
    consumer profile(s)
    lowering rule/version
    status: Exact | ExactWithSidecar | LossyDeclared | Unsupported | NotEvaluated
    loss/sidecar/status/coverage refs
    canonical digest
```

No input is dropped without a result.

## 3. Primitive types

Freeze exact mapping for ReferenceView primitives:

```text
nil -> nil
boolean -> boolean
integer -> integer when consumer semantics verified; otherwise number + loss
number -> number
string -> string
explicit any -> any only when source contract explicitly means unconstrained
explicit unknown -> unknown/consumer extension/sidecar according to profile
```

Do not use `any` for unresolved/unsupported input by convenience.

## 4. Literal types

Exact boolean/string/integer/number literal types lower only when consumer profile supports them semantically. Otherwise:

```text
base primitive + ExactWithSidecar or LossyDeclared
```

Retain literal value in sidecar/source map/loss record. Numeric representation must match the frozen number policy.

## 5. Named types

```text
Reference named entity/type -> AnnotationType::Named(exact declaration/entity ID)
```

Rules:

- target must resolve in exact selected ReferenceGeneration;
- rendered name derived by identifier/name profile, not source path;
- unresolved target -> Unsupported/NotEvaluated or explicit unresolved profile form with loss;
- ambiguous target -> conflict, no first match;
- cross-profile target forbidden;
- aliases/classes/interfaces are semantically distinguished even if rendered similarly.

## 6. Optionality and nilability

Represent separately:

```text
parameter may be omitted
field optional/missing
value type includes nil
return may be absent/no value
explicit default value
unknown optionality
```

Consumer syntax may combine these forms; lowering record must preserve original distinction and classify any collapse.

Examples:

```text
optional parameter with nonnil type
    semantic: optional=true, type=T
    rendered profile may use `param?` or `T|nil` only if exact semantics verified

nullable required parameter
    semantic: optional=false, type=T|nil
```

No blanket `?`/`|nil` substitution without consumer probe.

## 7. Arrays, maps, and collections

Semantic types:

```text
Array<T>
Map<K,V>
Record/inline fields when source explicitly defines
```

Render according to consumer profile (`T[]`, `table<K,V>`, named alias/class, etc.). Mixed tables or unknown key/value shapes become explicit loss/sidecar/unsupported.

Do not infer arrays from consecutive numeric keys in raw examples unless the source type contract says array.

## 8. Tuples and multiple returns

Preserve ordered return positions and tuple semantics separately from a single table/union.

- multiple Lua returns remain ordered member records;
- tuple type inside one value remains a semantic tuple only if source says so;
- consumer unable to express tuple gets loss record;
- omitted/unknown return is not `nil` or `any` automatically;
- return names/docs/restrictions stay position-bound.

## 9. Unions

- canonicalize exact duplicate variants;
- preserve variant structural order policy or canonical sort as profile defines;
- retain nil variant distinctly;
- no union member dropped for convenience;
- broad union/widening recorded as LossyDeclared;
- conflicting/unresolved variant blocks exact lowering.

Consumer-specific union syntax and size limits are profile-bound. Excessive union size cannot silently widen; explicit loss/budget record required.

## 10. Intersections and advanced types

Only activate when ReferenceView type model and consumer probe support exact semantics. Otherwise sidecar/unsupported.

Do not invent generics/intersections to imitate another tool without source and consumer contract.

## 11. Function/callback types

Semantic function type retains:

```text
ordered parameters/returns
optionality/nilability/variadic/defaults
receiver/self semantics
names/docs/restrictions
```

Renderer may emit callback alias/class/function tag according to profile. If consumer syntax cannot express receiver/multiple returns/variadic shape exactly, record loss.

## 12. Variadics

Distinguish:

```text
variadic parameter of T
variadic tuple/list of types
unknown extra arguments
ordinary array/table parameter
```

Only exact source form lowers. Consumer limitation/format change is loss-recorded.

## 13. Structured tables and inline records

Prefer exact named table/structure declaration when available. Inline anonymous record emission requires:

- exact source identity/shape;
- stable semantic declaration/type ID;
- consumer support;
- deterministic generated name if renderer requires one, derived from exact semantic identity—not path/order/randomness;
- source-map/loss record.

Do not emit arbitrary anonymous source bodies.

## 14. Enums

Possible semantic forms:

```text
named enum type + values
literal union
integer/string base with sidecar values
```

Profile selects based on source and consumer support. Value names/values exact. Never widen to number/string without loss record.

## 15. Widgets and script objects

Named receiver/class types link exact widget/script-object declarations. Method `self`/receiver is explicit. Unsupported inheritance/composition stays sidecar/loss; no full FrameXML hierarchy inferred.

## 16. Restriction and Secret projection

Known exact source facets can alter analysis type projection only through frozen rules:

### Always-secret value

```text
ordinary source type T
-> nominal WowSecret<T or category> analysis type
or nominal secret base + sidecar ordinary category
```

Exact rendered form/profile freezes through probes.

### Contextually/conditionally secret

Potential projection:

```text
T | WowSecretT
+ predicate/facet sidecar
```

Only if this does not falsely imply arbitrary operations are safe or that runtime object wrappers exist. Otherwise sidecar-only/NotEvaluated.

### Secret argument/member

Position-bound restriction projection. Do not transform all same-named types globally.

### Unknown facet/predicate/runtime gap

```text
ExactWithSidecar / Unsupported / NotEvaluated
```

Never ordinary exact/safe type by omission.

## 17. Nominal analysis types

A versioned restriction analysis module may define:

```text
WowSecretValue
WowSecretNumber
WowSecretBoolean
WowSecretString
other reviewed categories only when justified
```

Rules:

- clearly documented as static analysis projections;
- no runtime constructors/methods/fields unless actual runtime contract exists;
- no implicit conversion/declassification;
- consumer-specific behavior tested;
- changes enter type/dialect/artifact identity.

## 18. Unknown/unresolved source types

Different states:

```text
source explicitly unknown/untyped
normalizer unsupported type shape
named target unresolved
source partition partial/conflicted
consumer cannot express exact type
budget truncation
```

Each has a distinct lowering reason/status. No single `any` fallback.

## 19. Type aliases

Alias declaration retains exact alias target semantics and reference links. Alias cycles validated; legal recursive named types treated explicitly, accidental cycles rejected.

Renderer cannot inline aliases automatically if it changes identity/source maps/parity without a profile rule.

## 20. Invalid or reserved names

Name rendering is separate from type semantics. If a named type cannot render as a direct identifier:

- use explicit supported safe alias/index form;
- or deterministic generated safe name with exact map/loss when profile accepts;
- otherwise Unsupported.

Never silently strip/replace characters or merge names.

## 21. Documentation-derived type prohibition

Documentation/prose cannot create/refine types, optionality, defaults, restrictions, replacements, or enums unless the source field itself is a reviewed structured contract.

## 22. Consumer profiles

For each consumer, freeze/probe:

```text
primitive/literal/named/array/map/tuple/union/function/variadic syntax
generics/aliases/classes/fields/methods/enums/tags
optional parameter vs nil semantics
multiple returns
unknown/any behavior
nominal Secret types
source span and diagnostic effects
limits/known parser differences
```

Shared artifact only if all mandatory semantics pass declared consumers. Otherwise build separate profiles/artifacts.

## 23. Lowering budgets

Bound:

```text
type graph nodes/depth
union variants
tuple/member counts
callback params/returns
generated aliases/anonymous types
restriction projection expansions
rendered type bytes
loss records
```

Budget exceed is explicit Unsupported/Lossy/NotEvaluated and affects artifact eligibility.

## 24. Canonicalization

- structural type identity independent of rendered syntax;
- named references by exact declaration/entity IDs;
- ordered tuples/function members preserved;
- unordered union/intersection variants canonicalized by structural key as profile defines;
- exact duplicate variants collapsed with all input links retained;
- recursion represented through named IDs, not infinite expansion;
- no temp path/file/line in type ID.

## 25. Required operations

```text
build_type_lowering_profile
validate_type_lowering_profile
lower_reference_type
lower_primitive_literal_named_types
lower_collection_tuple_union_types
lower_function_callback_variadic_types
lower_optional_nilability_default_semantics
lower_enum_widget_script_object_types
lower_restriction_secret_types
resolve_consumer_type_syntax_capability
record_type_projection_status_and_loss
canonicalize_annotation_type_graph
validate_type_graph_closure
```

## 26. Required tests

- each primitive/literal/named/collection/tuple/union/function/variadic form;
- optional vs nullable vs missing vs default;
- multiple returns vs tuple/table;
- unresolved/ambiguous/cross-profile named refs;
- exact/lossy enum strategies;
- Secret always/contextual/unknown/runtime-gap positions;
- explicit any vs unsupported/unresolved;
- invalid/reserved names;
- union size/depth/budget;
- consumer EmmyLua/LuaLS semantic differences;
- no docs-derived type;
- deterministic structural IDs across rendered profiles/layouts;
- no silent omission/widening.

## 27. Hard stops

- no silent `any`/unknown/omission;
- no optionality/nilability collapse without loss;
- no inferred array/enum/type from examples/prose;
- no cross-profile named target;
- no runtime Secret wrapper claim;
- no permanent spell whitelist;
- no renderer syntax as canonical type ID;
- no consumer lowest-common-denominator loss without explicit profile/record;
- no unbounded recursive/union/type expansion.
