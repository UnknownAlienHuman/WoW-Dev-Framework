# Rejected ProjectStore physical models

**Status:** normative E2-D architecture rationale. These alternatives are not implementation options unless a later reviewed contract revision replaces the selected profile.

## Selected profile

```text
project-store-wal-manifested-partitions-v1
```

The selected model is one owned SQLite database per compatible ProjectStore epoch, WAL with one writer, immutable content-addressed partition versions, and a complete generation-to-partition membership map. A target generation is committed inactive, validated through an exact read snapshot, and activated by a separate compare-and-swap update of one coherent current publication record.

## Rejected: complete SQLite file per project generation

```text
generations/<StoreGenerationId>/project.sqlite
```

This model gives simple immutable files and independent readers, but it is rejected for the interactive project index because a normal one-file edit would require cloning, copying, or rebuilding the whole logical database image. Cross-platform copy-on-write/reflink behavior is not guaranteed, Windows replacement and active-reader deletion behavior complicate retention, and physical image production would become the dominant incremental cost.

The useful guarantees developed while evaluating this model remain required in the selected profile:

- exact project/graph publication-set binding;
- one coherent current selector;
- idempotent response-loss recovery;
- explicit inactive/validated/current states;
- reader and evidence retention roots;
- crash classification and quarantine;
- no last-known-good relabeling;
- no empty-row negative authority;
- Windows sharing/locking tests;
- deterministic logical manifests independent of physical SQLite bytes.

The generation-image design merged in PR #13 is superseded by this document and the selected profile. Git history remains the historical record; its image-specific files are not current implementation guidance.

## Rejected: full duplicated rows for every generation

Every retained generation would own a complete copy of every project and graph row. This avoids ancestry traversal but makes storage and write amplification proportional to the product of project size and retained generations. It also weakens partition-level reuse and makes small edits unnecessarily expensive.

## Rejected: recursive base-plus-delta generation chain

A generation would point to a base generation and store only changed rows/partitions. This minimizes writes but makes read cost, retention, corruption recovery, and deletion depend on potentially unbounded ancestry. It also makes exact historical views vulnerable to missing ancestors and complicates deterministic bounded queries.

The selected profile keeps complete membership maps, so unchanged partition versions are reused without recursive lookup.

## Rejected: separate project and graph databases or pointers

Independent stores/pointers cannot provide one atomic current view without a cross-database transaction and recovery protocol. E2-D requires project and graph records for one publication set to live in the same epoch database and be selected by one current record.

## Rejected: multiple writers

SQLite can serialize writers, but allowing independent writer owners would move stale-base, publication-order, checkpoint, retention, and GC races into every caller. V1 has one writer owner and finite busy behavior. Any future multi-process writer design requires a new explicit contract.

## Rejected: external process readers in v1

External readers would bypass the framework's generation leases, retention roots, registered query catalogs, and coverage semantics. V1 readers are process-local and snapshot-bound. A future external-read profile must define durable leases, locking, version negotiation, and GC behavior.

## Rejected: server or distributed database

The framework is local-first and rebuildable. A server introduces network identity, authentication, availability, consistency, migration, and operational requirements that are not justified by E2 workloads.

## Benchmark override rule

Benchmarks may demonstrate that the selected profile fails its frozen gates, but implementation agents may not silently switch models. A change requires:

1. measured failure under the frozen synthetic and pinned real-addon workloads;
2. a replacement contract with exact reader/publication/recovery/GC semantics;
3. migration or rebuild rules;
4. updated fixtures and checksums;
5. explicit owner review.
