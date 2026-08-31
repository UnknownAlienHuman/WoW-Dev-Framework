# E3-B Blizzard UI source data model

**Status:** normative semantic model. Concrete Rust and SQL names may differ; identity, authority, ownership and state invariants may not.

## Source provider

```text
BlizzardUiSourceProvider
    provider_id/version
    provider class: repository | archive | installation-derived | other reviewed class
    provider locator metadata: provenance only
    revision identifier and verification evidence
    materializer profile ID/version
    provider trust/claim class
    canonical digest
```

Provider locator, owner and repository name never become source entity semantics.

## Build binding

```text
BlizzardUiSourceBuildBinding
    binding_id
    source provider/revision/content manifest
    claimed game family/flavor/build/interface
    exact reference profile/generation compatibility target
    evidence IDs by authority class
    state:
        ExactBuildMatched
        ProviderDeclared
        ContentCorrelated
        Unverified
        Mismatch
    conflicts and unresolved checks
    reviewer/policy decision ID
    canonical digest
```

Build-binding state is independent from ingestion coverage and license state.

## Source profile

```text
BlizzardUiSourceProfile
    source_profile_id/version
    exact provider and build-binding IDs
    source collection schema/canonicalization profile
    logical root definitions and roles
    package/TOC selection profile
    XML/load profile
    analyzer and annotation-library profile
    fact-adapter and recognizer pack/profile IDs
    graph registry/bridge profile IDs
    store physical/schema/operation/validation profile IDs
    source/license/redistribution/security/budget policies
    publication eligibility policy
    canonical digest
```

## Logical root

```text
BlizzardUiSourceRootDefinition
    root_id
    logical role:
        shared_ui
        frame_xml
        addon_packages
        generated_api_glue
        embedded_library
        test_or_tooling_excluded
        unknown_reviewed
    snapshot-relative normalized root
    package discovery/manifest policy
    allowed file kinds
    inclusion/exclusion rules from reviewed profile
    expected coverage class
    license policy ID
```

A path string does not imply a role without this definition.

## Materialized snapshot

```text
MaterializedBlizzardUiSourceSnapshot
    materialized_snapshot_id
    provider/revision/materializer/build-binding IDs
    immutable source collection ID
    ordered root manifests
    complete admitted file manifest
    explicit omitted/excluded/unsupported entries
    symlink/reparse/submodule/LFS/archive report
    source-content manifest digest
    license/redistribution manifest
    materialization security report
    coverage/conflict/truncation records
    canonical digest
```

## Root manifest

```text
BlizzardUiSourceRootManifest
    root definition ID
    normalized root identity
    admitted package/global-unit manifests
    admitted file IDs/digests/counts/bytes
    omitted/excluded/unsupported records
    license decision refs
    completeness status
    canonical digest
```

## Source file

```text
BlizzardUiSourceFile
    source_file_id
    source collection/generation scope
    logical root ID
    normalized snapshot-relative path
    file kind
    canonical logical bytes digest and length
    provider object/blob identity: optional provenance
    encoding/newline/binary classification
    license/redistribution decision refs
    source handle ID
    canonical digest
```

Absolute checkout path is excluded.

## Source collection/project generation

```text
BlizzardUiSourceGeneration
    source_generation_id
    source profile ID
    materialized snapshot ID
    exact root/file/content manifests
    parser/analyzer/adapter/recognizer/graph profile IDs
    build-binding state and reference compatibility
    license/redistribution policy set
    capability/coverage/conflict state
    canonical digest
```

This is a source-project generation under universe `blizzard_ui_source`, not a user addon ProjectGeneration.

## Package and global unit

```text
BlizzardUiSourcePackage
    package_id
    root/source generation
    package class: addon_toc | global_root | shared_unit | generated_unit
    selected TOC variant: optional
    ordered source/load members
    direct dependency/include relations
    source/license/coverage refs
```

Global roots are not forced to have a TOC.

## Analyzer snapshot

```text
BlizzardUiSourceAnalyzerSnapshotBinding
    source generation ID
    exact wow-emmy implementation/config/annotation IDs
    physical and virtual Lua unit manifest
    library/source universe separation
    AnalyzerSnapshotId
    fact/finding/source-map manifests
    capability/coverage/conflict state
    canonical digest
```

## Source fact bundle

```text
BlizzardUiSourceFactBundle
    source generation/analyzer snapshot
    partition key and producer/adapter versions
    typed TOC/XML/load/analyzer/source facts
    source handles/evidence IDs
    confidence/provenance/coverage
    unsupported/loss records
    canonical digest
```

