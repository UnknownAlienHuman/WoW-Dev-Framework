# E2-C project index data model

**Status:** normative.

## Input request

```text
ProjectIndexRequest
    request_id
    ProjectConfiguration
    ProjectSourceSnapshot
    selected ProfileIdentity / ReferenceGenerationId
    TOC and XML dialect profile IDs
    accepted wow-emmy pin/probe/config identity
    recognizer core pack/fact adapter/evaluation profile IDs
    graph registry/proposal profile ID
    capability and budget policy IDs
    expected base ProjectIndexCandidate ID: optional
    cancellation
```

## Materialized source snapshot

```text
ProjectSourceSnapshot
    snapshot_id
    project/repository/revision provenance
    source content manifest
    root/universe declarations
    package/TOC candidate declarations
    ordered file records
    license/security/materialization report IDs
    canonical digest
```

```text
ProjectSourceFile
    ProjectFileId
    universe/role/root ID
    normalized relative path
    kind = toc | lua | xml | other-preserved
    content digest/length/object or supplied-byte handle
    executable = false
    source handle base
    canonical digest
```

No host path in canonical/public identity.

## Universe/root

```text
ProjectUniverseDeclaration
    universe_id
    role = first_party_project | declared_dependency_metadata | declared_dependency_source | analyzer_library
    project/package/revision identity
    allowed roots/file kinds
    first-party finding ownership policy
    cross-universe reference policy
```

```text
ProjectRootDeclaration
    root_id
    universe_id
    logical root
    source snapshot prefix
    path/case/symlink policy
    file/byte/depth budgets
```

## Package and TOC

```text
ProjectPackage
    package_id
    universe
    addon/package name
    selected TocVariantId
    alternate variant IDs retained separately
    dependency/load/variable/file manifest IDs
    source/provenance/coverage
```

```text
TocDocument
    toc_document_id
    package/universe/file/profile
    raw line records
    normalized directives
    ordered file entries
    dependency declarations
    SavedVariables declarations
    unknown/unsupported records
    diagnostics/coverage
    canonical digest
```

```text
TocVariant
    variant_id
    flavor/edition/interface applicability
    document ID
    selection evidence/policy
    selected = true|false
```

## TOC records

```text
TocDirectiveRecord
TocFileEntry
TocDependencyDeclaration
TocSavedVariableDeclaration
TocUnknownRecord
```

Each retains source ordinal/span/raw observation and normalized projection status.

## XML

```text
XmlDocument
    xml_document_id
    file/package/universe/generation
    root element
    normalized include/template/object/inheritance/script records
    embedded Lua unit IDs
    unknown element/attribute records
    diagnostics/coverage
    canonical digest
```

```text
XmlIncludeRecord
    source XML document/span
    normalized target path/file ID
    include kind
    ordinal
    resolution/cycle/coverage state
```

```text
XmlTemplateRecord
XmlObjectRecord
XmlInheritanceRecord
XmlScriptRecord
XmlUnknownRecord
```

## Virtual Lua unit

```text
ProjectLuaUnit
    lua_unit_id
    unit_kind = file | xml_script_file | xml_inline_script
    owning ProjectFileId or XmlDocumentId
    package/universe/role
    normalized logical path/virtual URI
    exact source span and source-map link for inline content
    canonical UTF-8 bytes digest/length
    semantic load ordinal/phase
    analyzer workspace/file identity
```

Inline units cannot masquerade as standalone physical files.

## Static load model

```text
ProjectLoadModel
    load_model_id
    selected packages/TOC variants
    package dependency graph
    required/optional resolution records
    load units and ordered edges
    LOD/bootstrap phase records
    XML include/script expansion edges
    reachable/unreachable/ambiguous units
    cycles/conflicts
    capability/coverage
    canonical digest
```

```text
ProjectLoadUnit
    unit_id
    package/variant
    source file/Lua unit/XML document
    phase = bootstrap | normal | dependency | optional-dependency
    ordinal/path/source refs
```

Static phase/order is not a runtime execution record.

## Parsed fact partitions

```text
ProjectFactPartition
    partition_id
    producer/profile/version
    exact project generation candidate inputs
    source dependencies
    TOC/XML/project normalized facts
    adapter loss/coverage/conflicts
    canonical digest
```

## Analyzer binding

```text
ProjectAnalyzerPlan
    target ProjectGenerationId
    exact Main/project and Library workspace manifests
    physical and virtual Lua units
    file add/update/remove set
    analyzer pin/config/profile
    expected snapshot/fact capabilities
    canonical digest
```

```text
ProjectAnalyzerBinding
    AnalyzerSnapshotId
    exact target generation/profile/reference/pin/config
    Main and Library manifests
    Lua unit/source-map closure
    fact set and generic finding IDs
    capability/coverage
```

## Recognizer execution

```text
ProjectRecognizerPlan
    plan_id
    target generation
    core pack/fact adapter/graph registry IDs
    ordered input partition bundles
    expected output producer partition keys
    budgets/cancellation
```

```text
ProjectRecognizerResult
    plan ID
    output partition IDs
    outcomes/matches/ambiguities/proposals
    coverage/truncation/errors
    canonical digest
```

## Graph proposal validation

```text
ProjectGraphProposalValidation
    validation_id
    exact graph registry/base candidate context
    recognizer/project proposal partitions
    accepted proposal mappings
    rejected proposals/conflicts/coverage effects
    no final graph generation
    canonical digest
```

## Invalidation

```text
ProjectPartitionDependencyGraph
    source/profile/tool/pack/registry input nodes
    derived TOC/XML/load/Lua/analyzer/adapter/recognizer/graph-input partitions
    typed dependency edges
    canonical digest
```

```text
ProjectInvalidationPlan
    base and target source/config generations
    changed inputs
    directly invalidated partitions
    transitively invalidated partitions with reason paths
    reusable partitions with proof
    conservative widened scopes
    expected removals/rebuilds
    canonical digest
```

## Candidate

```text
ProjectIndexCandidate
    candidate_id
    ProjectGenerationId candidate
    source snapshot/root/universe/package manifests
    selected TOC/XML/load model manifests
    Lua unit/analyzer binding manifests
    fact-adapter/recognizer output manifests
    graph proposal-validation manifest
    invalidation/reuse report
    capability/coverage/conflict/truncation/deferred records
    persistent publication state = NotPublishedE2C
    canonical digest
```

## Candidate view

Read-only operations expose exact candidate manifests/facts/results for tests and E2-D handoff. They do not create a current pointer, persistent store lease, or final GraphGeneration.

## Canonical order

- universes/roots/packages/TOC variants/files by stable IDs;
- TOC/XML records by document/source ordinal then stable ID;
- load units/edges by phase, package order, semantic ordinal, ID;
- Lua units by load order then ID;
- fact/recognizer/proposal partitions by producer/scope/ID;
- invalidation paths by target partition/reason/stable input ID;
- coverage/conflicts/errors by capability/partition/producer/ID.
