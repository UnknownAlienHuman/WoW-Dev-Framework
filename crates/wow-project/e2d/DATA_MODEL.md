# E2-D integrated publication data model

**Status:** normative.

## Request

```text
ProjectPublicationRequest
    request_id
    project_id
    exact E2-C ProjectIndexCandidate ID/digest
    exact expected current ProjectPublicationHead ID: optional for first publish
    exact selected ProfileIdentity/ReferenceGeneration
    exact graph registry and expected base GraphGeneration
    exact ProjectStore physical/bundle/durability profiles
    publication capability policy
    budgets/cancellation
```

## Base

```text
ProjectPublicationBase
    current head: optional
    current ProjectStore generation/artifact: optional
    current ProjectSnapshot/GraphSnapshot: optional
    exact reader/validation handles if needed
    last-known-good record: optional
    base capability/conflict/coverage state
```

## Project logical plan

```text
ProjectLogicalWritePlan
    plan_id
    target ProjectGeneration
    candidate/source/TOC/XML/load/Lua/analyzer/recognizer manifests
    project-owned direct graph proposal partition manifests
    project snapshot expectation
    registered project operation invocations
    expected counts/digests/references
    canonical digest
```

## Graph request and plan

```text
GraphPartitionReplacementRequest
    exact registry/base/target context
    project and recognizer validated proposal partitions
    removal/replacement set
    expected producer partition coverage
    graph budgets/cancellation
```

```text
GraphReplacementPlan
    plan_id
    target GraphGeneration candidate
    accepted/rejected proposal mappings
    assertion/conflict/coverage manifests
    registered graph operation invocations
    golden query catalog
    expected snapshot manifest
    canonical digest
```

## Publication bundle

```text
ProjectPublicationBundle
    bundle_id/version
    request/base/candidate IDs
    Profile/Reference/Project/Analyzer identities
    recognizer pack/rule/result manifests
    graph registry/base/replacement plan
    project logical write plan
    ProjectStore profile/registered bundle set
    ordered merged registered invocation plan
    object plan
    expected logical domain/generation manifests
    mandatory validation catalogs
    capability/conflict/coverage policy
    budgets/cancellation
    canonical digest
```

The bundle contains no SQL, database path, connection, or runtime callback.

## Store result

```text
SealedProjectStoreResult
    ProjectStoreGenerationId
    ProjectStoreArtifactId
    generation/artifact/object manifests
    transaction/seal/open reports
    exact read handle/validation lease
    state = OpenValidatedInactive
```

## Snapshot manifests

```text
ProjectSnapshotManifest
    ProjectSnapshotId
    Profile/Reference/ProjectGeneration
    candidate/source/TOC/XML/load/Lua/analyzer/recognizer manifests
    ProjectStore generation/artifact
    project record/count/digest manifest
    graph snapshot reference
    capability/conflict/coverage summaries
    validation reports
    canonical digest
```

```text
GraphSnapshotManifest
    GraphSnapshotId / GraphGenerationId
    registry/base/partition/assertion/conflict/coverage manifests
    ProjectStore generation/artifact
    ProjectSnapshot reference
    golden query results
    validation reports
    canonical digest
```

Cross-reference identity construction is noncyclic: shared `PublicationCoherenceManifest` is derived first, then each snapshot references it and stable counterpart key without hashing a structure containing itself.

## Coherence manifest

```text
PublicationCoherenceManifest
    coherence_id
    publication bundle
    profile/reference/project/analyzer/recognizer IDs
    graph registry/base/target-plan IDs
    store generation/artifact
    project and graph logical manifest digests
    capability/conflict/coverage manifests
    canonical digest
```

## Head

```text
ProjectPublicationHead
    head_id/schema/version
    project/head key
    PublicationCoherenceManifest ID
    ProjectStoreGenerationId / ArtifactId
    ProjectSnapshotId / ProjectGenerationId
    GraphSnapshotId / GraphGenerationId
    AnalyzerSnapshotId
    Recognizer result/proposal-validation manifest IDs
    ProfileIdentity / ReferenceGenerationId
    previous head ID: optional
    publication report ID
    eligibility/status
    canonical digest
```

## Result

```text
ProjectPublicationResult
    Published(head + snapshots + reports)
    AlreadyPublished(exact same head)
    Rejected(target failure + unchanged current/LKG)
    InactiveValidated(sealed target + recovery state)
    Cancelled(phase + unchanged current)
```

## Published view lease

```text
PublishedProjectViewLease
    exact head
    store generation lease/read handle
    ProjectView
    GraphView
    coherent component identity summary
```

## Failure record

```text
ProjectPublicationFailure
    failed phase/operation/code
    request/candidate/base/target IDs
    store/graph/domain reports
    current head unchanged proof
    inactive generation: optional
    last-known-good original head: optional
    recovery class
```

## Ordering

Project plan records, graph plan records, validation records, capability records, and head fields use frozen canonical order. Wall time, host path, process, transaction sequence, and SQLite row ID are excluded.
