# `wow-emmy` session and snapshot model

**Status:** normative E0-C lifecycle contract.

## 1. Ownership

One `AnalyzerSessionActor` owns one mutable upstream analysis instance. No other component mutates that instance directly.

```text
AnalyzerSessionActor
├── CandidatePin / accepted pin identity
├── AnalyzerConfiguration
├── WorkspaceRegistry
├── FileRegistry
├── upstream analysis instance
├── current mutable revision counter
└── last published AnalyzerSnapshot
```

Writes are serialized. Public reads use immutable snapshots or snapshot-bound operations.

## 2. Identity layers

### Project generation

`ProjectGenerationId` is supplied by the E0 harness and later by `wow-project`.

It identifies the coherent project file/configuration state used by the request.

### Analyzer snapshot

```text
AnalyzerSnapshotId
    derived from:
        accepted upstream pin identity
        analyzer configuration digest
        workspace registry digest
        sorted file identity/content digest set
        supplied ProjectGenerationId
        snapshot schema version
```

It excludes wall-clock time, memory addresses, temporary paths, and scheduling.

### Generation context

Every public fact/finding carries one `wow-core GenerationContext` containing:

```text
ProfileIdentity
ReferenceGenerationId
ProjectGenerationId
relevant tool/schema versions
```

`AnalyzerSnapshotId` is supplementary producer identity, not a replacement for project generation.

## 3. Configuration

```text
AnalyzerConfiguration
    configuration_schema_version
    upstream_pin_id
    language/dialect settings
    diagnostic settings
    workspace role policy
    file-size/count budgets
    diagnostic/fact/output budgets
    incremental update policy
    source coordinate policy
    annotation-library declaration
```

Configuration is explicit data supplied by the harness/service layer. It is never read implicitly from user/editor settings for correctness.

## 4. Workspace roles

```text
WorkspaceRole
    Main
    Library
```

### Main

- first-party project/fixture Lua;
- eligible for project source handles;
- eligible for generic diagnostic findings;
- eligible for semantic/local-flow facts.

### Library

- annotation/declaration source;
- used for resolution/inference;
- not a project implementation file;
- not the primary location for user-facing findings;
- has separate capability/coverage reporting.

A file cannot silently change roles between snapshots.

## 5. Workspace registry

```text
WorkspaceDeclaration
    workspace_id
    role
    logical_root
    source_origin_id
    normalized_file_ids[]
    configuration_digest
```

Rules:

- roots are logical/configured, not public absolute host paths;
- overlapping roots are rejected or explicitly classified;
- duplicate file identity across roles is rejected;
- workspace order is canonicalized;
- full Blizzard UI tree is excluded in E0.

## 6. File identity

```text
AnalyzerFile
    file_id
    workspace_id
    role
    normalized_relative_path
    content_digest
    byte_length
    language_kind
    supplied_project_generation
```

`file_id` is stable for the same logical workspace/path identity. A content change updates digest/revision, not logical file identity.

Rules:

- path is root-relative and slash-normalized;
- no absolute/local paths in public output;
- content bytes are UTF-8 or fail explicitly;
- content digest is verified before publication;
- duplicate path with incompatible content in one snapshot is invalid.

## 7. Session states

```text
Uninitialized
Configured
WorkspacesRegistered
Indexing
Ready
Degraded
Failed
Closed
```

### `Uninitialized`

No upstream instance/configuration.

### `Configured`

Accepted pin and validated configuration installed.

### `WorkspacesRegistered`

Main/library declarations and initial files accepted.

### `Indexing`

A coherent update batch is being applied. No new snapshot is published.

### `Ready`

Required session/library/index capabilities available.

### `Degraded`

A coherent snapshot exists, but named capabilities/files are partial/failed. Exact gaps are published.

### `Failed`

Session state is not trustworthy (panic, poisoned state, invariant violation, fatal pin/config/library failure). No new facts are published.

### `Closed`

No further operations.

## 8. Update batch

```text
AnalyzerUpdateBatch
    expected_previous_snapshot_id
    target_project_generation
    file_operations[]
    configuration_change: optional
```

```text
FileOperation
    Add(file identity + bytes)
    Update(expected old digest + new bytes)
    Remove(expected file identity/digest)
```

Rules:

- batch applies atomically at the framework boundary;
- stale expected snapshot/digest rejects the batch;
- operations sort/canonicalize for identity but preserve semantic final state;
- intermediate mutable state is not exposed;
- failure either leaves previous published snapshot active or marks session failed according to upstream recoverability;
- no silent retry against a new project generation.

