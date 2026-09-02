# E7-A generic session, lease, and response-journal storage seam

**Status:** normative cross-crate seam; implementation has not started.

`wow-store` supplies generic persistence primitives for frontend registry generations, durable sessions where enabled, workspace registrations, operation tickets, leases, response delivery journals, and retained immutable results. It does not interpret LSP, MCP, editor, project, diagnostic, or service semantics.

## Logical objects

```text
FrontendOperationRegistry bytes and manifest
FrontendSession durable metadata, excluding private ephemeral source
WorkspaceRegistration record
OperationTicket and durable effect references
ResponseJournalEntry
Session/operation/result lease and retention edges
consumer/privacy/license access metadata
reconciliation and audit references
```

Unsaved document content and overlay bodies are memory-only by default and are not generic store objects. An explicit encrypted recovery profile would supply a separate schema and lifecycle.

## Generic operations

Using existing store contracts:

```text
publish/read immutable object
append/read state/catalog record
acquire/renew/release exact lease
register/reconcile durable operation
append/read response delivery journal
snapshot-bound list/query
admit/release retention
backup/restore/GC reachability validation
```

## Response journal

A journal entry binds exact session/consumer/operation/result, transport response ID, prepared/sent/acknowledged/lost state, replay authorization, expiry, and retention. It records delivery only and never changes the service result status.

Replay returns exact retained bytes. It never re-invokes the service operation.

## Session persistence

The default embedded stdio LSP/MCP profile may use ephemeral sessions with only durable operation/result records persisted. The daemon profile may persist bounded session metadata and response journals for reconnect. Session persistence cannot imply unsaved overlay persistence.

## Leases

Leases protect exact owner/result artifacts needed by active operations or response replay. A lease binds owner artifact ID/digest, session/consumer, purpose, expiry/renewal policy, and release receipt. Local clocks cannot establish trusted expiry when the selected policy requires stronger evidence.

## Hard boundaries

- direct dependency remains `wow-core` only;
- no LSP/MCP/JSON-RPC framing or protocol method semantics;
- no client authentication, workspace path validation, document editing, diagnostics, search, context, or code-action logic;
- no raw SQL, connection, row ID, filesystem/object-store key, or transaction callback crosses the service seam;
- no unsaved source body in default persistent records;
- no response journal treated as semantic completion;
- no replay that re-executes an operation;
- no GC of live session/operation/result/evidence leases.

## Recovery

After crash/restore, the service uses exact generic store records to classify open/closing sessions, active/unknown operations, retained final results, delivery-lost responses, expired leases, and orphan journals. Store never decides whether to resume/cancel/reconcile a domain operation.

## Tests

Cover journal append/read/replay identity, crash between result commit and response send, cross-consumer replay rejection, lease renewal/expiry/release, active-evidence GC protection, backup/restore, corrupted journal/object, ephemeral-session behavior, absence of unsaved source, and deterministic logical records independent of SQLite layout.