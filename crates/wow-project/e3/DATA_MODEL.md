# E3-A data model

**Status:** normative semantic model; concrete Rust/SQL names may differ, ownership and identity may not.

## Source profile

```text
BlizzardUiSourceProfile
    profile_id/version
    source class = vendor_ui_source_mirror
    provider/repository metadata
    exact revision/tree/archive provenance requirements
    client product/flavor/channel
    build/interface/reference compatibility assertions
    configured roots and package selection policy
    path/case/encoding/line-ending policy
    ignored/excluded/generated-file policy
    materializer trust/security profile
    license/redistribution policy
    parser/analyzer/recognizer/graph/store profiles
    budgets
    canonical digest
```

## Materialized snapshot

```text
BlizzardUiSourceSnapshot
    source_snapshot_id
    source profile ID
    provider/repository/revision provenance
    exact root descriptors
    complete ordered file inventory
    content object IDs/digests/lengths
    file kinds, modes, case/collision state
    symlink/submodule/LFS/external records
    license/provenance records
    materialization/security report
    inventory coverage
    canonical content-manifest digest
```

No floating current/latest token.

## Platform-source project identity

```text
PlatformSourceProject
    ProjectId
    project kind = BlizzardUiPlatformSource
    universe_id = blizzard_ui_source
    exact source profile/snapshot
    client/profile/reference compatibility set
```

## Package and variant

```text
BlizzardUiPackage
    package_id
    project/source generation
    normalized package root
    package metadata
    available TOC variants
    selected variant ID or NotEvaluated
    required/optional dependencies
    load-on-demand/bootstrap roles
    source/license coverage
```

## File and Lua units

E3-A reuses E2-C `ProjectFile`, `ProjectLuaUnit`, `TocManifest`, `XmlManifest`, and `ProjectLoadModel` types, adding the platform-source universe/profile binding.

```text
BlizzardUiUnitManifest
    exact physical Lua units
    XML external-script units
    XML inline virtual units
    analyzer Main/Library classification
    load units and direct edges
    source handles/spans
    coverage and conflicts
```

## Index candidate

```text
BlizzardUiIndexCandidate
    candidate_id
    exact source/profile/client/reference identities
    ProjectGenerationId
    source/package/TOC/XML/load/Lua manifests
    AnalyzerSnapshotId and fact/finding manifests
    core recognizer pack/result manifests
    project and recognizer graph proposal partitions
    graph validation/rejection/conflict reports
    structural fingerprint manifest
    skeleton-input manifest
    capability/coverage/truncation/NotEvaluated summaries
    publication state = NotPublishedE3A
    canonical digest
```

## Graph producer partitions

```text
BlizzardUiGraphProposalSet
    exact project/source/analyzer/registry generation
    producer partitions:
        platform_source_inventory
        platform_toc_load
        platform_xml_structure
        platform_analyzer_structure
        core_recognizer_rule partitions
    accepted/rejected proposal refs
    evidence/coverage/conflicts
    canonical digest
```

## Structural fingerprint

```text
StructuralFingerprintRecord
    source entity key
    exact generation
    fingerprint schema/profile
    signature digest
    normalized declaration-shape digest
    direct structural-neighborhood digest
    source-content/span digest
    package/load-role digest
    evidence and coverage
```

Fingerprints are comparison inputs only, not cross-generation identity or lineage claims.

## Publication bundle

```text
BlizzardUiPublicationBundle
    validated candidate ID/digest
    exact expected current platform-source publication: optional
    target ProjectGenerationId / ProjectSnapshotId
    exact AnalyzerSnapshotId
    GraphPublicationPlan / GraphGeneration / GraphSnapshot
    project and graph logical partition manifests
    E2-D schema/operation/validation catalog IDs
    source/license/object-reference manifests
    expected counts/digests/golden reads
    budgets/cancellation
```

## Published view

```text
BlizzardUiProjectView
    exact CurrentPublicationRecord
    ProjectStoreReadSnapshot
    source profile/snapshot
    ProjectSnapshot/View
    GraphSnapshot/View
    AnalyzerSnapshot identity
    package/load/source/coverage view
    capability and validation records
```

## Skeleton-input view

```text
SkeletonInputView
    exact platform-source project/graph/analyzer generation set
    bounded root/package/file/entity selection
    declarations/signatures/types/source spans
    package/load/lifecycle roles
    direct graph relations and bounded reason paths
    docs/comment source handles under policy
    evidence/provenance/coverage/conflicts
    exact source-slice handles and byte budgets
    deterministic ordering
    truncation/continuation
```

It contains structured source facts, not rendered Project Maps, prose summaries, token counts, search rankings, or agent instructions.

## Update plan

```text
BlizzardUiIncrementalUpdatePlan
    exact base/target source snapshots and profiles
    file/content/fact final-state diff
    affected TOC/XML/load/Lua/analyzer/recognizer/graph/fingerprint/skeleton-input partitions
    exact reuse proofs
    removals and stale-closure checks
    target candidate/publication expectations
    budgets/cancellation
    canonical digest
```

## Coverage axes

Keep separate:

- source materialization/inventory;
- package/TOC variant;
- XML/include/script;
- Lua analyzer facts/findings;
- recognizer rule/input scope;
- graph proposal/acceptance/publication;
- structural fingerprint;
- skeleton-input projection;
- store publication/read validation.
