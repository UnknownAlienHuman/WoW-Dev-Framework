# E4-B ReferenceView lineage and migration input handoff

**Status:** normative supporting seam. `wow-reference` remains public contract authority; `wow-graph` owns cross-generation lineage assertions.

## Purpose

Expose exact comparable ReferenceView records for one ordered profile/generation pair without embedding cross-generation fuzzy matching or migration policy inside `wow-reference`.

```text
exact from/to ReferenceView
-> exact reference endpoints and typed facts
-> explicit owner aliases/deprecations/replacements/transitions
-> stable reference identity records where contractually defined
-> exact negative-authority and coverage/conflict records
-> E4-B lineage/change/migration input bundle
```

## `ReferenceLineageInputView`

```text
ReferenceLineageInputView
    exact Store/Profile/ReferenceGeneration/ReferenceView IDs
    product/client flavor/channel/build/Interface
    entity/fact/correction/alias/transition manifests
    source/evidence/coverage/conflict/negative-authority manifests
    field/type/restriction schema versions
    owner read catalog
    privacy/license/provenance
    canonical digest
```

No floating current/latest profile.

## Exact endpoints

Export exact generation-local entities for supported kinds:

```text
API system/function/method
signature/overload/parameter/return
structure/table/member/type/enum/literal
native event/callback contract
widget/script-object/template/mixin contract when ReferenceView owns it
restriction/Secret/protected/forbidden facet
CVar/predicate/deprecation/correction record
```

Each endpoint retains raw observation, normalized fact, correction, source handle, evidence, provenance, coverage, and conflicts.

## Stable reference identity

A reference key can support continuity only when the ReferenceView contract explicitly defines it as stable across the pair/profile family. Canonical string equality alone proves name equality, not identity continuity.

Stable identity records include:

```text
stable identity namespace/version
exact from/to endpoint IDs
owner rule/source record
entity kind/scope
proof class ceiling
coverage/conflicts
```

Unknown or changed identity schema caps comparison to candidates/NotEvaluated.

## Explicit aliases and transitions

Only owner records can establish:

```text
explicit alias
renamed/moved identity transition
deprecated in profile/generation
removed/introduced under exact negative authority
replaced by target for a declared capability/scope
split/merged contract surface
```

Search/fuzzy/source usage or documentation wording never creates these records.

An exact correction can alter normalized facts under its own digest-bound applicability, but it does not silently create cross-generation lineage unless the correction schema explicitly contains a reviewed transition record.

## Restriction changes

`wow-reference` supplies the only public-contract input for static restriction facet changes. Export:

- exact old/new restriction facet IDs/values;
- predicate/access-condition records;
- source/generated-doc evidence;
- correction/conflict state;
- build/profile applicability;
- runtime gaps and required probes;
- negative-authority scope.

Implementation source alone cannot replace this contract. Runtime Secret/spell state remains build/context-specific and can stay NotEvaluated.

## Signature/type comparison

Preserve exact distinctions:

```text
missing vs explicit null vs unknown vs unsupported vs known
optional vs nullable
multiple returns vs tuple/table
variadic vs fixed
named vs structural types
union/enum/literal/member identity
default/deprecation/restriction facets
```

No silent `any`, value omission, or normalization across incompatible profiles.

## Negative authority

For absence/removal/introduction/no-replacement evaluation, expose exact owner decisions by:

```text
entity kind and namespace/scope
profile/generation
source/evaluator/parser/normalization/correction coverage
conflicts/truncation/runtime gaps
negative authority permitted/denied and reason
```

An empty ReferenceStore lookup is not enough.

## Migration evidence

For exact project-use migration, ReferenceView can provide:

- old/new entity facts and source evidence;
- accepted explicit transition/replacement/deprecation records;
- signature/type/restriction changes;
- exact applicability and profile constraints;
- correction/coverage/conflict records;
- required runtime probes or unresolved contract gaps.

It does not inspect project source, generate edits, or decide use-site applicability.

## Source implementation boundary

Reference-generated documentation and Blizzard UI implementation source remain separate evidence classes. A source function with the same name does not become the public API endpoint. Cross-links require exact owner/graph relations.

## Patch-sensitive routing

Current API/restriction/security interpretation remains tied to exact ReferenceProfile and current external `wow-addon-engineering-kb` routing. E4-B profiles must not hard-code a current patch's symbol list, spell whitelist, event set, or Secret behavior.

## Dependency rule

`wow-reference` does not depend on `wow-graph` E4-B or `wow-search`. It exposes a narrow immutable view/artifact. `wow-service` acquires it and submits exact records to `wow-graph`.

## Security/privacy/license

- no source acquisition/network in the view;
- no arbitrary Lua execution;
- no raw SQLite/SQL;
- bounded exact entity/fact pages;
- source documentation remains inert data;
- private materializer paths/credentials excluded;
- redistribution/source notice policy preserved.

## Validation

- exact profile/generation binding;
- supported endpoint/fact kinds only;
- stable-ID/transition records explicitly owner-defined;
- raw/normalized/correction evidence closes;
- restriction facts retain runtime gaps;
- negative-authority records satisfy exact scope;
- no fuzzy/search/source-usage upgrade;
- deterministic ordering and canonical digest.
