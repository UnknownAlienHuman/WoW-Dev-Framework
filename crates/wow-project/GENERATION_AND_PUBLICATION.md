# Project generation and publication

**Status:** normative E0-D coherence contract.

## 1. Why generation exists

A project result is meaningful only when source files, selected reference profile, analyzer configuration/pin, and analyzer outputs describe one coherent state. `ProjectGenerationId` is the identity of that state.

It is not:

- a monotonically increasing database integer;
- a timestamp;
- a Git commit alone;
- an analyzer snapshot ID;
- permission to publish;
- a synonym for “latest.”

## 2. Generation derivation inputs

```text
ProjectGenerationDerivationInput
    project configuration schema/version
    project ID and kind
    workspace/source-origin declarations
    selected ProfileIdentity
    ReferenceGenerationId
    accepted analyzer pin ID
    analyzer compatibility-probe contract/report ID
    analyzer configuration digest
    canonical final first-party file manifest
    capability policy identity
    budget policy identity when output-affecting
    project-generation schema version
```

The file manifest contains file IDs, paths, roles, content digests, and byte lengths. It does not contain absolute host paths.

## 3. Excluded inputs

The generation hash excludes:

```text
wall-clock creation/update time
temporary checkout/root
process/thread/worker ID
memory address
filesystem enumeration order
hash-map iteration
local Git credentials/remote tokens
rendered diagnostic text
analyzer session memory identity
non-semantic log/profiling data
```

If an excluded field affects results, the implementation or generation contract is wrong and must be corrected.

## 4. Domain-separated generation identity

Use the `wow-core` domain-separated hash/canonicalization contract.

Conceptual domain:

```text
wow-project:project-generation:e0-d:1
```

Canonical derivation input must be independently serializable/testable. Do not hash an opaque in-memory struct or debug representation.

## 5. Candidate versus published generation

### Candidate

A valid derivation produces `ProjectGenerationCandidate` and target `ProjectGenerationId`.

Candidate means:

- the intended final project input state is known;
- update preconditions and basic configuration/inventory validation passed;
- analyzer publication has not yet been proven.

### Published

A generation becomes published only when:

1. analyzer update batch was accepted for the same target generation;
2. analyzer index/snapshot publication succeeded;
3. returned analyzer snapshot matches profile/reference/project/pin/config/files;
4. source registry and project capability records validate;
5. mandatory publication capabilities satisfy policy;
6. canonical project snapshot digest validates;
7. atomic publish operation commits the immutable snapshot.

A derived ID without these checks is not a successful project generation.

## 6. Publication transaction

```text
Begin
    read current immutable snapshot (optional)
    validate update preconditions
    compute final project configuration and file manifest
    derive target project generation candidate
    prepare target source registry/file records
    prepare generation-bound wow-emmy update batch

Analyze
    apply analyzer batch
    refresh/index
    request immutable analyzer snapshot

Validate
    validate analyzer snapshot context and manifests
    validate project/analyzer source-handle compatibility
    assemble project coverage/deferred capability records
    assemble project snapshot candidate
    validate all references/capabilities/digests

Commit
    atomically publish ProjectSnapshot
    make it current for exact project identity
    retain/retire previous snapshot according to policy

Abort
    publish nothing for target generation
    discard candidate state
    preserve prior snapshot under original identity when safe
    return typed failure
```

No consumer sees intermediate phases.

## 7. Analyzer snapshot matching

Before project publication, assert equality of:

```text
ProjectGenerationId
ProfileId / complete ProfileIdentity
ReferenceGenerationId
accepted analyzer pin ID
analyzer configuration digest
Main workspace identity
Main file IDs/paths/content digests/byte lengths
source origin mapping
required capability/coverage context
```

Library workspace identity is validated against analyzer binding policy, not copied into the first-party project manifest.

Any mismatch aborts publication.

## 8. Publication capability policy

### Mandatory E0 publication conditions

```text
project configuration valid
project file inventory complete
project source registry complete
project generation derivation valid
analyzer session/snapshot structurally valid
source-coordinate capability valid
analyzer Main manifest exactly matches project manifest
```

### Degradable analyzer capabilities

Per-file semantic/diagnostic/local-flow capabilities may be Partial/Failed only when:

- snapshot itself remains coherent;
- exact affected files/capabilities/partitions are reported;
- no fabricated facts/findings exist;
- higher rules will receive `NotEvaluated` blockers;
- project policy explicitly permits degraded publication.

### Deferred capabilities

TOC/XML/load/graph capabilities remain typed unavailable/NotEvaluated and do not block E0 publication because policy declares them outside milestone scope.

