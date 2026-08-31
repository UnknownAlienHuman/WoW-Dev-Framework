# Blizzard UI source graph — `wow-graph` owner seam

**Status:** normative E3-B supporting contract; extends E2-A through a versioned registry/profile, not a second graph implementation.

## Ownership

`wow-graph` owns:

- the `blizzard_ui_source` universe class;
- source-compatible entity/relation/attribute/axis registry definitions;
- semantic key and assertion validation;
- source, recognizer and reference/source bridge producer partitions;
- conflicts, coverage, GraphGeneration/GraphSnapshot and bounded exact queries;
- logical graph schema/operation/validation bundles used by E2-D.

It does not parse source, select a source profile, resolve provider/build/license policy, run recognizers or decide project publication/current selection.

## Universe profile

```text
BlizzardUiSourceGraphProfile
    profile_id/version
    universe class = blizzard_ui_source
    compatible source profile/generation schema
    graph registry bundle ID
    allowed entity/relation/attribute/axis definitions
    cross-universe relation policy
    source and bridge producer policies
    query/capability/budget profile
    canonical digest
```

One GraphGeneration binds one exact source generation and registry bundle.

## Identity

Source entity keys include exact source universe/generation and registry-defined semantic ingredients. Provider/repository/branch/checkout path, SQLite row IDs and display names are excluded.

Reference bridge targets retain exact `reference_api` keys. User project entities retain exact `first_party_project` keys. Graph never unifies them.

## Entity families

The E2 registry already covers many required kinds. Additive source-profile registry versions may activate:

```text
source_collection
source_root
package / toc_manifest / toc_variant / load_unit
file / virtual_source_unit / source_span
namespace / module / service / library
function / method / callback / event
xml_template / frame / region / mixin / factory / registry
state_root / state_path
```

Every new kind requires a stable identity schema, evidence requirements, allowed universe, attributes, query value and compatibility rules.

## Relation families

Source-internal:

```text
contains / declares / defines / exports / owns
loads / loads_before / depends_on / optional_depends_on
includes / references_template / inherits / parent_of
mixes_in / instantiates / created_by
calls / possible_calls
registers_event / handles_event
triggers_callback / subscribes_callback
hooks / sets_script
reads_state / writes_state
embeds_library / requires_library / implements_role
```

Cross-universe:

```text
blizzard_ui_source function/method --uses_api--> reference_api symbol
source registration/handler --registered exact relation--> reference_api event/entity
```

Project/source relations are registered but inactive until an exact project bridge package supplies project inputs.

## Axes

Source graph uses explicit axes:

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

No generic parent semantics. `parent_of` is object/XML only. Same entity may participate in multiple independent axes.

## Producer partitions

Graph validates independent producer classes:

```text
ui-source-inventory
ui-source-toc-xml-load
ui-source-analyzer-direct
ui-source-recognizer
ui-reference-bridge
project-ui-bridge [later]
```

Each partition names exact source/reference/project generations, producer/version, capability and coverage. Replacing a partition removes only that producer's stale assertions.

## Confidence ceilings

- direct source declaration/inventory facts may support `Proven` source assertions;
- deterministic adapter/recognizer relations are `Derived` or `Possible` according to inputs;
- cross-universe bridges are `Derived` at best;
- ambiguous/dynamic paths remain `Possible`;
- candidate/model/lineage hypotheses are excluded from E3-B production partitions.

Aggregation cannot upgrade confidence.

## Graph conflicts

Retain:

- source semantic key collisions;
- duplicate incompatible declarations;
- endpoint kind/profile/generation conflicts;
- forbidden object/load cycles and multiplicity violations;
- competing recognizer assertions;
- ambiguous reference/source bridge endpoints;
- source/reference build/profile incompatibility;
- evidence/coverage inconsistency.

No last-write, provider-majority or nearest-name resolution.

## Graph snapshot

```text
BlizzardUiSourceGraphSnapshot
    GraphGenerationId / GraphSnapshotId
    exact source profile/generation/snapshot
    exact registry bundle
    exact reference profile/generation for bridge partitions
    ordered source/recognizer/bridge partition manifests
    assertion/entity/relation/conflict/coverage manifests
    query capability summary
    canonical digest
```

The graph snapshot does not include the ProjectStore current pointer.

## Registered queries

E3-B requires bounded snapshot-bound variants of:

```text
entity_exact
neighbors
traverse_axis
bounded_paths
project_subgraph
explain_entity
explain_relation
partition/conflict/coverage lookup
```

Source-specific query profiles can filter exact root/package/file/entity kinds but cannot introduce fuzzy search or unbounded graph export.

## Negative authority

Graph may report an entity/relation absent only for an exact source universe/kind/relation/query scope when all relevant source/analyzer/recognizer/bridge partitions are complete and no conflict/truncation applies.

A source graph miss never becomes API absence, runtime absence or project absence.

## Persistence

Source graph supplies logical schema/operation/validation bundles and `GraphPublicationPlan` to E2-D. It does not own SQLite/WAL/current activation.

Read-back validation includes:

- exact source generation and registry binding;
- endpoint/reverse-index/assertion/evidence closure;
- partition replacement/stale removal;
- source/reference universe isolation;
- bridge target compatibility;
- conflict/coverage manifest reconciliation;
- deterministic golden queries.

## Tests

- same source symbol/name in source/reference/project universes remains distinct;
- provider/path/order mutations do not change semantic keys;
- source generation change creates scoped keys;
- direct/recognizer/bridge partitions replace independently;
- rejected proposal remains visible;
- object parent and inheritance stay separate;
- possible call/bridge cannot become proven/derived incorrectly;
- source graph miss cannot answer API absence;
- project bridge without exact project generation rejected;
- shuffled assertions and 1/2/N workers produce identical graph manifests;
- graph query results remain exact under E2-D old/new readers.
