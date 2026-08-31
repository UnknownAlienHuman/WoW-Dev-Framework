# Annotation semantic model

**Status:** normative E1-C reference-fact-to-consumer-neutral declaration model.

The semantic model is the contract between exact ReferenceView facts and all renderers/consumers. It removes file-layout and syntax concerns without discarding ownership, ordering, restrictions, provenance, coverage, or projection loss.

## 1. Input closure

A semantic build binds exactly:

```text
ReferenceView ID
ProfileId / ReferenceGenerationId
ReferenceDataManifest ID
Reference capability/coverage/conflict manifests
semantic/type/dialect/consumer profile IDs
build budgets/cancellation
```

Every input fact/declaration request is scoped to this exact context. Cross-generation links reject.

## 2. Source fact selection

Select only entity/fact kinds declared by `AnnotationSemanticProfile` and supported by ReferenceView capabilities.

For each requested kind/scope:

- query exact bounded facts;
- retain ReferenceLookupResult variant and decisive coverage/conflict records;
- create semantic elements only from `Found` exact facts;
- convert partial/conflict/NotEvaluated/unsupported conditions into projection status/loss/coverage records;
- never query another profile/kind/name/external source as fallback.

## 3. Module construction

Logical modules group declarations by exact semantic ownership, for example:

```text
Core globals/dialect
API system/namespace
Named types/tables
Events
Enums
CVars
Widgets/script objects
Restriction analysis types
```

Module grouping is versioned and layout-independent. A renderer may place several modules in one file or split one module, but semantic module ID remains stable under equivalent profile.

## 4. Declaration ownership

Ownership relationships are explicit:

```text
system/namespace owns callable
class/widget/script object owns field/method
structure owns field
module owns aliases/events/enums/globals
```

No string concatenation or path inference replaces exact owner/entity links. Missing/ambiguous owner produces Unsupported/Conflict status.

## 5. Callable declarations

```text
AnnotationCallableDeclaration
    function or method kind
    exact logical/qualified name
    receiver/owner
    ordered parameters
    ordered returns
    generic/type parameter data if supported
    deprecation/applicability/restriction/docs
    reference links/status/coverage/loss
```

Rules:

- preserve parameter/return order;
- distinguish optional, nullable, variadic, default, unknown, omitted;
- receiver method semantics explicit;
- no synthetic overload unless source/accepted projection profile explicitly defines it;
- unknown parameter/return metadata cannot disappear;
- body is inert fixed stub selected later.

## 6. Named tables/structures/classes

```text
AnnotationClassDeclaration
    class/alias/structure kind
    exact name/owner/namespace
    optional exact base/reference relation if persisted
    ordered fields
    docs/restrictions/reference links/status
```

A structure does not become a class with runtime constructor/method semantics unless exact consumer profile/source contract requires it. Representation difference gets classified.

## 7. Events

Event projection profile explicitly chooses among supported semantic forms, such as:

```text
named payload tuple alias
callback/function type
Event constant/name declaration plus payload alias
sidecar-only payload metadata
```

No assumption that an event is an ordinary function or that payload is readable at runtime. Restrictions/predicates/loss remain linked.

## 8. Enums and values

Semantic enum retains:

```text
exact enum name/namespace
value names
exact values/types/order when source contract defines
source/deprecation/restriction refs
```

Renderer chooses `---@enum`, alias literals, class fields, or sidecar according to consumer profile. Numeric/string value representation must be exact or loss-recorded.

## 9. CVars/globals

CVar semantics and global declarations are separate:

- CVar metadata can project as aliases/tables/docs only under accepted profile;
- Blizzard/runtime globals come from exact dialect/reference profile;
- no editor auto-global setting mutation;
- no global inferred from another addon/consumer environment.

## 10. Widgets and script objects

Semantic representation preserves exact receiver/class/method ownership, ordered signatures, source/restriction data, and any explicitly persisted inheritance/composition facts supported in E1.

Do not infer full UI inheritance/mixin graph from names or oracle layout.

## 11. Type graph

All declaration/member types reference structural `AnnotationType` nodes. Type graph:

- is acyclic unless explicit named/self reference through IDs;
- separates named reference from inline structure;
- preserves union/tuple/function/member order;
- records exact source type refs and lowering rule/status;
- never embeds rendered syntax strings as canonical identity;
- validates depth/node/budget constraints.

## 12. Documentation

Documentation fragments are separate semantic records:

