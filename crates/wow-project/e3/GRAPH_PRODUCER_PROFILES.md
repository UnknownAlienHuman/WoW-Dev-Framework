# Blizzard UI graph producer profiles

**Status:** normative producer ownership and graph handoff.

## Principle

E3-A does not write graph storage and does not collapse all facts into one producer. It submits exact, independently replaceable proposal partitions to `wow-graph`.

## Producer classes

### `platform_source_inventory`

Owns:

- source project/package/file/source-span entities;
- package/file containment and exact inventory relationships;
- content/source-handle/provenance attributes allowed by the registry.

Does not own TOC load, XML, analyzer, recognizer, or runtime facts.

### `platform_toc_load`

Owns:

- selected TOC/variant/load-unit entities;
- required/optional dependencies;
- TOC file order;
- LoadOnDemand/bootstrap/static-phase facts;
- direct reachability/load reason edges.

### `platform_xml_structure`

Owns:

- XML document/template/object/region/script/source-span entities;
- include/script/template/inheritance/object-parent relations;
- XML-to-virtual-Lua ownership and source mapping.

### `platform_analyzer_structure`

Owns direct deterministic adapters from exact `wow-emmy` facts that are explicitly assigned to `wow-project`, such as source declarations and source-unit membership required by the project graph profile.

It cannot reinterpret analyzer diagnostics or invent unresolved symbols.

### `core_recognizer:<pack>:<rule>:<version>`

Each E2-B core rule owns its own versioned producer partition containing accepted entity/relation proposals and exact match/evidence/coverage/ambiguity closure.

## Proposal lifecycle

```text
normalized source/project/analyzer facts
-> producer-specific ProposedEntityAssertion / ProposedRelationAssertion
-> graph registry/scope/evidence validation
-> accepted assertion IDs or rejected proposal records
-> conflicts and capability impact
-> GraphPublicationPlan
-> E2-D publication
```

Rejected proposals remain visible in candidate/publication reports. Project cannot weaken graph schema or silently drop them to claim success.

## Confidence

```text
Proven
    exact inventory/TOC/XML/source declaration facts when source evidence and parser coverage permit

Derived
    deterministic structural conclusion over proven normalized facts

Possible
    dynamic/ambiguous structure allowed but not proven

Candidate
    not produced by E3-A core source indexing
```

Recognizer output remains limited by its own E2-B contract. Aggregation cannot upgrade confidence.

## Cross-universe relations

Allowed only by explicit registry/profile, for example:

```text
blizzard_ui_source function -> reference api_symbol via exact resolved API identity
user project -> blizzard_ui_source template/mixin/function via exact target resolution in a later combined view
```

E3-A does not link by display name. It does not create lineage across Blizzard UI source generations.

## Partition replacement

Partition key includes:

```text
producer ID/version
blizzard_ui_source universe/profile/source/project generation
capability partition
package/file/rule scope as defined
```

On update:

- validate complete target partition before publication;
- remove stale assertions owned by that exact prior partition;
- preserve other producers;
- recompute conflicts/views/coverage;
- never mutate another producer's assertion;
- downgrade coverage when a producer/rule is disabled or incomplete.

## Complete-graph claim

E3-A may report `Complete` only for explicitly scoped capabilities where:

- configured source inventory is complete;
- parser/analyzer/adapter/recognizer inputs are complete for the partition;
- graph accepted/rejected/conflict accounting is complete;
- no relevant truncation or unresolved conflict remains;
- E2-D publication/read validation is complete.

It never means every runtime call, dynamic registration, object instance, or hidden client relation is known.

## Direct graph profile

The E3-A graph registry/profile must freeze:

- active entity/relation/attribute/axis definitions;
- exact cross-universe relation policy;
- producer permissions;
- confidence/evidence requirements;
- partition schemas;
- validation catalogs and golden queries;
- compatibility with E2-A base registry.

Unknown or incompatible registry versions block publication.
