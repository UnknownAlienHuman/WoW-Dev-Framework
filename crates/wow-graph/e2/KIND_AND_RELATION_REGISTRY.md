# Kind, relation, attribute, and axis registries

**Status:** normative.

## Entity kind definition

```text
EntityKindDefinition
    kind_id/version
    semantic purpose
    allowed universes/scopes
    identity field schema/order
    required/optional attribute IDs
    source/evidence requirements
    lifecycle/retention semantics
    query/display roles
    compatibility policy
```

A kind is not added to mirror an upstream table. It must have stable identity and query value.

## Initial E2 entity kinds

```text
repository
addon_package
toc_manifest
toc_variant
file
namespace
module
service
library
function
method
callback
event
api_symbol
xml_template
frame
region
mixin
factory
registry
state_root
state_path
source_span
```

Later kinds remain deferred until their owner/capabilities exist.

## Relation kind definition

```text
RelationKindDefinition
    relation_kind_id/version
    semantic purpose
    source/target kind sets
    direction and optional named inverse
    semantic qualifier schema
    required/optional attributes
    allowed universes/cross-universe policy
    evidence/confidence requirements
    multiplicity/uniqueness
    cycle policy
    axis memberships
    transitivity = none | query-derived-only
    compatibility policy
```

## Initial E2 relation kinds

```text
contains
declares
defines
exports
loads
loads_before
depends_on
optional_depends_on
inherits
mixes_in
instantiates
parent_of          object/XML semantics only
created_by
calls
possible_calls
registers_event
handles_event
triggers_callback
subscribes_callback
hooks
sets_script
references_template
uses_api
reads_state
writes_state
embeds_library
requires_library
owns
implements_role
```

`replaced_by`, `moved_to`, `same_lineage_as`, build-presence, impact, runtime, and external-candidate kinds are deferred to later profiles.

## Attribute definition

Allowed value families are bounded and canonical:

```text
boolean
signed/unsigned integer
finite decimal/number under profile
UTF-8 string with size/normalization policy
ID/reference
ordered list/set of allowed scalar/reference values
small tagged enum
small structured record declared by schema
```

No arbitrary JSON blob, source body, executable expression, SQL fragment, or model-generated schema.

Each attribute states:

- whether it participates in semantic identity;
- whether multiple producer values may coexist;
- merge/view policy;
- indexing policy;
- privacy/size rules;
- canonical ordering/serialization.

## Registry compatibility

- Additive kind/attribute/relation definitions require new immutable registry version.
- Changing identity fields, endpoint semantics, direction, cycle, or confidence policy is breaking.
- A graph snapshot pins exactly one registry bundle.
- Unknown registry definitions cause `NotEvaluated`/unsupported, never guessed behavior.
- Producer batches name the exact bundle they target.

## Validation

Reject:

- duplicate incompatible IDs;
- cyclic schema references without explicit bounded recursive form;
- relation endpoints with no possible kind pairing;
- generic parent relation outside object axis;
- transitive materialization declaration;
- candidate-to-proven confidence upgrade rule;
- identity depending on insertion order, row ID, host path, or display label;
- unbounded attributes or executable validators.
