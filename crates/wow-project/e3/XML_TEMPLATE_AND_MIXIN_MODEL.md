# Blizzard UI XML, template, object, mixin, and script model

**Status:** normative E3-A specialization; E2-C XML parser remains the only XML parser.

## XML safety

- streaming/bounded parser;
- DTD/external entities/XInclude/catalog/network resolution disabled;
- no script/handler execution;
- includes and script files resolve only inside the exact materialized snapshot;
- cycle, depth, fanout, file-count, byte, attribute, and text limits;
- exact raw records/spans preserved for unknown or malformed recoverable structures.

## Structural entities

```text
xml document
xml include
virtual template
concrete frame/object
region
script block
external script reference
inline Lua virtual unit
mixin/prototype reference
factory/constructor reference
source span
```

Exact entity kinds and relation endpoints are validated by the frozen E2-A graph registry extension.

## Independent relations

Keep distinct:

```text
contains
includes
loads script
references template
inherits
mixes in
parent_of          object/XML parent only
instantiates
created_by
sets_script
hooks script
owns
```

No generic parent relation and no conflation of XML inheritance, runtime object parentage, lexical containment, file ownership, or load order.

## Template identity and resolution

Template identity is scoped to exact universe/profile/generation and source declaration. Same display name in another build/universe is distinct.

Resolution records:

- exact declaration candidates;
- source/load visibility relevant to the static profile;
- zero/one/multiple compatible targets;
- evidence and coverage;
- conflict/ambiguity;
- `NotEvaluated` when dynamic/generated behavior exceeds capability.

First-found or last-write resolution is forbidden.

## Mixins and prototypes

Lua analyzer facts and core recognizers may propose mixin/prototype entities and `mixes_in`/factory relations. E3-A preserves:

- exact call/declaration facts;
- static literal targets;
- dynamic target prefixes/candidates;
- source handles;
- `Derived` or `Possible` confidence;
- ambiguity groups.

A mixin-like name or method set is not enough to prove identity.

## Scripts

External and inline scripts become deterministic source units with exact XML ownership and load ordinal. Inline Lua source mapping binds XML byte span to virtual Lua bytes without executing or reparsing Lua outside `wow-emmy`.

Malformed script text can downgrade only script/analyzer/recognizer capabilities that depend on it. Independent object/template facts remain available.

## Frame/object nonclaims

Static XML/source structure does not prove:

- runtime instance existence;
- exact creation timing;
- managed/protected/forbidden classification;
- secure execution state;
- combat mutability;
- taint safety;
- readable runtime attributes;
- performance.

Those require Reference Pack and/or runtime evidence.

## Skeleton-input requirements

For every exported XML/template/object/mixin/script item, downstream input includes:

- exact entity/relation IDs;
- declaration/source spans;
- signature/attributes under policy;
- selected load/package roles;
- direct structural edges;
- evidence, coverage, conflict, confidence;
- bounded source-slice handles.

It does not include an unbounded expanded XML tree or source body by default.