## 9. Index refresh

```text
apply update batch
-> update upstream VFS/file state
-> refresh required indexes
-> compute affected file/dependency set
-> run mandatory health probes
-> derive capability/coverage records
-> publish immutable snapshot
```

The adapter must not claim selective invalidation unless the probe proves it for the used upstream API.

## 10. Analyzer snapshot

```text
AnalyzerSnapshot
    snapshot_id
    upstream_pin_id
    generation_context
    configuration_digest
    workspace_digest
    file_manifest[]
    analyzer_capability_records[]
    library_health
    index_health
    published_fact_set_ids[]
    published_generic_finding_ids[]
    canonical_snapshot_digest
```

The snapshot is immutable. It may hold internal handles guarded by the actor, but public consumers receive only normalized snapshot-bound views/data.

## 11. Snapshot publication invariants

- exactly one profile/reference/project generation;
- file manifests match upstream indexed state;
- every source span validates against the exact content digest;
- library/main roles are preserved;
- all facts/findings reference files in the snapshot;
- capability/coverage records identify failed/partial files;
- no unpublished update data leaks;
- canonical ordering/digest is deterministic;
- failed/corrupt session publishes no new snapshot.

## 12. Required operations

### `build_analyzer_configuration`

Validate explicit configuration and derive canonical digest.

### `create_analyzer_session`

Create the upstream instance only after pin/probe acceptance.

### `register_workspace`

Register a Main or Library workspace declaration.

### `add_file`

Add validated UTF-8 content under a registered workspace.

### `update_file`

Require expected logical file identity/old digest and apply new bytes.

### `remove_file`

Remove exact file identity and invalidate dependent facts.

### `apply_update_batch`

Apply coherent file/config changes for one target project generation.

### `refresh_analyzer_index`

Run the upstream indexing/update path required by the selected pin.

### `publish_analyzer_snapshot`

Validate state/capabilities/facts/findings and derive immutable snapshot identity.

### `validate_analyzer_snapshot`

Check all invariants and internal references.

### `close_analyzer_session`

Release session-owned resources; idempotent at framework boundary.

## 13. Capability records

E0 capability/partition examples:

```text
emmy.session.ready
    partition: emmy.session

emmy.library.loaded
    partition: emmy.library:C_E0Fixture

emmy.file.parsed
    partition: emmy.file:<normalized path>

emmy.file.diagnostics
    partition: emmy.file:<normalized path>

emmy.fact.references
    partition: emmy.file:<normalized path>

emmy.fact.local_flow
    partition: emmy.file:<normalized path>
```

Coverage is producer/file/capability specific.

## 14. Failure isolation

### Parse failure

- file parse/fact capabilities fail;
- no fabricated semantic/local-flow facts for that file;
- exact generic parser diagnostic may still be published if upstream provides a valid span;
- unrelated files remain usable only within a validated coherent snapshot.

### Library failure

- `emmy.library.loaded` failed;
- resolution/inference capabilities depending on library become failed/NotEvaluated;
- generic syntax diagnostics not requiring library may remain usable;
- root cause remains explicit.

### Upstream panic or poisoned state

- session transitions `Failed`;
- previous immutable snapshot may remain last-known-good but is not relabeled as the new project generation;
- no partial new snapshot.

### Budget/cancellation

- incomplete operation has typed state;
- no partial facts escape unless the contract explicitly publishes a coherent truncated snapshot with coverage; E0 prefers rejection over truncated fact sets.

## 15. Incremental update requirements

The implementation must demonstrate:

- update one main file changes only proven affected facts/diagnostics;
- library update invalidates dependent member/reference facts;
- removed file facts/findings disappear;
- unchanged file outputs remain byte-identical when reused;
- final output is independent of update order leading to identical contents;
- cross-generation stale update is rejected.

## 16. Actor boundary rules

- no reentrant mutation during diagnostic/fact extraction;
- no consumer callback may mutate session state;
- no raw mutable upstream reference escapes;
- cancellation/close cannot publish half-applied state;
- public methods report session/snapshot identity;
- synchronization implementation remains private.

## 17. E0 fixture snapshots

Expected logical snapshots:

```text
snapshot: baseline-clean
snapshot: baseline-with-generic-error
snapshot: missing-api
snapshot: secret-local-unsafe
snapshot: secret-local-guarded
snapshot: broken-library
snapshot: one-file-updated
```

Exact expected facts/findings are in `examples/` and `TEST_MATRIX.md`.