```text
summary/detail/parameter/return/field/deprecation note kinds
source raw observation IDs
normalized text with no renderer syntax
sanitization/render policy status
```

Docs are optional for artifact capability only when policy says so; omission/truncation/sanitization remains visible.

## 13. Deprecation and availability

Only exact ReferenceView facts project. Semantic declaration can carry:

```text
deprecated flag/status
exact deprecation message/reference
explicit replacement/transition link when source states it
profile/build applicability
```

No inferred replacement or currentness.

## 14. Restrictions and Secret analysis

Semantic declaration/member/type holds exact projection requests from ReferenceView facets:

```text
always-secret nominal type candidate
contextual-secret union/sidecar candidate
secret argument/return/member position
predicate/conditional facet refs
protected/forbidden/private/runtime-gap sidecar refs
```

The semantic model does not decide runtime accessibility. It records analysis projection status and exact raw/source references.

## 15. Projection statuses

Every selected source fact/member/type has a status record. Examples:

```text
Exact
    all selected semantics represented consumer-neutrally

ExactWithSidecar
    core declaration exact; extra restriction/raw/provenance only in sidecar

LossyDeclared
    a declared approximation is emitted and precisely recorded

Unsupported
    no acceptable declaration/type form

NotEvaluated
    required ReferenceView capability/coverage unavailable
```

A parent declaration can be Found/Exact while one member/documentation/restriction field is partial/lossy.

## 16. Reference links

At minimum link:

```text
semantic declaration -> ReferenceEntity/Fact
member/type -> exact member/type/restriction facts
semantic docs -> raw observation/source
corrected field -> CorrectionApplication
all material elements -> Evidence/SourceHandle/coverage/conflict/lowering rule
```

Derived semantic elements list exact input IDs and producer version.

## 17. Duplicate/conflict handling

- exact same reference fact maps once with all source/evidence links;
- multiple semantic declarations for one unique exact identity require an explicit overload/variant profile or conflict;
- ReferenceView `Conflict` never becomes first/last declaration;
- consumer name collision after rendering is detected before files and recorded/rejected/loss-classified;
- sanitization cannot merge distinct identities silently.

## 18. Canonical semantic order

Semantic ordering is based on versioned keys, not ReferenceStore query/row order:

```text
module kind/system/namespace/module ID
declaration kind/owner/name/signature/declaration ID
member semantic ordinal/kind/name/member ID
type structural key/type ID
docs/restrictions/links/status/loss by subject/rule/ID
```

## 19. Semantic manifests

Record exact counts/digests by:

```text
module/declaration/member/type/documentation/restriction kind
projection status
reference source capability/coverage state
consumer profile applicability
unknown/unsupported/conflict/loss category
```

Counts are observability, not correctness proof alone.

## 20. Validation closure

Validate:

- every element exact context/profile/reference;
- every declaration belongs to one module;
- every member/type/doc/restriction link resolves;
- all selected ReferenceView inputs have projection status;
- all emitted semantic elements have reference/derivation closure;
- no invalid ownership/member order/name collision;
- no semantic type orphan/cycle violation;
- source conflict/partial state not hidden;
- budgets/truncation explicit;
- deferred capabilities not represented as empty success.

## 21. Required operations

```text
select_reference_facts_for_annotation
build_annotation_modules
build_callable_declarations
build_structure_table_class_declarations
build_event_enum_cvar_widget_declarations
build_annotation_members
build_annotation_type_graph
build_documentation_records
build_restriction_projection_requests
build_reference_projection_links
build_projection_status_records
canonicalize_semantic_model
validate_semantic_model_closure
build_semantic_manifest
```

## 22. Required tests

- every active module/declaration/member kind;
- exact owner/order/signature/type links;
- same name/different scope;
- overload/variant vs conflict;
- partial member under exact declaration;
- source conflict/NotEvaluated propagation;
- raw/correction/evidence/source-map input links;
- semantic IDs stable across layout profiles;
- invalid identifier not renamed at semantic layer;
- deterministic model under randomized ReferenceView row/query/worker order;
- all selected inputs receive one status.

## 23. Hard stops

- no rendered syntax as canonical semantic identity;
- no direct ReferenceStore row formatting;
- no first/last conflict selection;
- no source/consumer/oracle fallback;
- no runtime behavior inference;
- no declaration without reference/derivation closure;
- no missing status for selected input;
- no file path/span in semantic ID.
