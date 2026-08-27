# Graph, search, skeletons, and planning

**Status: normative design**

## 1. One graph, multiple views

The framework stores one typed domain graph and exposes bounded projections. Owner, load, object, inheritance, call, event, state, lineage, and impact trees are views, not separate truth stores.

A WoW symbol may have several independent parent relationships. The data model never forces them into one ambiguous `parent` field.

## 2. Node registry

The registry is open and versioned. Initial kinds include:

```text
repository, build, addon_package, toc_manifest, toc_variant, file,
namespace, module, service, library, function, method, callback, event,
api_symbol, enum, cvar, xml_template, frame, region, mixin, prototype,
factory, registry, style, element, plugin, feature, state_root, state_path,
extension_point, restriction_facet, runtime_finding, source_span
```

Adding a kind requires defined identity, source evidence, lifecycle, and query semantics. A kind is not added merely to mirror an upstream table name.

## 3. Relation registry

Initial relation kinds include:

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

Every edge stores source or source artifact, producer ID/version, generations, confidence, coverage partition, and optional competing evidence.

## 4. Independent parent axes

Queries may expose:

- `lexical_chain` — enclosing function/table/file;
- `owner_chain` — package/module/namespace/mixin/service;
- `load_chain` — TOC variant, load unit, dependencies, order;
- `object_chain` — frame/XML parent-child and factory relation;
- `inheritance_chain` — XML inherits, mixins, prototype/metatable facts;
- `registration_chain` — event/callback/style/element/plugin registries;
- `lifecycle_chain` — initializer, enable handler, encounter/module factory;
- `state_chain` — state root/path readers and writers;
- `call_neighborhood` — proven and possible callers/callees.

A search result may include several chains because each answers a different engineering question.

## 5. Fact extraction

Canonical facts come from:

- Emmy Lua syntax and semantic facts;
- parsed TOC order, flavor variants, dependencies, and SavedVariables;
- structural XML templates, inheritance, frames, scripts, anchors, and children;
- APIDocumentation and Reference Pack facts;
- deterministic recognizers over normalized facts;
- explicit curated lineage/correction evidence;
- optional structured runtime observations.

External semantic or community evidence is joined only as candidate or implementation evidence.

## 6. Universal recognizers

Core recognizer families include:

```text
TOC package/load/dependency
XML template/frame/parent/inherits/scripts
CreateFrame factory/template/parent
CreateFromMixins and Mixin assignment
EventRegistry/CallbackRegistry/AceEvent registration
SetScript/HookScript/hooksecurefunc
LibStub and library embeds
module/addon factories
plugin/region/style/element registries
SavedVariables roots and literal state paths
slash commands and message buses
flavor/edition partitions
Secret guards and unsafe sinks
```

Named calibration packs may encode structural patterns observed in Ace3, oUF, ElvUI, WeakAuras, BigWigs, Details, or Plater. They emit only universal roles.

Recognizer changes require corpus fixtures and mutation tests that demonstrate precision beyond the named source repository.

## 7. Source skeletons

### L0 — identity and role

```text
signature
owner/load chains
direct API/event/state roles
caller/callee/registration counts
restriction and migration status
```

### L1 — control and effects

```text
signature
branches and loops
calls and callbacks
early returns
guard conditions
state reads/writes
implementation bodies collapsed
```

### L2 — exact source

Exact source span or full file detail resolved through a stable handle.

L1 is the default agent read. A query response should normally fit within approximately 2 KB and provide handles for deeper detail.

## 8. Search stages

```text
0. classify intent and requested profile/universe
1. exact active canonical name
2. aliases, deprecations, replacements, and build lineage
3. namespace/member/prefix
4. receiver/signature/return/restriction shape
5. normalized edit-distance/trigram candidates
6. FTS5 BM25 over docs, comments, role labels, and L0/L1 skeletons
7. graph-neighborhood expansion
8. optional Codebase Memory semantic candidates
9. deterministic reranking and evidence classification
```

Exact and historical evidence precede fuzzy and semantic discovery.

## 9. Ranking authority

Signals rank in approximate authority order:

1. explicit replacement/deprecation relation;
2. exact canonical or alias match in the active profile;
3. lineage evidence across builds;
4. entity kind and namespace;
5. receiver and parameter/return shape;
6. restriction-facet compatibility;
7. package and load affinity;
8. graph-neighborhood overlap;
9. documentation/skeleton FTS;
10. name or semantic similarity.

The final result includes `why` signals and identifies which lanes were used.

## 10. Search result contract

```rust
struct SearchHit {
    entity: EntityKey,
    canonical_name: String,
    kind: String,
    active_profile: ProfileId,
    owner_chain: Vec<EntityKey>,
    load_chain: Vec<EntityKey>,
    source: SourceHandle,
    migration: Option<MigrationStatus>,
    confidence: EvidenceLevel,
    coverage: CoverageSummary,
    why: Vec<SearchSignal>,
    detail_handle: StableHandle,
}
```

The public schema will additionally carry generation context and provenance references.

## 11. Removed or missing symbols

When an exact active symbol is absent:

1. confirm negative authority for the active partition;
2. inspect migration/deprecation journals;
3. inspect historical lineage;
4. compare receiver, signature, return, and restriction shape;
5. inspect current callers or examples in the pinned Blizzard source;
6. return an explicit replacement only when evidence supports it;
7. otherwise return ranked `Candidate` items and the missing evidence.

Text similarity never authorizes a replacement or autofix.

## 12. Build lineage

Path identity is insufficient because Blizzard UI packages and files move across releases. Lineage may combine:

- source-control rename/move evidence;
- normalized syntax fingerprints;
- signature and receiver shape;
- owner and load neighborhoods;
- call, API, event, and template neighborhoods;
- deprecation or transition documentation.

Each lineage edge states the evidence and confidence separately. A surviving semantic symbol may have a changed package, callback contract, restriction facet, or security context.

## 13. Patch impact

```text
new Reference Pack
→ diff APIs, events, facets, templates, packages, and lineage
∩ project uses, hooks, templates, state paths, and load graph
→ affected modules/files/functions
→ L0/L1 study set
→ deterministic checks and manual runtime scenarios
```

Impact output distinguishes:

- direct proven dependency;
- derived transitive dependency;
- possible dynamic reachability;
- external candidate evidence;
- not-evaluated areas.

The goal is a bounded study and test plan, not a giant list of every graph neighbor.

## 14. Planning

`wow_plan` should convert a task or impact set into:

```text
target entities and owner/load chains
required reference/profile facts
L0/L1 source handles
known restrictions and dynamic gaps
candidate extension points
files likely to change
checks required before completion
runtime scenarios that static analysis cannot settle
```

Plans are evidence-backed. They do not invent fix shapes when the source contract is unknown.

## 15. Storage and query bounds

Persistent entities, edges, aliases, lineage, restrictions, and source spans live in SQLite with covering indexes in both edge directions. FTS5 indexes canonical/alias names, documentation, comments near declarations, role labels, migration notes, and skeletons.

Only a requested neighborhood is loaded into `petgraph` or equivalent in-memory structures. Query budgets limit node count, depth, source bytes, and time. Truncation is explicit and affects coverage.
