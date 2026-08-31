# E2-B declarative recognizer pack schema

**Status:** normative non-executable rule format.

## Format

The canonical E2 pack interchange is bounded canonical JSON. YAML anchors, includes, templating, environment substitution, executable expressions, regex programs, embedded Lua, dynamic libraries, and remote references are not supported.

## Pack header

```text
schema_version
pack_id/version
trust_class
fact_schema_profile_id
graph_registry_bundle_id
applicable universe/profile/fact-schema ranges
rule IDs in canonical order
evaluation profile/corpus IDs
pack budgets
license/provenance/review metadata
canonical digest
```

Pack metadata may name a calibration corpus; rule conditions may not branch on repository/addon identity.

## Rule shape

```text
rule_id/version
title/purpose
universal role/relation contract
required capabilities
accepted input partition/scope kinds
clauses[]
captures[]
outputs[]
confidence policy
ambiguity policy
coverage/no-match policy
budgets
fixture/evaluation IDs
rollout state
```

## Clause primitives

### `fact`

Select a typed fact kind into an alias with exact field constraints.

```json
{"op":"fact","as":"call","kind":"LuaCallFact"}
```

### `join`

Typed equality between declared fields/references.

```json
{"op":"join","left":"call.callee_reference_fact_id","right":"callee.source_fact_id"}
```

### `field_eq` / `field_in`

Exact scalar/tag/symbol comparison against a bounded literal declared in the pack profile.

### `same_scope`

Require same file/function/package/XML document/declared partition.

### `exists`

Require a bounded subclause match.

### `not_exists`

Allowed only when the rule declares the closed search scope and Complete capability required to prove absence.

### `ordered_relation`

Use an existing source ordinal or exact source-span order; no matcher-generated control-flow inference.

### `control_flow_relation`

Reference an exact supplied relation such as `dominates`; no CFG reconstruction.

### `all_of` / `any_of`

Bounded acyclic composition. Nesting depth is profile-limited.

## Predicates

Allowed predicate families:

```text
exact equality/inequality
enum membership
ID/reference equality
bounded numeric comparison for source ordinal/arity only
literal presence/type
resolution status
confidence/coverage status
same declared scope
```

Disallowed:

```text
arbitrary regex or glob over raw text
edit distance/semantic similarity
source-code evaluation
arbitrary arithmetic/string scripts
reflection or field-name iteration
filesystem/repository queries
network/model calls
unbounded recursive traversal
```

A future reviewed regex predicate would require a new schema version, bounded engine, security corpus, and explicit need. E2-B has none.

## Capture schema

Each capture declares:

```text
name
type/domain
source alias/field
cardinality = one | optional | bounded_many
canonicalization rule
whether used in semantic key, attribute, evidence, or explanation
```

Undeclared capture or type coercion is invalid. Source snippets are not capture values.

## Output declaration

```text
output_id
kind = entity_assertion | relation_assertion
registered graph kind/relation
source/target key ingredient mappings
typed attribute mappings
semantic qualifiers
source/evidence/coverage mappings
confidence expression restricted to reviewed policy
ambiguity behavior
```

Outputs cannot:

- invent a graph kind or attribute;
- set final graph assertion/generation IDs;
- mark a recognizer derivation `Proven`;
- promote `Possible` inputs to `Derived` unless the exact uncertainty is irrelevant under the frozen rule proof;
- emit diagnostics or source edits.

## Rule compatibility

Breaking changes include:

- clause/operator semantics;
- capture identity/cardinality;
- semantic key ingredients;
- graph output kind/direction;
- confidence/ambiguity/coverage policy;
- required capability scope.

They require a new rule version and producer partition replacement. Prose or fixture additions alone do not change semantic version unless outputs can change.

## Trust classes

### `core`

Repository-owned universal rules. Active in E2 after all gates.

### `calibration`

Named corpus-derived patterns that emit universal roles only. Deferred E5, explicit enablement/profile, no repository-name conditions.

### `experimental`

Shadow-only investigation. Cannot publish default graph assertions or satisfy mandatory coverage.

## Validation

Reject packs with:

- duplicate/incompatible IDs;
- unknown fact/graph schema versions;
- cycles in clauses/captures/derivations;
- unbounded joins/outputs/nesting;
- unsafe operators or executable content;
- undeclared literals or repository conditions;
- invalid graph endpoints/attributes;
- negative clause without complete-coverage contract;
- missing positive/negative/partial/mutation fixture IDs;
- default rollout without evaluation gate.
