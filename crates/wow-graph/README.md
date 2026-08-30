# `wow-graph` implementation contract

**Status:** deferred to E2/E3; contract scaffold only.

## Mission

`wow-graph` owns the typed WoW domain graph model, evidence-bearing entity/relation storage, partition replacement, lineage primitives, and bounded graph queries. It provides one graph with multiple explicit views rather than separate inconsistent trees.

## Owned responsibilities

- open/versioned entity-kind and relation-kind registries;
- stable entity identity rules;
- evidence-bearing directed relations;
- independent parent axes;
- partition/generation replacement semantics;
- graph validation and conflict retention;
- bounded neighbor/traversal/path/subgraph queries;
- lineage edge storage/query primitives;
- impact traversal primitives;
- persistence adapters through `wow-store`;
- deterministic graph serialization/export for tests;
- query budgets, truncation, and cancellation.

## Explicit non-responsibilities

`wow-graph` does not:

- parse Lua, TOC, XML, APIDocumentation, or external repositories;
- run recognizers;
- decide whether an API exists;
- rank search hits;
- generate source skeletons;
- infer missing edges from name similarity or model output;
- collapse independent parent axes into one `parent` relation;
- call Codebase Memory;
- own project/reference generation publication.

## Initial entity kinds

The registry begins with the kinds defined in [`../../docs/GRAPH_SEARCH_AND_PLANNING.md`](../../docs/GRAPH_SEARCH_AND_PLANNING.md), including:

```text
repository, build, addon_package, toc_manifest, toc_variant, file,
namespace, module, service, library, function, method, callback, event,
api_symbol, enum, cvar, xml_template, frame, region, mixin, prototype,
factory, registry, style, element, plugin, feature, state_root, state_path,
extension_point, restriction_facet, runtime_finding, source_span
```

A new kind requires identity, source evidence, lifecycle, and query semantics. Do not mirror every upstream table as a graph kind.

## Initial relation kinds

```text
contains, declares, defines, exports, loads, loads_before,
depends_on, optional_depends_on, inherits, mixes_in, instantiates,
parent_of, created_by, calls, possible_calls, registers_event,
handles_event, triggers_callback, subscribes_callback, hooks,
sets_script, references_template, uses_api, reads_state, writes_state,
embeds_library, requires_library, owns, implements_role, replaced_by,
moved_to, same_lineage_as, present_in_build, removed_in_build,
runtime_touches
```

Relation definitions must state direction, allowed endpoint kinds, evidence requirements, transitivity, uniqueness, and whether reverse traversal has a named semantic.

## Independent parent axes

The graph exposes at least:

```text
lexical
owner/package/module
load/dependency/order
object/frame/XML
inheritance/mixin/prototype
registration/event/callback/style/plugin
lifecycle
state root/path
call
lineage
```

An entity can participate in several axes simultaneously. APIs and result schemas name the axis explicitly.

## Evidence contract

Every entity/relation stores or references:

```text
source handle or source artifact
producer ID/version
profile/reference/project/external generation
provenance
confidence = Proven | Derived | Possible | Candidate
coverage partition/status
optional competing evidence
attributes validated by the kind/relation schema
```

The graph never upgrades confidence. Producer adapters assign evidence according to their owning contracts.

## Required operations

| Operation | Required behavior |
|---|---|
| `register_entity_kind` | Validate a versioned kind definition before use. |
| `register_relation_kind` | Validate endpoint/direction/evidence rules and reject incompatible redefinition. |
| `intern_entity` | Create/reuse identity within one universe/profile/generation under explicit key rules. |
| `add_relation` | Validate endpoint kinds, evidence, generation, and uniqueness; retain conflicts. |
| `replace_partition` | Atomically replace all facts from one producer partition/generation. |
| `validate_graph_partition` | Detect dangling handles, invalid endpoints, cycles where forbidden, and evidence gaps. |
| `neighbors` | Return deterministic bounded one-hop results by relation/direction/filter. |
| `traverse_axis` | Traverse one named parent axis with depth/node/time budgets and explicit truncation. |
| `bounded_paths` | Return evidence-bearing paths under relation whitelist and path-count budget. |
| `project_subgraph` | Load/export only the requested bounded neighborhood. |
| `query_lineage` | Return build-scoped lineage edges without converting candidates into replacements. |
| `impact_reachability` | Compute direct/derived/possible affected nodes with reason paths. |
| `explain_relation` | Return the exact evidence/producer/coverage supporting one edge. |
| `remove_generation` | Remove only unretained facts without breaking referenced retained generations. |

## Identity rules

- Entity identity is universe/profile/generation aware where the entity can differ by build.
- Source path alone is never a semantic identity across builds.
- Same display name does not imply same entity.
- Dynamic/possible call targets remain distinct edges and confidence.
- Historical lineage links entities; it does not merge their records.
- External candidate entities cannot collide with platform/project entities merely by name.
- Stable keys must be reproducible from normalized facts, not insertion order.

## Query rules

1. Every query declares universe, profile/generation, relation set, direction, and budget.
2. Unbounded traversal is unavailable from the public API.
3. Truncation is explicit and downgrades result coverage.
4. Query output order is deterministic.
5. Cycles are represented safely and cannot cause infinite traversal.
6. Candidate edges are excluded from proven impact unless requested and labeled.
7. The graph returns source/evidence handles, not full source bodies.
8. SQL/internal IDs are never public stable identities.

## Milestone sequence

### E2

Implement project entities/relations required for TOC/XML/load/state/event/hook facts and partition replacement.

### E3

Add Blizzard package/UI entities, parent-axis projections, source-handle neighborhoods, and L0/L1 support inputs.

### E4

Add lineage and impact query primitives required by search/patch impact; ranking remains in `wow-search`.

## Required tests

- entity/relation registry validation;
- stable identity under randomized insertion;
- independent parent axes for one entity;
- valid/invalid endpoint kinds;
- conflict retention;
- atomic partition replacement;
- cross-generation/profile edge rejection;
- bounded traversal, cycle handling, cancellation, truncation;
- candidate versus proven impact separation;
- deterministic paths/subgraphs;
- lineage does not merge entities;
- malicious oversized query budgets rejected;
- persistence round-trip.

## Documentation sources

- [`../../docs/GRAPH_SEARCH_AND_PLANNING.md`](../../docs/GRAPH_SEARCH_AND_PLANNING.md)
- [`../../docs/PROVENANCE_AND_COVERAGE.md`](../../docs/PROVENANCE_AND_COVERAGE.md)
- [`../../docs/ARCHITECTURE.md`](../../docs/ARCHITECTURE.md)
- [`../../docs/TEST_STRATEGY.md`](../../docs/TEST_STRATEGY.md)
- [`../../docs/SECURITY_MODEL.md`](../../docs/SECURITY_MODEL.md)

## Definition of done

The graph milestone is complete when producers can atomically replace evidence-bearing partitions, callers can request explicit bounded views across independent axes, and no query can confuse name similarity, candidate evidence, SQL IDs, or cross-generation data with proven graph truth.
