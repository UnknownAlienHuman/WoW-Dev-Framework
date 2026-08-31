# E3-B platform source data model

**Status:** normative semantic model. Provider transport and physical storage details remain outside domain identity unless explicitly pinned by profile.

## Acquisition output

```text
MaterializedPlatformSourceSnapshot
    snapshot ID
    provider identity/trust class
    repository/archive/client-extract provenance
    immutable revision/tree/content manifest
    requested and observed flavor/build/profile values
    root/file/path/type/content manifests
    provider inventory observations
    license/redistribution manifest
    security/materialization report
    excluded/unsupported entries
    coverage/conflicts
    canonical digest
```

## Provider and build observations

```text
PlatformSourceProvider
    provider ID/type/trust class
    provider repository/location identity
    adapter implementation/profile
    declared source role/scope
    provenance evidence
```

```text
PlatformBuildObservation
    observation ID/kind
    provider/snapshot/source handle
    product/flavor/version/build/interface values
    confidence/provenance
    conflict links
```

Kinds include request expectation, repository commit message, `version.txt`, TOC Interface, generated docs, local client extraction, and runtime probe. No automatic winner.

## Source entry

```text
PlatformSourceEntry
    source entry ID
    normalized root-relative semantic path
    entry type
    content ObjectId/digest/length
    provider blob/tree/archive member refs
    source role
    license/provenance/security refs
    case/Unicode/collision state
    inclusion/exclusion status
    canonical digest
```

## Source roles

```text
implementation_lua
implementation_xml
implementation_toc
generated_api_documentation
api_documentation_framework
provider_inventory_hint
repository_metadata
repository_automation_ignored
test_or_tool_source_ignored
asset_or_noncode
unknown_supported
unsupported
excluded_by_profile
```

Roles do not create API/runtime authority.

## Corpus identity

```text
PlatformSourceCorpusProfile
    profile ID/version
    target product/flavor/build/interface expectations
    source roots and role rules
    package/TOC/XML/load policies
    analyzer workspace/sharding profile
    recognizer pack and graph registry
    publication/source-detail/security/license policies
    budgets/canonicalization
```

```text
PlatformSourceCorpusGeneration
    corpus generation ID
    materialized snapshot ID
    exact profile/tool/registry/pack IDs
    selected build observation/conflict set
    package/source/load/analyzer/recognizer/graph manifest IDs
    coverage/conflict/license state
    canonical digest
```

## Package inventory

```text
PlatformSourcePackage
    package ID
    corpus generation
    exact package root
    selected TOC variant IDs
    dependency/optional dependency/LOD/bootstrap metadata
    ordered source entry IDs
    XML include/script units
    package role
    coverage/conflicts
```

```text
PlatformSourcePackagePartition
    partition key
    package/source/profile/tool dependencies
    source/TOC/XML/load/analyzer/recognizer/graph output manifests
    reuse/invalidation proof
```

## Global load model

```text
PlatformSourceLoadModel
    model ID
    exact selected packages/TOCs
    package dependency and order relations
    file/source-unit order
    XML include/external/inline script expansion
    bootstrap/normal/LOD/conditional roles
    reachable/conditional/unreachable/unknown states
    direct-edge and reason-path manifests
    conflicts/coverage
```

## Analyzer model

```text
PlatformAnalyzerWorkspacePlan
    logical workspace ID
    all selected physical and virtual Lua units
    library/reference annotation profile
    deterministic update/shard plan
    cross-package/global-name assumptions
    expected analyzer snapshot and fact capabilities
    budgets/cancellation
```

```text
PlatformAnalyzerPartition
    partition ID
    logical workspace ID
    source unit/package/load closure
    analyzer implementation/config/profile
    fact/finding manifests
    cross-shard dependency/coverage/loss
```

## Recognizer and graph handoff

```text
PlatformRecognizerFactBundle
    exact corpus/package/analyzer generation
    typed project/TOC/XML/load/Lua facts
    source/evidence/coverage
```

```text
PlatformGraphProposalManifest
    project-direct and recognizer producer partitions
    exact graph registry/universe/corpus generation
    proposed entities/relations
    accepted/rejected/conflict/coverage expectations
```

## Candidate and publication

```text
PlatformSourceCorpusCandidate
    candidate ID/status
    corpus generation/profile/snapshot IDs
    package/source/load/analyzer/recognizer/graph-input manifests
    source handle/object/reference manifests
    coverage/conflicts/license/security/truncation
    incremental update report
    persistent state = NotPublishedE3B
    canonical digest
```

```text
PlatformSourcePublicationSet
    publication set ID
    candidate and graph generation/snapshot IDs
    platform project/snapshot/analyzer IDs
    complete logical partition membership
    source object/reference manifest
    schema/operation/validation catalogs
    capability/license/security state
    canonical digest
```

```text
PlatformSourceCorpusView
    exact ProjectStore epoch/generation/publication
    PlatformSourceCorpusGeneration
    platform ProjectView/GraphView
    package/load/source/analyzer/recognizer query catalogs
    exact source-detail resolver
    coverage/conflict/license/security manifests
```

```text
SourceUniverseManifest
    universe = pinned_platform_ui_source
    provider/snapshot/corpus/project/graph/store publication IDs
    build/flavor/profile observations
    query/source-detail catalog IDs
    provenance/coverage/conflict/license/security state
    canonical digest
```

## Incremental update

```text
PlatformSourceUpdatePlan
    exact base and target snapshots/profiles/tools
    added/removed/changed entries
    affected packages/TOCs/XML/load/analyzer/recognizer/graph partitions
    exact reusable partitions and proof
    widened unknown-impact scopes
    expected target manifests
```

No authoritative rename/lineage field in E3-B.

## Canonical ordering

```text
providers/observations by trust class/kind/source/ID
entries by normalized semantic path/type/content ID
packages by selected load/order key then package ID
TOC/XML/source units by source order
partitions by owner/kind/package/source/capability/ID
relations/proposals by registry semantic ordering
conflicts/coverage/license/security records by scope/kind/ID
```

No archive/Git/filesystem/worker/completion/clock order.
