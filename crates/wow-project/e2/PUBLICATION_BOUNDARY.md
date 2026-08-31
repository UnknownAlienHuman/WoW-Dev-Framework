# E2-C candidate boundary and E2-D persistent publication handoff

**Status:** normative separation of candidate construction from ProjectStore/GraphSnapshot publication.

## E2-C output

E2-C produces a validated immutable `ProjectIndexCandidate` with:

- exact ProjectGeneration candidate identity;
- source/root/universe/package/TOC/XML/load manifests;
- physical/virtual Lua unit and analyzer snapshot binding;
- project/TOC/XML/load facts;
- recognizer input/output producer partitions;
- graph proposal-validation mappings/rejections/conflicts;
- invalidation/reuse/removal closure;
- capability/coverage/truncation/deferred state;
- canonical digest.

It does not create a persistent store, current pointer, final GraphGeneration, or published E2 ProjectSnapshot.

## Why the boundary exists

E2-D still must choose and prove the physical ProjectStore generation model, WAL/read snapshot behavior, one-writer transaction, multi-manifest atomicity, crash recovery, retention, and GC. Parser/recognizer code must not silently predetermine these choices.

## Candidate immutability

After validation:

- candidate records/bytes are immutable;
- no parser/analyzer/recognizer result can be appended in place;
- correction requires a new target candidate/generation;
- candidate is identified by canonical content, not memory pointer or sequence number;
- candidate remains `NotPublishedE2C` even if complete.

## Candidate read view

```text
ProjectIndexCandidateView
    exact candidate/generation/profile identities
    source/package/TOC/XML/load manifests
    file/Lua unit/source-handle queries
    analyzer facts/findings by exact capability
    recognizer outcomes/proposals
    graph proposal-validation report
    invalidation/coverage/conflict/deferred records
```

The view is for tests, inspection, service/E2-D handoff. It cannot claim persistent/current status or open a database.

## E2-D handoff bundle

```text
ProjectPublicationBundle
    validated ProjectIndexCandidate ID/digest
    exact base published Project/Graph/Store generation IDs: optional
    target ProjectGenerationId
    graph registry and validated proposal partitions
    project logical partition manifests
    required ProjectStore schema/operation/validation profile IDs
    expected project/graph counts/digests/query vectors
    object/reference manifests
    publication capability policy
    budgets/cancellation
```

The bundle contains registered logical operations, not raw SQL.

## E2-D responsibilities

- select/freeze physical ProjectStore model after benchmarks/fixtures;
- activate `wow-store` direct dependency in project implementation;
- one writer and stale-base rejection;
- WAL/runtime profile where selected;
- atomic logical partition replacement and current generation publication;
- construct final GraphGeneration/GraphSnapshot through `wow-graph`;
- bind ProjectSnapshot to exact persisted ProjectStore/GraphSnapshot/analyzer/source identities;
- reopen consistent read snapshots;
- crash/cancel/failure isolation;
- last-known-good/current/failed target reporting;
- retention/lease/backup/rebuild/GC.

## Atomicity target

Readers must observe either:

```text
old published ProjectSnapshot + old GraphSnapshot + old ProjectStore generation
```

or:

```text
new coherent ProjectSnapshot + new GraphSnapshot + new ProjectStore generation
```

Never mixed source/analyzer/recognizer/graph/store generations.

## Failure before E2-D

An E2-C candidate failure:

- produces no publication bundle;
- leaves prior published state untouched;
- can retain exact failure/candidate IDs for status/debugging;
- does not alter current pointers.

## Failure during E2-D

The E2-C bundle remains immutable evidence. E2-D records exact store/graph/publication failure and does not mutate/relabel candidate. Prior published generation remains under its original identity.

## Last-known-good

A previous published state or E2-C candidate can be referenced as last-known-good only with exact original IDs. It cannot satisfy a request for the failed target or merge old facts with new inputs.

## Persistent versus canonical determinism

E2-C freezes logical candidate determinism. E2-D separately classifies:

- logical project/graph/store generation determinism;
- SQLite physical byte reproducibility;
- WAL/checkpoint/operational state;
- backup/archive bytes.

Physical nondeterminism cannot change logical IDs or query results.

## E2-C tests

- candidate complete/partial/fail/cancel/no-change;
- candidate immutable after validation;
- no current pointer/store/GraphGeneration fields;
- publication bundle contains exact registered logical manifests only;
- stale base not hidden;
- prior state not relabeled/mixed;
- candidate digest independent of future physical store model.

## E2-D entry gate

Do not begin E2-D implementation until:

- E2-A graph and E2-C candidate code/seams pass;
- E2-B recognizer producer partitions pass replacement tests;
- selected ProjectStore physical model is benchmarked/frozen;
- store/project/graph schema-operation-validation bundles exist;
- crash/WAL/read-snapshot/current-pointer fixtures exist;
- full publication and rollback vectors/checksums freeze.
