# Raw metadata preservation and normalized reference facts

**Status:** normative E1-B raw-observation, identity, projection, conflict, and deterministic lowering contract.

## 1. Two mandatory layers

```text
Raw observation layer
    exact parsed/evaluated source values, including unknown/duplicate/conflicting fields

Normalized reference layer
    typed projections for supported E1 query contracts
```

Annotations are a third later projection owned by `wow-annotations`.

A normalized field cannot exist without source/raw/correction evidence unless it is an explicit deterministic derived value with identified derivation inputs.

## 2. Raw observation identity

Raw identity includes:

```text
Profile/source snapshot candidate context
partition/file/registration IDs
entity candidate key/kind
field path/name
canonical raw value ID/digest
source handle/span/order
parser/evaluator versions
observation schema version
```

Raw observations are immutable. A correction or normalization update creates another projection/application record, not a rewritten raw row.

## 3. Canonical field paths

Field path is a structured sequence:

```text
FieldName(name)
ArrayIndex(index)
MapKey(canonical key value ID)
Variant/member discriminator where schema requires
```

No dot-string ambiguity. Human path rendering is noncanonical.

## 4. Missing, null, unknown, unsupported, default

Represent separately:

```text
Missing
    no source observation for expected field

ExplicitNull
    source explicitly supplies nil/null under the field/table semantics

UnknownField
    source field exists but normalizer has no accepted semantic contract

UnsupportedValue
    source exists but evaluator/normalizer cannot safely lower it

DefaultedProjection
    accepted schema defines a default derived from explicit absence; derivation is recorded

KnownValue
    supported exact projection
```

Do not turn unknown/unsupported into missing/default.

## 5. Raw value storage

Persist canonical value nodes/tables by stable IDs, content-addressed objects, or normalized rows according to measured size/query needs. The logical contract requires:

- exact semantic value tree round-trip;
- bounded traversal/detail reads;
- source observation and field-path links;
- duplicate source fields retained;
- no JSON-string-only opaque blob as the sole canonical representation;
- object references validated through `wow-store`;
- deterministic value digest independent of SQL row order.

## 6. Entity candidate assembly

Registrations/raw tables are first classified into candidate entity kinds/systems/owners using exact schema rules.

```text
EntityCandidate
    candidate ID
    kind/system/namespace/name/owner/signature discriminator
    registration/raw observation IDs
    source order and applicability
    classification status
```

Unknown/unrecognized entity shape becomes quarantine/unsupported record; do not coerce into a similar known kind.

## 7. Stable normalized entity identity

Entity identity contract per kind declares:

```text
required canonical name/system/namespace/owner
receiver/member/signature discriminator when needed
profile/reference generation binding
applicability/flavor discriminator when needed
identity schema version
```

Examples:

```text
api function: system + canonical function name + kind
widget method: widget/receiver entity + method name + kind
structure field: structure entity + field name/order discriminator
system event: system + event name
restriction facet: target entity/member/argument-return-field position + facet kind/version
```

No path alone and no fuzzy name identity.

## 8. Canonical names

- Preserve source canonical spelling.
- Define namespace/system/member separators structurally.
- Case sensitivity follows source/API contract; no locale folding.
- Optional lookup aliases require explicit source/correction fact; no generated fuzzy aliases.
- Display names/docs are nonidentity unless entity-kind contract says otherwise.
- Invalid/missing name prevents normalized entity and affects coverage.

## 9. Parameters, returns, payloads, fields

Ordered member records include:

```text
position/ordinal
source name
canonical type expression/value refs
nilability/optional/default semantics where source states them
documentation/raw metadata refs
restriction/predicate refs
source/raw observation/evidence IDs
```

Ordering is semantic and retained. Duplicate names at different positions remain distinct. Unknown member metadata remains raw and can block signature completeness without deleting known fields.

## 10. Type representation

E1 stores a versioned structured type/reference model sufficient for exact APIDocumentation facts, such as:

```text
primitive
named table/structure/enum/widget/script object
array/map/tuple/union/optional where source contract supports
callback/function signature if represented
unknown/raw type value reference
```

Do not lower directly into LuaCATS text as canonical form. `wow-annotations` renders later.

Unresolved named type remains an exact unresolved reference/conflict/coverage state, not silently `any`.

## 11. Documentation/prose

Documentation strings can be stored as raw metadata and bounded normalized text fields, with exact source evidence. Prose is not used to infer:

```text
replacement relations
restriction semantics
argument/return shape
runtime safety
entity identity
```

unless a specific source field has an accepted structured contract.

## 12. Known field registry

```text
ReferenceFieldRegistry
    registry ID/version
    entity kind/path patterns
    accepted raw value shapes
    normalizer operation/version
    produced fact fields/capabilities
    unknown/unsupported policy
    default derivations
```

Adding field support requires fixtures, coverage mapping, raw backward compatibility, and schema migration/operation review.

## 13. Unknown fields

For every unknown field:

- preserve raw name/path/value/source;
- classify parent entity/system/value kind;
- record affected capability families;
- include in quarantine/report/manifests;
- determine whether dependent normalized facts can remain Complete;
- never suppress due to leading underscore/date/provider/source popularity;
- allow future normalizer to project it without reacquiring source.

## 14. Unsupported values

Known field with unsupported value shape:

