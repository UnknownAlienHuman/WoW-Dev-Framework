# E2-C static package and file load model

**Status:** normative static reachability/order model; not runtime evidence.

## Purpose

Build one deterministic project load plan from selected TOC variants, declared package dependencies, TOC file order, XML includes/scripts, and explicit bootstrap/LOD metadata.

## Inputs

```text
selected ProjectPackage/TocVariant records
resolved required/optional dependency declarations
resolved TOC file entries
parsed XML include/script records
ProjectLuaUnit and XmlDocument manifests
load-model profile and budgets
exact project generation candidate context
```

## Package graph

```text
ProjectPackageLoadGraph
    package nodes
    required dependency edges
    optional dependency edges
    resolution status
    selected package roots
    cycles/conflicts
    canonical order groups
```

Rules:

- required unresolved dependency blocks a complete load plan for the dependent package;
- optional unresolved dependency remains explicit and may yield conditional/Possible edges;
- dependency source is not fetched automatically;
- cycles are retained/classified; no arbitrary break by alphabetical/source-discovery order;
- same package name in multiple universes requires explicit resolution policy;
- load/dependency relation is not ownership or API authority.

## Package phases

Static phases can include:

```text
dependency prerequisite
bootstrap
normal/full package
optional dependency conditional
unreachable/deferred
```

Phase assignment derives only from selected TOC/dependency/bootstrap facts. It does not predict the game client's actual trigger, success, timing, combat state, or frame readiness.

## File and unit ordering

```text
TOC entry order
-> for XML entries, document encounter order
   -> include expansion order
   -> external Script file order
   -> inline script unit order
-> explicit later TOC entries
```

Exact expansion semantics are frozen by the TOC/XML/load profiles. The model records reason edges and source ordinals for every ordering relation.

Do not use filesystem order, path lexicography, worker completion, source map order, or analyzer output order.

## Load units

```text
ProjectLoadUnit
    unit ID
    package/variant/phase
    source type = lua_file | xml_document | xml_inline_lua
    physical/virtual source identity
    semantic ordinal path
    direct predecessor/successor reason edges
    reachability/conditional state
    source/evidence/coverage
```

XML object/template facts belong to the XML document unit; executable Lua semantics remain in Lua units analyzed by `wow-emmy`.

## Reachability

```text
Reachable
    selected package/variant and all required path/phase edges resolve

ConditionallyReachable
    optional dependency, dynamic/unknown profile condition, or unresolved optional include affects reachability

Unreachable
    source file exists but no selected load root/edge reaches it

Unknown/NotEvaluated
    mandatory parser/dependency/include/load capability incomplete
```

Unreachable source can remain inventory/search evidence but does not enter default runtime-like project analysis scope. Test/support files may use a separate explicit analysis scope.

## Load order relations

Emit project facts/proposals for exact direct relations:

```text
loads
loads_before
depends_on
optional_depends_on
contains
```

Do not materialize every transitive order pair. Transitive reachability is a query/result with reason path.

## Bootstrap

A unit explicitly tagged Bootstrap:

- is marked small/early static bootstrap role;
- can register only what source later analysis proves, not what the marker implies;
- cannot assume full package XML/UI state;
- cannot be used to assert self-load timing/success;
- full attach-once/lifecycle behavior is analyzed separately from Lua facts/recognizers/rules.

The project model does not enforce runtime coding style; it preserves the phase/role for downstream checks.

## `ADDON_LOADED` and lifecycle

Static model records package/unit dependencies and source registrations. It does not infer:

```text
all frames/children exist at ADDON_LOADED
variables/player/login readiness beyond exact helper/source evidence
combat-safe configuration
callback registered only once
cleanup occurred
runtime load succeeded
```

Those require analyzer/rules/runtime evidence. This follows the KB lifecycle boundary.

## Flavor and selected variant

Only the selected TOC variant contributes active load roots. Historical/other-flavor variants cannot make a missing file/dependency appear reachable or fill coverage.

## Conflict and coverage

Conflicts include:

```text
required dependency cycle/unresolved target
same file/unit with incompatible phase/order
include cycle
missing file/script/include
ambiguous package resolution
unknown file tag/directive affecting load
TOC/XML order inconsistency
```

Coverage partitions separate package graph, variant, file entries, XML expansion, phases, and reachability. One missing optional dependency does not erase complete first-party file order, but affected conditional paths remain partial.

## Incremental behavior

- Lua content change does not change static load order unless path/manifest/XML reference changes;
- TOC entry/directive/dependency change invalidates selected package load partitions and dependent packages;
- XML include/script change invalidates affected document expansion/downstream unit order;
- selected flavor/profile change rebuilds variant/load model;
- missing dependency resolution change invalidates dependent reachability;
- unchanged load partitions can be reused only by exact dependency/digest proof.

## Operations

```text
build_package_dependency_graph
classify_package_load_phases
expand_selected_toc_load_units
expand_xml_load_units
build_direct_load_order_edges
classify_static_reachability
validate_load_model
explain_load_path
build_load_coverage_report
```

## Tests

- one package exact TOC order;
- required and optional dependency chains;
- missing dependencies and cycles;
- LOD/bootstrap phases;
- nested XML includes/script/inline units;
- include cycles/duplicates;
- reachable/conditional/unreachable/unknown files;
- no transitive-edge explosion;
- no frame/runtime/load-success inference;
- selected variant isolation;
- update invalidation/reuse;
- 1/2/N deterministic unit/edge/order manifests.
