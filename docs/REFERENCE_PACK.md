# Reference Pack contract

**Status: normative design; public schema pending E1**

A Reference Pack is the immutable platform input consumed by diagnostics, search, graph queries, and project indexing. It represents one exact World of Warcraft profile and never means “latest” without a build and digest.

## 1. Goals

A pack must provide:

- exact API systems, functions, methods, tables, events, enums, CVars, widgets, templates, mixins, packages, and source locations;
- raw Blizzard metadata, including unknown fields and restriction facets;
- Ketho-compatible annotation projections for EmmyLua and LuaLS parity tests;
- package/TOC/XML/UI relationships and compact source skeletons;
- historical presence, removal, move, and replacement facts where configured;
- evidence, coverage, corrections, checksums, licenses, and builder identity;
- deterministic output from equivalent logical inputs.

A normal user should not need Ketho, Numy, PHP, LuaRocks, VS Code, or a local Blizzard UI checkout to consume a released pack.

## 2. Inputs and authority

Canonical content comes from a pinned Blizzard UI snapshot. Acquisition may use a Gethe mirror, local official export, InterfaceExport, or another verified provider. Provider identity is provenance, not platform authority.

Expected input classes:

```text
Blizzard_APIDocumentation
Blizzard_APIDocumentationGenerated in declared TOC order
Blizzard_Deprecated and transition material
Interface/AddOns package TOCs
XML templates and frame declarations
Lua UI implementation
optional interface resource metadata
reviewed corrections
Ketho/Numy outputs for differential comparison
```

## 3. APIDocumentation ingestion

The builder uses Emmy's Lua CST plus a restricted declarative evaluator.

Allowed behavior is explicitly bounded:

- literals and table constructors;
- local bindings;
- field/index access to known constants;
- known documentation registration calls;
- bounded constant expressions.

Arbitrary calls, file IO, dynamic loading, metaprogram execution, unbounded loops, or unknown side effects are never executed.

The pipeline is:

```text
Lua CST
→ raw canonical Lua value tree
→ schema-aware lowering
→ API/event/table/widget/predicate facts
→ raw unknown-field preservation
```

An unsupported construct creates an ingestion diagnostic. If a required contract cannot be completed, the relevant partition loses negative authority.

## 4. FrameXML and package ingestion

```text
TOC order and variants
+ bounded streaming XML parse
+ Emmy Lua syntax/semantic facts
→ packages, files, templates, frames, regions, scripts, anchors,
   inheritance, mixins, factories, methods, source spans
```

The builder emits package-local graph shards so a query need not load the full UI graph.

Ketho and Numy outputs are compared against the same snapshot. Disagreement is retained and classified; neither oracle blindly overwrites structural source extraction.

## 5. Corrections

Corrections are reviewed data, never hidden code branches:

```rust
struct CuratedCorrection {
    target: EntityKey,
    field: FieldPath,
    expected_source_digest: Digest,
    replacement: CanonicalValue,
    evidence: Vec<EvidenceRef>,
    reviewed_by: String,
}
```

When the upstream digest changes, the correction expires and must be re-reviewed.

## 6. Profile isolation

A profile includes at least:

```text
profile ID
WoW flavor/edition
Interface number
client build
source revision and digest
builder version
schema versions
correction-set digest
capability/coverage report
creation time
```

A project selects one active profile from TOC/configuration. Current, historical, PTR, beta, or flavor profiles are physically and logically separate. PTR data is advisory unless the project explicitly selects it.

Diagnostics never merge API signatures or restriction facets from different profiles.

## 7. Pack layout

Planned logical layout:

```text
manifest.json
reference.sqlite
annotations/
source-map.sqlite
raw-apidoc.zst
ui-source-skeletons.zst
checksums.json
licenses/
```

The physical layout may evolve before E1, but these responsibilities remain separate:

- manifest and compatibility identity;
- queryable normalized facts;
- editor/analysis annotations;
- source-handle resolution;
- raw preserved metadata;
- compact source detail;
- integrity and licensing.

## 8. Annotation projection

The annotation tree remains Ketho-compatible in concept:

```text
Annotations/Core/Blizzard_APIDocumentationGenerated/
Annotations/Core/Data/Enum.lua
Annotations/Core/Data/Event.lua
Annotations/Core/Data/CVar.lua
Annotations/Core/Widget/
Annotations/Core/ScriptObject/
Annotations/Core/Type/
Annotations/Core/FrameXML/
Annotations/Core/WowDialect/
```

Parity is semantic: canonical symbols, signatures, and types for the same source snapshot. Byte identity is not required.

Annotations may project nominal Secret types for linting, but the raw facet store remains canonical.

## 9. Determinism and integrity

A release build must:

- canonicalize input ordering;
- normalize paths and line endings;
- sort public outputs deterministically;
- record every input digest and builder dependency;
- produce checksums for every pack artifact;
- rebuild byte-identically from equivalent logical inputs where compression metadata permits;
- report nondeterministic fields separately when unavoidable;
- reject a mismatched manifest or checksum at load time.

## 10. Capability report

The manifest exposes capability and coverage partitions, for example:

```text
apidoc.functions = Complete
apidoc.events = Complete
restriction.secret_facets = Partial
ui.package.Blizzard_ActionBar = Complete
lineage.120001_to_120100 = Partial
annotations.ketho_semantic_parity = Complete
```

Consumers use this report to decide whether a query can return authoritative negative results.

## 11. Validation gates

Before release, a pack must pass:

- all declared APIDocumentation inputs ingested or explicitly diagnosed;
- raw unknown fields preserved;
- Ketho semantic parity report generated;
- Numy differential report generated for configured FrameXML partitions;
- schema and database integrity checks;
- annotation compatibility probe;
- deterministic repeated build comparison;
- exact search and negative-authority fixtures;
- profile isolation tests;
- license/provenance manifest checks.

## 12. Retention and lineage

Normal installations retain current and configured target profiles plus compact lineage journals for selected older builds. Full historical source snapshots are not required for every query once lineage evidence and source handles are materialized.