- preserve raw syntax/value/record as far as safe;
- do not emit invalid known projection;
- mark exact field/entity/partition capability Partial/Failed;
- retain parser/evaluator reason;
- later support requires new normalizer version and generation.

## 15. Duplicate observations

Exact duplicates:

- raw duplicates remain or are represented by a lossless occurrence list;
- normalized entity/fact may canonicalize once;
- evidence includes every occurrence/source order;
- dedup identity exact.

Conflicting duplicates:

- preserve each raw observation;
- create `ReferenceConflictRecord`;
- do not choose first/last unless exact source contract says one wins while retaining conflict/raw evidence;
- downgrade dependent coverage/authority.

## 16. Cross-file/system references

Resolve only through exact selected generation indexes:

```text
named table/type refs
enum refs
system/entity refs
predicate targets
deprecation target/replacement when explicit
```

Resolution result:

```text
Resolved exact
Unresolved
Ambiguous/conflict
NotApplicable
```

No external/current/other-profile fallback.

## 17. Restriction metadata

Known restriction fields lower to open facet facts with exact target scope and raw payload. Unknown restriction field/facet remains raw/quarantined and blocks dependent safety/Secret capability as needed.

Do not flatten:

```text
SecretArguments
SecretReturns
ConditionalSecret
predicates
forbidden/protected/private metadata
```

into prose or a single boolean.

## 18. Deprecations and transitions

Normalize only explicit source facts:

```text
deprecated status/message/build
alias only when exact field/source says alias
replacement/moved-to only when exact structured evidence exists
```

Name similarity/documentation wording does not create a replacement edge.

## 19. Conflict model

Conflict kinds include:

```text
duplicate-different-value
type-reference-ambiguous
raw-versus-correction-mismatch
source-partitions-disagree
explicit-transition-target-invalid
normalizer-invariant-violation
unknown-dependent-field
```

Conflict record lists competing raw/fact/correction/evidence IDs and affected capabilities. Resolution creates a new reviewed correction/normalizer/source generation, not silent mutation.

## 20. Derived facts

Only deterministic derivations with exact inputs/version can emit `Derived` facts, for example:

```text
fully-qualified structured name
signature digest from ordered parameters/returns
entity source occurrence count
explicit applicability intersection
```

Derived fact records list input IDs and cannot exceed their evidence/coverage authority.

## 21. Canonical manifests and counts

Build records:

```text
raw observation count/digest by partition/kind
unknown field count/digest
unsupported record count/digest
normalized entity/fact count/digest by kind/system
conflict count/digest
restriction/predicate/deprecation counts
source/evidence coverage counts
```

Counts are diagnostic/evaluation data, not correctness proof alone.

## 22. Storage mapping

Persistent schema must support:

- exact indexed entity/kind/name/system/profile queries;
- ordered members/parameters/returns/payload fields;
- raw field/value/source lookup;
- evidence/provenance/source handles;
- corrections/applications;
- conflicts;
- coverage/capability partitions;
- restriction/predicate/deprecation refs;
- deterministic build/read validation.

Store schema/operation bundle is defined separately. `wow-reference` encodes/decodes domain records; `wow-store` executes registered operations.

## 23. Budgets

Bound:

```text
raw value nodes/depth/bytes
observations per entity/file/build
unknown/unsupported/conflict records
normalized entities/members/type nodes
cross-reference resolution candidates
raw metadata read depth/count/bytes
manifest/report bytes
```

Truncation/limit changes exact coverage; never drop records silently.

## 24. Determinism

Equivalent source/evaluator/normalizer/field registry/correction/profile yields equivalent:

```text
raw value/observation IDs
entity/fact/member IDs
unknown/unsupported/conflict IDs
reference resolution outcomes
restriction/predicate/deprecation facts
manifests/counts/digests
```

Independent of SQL insertion/page order, worker order, temp path, prose rendering, provider path/time.

## 25. Required operations

```text
build_reference_field_registry
validate_reference_field_registry
build_raw_field_path
emit_raw_observation
emit_unknown_field_record
emit_unsupported_value_record
classify_entity_candidate
build_reference_entity_key
normalize_entity_and_facts
normalize_ordered_members
normalize_type_reference
normalize_restriction_facets
normalize_predicate_facts
normalize_deprecation_transition_facts
resolve_exact_reference_links
detect_and_record_reference_conflicts
canonicalize_exact_duplicates
build_raw_and_normalized_manifests
validate_raw_projection_closure
```

## 26. Required tests

- raw canonical value/field-path round-trip;
- missing/null/unknown/unsupported/default distinction;
- every supported entity/fact/member/type shape;
- unknown known-field/new-field/nested field preservation;
- exact duplicate vs conflict cases;
- source order/member order;
- unresolved/ambiguous/cross-profile references;
- open restriction facet and unknown restriction field;
- explicit vs inferred replacement rejection;
- raw survives correction/normalizer/schema evolution;
- budgets/truncation coverage impact;
- randomized source/worker/SQL insertion order determinism;
- no annotation text or raw SQL as canonical fact.

## 27. Hard stops

- no unknown/raw loss;
- no unknown-as-absent/default;
- no opaque JSON-only canonical raw store;
- no fuzzy identity/reference/replacement;
- no conflict collapse;
- no unresolved type to silent `any`;
- no prose-derived security/shape facts;
- no annotation format as canonical model;
- no cross-profile resolution;
- no silent truncation.
