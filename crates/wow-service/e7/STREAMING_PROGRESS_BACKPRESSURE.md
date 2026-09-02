# E7-A streaming, progress, backpressure, and output integrity

**Status:** normative.

## Principle

The canonical service operation has one final result envelope. Progress notifications and artifact chunks are transport aids, not alternate result semantics.

## Progress events

A progress event binds:

```text
exact transport request and service operation IDs
session generation and client scope
monotonic sequence
phase/stage code
bounded counters and optional total
parent/child progress relationship
redaction profile
nonsemantic = true
```

Progress cannot contain unbounded source, raw provider responses, credentials, private paths, owner handles, or confidential review/cohort data.

Allowed stage codes come from the operation descriptor. Arbitrary source/provider text cannot become a stage name that a client interprets as control data.

## Progress lifecycle

```text
Begin
Report
End
```

- `Begin` may be emitted only after request admission.
- `Report` sequences are monotonic and bounded.
- `End` means progress reporting ended, not that the operation succeeded.
- The final result/error envelope remains mandatory unless the connection is lost.
- Missing progress never changes semantic status.

A client cancellation request may race with progress. The final service/cancellation/reconciliation state controls.

## Artifact streams

Large exact artifacts may be retrieved through `artifact_stream_open` and `artifact_stream_continue`.

The stream descriptor freezes:

```text
artifact ID/digest/size/schema/media type
consumer/session/privacy/license scope
chunk profile and maximum total bytes
stream expiry/lease
integrity chain and final digest
```

Each continuation supplies exact stream ID, expected next sequence, prior chunk digest, and cumulative budget. It cannot change artifact, consumer, policy, chunk profile, or reset limits.

Artifact stream output is exact byte transport. It does not reserialize, pretty-print, normalize newlines, decompress unknown archives, or infer media type.

## Stream integrity

Every chunk records exact byte range and digest. The final chunk binds the whole-artifact digest and expected size. Duplicate/reordered/skipped/overlapping chunks fail validation.

A client may resume only from a retained validated stream state. If the artifact or stream lease is unavailable, return stale/unavailable; never open a newer or same-name artifact automatically.

## Privacy and license

A stream is admitted only if the selected consumer profile may receive the entire artifact. The service does not stream a broad artifact and expect the transport/client to redact it.

Where only bounded fields are permitted, the owner/service first creates a separately identified redacted artifact and streams that artifact.

## Backpressure budgets

Every host profile freezes limits for:

```text
connections and sessions
in-flight requests per client/session
queued request count and bytes
progress events and bytes
open artifact streams
chunk bytes and outstanding chunks
stdout/stderr/pipe/socket buffered bytes
owner/service tasks and memory
```

Unlimited values are invalid. Negotiation can only reduce limits.

## Queue behavior

Requests are admitted in deterministic policy order. Overload produces an explicit bounded `Busy`, `Backpressure`, or `ResourceLimit` error before service dispatch whenever possible.

The host never silently drops, duplicates, coalesces, or reorders semantic requests. Transport-specific document-change coalescing is forbidden unless the resulting exact full document/version is supplied as one new explicit overlay request and skipped versions are recorded as transport-only superseded messages.

## Slow clients

A slow client cannot hold unbounded result/progress memory. The profile may:

```text
pause transport reads
stop optional progress
cancel a not-yet-effecting request
close an artifact stream
close the client session
```

It cannot report success before delivery, discard a committed effect record, or silently repeat a request.

## Broken pipes and disconnects

Output failure after a service effect does not undo the effect. The host records final-delivery state separately and returns/reconciles it on reconnect where supported.

One-shot CLI maps broken pipe/output failure to its defined nonzero exit without a second service call. Stdio LSP/MCP closes/reconciles according to session class. Daemon sessions retain exact request receipts under their lease profile.

## Ordering

Within one request, progress and chunks are sequence-ordered. Across requests, no global semantic order is inferred from arrival/completion. Effect ordering is established only by exact owner operation/state receipts.

## Determinism

Canonical final envelopes and artifacts are transport-independent. Chunk boundaries, progress frequency, buffering, and scheduling are operational and excluded from semantic digests, while each emitted chunk/event has its own integrity and request binding.