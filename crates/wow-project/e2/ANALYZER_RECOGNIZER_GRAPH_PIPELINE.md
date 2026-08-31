# E2-C analyzer, recognizer, and graph-proposal pipeline

**Status:** normative cross-crate orchestration without persistent publication.

## Stage order

```text
SourceSnapshotValidated
-> TocVariantsSelectedAndParsed
-> XmlClosureParsed
-> StaticLoadModelBuilt
-> LuaUnitsMaterialized
-> ProjectGenerationCandidateDerived
-> AnalyzerPlanBuilt
-> AnalyzerSnapshotValidated
-> RecognizerFactBundlesBuilt
-> RecognizerOutputPartitionsBuilt
-> GraphProposalsValidated
-> InvalidationAndCoverageFinalized
-> ProjectIndexCandidateValidated
```

Failure/cancellation does not skip ahead or produce a complete candidate.

## Project generation derivation

E2-C `ProjectGenerationId` adds to E0-D inputs:

```text
source snapshot/content manifest
root/universe/package/TOC variant selection
TOC/XML/load parser/profile versions
normalized TOC/XML/load manifests
physical and virtual Lua unit manifest
recognizer fact adapter/core pack/evaluation profile
E2-A graph registry/proposal profile
incremental invalidation schema/profile
capability/budget policy when output-affecting
project index contract/canonicalization versions
```

It excludes worker order, temp root, row ID, wall clock, cache state, logs, and physical future ProjectStore details.

## Analyzer plan

Project builds exact `ProjectAnalyzerPlan` from reachable/analyzed Lua units:

```text
Main workspace
    selected first-party physical Lua files
    external XML Script files reachable from selected closure
    source-mapped XML inline virtual Lua units

Library workspace
    exact annotation library/artifact from selected reference profile
```

Optional explicit nonloaded/test support scopes use separate workspace/role/capability and cannot masquerade as runtime-reachable first-party Main units.

## Analyzer update

- map add/update/remove units against exact base snapshot;
- use target ProjectGenerationId supplied by project;
- verify every byte digest/length/virtual-source map;
- no duplicate physical/virtual unit identity;
- no analyzer-discovered extra Main file;
- analyzer library remains separate;
- `wow-emmy` returns immutable snapshot/facts/findings for exact target generation.

## Analyzer validation

Validate:

```text
project/profile/reference generation
accepted pin/probe/config/library identities
Main/Library workspace manifests
physical and virtual file/unit IDs/digests/lengths
XML-inline source maps
required capability/coverage records
no facts/findings for removed units
all project source handles resolve against target snapshot
```

Mismatch aborts later stages.

## Recognizer bundle planning

Project assembles narrow exact bundles based on parsed/load/analyzer partitions and E2-B rule applicability.

Examples:

```text
TOC package/variant bundle
XML document/object/template/script bundle
CreateFrame/mixin Lua unit bundle
native signal bundle
custom producer/subscriber declared scope bundle
hook bundle
library bundle
SavedVariables root + Lua access bundle
```

Each bundle names all cross-partition dependencies. No implicit project-wide scan.

## Recognizer execution

- invoke exact core pack/rule versions;
- retain rule outcomes, ambiguity, adapter loss, budget/truncation;
- outputs are proposed graph assertions only;
- a partial/failed bundle does not create a complete producer partition;
- rule removal/version changes reflected in target partition plan;
- no project-specific rule mutation.

## Graph proposal validation

Project calls graph proposal-validation seams using:

```text
exact E2-A registry bundle
project/reference/universe scope
base graph candidate context if any
recognizer/project proposal partitions
source/evidence/coverage closure
```

Graph returns accepted mappings, rejections, conflicts, and coverage impacts. E2-C retains these in `ProjectGraphProposalValidation`; it does not create final graph assertion IDs beyond returned validated mappings or publish a GraphGeneration.

## Project-owned direct proposals

Project may propose graph facts directly only for semantics it owns exactly, such as:

```text
repository/package/TOC/file/XML document/load unit identities
direct selected TOC contains/loads/dependency/order facts
source ownership and load edges
```

Higher structural roles remain recognizer-owned. Direct proposals use separate producer IDs/partitions.

## Error isolation

- TOC failure can leave independently known source inventory but blocks selected package/load/analyzer closure.
- One XML script failure can retain independent XML template/object facts but blocks the script/analyzer/recognizer partitions.
- One Lua parse/fact capability failure can preserve other files and exact coverage, subject to candidate policy.
- Recognizer failure affects its producer partition only.
- Graph proposal rejection remains explicit and can block candidate capability/promotion without erasing source/analyzer facts.

## Candidate status

```text
CompleteCandidate
    all mandatory E2-C source/parser/load/analyzer/recognizer/proposal/candidate gates pass

PartialCandidate
    coherent useful candidate with exact permitted partial/NotEvaluated partitions

Failed
    mandatory context/invariant/security/generation/candidate closure failed

Cancelled
    no complete target candidate

NoChange
    target inputs equal base
```

Complete/Partial candidate still has `persistent publication = NotPublishedE2C`.

## Last-known-good

Base candidate may remain available under old identity after target failure. It cannot be mixed with target source/facts/proposals or relabeled current for target. Persistent current-pointer policy begins in E2-D.

## Operations

```text
derive_e2_project_generation
build_project_analyzer_plan
apply_and_validate_project_analyzer_update
plan_recognizer_fact_bundles
execute_project_recognizers
build_project_direct_graph_proposals
validate_all_project_graph_proposals
assemble_project_index_candidate
classify_project_index_candidate_status
```

## Tests

- full stage path and stage omission/reordering mutation;
- analyzer Main/Library/virtual-unit closure;
- source-map mismatch for XML inline Lua;
- one exact bundle per rule scope;
- custom producer/subscriber bounded scope;
- recognizer partial/rejected proposal isolation;
- direct project versus recognizer producer partitions;
- no final GraphGeneration/persistent pointer;
- last-known-good not mixed/relabelled;
- 1/2/N deterministic stage manifests/candidate.
