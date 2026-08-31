# Generation coherence and publication head

**Status:** normative.

## Coherence tuple

One headed publication fixes:

```text
ProfileId / full ProfileIdentity
ReferenceGenerationId
ProjectGenerationId
ProjectIndexCandidateId
AnalyzerSnapshotId + analyzer pin/config/unit/file manifests
Recognizer pack/rule/result/output-partition manifests
GraphRegistryBundleId
base and target GraphGenerationId
ProjectStoreProfileId / RegisteredBundleSetId
ProjectStoreGenerationId / ProjectStoreArtifactId
ProjectSnapshotId
GraphSnapshotId
PublicationCoherenceManifestId
ProjectPublicationHeadId
```

Every component must agree on project/universe/profile/reference/source digests.

## Validation order

1. Candidate closure and publication eligibility.
2. Base/head/store/graph identity.
3. Analyzer manifest and source-handle closure.
4. Recognizer output and proposal-validation closure.
5. Graph plan registry/base/target closure.
6. Store profile/bundle compatibility.
7. Publication bundle noncyclic identity and expected manifests.
8. Sealed store read validation.
9. ProjectSnapshot and GraphSnapshot coherence.
10. New head payload and CAS precondition.

## One head

A single head prevents these invalid states:

```text
new project + old graph
old project + new graph
new graph + wrong analyzer facts
new store + old manifest
new project + another reference profile
```

Store registry atomically stores one typed payload. Project validates all semantic fields before and after retrieval.

## Head resolution

```text
Exact(head ID)
Current(project ID)
```

`Current` is an operation selector only. A read immediately records the exact resolved head and leases its exact store generation.

No unscoped `latest`, no second resolution for graph, no fallback after mismatch.

## Head transition

```text
Absent -> H1
H1 -> H2
```

Requires expected prior head. A target derived from H1 cannot overwrite H2.

## Eligibility

Head status:

```text
complete
partial
```

`partial` is allowed only under explicit publication policy. It retains all `NotEvaluated`, conflict, failed-degradable, and truncation records. It is never rendered as clean/complete.

## Historical/LKG

Historical head is immutable. Last-known-good is a retention/status designation pointing to an existing head. It cannot acquire a new ProjectGeneration, GraphGeneration, profile, or target status.

## Cross-reference construction

Avoid ProjectSnapshot ↔ GraphSnapshot hash cycle by deriving a shared coherence manifest from stable component/logical manifests first. Snapshot IDs derive from the coherence ID plus their owned manifest. Head derives from both snapshot IDs and coherence ID.

## Head validation after read

Any caller opening a headed view verifies:

- head payload digest;
- exact store generation/artifact;
- project/graph snapshot manifests;
- shared coherence ID;
- component IDs;
- store lease/read handle;
- capability/conflict/coverage summaries.

A registry record alone is not sufficient.
