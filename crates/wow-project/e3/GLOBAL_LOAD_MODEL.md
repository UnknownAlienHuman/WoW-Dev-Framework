# Platform source global static load model

**Status:** normative E3-B source-order and reachability contract.

## Purpose

Construct one deterministic static model across all selected platform UI packages. It answers which packages/files/virtual units are ordered or conditionally reachable under the selected source profile. It does not claim client execution.

## Inputs

```text
selected packages and TOC variants
required/optional dependencies
LoadOnDemand/bootstrap/conditional metadata
exact TOC entry order
XML includes/external scripts/inline units
profile conditions
closed source manifest
```

Provider inventory lists are validation observations, not load edges.

## Nodes and edges

Nodes:

```text
corpus_root
bootstrap_phase
package
selected_toc_variant
file
xml_document
xml_inline_lua_unit
conditional_or_lod_trigger_placeholder
```

Direct edges:

```text
requires_package
optional_package
loads_before
ordered_toc_entry
xml_includes
xml_external_script
xml_inline_script
bootstrap_member
conditional_or_lod_member
```

Transitive reachability is a reason path, never a persisted direct edge.

## Static statuses

```text
Reachable
ConditionallyReachable
Unreachable
Unknown
Conflict
NotEvaluated
```

Conditional placeholders record known source conditions without pretending a runtime trigger occurred.

## Ordering and cycles

TOC/XML source order is semantic. Filesystem/provider/archive order never substitutes. Ties without source order use stable presentation order and create no ordering assertion.

Classify required dependency, loads-before/order, XML include, and optional dependency cycles separately. Do not silently break cycles by lexical/name order.

## LOD/bootstrap nonclaims

Static roles do not prove package load, `ADDON_LOADED` readiness, frame/template existence, callback delivery, API payload readability, taint/combat/protected/Secret legality, or performance.

## Structural entrypoints

May emit exact candidates from bootstrap/first units, XML scripts, proven initialization calls, or registered handlers. Names such as `Init`/`OnLoad` alone are insufficient.

## Incremental invalidation

TOC/dependency/LOD/order/XML changes invalidate affected load closure and downstream analyzer/recognizer/graph partitions. A Lua body edit can preserve topology but invalidates its analyzer/downstream partitions. Unknown impact widens conservatively.

## Required tests

Global ordering, dependency presence/absence, LOD/conditional packages, no variant merge, XML order, cycle classes, present-unreachable files, inventory disagreement, presentation ties, no transitive-edge materialization, exact invalidation, and no runtime claims.
