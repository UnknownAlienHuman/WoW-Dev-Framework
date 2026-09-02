# E7-A JSON-RPC framing, parsing, results, and errors

**Status:** normative common semantic boundary. Application contracts define protocol-specific byte framing.

## Layering

```text
byte transport/framing parser
-> exact JSON value parser
-> JSON-RPC envelope validation
-> protocol lifecycle/method/tool validation
-> typed params validation
-> service request/result
-> typed JSON-RPC result or error
-> canonical JSON bytes
-> protocol-specific framing
```

Each layer has separate limits and errors. A framing failure never reaches semantic dispatch.

## JSON parsing

The exact profile freezes:

- UTF-8 and BOM policy;
- duplicate object-key behavior;
- maximum message bytes, nesting depth, string bytes, array/object members, number length;
- JSON number interpretation;
- invalid Unicode/control behavior;
- trailing bytes/whitespace behavior;
- canonical output serialization.

Duplicate keys are rejected. No comments, trailing commas, NaN, Infinity, extensions, or polyglot fallback.

## JSON-RPC envelope

Validate:

```text
jsonrpc exact required value under pinned profile
request/response/notification object shape
method string and length
params allowed object/array form under exact operation schema
ID type/value under profile
result XOR error for responses
error code/message/data schema
no extra top-level fields unless exact profile allows/preserves them
```

A source/client string cannot become the method field after parsing.

## Request, notification, and response

```text
Request
    method + ID + optional params
    exactly one response unless transport closes before delivery

Notification
    method + no ID + optional params
    no response

Response
    exact echoed ID
    exactly one of result or error
```

Batch arrays are rejected in initial E7-A profiles.

## Protocol-specific framing

The common service never parses raw headers/lines. Applications provide a validated `FramedJsonRpcMessage` containing exact bytes/digest and framing receipt.

- LSP app owns its exact pinned stdio header/body framing.
- MCP app owns its exact pinned stdio message framing.

Neither application can feed bytes valid only under the other profile.

## Result mapping

A domain operation that executed and produced a valid typed service envelope usually maps to a JSON-RPC `result`, including:

```text
Complete
NoChange
CandidateOnly
Partial
Blocked
ConflictBlocked
Truncated
OutcomeUnknown
NotEvaluated
Cancelled where the protocol/result profile permits
```

The result includes exact domain status, evidence/coverage/nonclaims, and continuation/reconciliation IDs. JSON-RPC success framing does not mean domain proof/pass.

## Error mapping classes

```text
ParseError
InvalidRequest
MethodNotFound
InvalidParams
LifecycleError
CapabilityNotNegotiated
BindingOrDocumentStateError
PermissionDenied
RequestCancelled
ContentModifiedOrDesynchronized
ResourceLimitExceeded
InternalProtocolError
TransportOrDeliveryError
OwnerInfrastructureError
```

Exact numeric/string protocol codes are pinned in the LSP/MCP/JSON-RPC error profile before implementation.

## Domain error versus result

The registry entry defines whether a service outcome is represented as result or error. Rules:

- malformed protocol/params/lifecycle/permission failures are errors;
- completed domain validation returning `Invalid` is normally a result with validation state;
- Candidate/partial/blocked/NotEvaluated is normally a result;
- unsupported negotiated owner capability can be a result or protocol error only as frozen per entry;
- internal serialization/framing failures are protocol/transport errors;
- `OutcomeUnknown` remains an explicit domain result or specialized error with exact recovery data; it is never converted to ordinary failure/no-effect.

## Error object

```text
ProtocolErrorData
    stable framework error code
    session/request/binding/document IDs when safe
    method/tool registry entry
    domain operation/result/recovery refs when safe
    lifecycle/capability/coverage/conflict state
    response-loss/OutcomeUnknown state
    privacy-redacted structured arguments
```

Error `message` is a stable bounded framework string. Untrusted source/provider/client text is not interpolated without strict escaping/redaction and is never instructions.

## Unknown method/tool

Unknown names map to method/tool-not-found according to the exact protocol profile. No case folding, prefix match, nearest name, dynamic extension, or generic service call.

## Invalid params

Includes unknown fields, schema/type/range/enum errors, exact ID/digest mismatch, forbidden credential/URI/tool field, invalid document position/version, unbounded limits, and cross-operation fields.

No partial typed request enters service.

## Cancellation error/result

Cancellation mapping follows the pinned protocol and registry entry. Regardless of wire code:

- exact target request ID is preserved;
- completed domain effect can still be returned/reconciled;
- cancellation never proves no effect;
- no second response is emitted.

## Content modified/desynchronized

Document binding/version/digest/position mismatch maps to an exact protocol error/result requiring explicit resynchronization. The server never silently reopens disk or applies changes to a different overlay.

## Response serialization

Canonical semantic response JSON is generated from typed result/error records:

- deterministic field ordering/profile;
- exact ID type/value;
- canonical strings/numbers/nulls/arrays;
- no nonfinite floats;
- bounded bytes;
- source/provider text structurally escaped;
- no incidental timestamp/host/process/connection state in semantic digest.

Application framing wraps exact JSON bytes without modifying them.

## Delivery

Write/flush state is distinct from domain completion. A partial write or disconnect records response-delivery uncertainty and closes/fails the transport; it never emits a second repair response.

## stderr/stdout

Protocol stdout contains only framed messages. Logs/progress not defined as protocol messages use bounded stderr. No banner, panic text, debug dump, ANSI, or human prose on stdout.

## Tests

- malformed UTF-8/JSON/duplicate keys/depth/number bombs;
- request/notification/response shape;
- ID type echo and no coercion;
- batch rejection;
- unknown method/tool;
- invalid params before service;
- Candidate/Partial/Invalid validation status preserved as result;
- OutcomeUnknown not folded failed/no-effect;
- stable error data redaction;
- source text with JSON/framing/control sequences;
- response serialization golden bytes;
- partial frame write and no duplicate response;
- LSP/MCP framing cross-use rejection.
