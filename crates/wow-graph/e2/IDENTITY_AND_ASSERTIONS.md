# Semantic identity and assertion model

**Status:** normative.

## Entity identity

Identity is derived from the entity kind's frozen identity schema and exact scope. Examples:

```text
file
    universe + project generation + normalized project file ID

function/method
    universe + project generation + analyzer semantic symbol ID/source declaration identity

api_symbol
    reference universe + profile/reference generation + canonical API entity key

module/service
    project universe + project generation + universal role key emitted by recognizer
```

Display name, path, line number, or source text alone is insufficient unless the kind explicitly owns it as a canonical field.

## Assertion identity

Assertion ID is domain-separated from semantic key and includes:

- subject key;
- producer ID/version and partition key;
- generation context;
- exact typed attribute payload;
- evidence/source/coverage/derivation IDs.

Equivalent logical assertions from reordered input produce the same ID. Two producers may produce different assertion IDs for the same semantic key.

## Relation identity

A directed relation key includes source, target, kind, scope, and relation-defined semantic qualifiers. Evidence, confidence, timestamps, and producer do not belong to the relation key.

Repeated identical producer assertions deduplicate by assertion ID. Distinct evidence may remain separate assertions unless the producer contract explicitly bundles evidence canonically.

## Assertion acceptance

Validation checks:

- kind/relation registry compatibility;
- exact universe/profile/generation;
- endpoint existence or same-batch declaration;
- source/evidence/coverage closure;
- confidence allowed for producer/rule;
- typed attributes and budgets;
- no forbidden cross-universe relation;
- no cyclic derivation evidence;
- no source handle outside registered source origin.

## Derived view policy

For one semantic field/relation:

- identical compatible assertions retain all supporting assertion/evidence IDs;
- different nonexclusive values may coexist as a multi-value view when schema permits;
- incompatible exclusive values create a conflict and block/downgrade dependent capabilities;
- view confidence is no stronger than the strongest directly allowed conclusion under all blockers and never upgrades candidates;
- a missing assertion is not proof of absence without producer coverage/negative authority.

## Deletion

Assertions are immutable. Deletion occurs only by partition replacement/retention GC. A producer cannot directly delete another producer's assertion.

When the last assertion for an entity/relation disappears, the snapshot view disappears unless retained by another explicit partition. Referencing stale edges are rejected during publication validation.

## Cross-generation and lineage

Same semantic symbol across project/reference generations is a distinct scoped key. Later lineage relations link them; they are never merged into one entity record.

## External/candidate isolation

A candidate assertion belongs to an external/candidate universe and cannot share the project/reference key merely by canonical name. Any later link is an explicit candidate relation with candidate confidence.
