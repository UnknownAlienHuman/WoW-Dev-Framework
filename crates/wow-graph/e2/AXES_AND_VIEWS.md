# Independent graph axes and views

**Status:** normative.

## Axis model

An axis is a versioned query projection over declared relation kinds/directions. It is not a separate graph or duplicated truth table.

Initial axes:

```text
lexical
ownership
load
object
inheritance
registration
lifecycle
state
call
```

Lineage and impact activate later.

## Axis definitions

### Lexical

Enclosing declaration/file structure. Usually acyclic, multi-level.

### Ownership

Package/module/namespace/service/library ownership. Multi-parent may be valid when explicit; no generic parent assumption.

### Load

TOC variant, dependency, optional dependency, load unit, and ordering. `loads_before` cycles are conflicts/invalid for a selected variant; dependency cycles follow explicit policy.

### Object

XML/frame/region/factory parentage. `parent_of` belongs only here.

### Inheritance

XML inherits, mixins, prototypes/metatables. Multi-parent directed graph; cycles may be invalid depending on relation.

### Registration

Events, callbacks, registries, styles, elements, plugins, hooks, scripts. General directed; not a parent hierarchy.

### Lifecycle

Initialize/enable/disable/factory ownership and transitions. Explicit relation semantics; cycles may be valid only by definition.

### State

State roots/paths and readers/writers. General directed, often cyclic through functions.

### Call

Proven and possible calls remain separate relation kinds/confidence. Cycles/recursion valid.

## Query forms

```text
axis_roots
axis_parents
axis_children
axis_chain
axis_neighborhood
```

Every result names the axis and underlying relation assertions. No axis result invents a generic parent edge.

## Parent/hierarchy constraints

Axis definition declares:

- hierarchy model;
- allowed root semantics;
- one/many parent policy;
- cycle behavior;
- deterministic parent ordering;
- confidence/evidence policy;
- truncation behavior.

A single-parent query against a multi-parent axis is invalid unless it supplies a deterministic selection policy that remains a presentation choice, not graph truth.

## View closure

Views include:

- exact entity/relation keys;
- supporting assertions/evidence/coverage;
- conflicts;
- query derivation/path;
- truncation and continuation.

They exclude full source bodies and search ranking.