## Source graph proposal partition

```text
BlizzardUiSourceGraphProposalPartition
    source generation ID
    universe = blizzard_ui_source
    producer ID/version/partition
    registry bundle ID
    proposed entity/relation assertions
    evidence/source/coverage/conflict closure
    validation request/report IDs
    canonical digest
```

## Reference/source bridge input

```text
ReferenceUiBridgeInput
    exact source generation/profile
    exact reference profile/generation/graph view
    bridge profile/registry IDs
    source/reference entity indexes
    source facts/graph assertions used for resolution
    capability/coverage/conflict state
    canonical digest
```

## Reference/source bridge assertion

```text
ReferenceUiBridgeProposal
    bridge proposal ID
    source EntityKey in blizzard_ui_source universe
    reference EntityKey in reference_api universe
    registered relation kind/direction
    exact resolution rule/version
    supporting source/reference assertions and evidence
    confidence/provenance/coverage/conflicts
    ambiguity group: optional
    canonical digest
```

String/name equality alone cannot create this record.

## Candidate

```text
BlizzardUiSourceIndexCandidate
    candidate_id
    source profile/generation/snapshot IDs
    packages/roots/files/TOC/XML/load manifests
    analyzer snapshot/fact manifests
    recognizer output partitions
    source graph proposal partitions
    reference/source bridge proposal partitions
    rejected proposal/conflict reports
    source/build/license/redistribution/coverage states
    invalidation/reuse manifest
    publication eligibility decision
    state:
        CompleteCandidate
        PartialCandidate
        Failed
        Cancelled
        NoChange
    persistent_publication_state = NotPublishedE3B
    canonical digest
```

## Source graph generation

```text
BlizzardUiSourceGraphGeneration
    GraphGenerationId / GraphSnapshotId
    universe = blizzard_ui_source
    exact source generation and registry bundle
    accepted source and bridge assertion partitions
    conflicts/coverage/capability summaries
    graph logical partition manifests
    canonical digest
```

Reference entities remain reference-universe endpoints; they are not copied into source-universe identity.

## Publication set

```text
BlizzardUiSourcePublicationSet
    publication_set_id
    source profile/generation/snapshot IDs
    source candidate ID/digest
    analyzer snapshot ID
    source graph generation/snapshot IDs
    reference generation/profile and bridge manifests
    source project and graph logical partition manifests
    source/license/redistribution/capability policy
    object/member reference manifest
    validation/golden-read plan
    canonical digest
```

## Store generation and current record

```text
BlizzardUiSourceStoreGeneration
    ProjectStoreGenerationId
    dedicated ProjectStoreId/EpochId
    publication set ID
    complete partition membership
    state per E2-D
```

```text
CurrentBlizzardUiSourcePublicationRecord
    selector:
        source profile ID
        game family/flavor/build/interface compatibility key
    exact store/source/graph/analyzer/reference/publication IDs
    previous current record/generation: optional
    activation validation report ID
    CAS base record digest
    canonical digest
```

No unqualified global “latest” current record.

## Redistribution decision

```text
SourceArtifactRedistributionDecision
    decision_id
    subject:
        raw_source_bytes
        source_excerpt
        normalized_fact
        source_map
        graph_artifact
        skeleton_or_context_artifact
        database_or_pack
    source/root/file/license IDs
    state:
        LocalAnalysisOnly
        RedistributableWithNotice
        RedistributableRestricted
        RedistributableUnrestricted
        Forbidden
        Unknown
    required notices/attribution/limits
    reviewer/evidence IDs
    canonical digest
```

## Publication validation report

```text
BlizzardUiSourcePublicationValidationReport
    exact target store/publication/source/graph/reference IDs
    build-binding/profile checks
    root/file/content/source-map closure
    analyzer/fact/recognizer checks
    graph and bridge validation/golden queries
    stale-removal/cross-universe/cross-generation leakage checks
    license/redistribution checks
    store/integrity/object checks
    budgets/cancellation
    status and canonical digest
```

## Identity order

```text
materialized snapshot + source profile
-> BlizzardUiSourceGeneration
-> AnalyzerSnapshot and source facts
-> source/bridge graph proposals
-> GraphGeneration / GraphSnapshot
-> BlizzardUiSourceIndexCandidate
-> BlizzardUiSourcePublicationSet
-> ProjectStoreGeneration
-> read-back validation
-> CurrentBlizzardUiSourcePublicationRecord
```

No semantic ID depends on current pointer, SQLite row/page/WAL state, checkout path, clock or worker order.
