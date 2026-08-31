# E2-C incremental invalidation and reuse

**Status:** normative dependency-driven update contract.

## Principle

Incremental indexing starts from exact immutable base and target source/configuration manifests. It never trusts filesystem mtimes, watcher order, editor events, path presence, or a producer's claim that cached facts remain valid.

## Dependency graph

```text
ProjectPartitionDependencyGraph
    input nodes:
        source file/content
        root/universe/package/TOC variant selection
        profile/reference/analyzer/parser/adapter/recognizer/graph/budget profiles
    derived nodes:
        TOC document/variant/file/dependency/variable partitions
        XML document/include/template/object/script/embedded-Lua partitions
        load package/unit/reachability partitions
        Lua unit/analyzer fact partitions
        recognizer fact/output partitions
        graph proposal-validation partitions
        candidate coverage/conflict/manifests
    typed edges with exact reason and propagation policy
```

The graph is versioned, acyclic by derivation stage, and canonical.

## Update input

```text
ProjectIndexUpdateRequest
    expected base ProjectIndexCandidate/ProjectGeneration IDs
    target ProjectIndexRequest/source snapshot
    explicit source/config/profile changes or target final snapshot
    cancellation/budgets
```

The target final state is canonicalized before invalidation planning. Stale base rejects the update.

## Change classification

```text
file added/updated/removed/moved/role-changed
TOC content/variant/selection changed
XML content/include/script path changed
Lua physical or virtual unit bytes changed
package/dependency resolution changed
profile/reference/analyzer pin/config changed
TOC/XML/load/adapter/recognizer/graph profile changed
capability/budget policy changed
source universe/root/materialization policy changed
```

A move is remove+add unless the source snapshot supplies an exact stable logical file identity under project policy.

## Direct invalidation

### Lua file or virtual unit content

Invalidate:

- analyzer file/fact/generic-diagnostic partitions;
- project Lua adapter partitions;
- recognizer bundles/output partitions depending on those facts;
- graph proposal-validation/candidate manifests depending on those outputs;
- source handles/spans/digests for the target generation.

Static TOC/XML load order can be reused if exact path/entry/XML structure did not change.

### TOC document

Reparse document and invalidate:

- variant selection if applicability fields changed;
- file entries/order/dependencies/LOD/bootstrap/SavedVariables partitions;
- package load graph and dependent package reachability;
- affected XML/Lua unit closure;
- analyzer workspace add/remove/order metadata where output-affecting;
- TOC/load/state recognizer bundles/outputs;
- graph proposals/candidate manifests.

If variant selection changes, all active partitions from the old selected variant become stale.

### XML document

Invalidate:

- XML document facts;
- recursive include closure whose resolution/order depends on it;
- templates/objects/inheritance/scripts;
- external/inline Lua unit manifests and analyzer partitions for changed/removed units;
- XML/frame/script recognizer bundles/outputs;
- affected load units/graph proposals.

### Dependency/package metadata

Invalidate package graph, load phases/reachability, downstream selected package scopes, and related recognizer/graph input partitions. Dependency source facts remain separate and are not copied.

### SavedVariables declaration

Invalidate state-root facts and all state-path bundles/outputs for that root/package. Do not read or migrate runtime SavedVariables content.

### Analyzer pin/config/library profile

Invalidate all analyzer snapshots/facts/generic findings and every recognizer/output partition depending on Lua facts. TOC/XML normalized facts can be reused only if their own profiles/generation inputs remain exact.

### Recognizer pack/rule/adapter profile

Invalidate affected recognizer input/output producer partitions and graph proposals; do not reparse source or rerun unrelated analyzer partitions unless fact adapter dependencies changed.

### Graph registry/proposal profile

Revalidate/rebuild all affected graph proposals and candidate graph mappings. Source/parser/analyzer/recognizer matches can be reused only if exact output shapes remain compatible.

## Reuse proof

A partition is reusable only when:

- producer/schema/profile versions identical;
- all direct/transitive dependency IDs/digests identical;
- generation scoping permits semantic reuse into target through an explicit rebind/validation contract;
- source handles/spans remain valid for target content;
- capability/coverage/conflict/truncation state unchanged;
- canonical partition digest recomputes/validates;
- no removed dependency or stale producer output remains.

Path/name/mtime equality alone is never proof.

## Generation rebinding

Derived facts are immutable under the base generation. Target candidate can reuse computation results only through a validated semantic reuse record that constructs new target-generation records/IDs where required. It never relabels old records in place.

## Conservative widening

If dependency impact cannot be proven narrowly:

```text
unknown Lua semantic dependency -> invalidate containing analyzer workspace/package scope
unknown XML include/template impact -> invalidate containing XML/package closure
unknown TOC directive effect -> invalidate selected package load model
unknown recognizer output compatibility -> invalidate the rule/pack output partition
unknown graph registry compatibility -> revalidate all affected proposals
```

Widening is explicit in `ProjectInvalidationPlan` and metrics. Stale reuse is prohibited.

## Removal closure

Target validation proves that removed inputs have no current:

- file/source registry record;
- TOC/XML/load/Lua unit record;
- analyzer fact/finding/source handle;
- adapter fact;
- recognizer match/proposal/output partition;
- graph proposal mapping;
- candidate manifest/count/digest reference.

Old immutable base candidates can retain them under old identity.

## NoChange

If target canonical source/config/tool/profile inputs equal base:

```text
outcome = NoChange
no parser/analyzer/recognizer/graph-validation work
base candidate returned by identity
no new ProjectGenerationId
```

## Cancellation/failure

No target complete candidate is exposed. Completed intermediate target partitions may be discarded or retained in an internal exact cache only under a future cache contract; they are not published/current. Base candidate remains unchanged.

## Determinism

Equivalent final target state reached through different independent update orders yields identical invalidation final set, reused/rebuilt partition manifests, target ProjectGenerationId, and candidate bytes. Reason paths/order are canonical.

## Operations

```text
build_project_partition_dependency_graph
diff_project_index_inputs
classify_project_changes
plan_direct_invalidation
propagate_project_invalidation
validate_partition_reuse
build_target_partition_plan
validate_removed_output_closure
classify_project_no_change
build_project_invalidation_report
```

## Tests

- Lua-only edit minimal invalidation;
- TOC file-order/dependency/variant/SavedVariables changes;
- XML include/template/script/inline-Lua changes;
- file removal/add/move;
- profile/analyzer/adapter/recognizer/graph version changes;
- unknown dependency conservative widening;
- stale fact/output/source-handle mutation detection;
- no-op and same-final-state different update order;
- failed/cancelled target leaves base unchanged;
- deterministic 1/2/N invalidation and candidate results.
