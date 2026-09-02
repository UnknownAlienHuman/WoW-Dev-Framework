# E7-A session, request, overlay-reference, and stream storage seam

**Status:** normative cross-crate seam; implementation has not started.

`wow-store` provides generic durable object/catalog/effect/lease/retention mechanisms for E7-A. It does not interpret transport, session, editor, overlay, diagnostic, LSP, MCP, or authorization semantics.

## Logical objects supplied by owners/service

```text
TransportCompatibilityManifest
ServiceOperationRegistry
ClientSession and SessionGeneration records
project/profile binding and resolution receipts
ProjectOverlayGeneration references and retained content objects
SessionSnapshot manifests
TransportRequestReceipt and operation reconciliation records
progress metadata when retained by profile
ArtifactStreamDescriptor and chunk-integrity state
final service envelope/artifact references
lease/expiry/close/audit/quarantine records
```

The producing owner supplies registered schema, canonical bytes, logical ID/digest, prepared operation, validation function through a narrow typed contract, and retention edges.

## Generic operations

Using existing store primitives:

```text
publish immutable object
append session generation/state record
compare-and-swap current session generation
read exact object/record
read snapshot-bound session/request list page
record/reconcile durable operation effect
create/renew/release exact lease
admit/release retention edges
validate backup/restore and GC reachability
```

`wow-store` does not expose raw SQL, connection, transaction callback, table/row ID, filesystem path, object key, serializer, or mutable session handle to service/applications.

## Session state

The store may maintain an append-only session state machine and one guarded current-generation pointer per exact session. It does not decide whether a project/profile/overlay update is semantically valid; service/owners validate prepared transitions before commit and on fresh read-back.

Session current CAS uses exact expected prior generation/digest. Store never silently rebases or picks latest.

## Overlay content

Bounded document bytes may be stored as content-addressed private objects when the selected privacy/license profile allows. The store does not parse source, apply changes, convert coordinates, or infer file identity.

Overlay owner records reference exact content objects. Encryption-at-rest or OS-protected storage is a platform profile; its keys never cross public seams or enter semantic identities.

## Request/effect records

For effecting requests, retain:

```text
OperationId + CanonicalRequestDigest
transport request/session generation
prepared/dispatched/committed/no-effect/unknown state
owner/store effect references
final result and delivery state
cancellation/disconnect/reconciliation
```

A possible commit with lost response remains unknown until exact reconciliation. Same operation ID/different digest fails.

## Progress

Progress is nonsemantic and ephemeral by default. If a daemon recovery/audit profile retains bounded progress, store persists only registered stage codes, sequences and counts under redaction; no source/body/private data.

## Artifact streams

The store can retain stream descriptors and delivered/acknowledged chunk state. It does not transform artifact bytes or authorize disclosure. Stream continuation validates exact artifact/digest/consumer/session/profile/sequence/budget as supplied by service.

## Leases and retention

Leases keep exact session snapshots and referenced owner artifacts available for in-flight/reconnectable requests. Retention edges protect:

```text
current and prior in-flight session generations
project/profile resolution receipts
open/saved-pending overlay generations and content
owner views used by retained editor results
request/effect/reconciliation records
open artifact streams and final artifacts
security/incident/legal/privacy/license holds
```

Closing/expiring a session releases only eligible session-owned leases. It cannot collect evidence referenced by durable results or unresolved effects.

## Recovery

On restore/startup, store exposes exact durable records to `wow-service`; it does not decide to rerun semantic work. Corrupt/missing/incompatible object, state transition, lease, or retention closure is explicit and can quarantine the session/result.

## Hard boundaries

- direct dependency remains `wow-core` only;
- no LSP/MCP/daemon framing;
- no client authentication or service authorization decisions;
- no project/source/overlay/editor semantics;
- no implicit current/latest selection;
- no source execution or provider access;
- no cross-session sharing by default;
- no GC of unresolved/in-flight/reconnectable evidence;
- no public success inferred from physical commit alone.

## Tests

Required tests cover session-generation CAS, concurrent updates, effect reconciliation, lease renewal/expiry, crash between each write boundary, content corruption, backup/restore, GC reachability, private object policy, stream resume, cross-session key substitution, and deterministic logical records independent of SQLite/WAL/layout details.