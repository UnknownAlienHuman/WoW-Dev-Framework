# Context invalidation, reuse, and publication boundary

**Status:** normative E3-A derived-artifact lifecycle.

## Ownership

`wow-context` owns:

- dependency manifests for context partitions;
- deterministic invalidation/reuse plans;
- artifact candidate construction and validation;
- artifact/member logical IDs and digests;
- context-specific coverage, omission, truncation, and redaction reports.

`wow-project` owns:

- project generation transitions and diff/invalidation input;
- coherent publication-set construction;
- deciding whether validated context partitions join a future project publication.

`wow-store` owns physical persistence/publication/retention/GC when context persistence is activated. `wow-context` never opens the store.

## E3-A output

```text
ContextArtifactCandidate
    target ContextInputSet/Profile
    L0/L1/ProjectMap/source partition manifests
    logical members and object references
    dependency and invalidation manifests
    coverage/conflict/truncation/redaction manifests
    validation report
    publication_state = CandidateNotPublishedE3A
```

E3-A does not create or advance a ProjectStore current record.

## Dependency graph

Each context partition declares exact dependencies on a bounded set of:

```text
project/graph/reference/UI snapshot and registry IDs
entity/relation/assertion/query result IDs and digests
source handles/spans/source-map records
project load/TOC/XML/analyzer/recognizer manifests
coverage/conflict records
ContextProfile, rendering, budget, redaction, tokenizer profiles
required other context partitions
```

The dependency graph is acyclic and canonical.

## Change inputs

```text
ContextChangeSet
    base/target ContextInputSet
    project partition additions/removals/changes
    graph assertion/entity/relation/conflict/coverage changes
    source handle/content/source-map changes
    profile/rendering/budget/redaction/tokenizer changes
    optional reference/UI graph generation changes
```

Mtime, watcher order, process cache state, host path, or unchanged display label is insufficient.

## Invalidation rules

### Entity/assertion change

Invalidate the entity's L0/L1 partitions and map sections that directly summarize it. Invalidate neighboring skeleton partitions only when their displayed direct counts/handles/paths depend on the changed assertion.

### Relation change

Invalidate source/target skeleton relation sections, affected axis/path summaries, map section/navigation entries, and bundles whose selected reason closure includes the relation.

### Coverage/conflict change

Invalidate every field/section/bundle that cites the changed record even if visible entity/relation values remain the same.

### Source bytes/span/source-map change

Invalidate affected excerpt partitions and any skeleton field whose exact displayed source coordinate/digest changes. Source-independent skeleton structure may be reused if dependency proof remains exact.

### Project/graph snapshot change

Do not invalidate everything solely because a generation ID changed if owner-provided diff manifests and partition digests prove exact unaffected dependencies. If proof is missing, widen to the enclosing artifact set.

### ContextProfile/rendering change

- semantic field/selection profile change invalidates affected artifact partitions and all dependent bundles;
- rendering-only change may reuse canonical machine artifacts but invalidates rendered members;
- budget-only change invalidates bundle selection/rendering, not necessarily skeleton machine artifacts;
- tokenizer change invalidates token-dependent selection/counts/rendering while byte-only artifacts may remain reusable under exact proof;
- redaction/license change invalidates source excerpts and any rendered member containing them.

### Optional reference/UI graph change

Invalidate only cross-universe fields/sections/paths and dependent bundles when exact dependency maps exist; otherwise widen according to the configured universe partition.

## Removal closure

For a deleted entity/relation/source/partition/profile field, target artifacts must contain no stale:

```text
skeleton or map member
navigation/index entry
relation/path/axis summary
source excerpt/source handle presentation
selection/dependency reference
coverage/conflict reference
rendered text fragment
continuation cursor target
logical object/reference
```

A test queries every reverse dependency class.

## Reuse proof

```text
ContextReuseProof
    exact base/target input/profile IDs
    partition key and prior member IDs
    identical dependency IDs/digests
    identical required source/evidence/coverage/conflict state
    compatible schema/canonicalization/rendering/tokenizer policy
    no removal/tombstone dependency
    validation result
```

No proof means rebuild or `NotEvaluated`; never silently retain stale content.

## No-change

A canonical no-change result occurs only when target input/profile and every logical artifact dependency/member digest are equivalent. It creates no new artifact-set semantic ID or expensive rebuild.

Operational regeneration of identical rendered bytes may be recorded separately without a new semantic set.

## Deterministic target

Independent update orders reaching the same exact final project/graph/source/profile state produce the same context partitions, artifact-set ID, selection order, rendered bytes, and target diff.

## Future persistent publication

A later integration package may add context logical partitions to the E2-D `ProjectPublicationSet`:

```text
validated ContextArtifactCandidate
-> project-owned publication bundle
-> registered context partition operations/validation
-> inactive ProjectStore generation
-> fresh read-back context/project/graph validation
-> CAS activation
```

The future integration cannot make `wow-context` depend directly on `wow-store`.

## Last-known-good

If target context construction or future persistence fails:

- prior context artifact set retains original input/profile/generation identity;
- failed target retains target identity/failure record;
- old context is not relabeled as the target;
- callers may request an explicit last-known-good view with mismatch disclosure;
- current project/graph publication is not rolled back merely because optional context caching failed unless higher policy makes context mandatory.

## Garbage collection handoff

Context supplies complete logical artifact/member/source-object reference manifests. The owning project/store retention policy decides physical retention and GC. No age-only or path-based deletion.
