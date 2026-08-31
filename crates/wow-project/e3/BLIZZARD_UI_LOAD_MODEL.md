# Blizzard UI static load model

**Status:** normative E3-A specialization of the E2-C load model.

## Inputs

```text
exact selected package/TOC variants
required and optional dependency facts
TOC source order
XML include and script expansion
physical and virtual Lua units
LoadOnDemand/bootstrap/conditional metadata
explicit source-profile package roots
```

No runtime logs, installed addon state, frame registry, SavedVariables contents, or client process observation.

## Node classes

```text
platform source project
package
selected TOC variant
TOC entry
XML document/include
XML external script
XML inline virtual script
physical Lua unit
load phase/gate
unresolved external dependency
```

## Direct edge classes

```text
package_requires
package_optionally_depends_on
package_contains_entry
entry_loads_after
entry_includes_xml
xml_includes_xml
xml_loads_script
xml_contains_inline_script
unit_in_phase
unit_conditionally_reachable
```

Transitive reachability is a bounded reason path, not a persisted all-pairs relation.

## Phases and gates

E3-A may model exact static roles such as:

```text
bootstrap candidate
normal package load
load-on-demand package
conditional/dependency-gated unit
unreachable in selected profile
unknown/not evaluated
```

The profile must define how metadata maps to these static roles. A label never proves runtime initialization or availability.

## Reachability

```text
Reachable
ConditionallyReachable
Unreachable
Unknown
NotEvaluated
Conflict
```

Each result includes direct reason edges and exact source evidence. Unknown/partial dependency resolution cannot become unreachable or reachable by convenience.

## Ordering

TOC order is semantic. XML include/script expansion order follows exact XML/source rules. Filesystem order, hash-map order, package name sorting, or analyzer completion order never replaces source order.

When multiple packages share a source unit, each load membership remains separate. The unit's semantic source identity is not duplicated merely to force one parent.

## Cycles

- include cycles are bounded parser conflicts;
- required dependency/load-order cycles follow the exact package/load profile and remain explicit conflicts;
- call/state cycles are unrelated to load cycles;
- no cycle is broken by dropping the last/first edge.

## Cross-package resolution

Only explicitly supplied packages/dependency manifests in the same materialized snapshot/profile participate. Missing dependencies remain unresolved records; E3-A does not download or infer them from names.

## Runtime nonclaims

The static load model does not prove:

- that the package was enabled;
- that all dependencies were installed;
- successful execution;
- event ordering in a client session;
- frame/object existence at a given event;
- addon initialization completion;
- Secret Value readability;
- combat/protected/taint safety;
- performance.

## Graph handoff

Project-owned proposals include exact package/file/unit/load entities and direct load/dependency/include/script relations. `wow-graph` validates endpoints, scope, multiplicity, cycle policy, evidence, coverage, and confidence before publication.

## Update behavior

Changes to TOC order, dependencies, selected variant, XML includes/scripts, phases, or referenced-file resolution invalidate the affected direct load partitions and every dependent reachability/skeleton-input partition. Unaffected source/analyzer partitions may be reused with exact proof.

Removed direct edges and nodes must not survive in the target graph or skeleton-input view.
