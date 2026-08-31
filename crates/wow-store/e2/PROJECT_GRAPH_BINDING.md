# Project, analyzer, recognizer, graph, and store binding

**Status:** normative cross-crate E2-D seam; ownership remains separated.

## Identity DAG

```text
materialized source + profiles
-> ProjectGenerationId
-> AnalyzerSnapshotId
-> ProjectIndexCandidateId

validated project/direct proposals
+ recognizer producer partitions
+ graph registry/base
-> GraphPublicationPlanId
-> GraphGenerationId / GraphSnapshotId

ProjectIndexCandidateId
+ GraphPublicationPlanId
+ project/graph logical partition manifests
-> ProjectPublicationSetId
-> ProjectSnapshotId

ProjectPublicationSetId
+ complete partition membership
+ schema set and object refs
-> ProjectStoreGenerationId

validated ProjectStoreGenerationId
-> CurrentPublicationRecord
```

No domain ID depends on SQLite row/page/order or the current pointer.

## `wow-project` responsibilities

- verify E2-C candidate state and exact base publication;
- request graph plan from `wow-graph`;
- assemble project logical partition versions;
- combine exact project and graph plans into one `ProjectPublicationSet`;
- invoke `wow-store` public E2-D operations;
- validate returned inactive/current bindings;
- expose the domain `PublishedProjectView`.

## `wow-graph` responsibilities

- validate producer partition replacements;
- derive semantic keys/assertion IDs/conflicts/coverage;
- derive `GraphGenerationId` and `GraphSnapshotId`;
- provide graph logical partitions, write operations, validation catalog, and golden queries;
- validate the persisted graph view.

## `wow-store` responsibilities

- validate schemas/catalogs/parameters;
- materialize/reuse physical partition versions;
- build complete membership;
- commit inactive store generation;
- expose exact read snapshots;
- execute generic and owner validation operations;
- activate the current record by CAS;
- manage WAL/checkpoint/recovery/retention/GC.

## Analyzer binding

The publication set references the exact analyzer snapshot, pin/config/profile, file/unit manifest, fact/finding partition manifests, and coverage. Store never opens or controls the analyzer.

After restart, a live analyzer may be rebuilt, but persisted facts retain the original analyzer snapshot identity. Rehydration cannot relabel them.

## Recognizer binding

Every recognizer partition retains exact pack/rule/version/input/generation identity. Graph maps accepted proposals to assertions and project records retain rejected proposals. Rule update or disablement creates new partitions and a new publication.

## Current coherence

Domain open performs:

```text
open one store read transaction
-> read current record
-> validate epoch/schema/profile
-> open exact generation membership
-> validate ProjectPublicationSet
-> open exact ProjectSnapshot and GraphSnapshot
-> verify analyzer/source/profile bindings
```

Any mismatch fails; no component substitutes another generation.

## Partial candidates

A `PartialCandidate` can publish only when policy explicitly allows every incomplete partition and all `NotEvaluated`, conflict, and truncation state is retained. Storage success never upgrades coverage.

## Last-known-good and rollback

Failed targets keep target IDs. Last-known-good keeps original IDs. Explicit rollback activates a retained fully validated publication set through a new CAS record; it never merges old and new state.
