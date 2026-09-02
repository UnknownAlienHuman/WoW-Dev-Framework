# E7-A LSP framing, document synchronization, and positions

**Status:** normative.

## Transport

Initial candidate transport:

```text
stdio
single client per process
one exact LSP protocol profile per process/session
```

The adapter implements the framing required by the frozen official LSP profile. It does not autodetect alternate protocols, expose a TCP listener, or mix log/progress text into protocol stdout.

## Framing

- Parse headers under strict ASCII/name/value/length limits.
- Require and validate the profile’s content-length semantics.
- Reject duplicate/conflicting length or unsupported transfer encodings.
- Bound body length before allocation.
- Parse one JSON-RPC message under strict JSON limits and duplicate-key policy.
- Preserve request ID type/value exactly within the protocol profile.
- Reject trailing/polyglot bytes that are not another valid framed message.
- Protocol stdout contains frames only; logs use stderr or configured telemetry.

Framing errors never enter domain owners.

## JSON-RPC

- Validate version, method, params, request/notification distinction, and ID.
- Unknown methods receive exact method-not-found behavior.
- Malformed requests receive protocol errors with no service call.
- Batch messages are unsupported unless a later frozen profile implements them.
- Notifications never receive responses except as required by the protocol.
- Duplicate active request IDs are rejected/close according to profile.
- A late response is not emitted after the request has already received a terminal protocol response.

## Initialization ordering

Before successful initialize:

- only profile-allowed pre-initialize messages are accepted;
- no document or analysis service operation executes;
- workspace/current resolution occurs only through the initialization/workspace-bind contract;
- advertised capabilities are computed from actual service/session availability.

After shutdown:

- no new normal requests are accepted;
- `exit` closes transport without a second service shutdown effect.

## Position encoding

The negotiated `PositionEncodingProfile` freezes:

```text
supported encodings and preference order
line/character interpretation
UTF-8/UTF-16 code-unit conversion
newline recognition
invalid Unicode handling
source-map conversion for virtual/generated documents
out-of-range/clamping policy
```

Default behavior is exact validation, not clamping.

Every position conversion binds exact document content digest and version. A position from another overlay generation is stale.

## Unicode rules

Test and handle:

- ASCII;
- multibyte UTF-8;
- BMP characters;
- astral characters represented by UTF-16 surrogate pairs;
- combining marks;
- CRLF, LF, and profile-supported newline forms;
- empty/final lines;
- invalid UTF sequences under byte-oriented inputs;
- positions inside a code point or surrogate pair;
- integer overflow and huge lines.

The adapter does not normalize source text unless the document profile explicitly defines and records the transformation.

## Document synchronization

The initial profile freezes either incremental sync with full-open content or a stricter full-sync fallback. For incremental changes:

```text
exact open version/content
+ exact prior overlay generation
+ strictly valid new version
+ ordered range edits under negotiated position encoding
-> one document_change service request
```

The app does not apply edits independently to derive semantic results; it may validate/construct the exact service request and then relies on service’s canonical overlay result.

## Resynchronization

When versions/ranges/content diverge:

- reject the change;
- report the exact synchronization error;
- request/require a profile-defined full resync or close/reopen;
- do not guess missing edits;
- do not read disk to repair the buffer;
- do not rebind to current.

A full resync creates a new exact overlay record under the same session only if the profile permits it and records the discontinuity.

## URI handling

- Parse and normalize under the protocol/platform profile.
- Map only through the service workspace/document owner seam.
- Do not open URI paths directly.
- Reject unsupported schemes, traversal, device/UNC/ADS/reparse escape, normalization collision, or cross-workspace mapping.
- Return authorized protocol URIs without leaking private absolute paths when the privacy profile forbids them.

## Save and close

`didSave` maps to observation only. Text, if supplied, is validated and recorded by service; it does not cause filesystem writes.

`didClose` maps to one close operation and drops session overlay state. The app does not save or publish.

## Output ordering

Responses may complete out of request order as permitted by JSON-RPC, but semantic lists inside each response use service canonical order. Notifications/progress preserve protocol sequencing requirements.

## Backpressure and broken pipe

- Bound input/output frame queues.
- Stop reading or reject under the profile when saturated.
- No frame is interleaved with logs.
- Broken stdout/pipe stops projection and initiates session disconnect handling.
- Service operation is never automatically repeated.
- Already committed owner effects retain exact state/reconciliation records.

## Conformance vectors

Freeze exact bytes for initialize, shutdown, open/change/save/close, diagnostics, navigation, code action, cancellation/progress, protocol errors, Unicode positions, malformed headers, oversized bodies, duplicate keys, broken frames, and shutdown races on every supported platform.
