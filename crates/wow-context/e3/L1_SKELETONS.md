# L1 semantic skeletons

**Status:** normative E3-A semantic-detail projection.

## Purpose

L1 answers “what exact semantic facts are already known about this entity, and where should I expand next?” without loading an unrestricted implementation body or generating an architectural interpretation.

L1 is always rooted in an existing L0 skeleton.

## Field families

A ContextProfile may include bounded subsets of:

```text
exact signatures, parameters, returns, type/literal constraints
visibility/export/declaration attributes represented by source/analyzer facts
package/TOC/load-unit and static reachability facts
XML object/template/parent/inheritance/script facts
module/service/library/state-role assertions
native frame-event, EventRegistry, CVar, callback and hook structure
state-root and literal state-path reads/writes
proven/derived calls and explicitly requested possible calls
API-use relations to exact reference entities
object/factory/mixin/ownership/lifecycle relations
selected direct dependency and cross-universe bridge relations
exact restriction/capability fields from authoritative reference/rule inputs
bounded source excerpts under the excerpt profile
conflicts, coverage, truncation, redaction, and NotEvaluated state
query and expansion recipes
```

## Field record

Each semantic field is represented conceptually as:

```text
L1Field
    field ID/type
    canonical typed value or ordered values
    semantic owner EntityKey
    source fact/assertion/query/derivation IDs
    source handles/evidence IDs
    provenance/confidence
    capability/coverage/conflict state
    truncation/redaction state
    rendering label/order
```

A rendered sentence without this machine record is noncanonical.

## Signatures and types

- Use exact analyzer/reference facts and their source coordinates.
- Preserve unknown, unresolved, optional, nilable, variadic, tuple, and multi-return states.
- Do not synthesize a type from usage popularity.
- Do not collapse Secret/restricted analysis types into ordinary safe values.
- Do not claim runtime callability or payload accessibility from a static signature.

## Relation summaries

L1 may include exact direct relations and bounded reason paths. Every relation states:

```text
relation kind and direction
source/target exact keys and universes
supporting assertion IDs
confidence/provenance/evidence/coverage
conflicts
path derivation when not direct
```

A reason path does not become a direct edge. A path containing `Possible` remains possible.

## Event and callback structure

Keep separate:

```text
native Frame event registration
EventRegistry native frame-event bridge
custom EventRegistry producer/subscriber
CVar callback
other callback registries
```

L1 may say that source statically registers/subscribes/triggers under exact facts. It cannot claim that the client delivered the event, payload values were readable, or the handler was safe in combat.

## Hooks and protected behavior

L1 may expose exact hook structure such as target, hook family, handler and source handles. It cannot infer:

- taint safety;
- combat safety;
- protected/forbidden/managed-object legality;
- execution order beyond the exact contract;
- performance cost or absence of recursion;
- runtime receiver identity when static facts are ambiguous.

Those belong to exact reference/rule/runtime evidence.

## State

SavedVariables roots require exact TOC declarations. Literal state paths may be shown exactly. Dynamic keys retain exact prefixes/unknown segments and `Possible` confidence where appropriate.

Never read SavedVariables contents or render private user data.

## Source excerpts

An L1 source excerpt is optional and explicit. Default is source handles only.

Permitted excerpt purposes include:

- exact declaration/signature context;
- exact registration/hook/state operation site;
- a bounded reason-path anchor;
- a small source span specifically requested by the caller.

Forbidden default behavior:

- whole file/function/class/module body;
- all callers/callees source;
- arbitrary comments/documentation blocks;
- expanding around a token until a byte budget is exhausted;
- including dependency/reference source without exact profile/license permission.

Every excerpt uses [`SOURCE_EXCERPTS_AND_REDACTION.md`](SOURCE_EXCERPTS_AND_REDACTION.md).

## Ambiguity and conflicts

L1 retains all compatible competing assertions and explicit ambiguity groups. It does not select the first, last, most popular, or closest-named owner/target.

Exclusive disagreement renders as a conflict record and may block a semantic field or relation section. The skeleton can remain partially usable if unaffected fields are complete.

## Size and detail budgets

Per-skeleton limits include:

```text
fields
values per field
relations per family
reason paths and path depth
source excerpts/excerpt bytes
conflict/evidence/source refs
serialized bytes
optional exact tokenizer tokens
```

Required identity/uncertainty closure is retained before optional fields. If even required closure cannot fit, return a typed budget failure or explicit minimal/truncated artifact according to profile—never silently drop coverage/conflict state.

## Rendering

Default deterministic order:

```text
L0 reference
signature/type
load and ownership
object/inheritance/lifecycle
registration/events/callbacks/hooks
state
calls/API/dependencies
source excerpts
coverage/conflicts/NotEvaluated
expansion recipes
```

The profile may omit inapplicable sections but cannot hide a requested section's incomplete/conflicted state.

## Completion criteria

- every L1 field has exact traceability;
- no full implementation body is present by default;
- all relation systems remain typed and separate;
- cross-universe endpoints remain scoped;
- no static structure becomes runtime/safety proof;
- excerpt/redaction/license policies are enforced;
- shuffled input/assertion order produces identical machine and rendered output;
- profile downgrade/removal deletes the target field/section rather than leaving stale text.
