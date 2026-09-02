# E7-A sessions, workspace registration, and document overlays

**Status:** normative.

## Session opening

`frontend_session_open` validates one exact protocol profile, client identity, consumer/privacy/license profile, registry generation, resource limits, and optional parent daemon connection. It returns a session ID and negotiated capability set. Client/version metadata is provenance and compatibility input, not semantic authorization.

A session starts with no implicit project, workspace, document, profile, current generation, provider, or credential.

## Workspace registration

A workspace is registered only through `frontend_workspace_register` with:

```text
explicit URI/path
platform path profile
intended project/source universe
explicit source/project/reference profile selector
include/exclude/access/privacy/license policy
watch/snapshot behavior
OperationId when durable publication may occur
```

The service delegates path/source/project validation to `wow-project`. It never searches parent directories, selects a Git root, scans an AddOns directory, reads editor settings, or guesses the WoW installation.

Workspace registration states:

```text
RegisteredExactProject
RegisteredUnmaterialized
SingleDocument
Blocked
Conflict
NotEvaluated
Closed
Failed
```

`RegisteredUnmaterialized` does not claim successful project analysis.

## Document opening

`frontend_document_open` requires a registered workspace or explicit single-document profile, canonical URI, language/profile, client version, and full content. The project owner returns an immutable overlay snapshot based on the exact persisted source/project generation or an explicit no-base state.

Duplicate open with identical session/URI/version/content can be `NoChange`; incompatible duplicate state is conflict.

## Incremental changes

Each `frontend_document_change` binds:

```text
exact prior overlay snapshot ID/digest
strictly advancing client version
negotiated position encoding
ordered nonoverlapping change sequence as declared by profile
new content length/digest limits
cancellation
```

The owner validates every range against the exact prior snapshot. Ranges cannot split invalid UTF encoding units or point outside the document. Failure of any change rejects the entire new snapshot; no partial application.

If versions are skipped, repeated with different content, or arrive out of order, return `ResynchronizationRequired`. The service never guesses order or requests disk content as a substitute.

## Position encoding

The LSP profile supports exact negotiated `utf-16` and `utf-8` position encodings. The session records one encoding. Every line/character position is converted against the overlay's exact line index and content bytes.

Invalid surrogate boundaries, malformed UTF-8, overflow, negative values, stale document versions, or line-ending mismatches are typed failures. Internal source handles use canonical UTF-8 byte offsets and exact source digests.

## Save

`frontend_document_save` distinguishes:

```text
ContentAcknowledgedOnly
DiskReconciled
NewProjectGenerationPublished
ConflictWithDisk
NotEvaluated
Failed
```

A save notification without text does not prove the disk bytes. When exact disk reconciliation is required, the project owner reacquires the explicit registered path under its source policy and compares digests. Publishing a new project generation is a separate durable effect with its own receipt.

## Close

`frontend_document_close` releases session overlay ownership after active operations stop or retain exact leases. It does not delete published project generations or durable results. Unsaved bytes are destroyed by default after mandatory close unless an explicit secure recovery profile retained them.

## Overlay analysis

Overlay-aware diagnostics/hover/definition/references/symbol/completion operations bind one exact overlay snapshot. Owner capabilities may be:

```text
OverlayLocalComplete
OverlayLocalPlusPublishedDependencies
OverlayPartialGraph
SavedGenerationOnly
NotEvaluated
```

A result must expose which model applies. Published graph/search/context data cannot be represented as current for changed unsaved content without an explicit partial/staleness boundary.

## Workspace changes

Changing root, profile, includes, trust/privacy/license, or base project generation creates a new registration identity. Existing overlays remain bound to the prior registration until closed or explicitly migrated through a validated operation.

## File watching

File-watch notifications are hints, not source truth. The project owner reacquires exact bytes and publishes a new generation under its contracts. High fanout is bounded and coalesced; dropped hints require a rescan request, not silent completeness.

## Multi-client behavior

Two clients editing the same file receive separate overlays unless an explicit collaborative-session profile exists. One client's unsaved content never changes another client's results. Persisted project generation updates are shared only after publication and exact reacquisition.

## Privacy and retention

Unsaved source is private session-scoped data. It is excluded from default logs, crash reports, progress, MCP resources, and durable stores. Any opt-in recovery retention requires encryption/consumer scope/expiry/audit and remains distinct from project publication.