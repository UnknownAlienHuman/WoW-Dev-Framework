# E7-A sessions, project/profile binding, and editor overlays

**Status:** normative.

## Session opening

`session_open` creates one transport-scoped session from explicit host/client/exposure/privacy/resource profiles. It does not discover a project, repository, WoW installation, profile, editor, or current store state.

```text
validate transport/client/profile request
-> register effect when durable
-> create ClientSession
-> create initial empty SessionGeneration
-> admit lease/retention when required
-> close resources
-> return exact session/generation IDs
```

Stdio sessions are process-scoped by default. Local-daemon sessions may be reconnectable and durable under an exact lease profile.

## Project binding

`session_project_bind` accepts one explicit project registration request or one exact project publication/generation selector. A permitted symbolic current selector is resolved once by service and replaced with exact IDs plus a resolution receipt.

The binding validates:

```text
ProjectStore/publication/generation identity and retention
source-root identity through project owner records
selected project profile and capabilities
privacy/license/consumer scope
compatibility with the session data root and selected ReferenceProfile
```

Transport cwd, LSP workspace folder, MCP root, Git repository, open document URI, or daemon data directory is only a registration candidate. None becomes an owner project identity without service/project validation.

Project rebinding creates a new `SessionGeneration`. Existing in-flight requests keep the prior generation and its retained owner views.

## Profile binding

`session_profile_bind` resolves and validates one exact WoW/reference/analyzer/recognizer/rule/graph/context profile set. It records:

```text
flavor/build/Interface identity
ReferenceProfile/ReferenceGeneration
analyzer implementation/profile
recognizer/core-pack execution profile
rule/graph/context profiles
capability report
resolution and retention receipts
```

No automatic refresh occurs when a newer profile/reference/core pack appears. Rebinding is explicit and creates a new session generation.

## Overlay opening

`session_overlay_open` requires an exact bound project generation and a URI/path that the project owner maps to one exact source file. Service passes the request to the `wow-project` overlay port and never opens the file itself.

The request binds:

```text
session/client/project generation
transport document URI
editor document version
full UTF-8 bytes and digest
line-ending/encoding/coordinate profile
privacy/license/resource limits
OperationId + CanonicalRequestDigest
```

A URI outside the exact project owner scope, an ambiguous mapping, invalid encoding, oversized content, or stale/conflicting document identity fails.

## Overlay change

Baseline `session_overlay_change` is full-document replacement with a strictly increasing editor document version. The owner validates prior overlay identity and publishes a new immutable overlay document/generation.

Optional incremental change profile additionally binds:

```text
exact prior document version/digest
ordered nonoverlapping LSP UTF-16 ranges
range lengths and replacement bytes
UTF-16-to-UTF-8 coordinate map digest
```

Invalid surrogate boundaries, stale versions, overlapping/out-of-range edits, coordinate overflow, or final digest mismatch fail. Service and app do not attempt heuristic recovery or request reordering.

## Overlay save

`session_overlay_save` records editor save intent and the exact saved bytes/digest when supplied. It does not assume disk contents or publish a new project generation.

State becomes `SavedPendingBaseRefresh` until one of:

```text
explicit project refresh publishes a new exact ProjectGeneration containing the same file digest
explicit close discards the session overlay and returns to the bound base generation
explicit rebind chooses another exact project generation
```

If a refreshed project generation contains different bytes, return conflict; do not silently discard the overlay.

## Overlay close

`session_overlay_close` creates a new overlay/session generation without that open overlay. It reverts semantic requests to the exact bound base project generation, not to floating current or disk state.

Closing an overlay does not delete retained generations still referenced by in-flight requests, results, diagnostics, streams, or audit.

## Session snapshot

Before any semantic operation, service freezes:

```text
SessionGeneration
project/profile/reference/graph/analyzer/core-pack bindings
overlay generation set
capability/exposure/privacy/license/budget profiles
```

The resulting `SessionSnapshot` is the only session input to owner acquisition. A later session update cannot affect it.

## Concurrent updates

Session generation update uses exact expected-current compare-and-swap. Two concurrent project/profile/overlay updates cannot both claim the same prior generation. A stale update fails and must be reconstructed against an explicitly fetched newer generation.

Semantic requests can execute concurrently against immutable snapshots. Mutable owner sessions remain inside their owners.

## Lease and expiry

A durable daemon session binds an exact lease/expiry profile. Expiry stops new requests and closes transport/session resources; it does not delete referenced owner artifacts or assume effect cancellation. Renewal is explicit, bounded, authorized, and creates a receipt.

## Session close

`session_close` prevents admission of new requests, cancels or drains in-flight requests according to the exact profile, reconciles effects, releases session-owned leases, closes resources, and records `Closed`, `OutcomeUnknown`, or `Failed`.

No detached cleanup continues after the final response.

## Hard stops

- no implicit cwd/workspace/root project registration;
- no floating profile/current refresh;
- no app/service source patching;
- no overlay cross-session visibility;
- no document-version rollback or guessed ordering;
- no save-equals-published-base assumption;
- no close/expiry deleting retained evidence;
- no success before mandatory lease/retention/close records.