## 9. Atomic current-pointer semantics

The project actor/view registry may maintain a current snapshot pointer for one project identity.

Rules:

- pointer changes only after full snapshot validation;
- pointer change is atomic;
- readers holding an old snapshot continue to see immutable old state;
- current pointer never references a candidate/failed snapshot;
- a request can require an expected current generation and fail on mismatch;
- no implicit “latest” lookup without project identity/explicit service policy.

## 10. Last-known-good

When target publication fails:

- prior snapshot may remain retrievable/current only according to explicit service state;
- it retains original project/analyzer/reference identity;
- status reports target failure and retained generation separately;
- it cannot satisfy a request requiring the failed target generation;
- it cannot be merged with target candidate files/facts;
- repeated blind retry is prohibited.

Conceptual status:

```text
current_published_generation: old-id
failed_target_generation: candidate-id
last_known_good: old-id
is_current_for_failed_target: false
```

## 11. Project generation changes

A new generation is required when any semantic input changes, including:

- first-party file add/update/remove;
- file role/path/origin change;
- selected profile/reference generation change;
- analyzer pin/probe/config change;
- project configuration/capability policy change affecting outputs;
- source-root/workspace identity change;
- generation schema/canonicalization version change.

No new generation is required for purely external presentation/logging changes that do not affect project state/results.

## 12. Same final state through different updates

Different valid update sequences resulting in identical canonical final derivation inputs must produce:

```text
same ProjectGenerationId
same canonical project file manifest
same analyzer final-state snapshot/facts/findings when analyzer determinism holds
same canonical ProjectSnapshot bytes/digest
```

Operation order is not generation identity unless the final semantics actually depend on it. Conflicting operations in one batch are rejected rather than order-resolved silently.

## 13. Cross-generation guards

Operations consuming multiple project/analyzer inputs call core `require_same_generation`/equivalent guards.

Reject:

- analyzer facts from old snapshot in new project candidate;
- source handle from removed/updated old content;
- generic finding with mismatched project generation;
- update request based on stale current generation;
- project snapshot using reference generation different from configuration;
- service/rules input mixing two project generations.

## 14. Canonical snapshot digest

Project snapshot digest covers:

```text
snapshot schema version
project generation
selected profile/reference generation
project configuration digest
source origin declaration
canonical project file manifest
analyzer binding identity
project-owned coverage/deferred capability records
publication status
```

It references analyzer snapshot/fact/finding identities/digests but does not duplicate their full canonical payloads unless the public contract explicitly requires embedding.

## 15. Publication operations

Required semantics:

```text
derive_project_generation_id
validate_generation_candidate
build_analyzer_update_batch
validate_analyzer_snapshot_for_project
assemble_project_snapshot
validate_project_snapshot
publish_project_snapshot
retain_last_known_good_snapshot
require_current_project_generation
open_project_view
```

## 16. Publication failures

Typed failures include:

```text
project_generation_derivation_invalid
expected_project_generation_mismatch
analyzer_snapshot_project_generation_mismatch
analyzer_snapshot_profile_or_reference_mismatch
analyzer_snapshot_pin_or_configuration_mismatch
analyzer_file_manifest_mismatch
project_mandatory_capability_unavailable
project_snapshot_invalid
project_snapshot_digest_mismatch
project_publication_aborted
```

Failure messages do not substitute for these codes/context fields.

## 17. Required publication scenarios

### Initial publish

No prior snapshot; closed configuration/files analyze and publish one snapshot.

### One-file update

Expected old generation/digest match; new target generation/analyzer snapshot/project snapshot publish.

### Stale update

Expected generation/digest mismatch; analyzer is not mutated or no project snapshot publishes.

### Analyzer update failure

Target candidate derived; analyzer fails; no target project snapshot; prior snapshot retained under old identity.

### Analyzer generation mismatch

Analyzer returns healthy but wrong generation; project publication rejects it.

### Degraded per-file facts

Snapshot publishes only if policy permits and exact failure/NotEvaluated coverage remains explicit.

### Deferred E2 capability request

Returns typed unavailable/NotEvaluated; does not affect E0 snapshot coherence or fabricate empty data.

## 18. Hard stops

- no timestamp/sequence counter as canonical generation identity;
- no publication before analyzer validation;
- no partial file manifest with old analyzer facts;
- no automatic relabel of last-known-good;
- no silent retry against new generation;
- no mixed profile/reference generation;
- no mutable published snapshot;
- no “latest” project generation selected without explicit project/service context.
