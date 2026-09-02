# E7-A MCP framing, lifecycle, results, cancellation, and progress

**Status:** normative.

## Transport

Initial candidate:

```text
stdio
single client per process
one exact MCP specification/profile per session
```

The exact message framing, JSON-RPC behavior, initialization sequence, capability fields, notifications, progress, cancellation, and errors are frozen from the official MCP specification and repository compatibility vectors before implementation.

The adapter does not autodetect protocols, open network listeners, or multiplex logs with protocol stdout.

## Framing and JSON-RPC

- Bound message bytes/depth/keys/arrays/strings before service dispatch.
- Reject duplicate keys, invalid JSON-RPC version/ID/method/params, nonfinite numbers, malformed Unicode, and unsupported batch behavior.
- Preserve request ID type/value as transport metadata.
- Unknown methods/tools/resources return exact protocol errors.
- Notifications receive no response unless required by the frozen profile.
- stdout contains MCP protocol messages only; diagnostics use stderr/telemetry.

## Initialization

```text
parse initialize
-> validate exact MCP revision/profile
-> acquire transport authentication and operation authorization context
-> normalize client capabilities
-> call service session_initialize once
-> compute static tool/resource capability intersection
-> emit initialize result
-> observe initialized lifecycle notification under profile
-> accept normal operations
```

A client name/version/capability cannot register tools, authorize operations, or change semantic behavior.

## Tools/list and resources/list

Lists are deterministic for exact implementation/profile/session authorization/capability state. They include only static descriptors permitted in the session.

- No source/provider/project-controlled descriptors.
- No hidden generic dispatch tool.
- Bounded output and continuation only if the exact MCP profile defines it.
- Empty authorized list is not global absence.
- Tool/resource descriptions are static repository-owned text.

## Tools/call

```text
validate static tool descriptor and closed arguments
-> validate session/authorization/privacy/budgets
-> construct exactly one service request
-> invoke exactly one service operation
-> validate result identity/status
-> project to MCP structured content
-> close resources/output
```

No adapter-side owner composition, retry, candidate selection, source read, or effect inference.

## Resources/read

```text
validate opaque exact resource URI
-> authorize exact artifact/result/source class
-> invoke one exact service/artifact read
-> validate retention/digest/privacy/license
-> project exact bounded content
```

Embedded URLs, file paths, provider locators, or source links are not followed.

## Results

Structured results preserve the service envelope fields required for trust:

```text
status
exact session/view/overlay/artifact/result IDs
provenance/confidence/candidate state
coverage/conflicts/blockers/omissions
partial/truncated/NotEvaluated/OutcomeUnknown
privacy/license/source-boundary state
nonclaims
```

Text content is a static projection only. Source/provider text remains structurally marked untrusted data.

## Error mapping

Malformed protocol/tool/resource/schema requests use MCP/JSON-RPC errors with no service call. A valid service operation returning `CandidateOnly`, `Blocked`, `Partial`, `NotEvaluated`, or authorization denial remains a structured domain result according to the frozen projection profile.

Stable service error codes are retained in structured data where allowed. Secrets/private roots/source bodies/raw owner handles are excluded.

## Cancellation

Protocol cancellation maps exact request ID to one service `operation_cancel` call. The result reflects actual state, including completed, cancelled, committed effect, stale request, unsupported cancellation, or `OutcomeUnknown`.

No automatic retry or repeated provider/owner effect.

## Progress

Progress tokens are bounded transport metadata. Service progress records map mechanically with monotonic sequence/stage. Progress cannot reveal hidden source/holdout/review/private-provider content beyond the authorization profile.

100% progress is not final success before the terminal result and closure.

## Partial results and continuation

A tool can return an exact continuation handle only when the mapped service operation supports it. The handle binds original session/view/generation/request/profile/budget/authorization/privacy state. A later call cannot reset budget, refresh current, change provider generation, or widen disclosure.

MCP pagination/listing behavior is separately profiled; arbitrary client cursor data is not passed to lower owners.

## Shutdown and transport loss

The exact MCP lifecycle/profile defines graceful shutdown/close behavior. On EOF/broken pipe/client disconnect:

- stop admitting requests;
- cancel/reconcile active operations;
- do not blind-retry effects;
- close resources/session synchronously;
- retain committed artifacts under real state;
- emit no malformed mixed protocol/log output;
- perform no source save/edit/publication by convenience;
- leave no detached work.

## Backpressure

Bound inbound/outbound queues, active calls, progress messages, listed resources/tools, result bytes, and source/context payloads. Slow clients produce backpressure/typed close, not unbounded buffering or silent response loss.

## Determinism

For identical exact service result/profile, structured result/resource bytes are deterministic independent of request ID, client name/version, progress cadence, queue scheduling, host, or cache history. Transport request IDs remain outside semantic result identity.
