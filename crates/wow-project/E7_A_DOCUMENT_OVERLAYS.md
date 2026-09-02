# E7-A project-owned workspace registration and document-overlay seam

**Status:** normative cross-crate seam; implementation has not started.

`wow-project` owns validation of explicit workspace roots, source acquisition under project policy, immutable unsaved document overlays, disk reconciliation, and publication of new exact project generations. `wow-service` coordinates frontend sessions and transports.

## Operations

```text
project_workspace_register
project_workspace_validate
project_workspace_unregister
project_document_overlay_open
project_document_overlay_change
project_document_overlay_save
project_document_overlay_close
project_document_overlay_get
project_document_overlay_analyzer_input
```

## Workspace request

```text
explicit root URI/path
platform path profile
source/project/reference profile selector and exact resolution guard
include/exclude/access boundaries
trust/privacy/license policy
watch/snapshot behavior
budgets and cancellation
```

A supplied root is untrusted. The owner normalizes it, enforces the permitted root, validates file-system identity and path-policy constraints, and returns an exact registration record. It does not infer a root from cwd, Git, editor state, an AddOns folder, a WoW installation, or a parent directory.

## Overlay model

Each accepted document state is immutable:

```text
session/workspace/document URI
canonical owner-relative path
language/profile
client version
exact prior overlay snapshot
UTF-8 content bytes/digest/line index
applied change manifest
persisted source/project base identity
saved/dirty/conflict state
privacy/retention policy
```

The owner representation uses canonical UTF-8 byte offsets. LSP UTF-16/UTF-8 positions are converted and validated by the E7-A service projection against the exact overlay line index.

## Change application

- Require exact prior overlay ID/digest and strictly advancing client version.
- Validate every ordered change range against the same declared prior/intermediate snapshot according to the frozen synchronization profile.
- Reject the entire new snapshot if any range, encoding boundary, size, overlap/order, or digest guard fails.
- Never apply a subset or guess a missing version.
- Return `ResynchronizationRequired` for stale, repeated-with-different-content, skipped, or out-of-order changes.

## Save and disk reconciliation

A save notification is not proof of disk state. Supported results:

```text
ContentAcknowledgedOnly
DiskReconciled
NewProjectGenerationPublished
ConflictWithDisk
NotEvaluated
Failed
```

When exact reconciliation is requested, the owner reacquires the explicit registered source path under the normal nonexecuting source policy and compares bytes/digests. Publishing a new project generation is a separate durable operation. Unsaved overlay state never mutates a retained generation in place.

## Overlay analyzer input

`project_document_overlay_analyzer_input` produces one exact bounded owner view for `wow-emmy` and dependent read-only operations. It identifies which files/units are overlay-backed, persisted-generation-backed, stale, missing, or excluded. It does not run the analyzer itself.

## File-watch hints

File-watch notifications can request reacquisition but are not source evidence. Coalesced/dropped/high-fanout events require explicit rescan state and cannot yield clean complete publication by assumption.

## Multi-client isolation

Overlay keys include session and consumer scope. Two sessions editing the same file receive different overlay snapshots. One session cannot read another session's unsaved bytes. Shared state begins only at a separately authorized immutable project publication.

## Close and retention

Closing an overlay releases session-private bytes after active leases finish. Default retention is memory-only and destroyed synchronously. Optional encrypted recovery requires an explicit profile, owner, expiry, consumer binding, audit, and separate artifact identity.

## Hard boundaries

- no dependency on `wow-service`, `apps/wow`, LSP, MCP, editor SDKs, or transport libraries;
- no repository hooks, tasks, binaries, package managers, Lua, XML scripts, generated code, or project-local tools executed;
- no auto-discovery or editor-setting mutation;
- no graph/search/context/external-provider semantics;
- no source edit authorization merely because an overlay exists;
- no complete saved-generation claim for dirty overlays;
- no raw filesystem handles or unrestricted roots crossing the owner port.

## Tests

Cover Windows and Unix path policy, case/normalization collisions, symlink/junction/reparse/device/UNC/ADS boundaries, full and incremental changes, UTF-8/UTF-16 range conversion, stale versions, partial failures, save conflicts, watcher hints, privacy cleanup, two-client isolation, cancellation, response loss around generation publication, and deterministic bytes.