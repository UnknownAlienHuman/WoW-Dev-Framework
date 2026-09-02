# E7-A protocol and session data model

**Status:** normative transport-independent model.

## Protocol profile bundle

```text
ProtocolProfileBundle
    bundle ID/version
    JSON-RPC specification profile
    LSP specification profile: optional
    MCP specification profile: optional
    transport/framing profile
    URI/workspace profile
    position-encoding profile
    document synchronization profile
    lifecycle profile
    capability registry profile
    request/notification/progress/cancellation profile
    protocol-error mapping profile
    privacy/permission/resource profile
    canonicalization and transcript profile
    canonical digest
```

A bundle activates exactly one protocol kind and transport profile.

## Protocol implementation profile

```text
ProtocolImplementationProfile
    implementation ID/version/commit
    supported protocol/profile bundle IDs
    JSON parser and framing implementation IDs
    maximum resource limits
    platform/stdio adapter ID
    conformance/golden/benchmark report IDs
    canonical digest
```

## Capability registry

```text
AllowedServiceOperationRegistry
    registry ID/version
    protocol kind/profile
    immutable ordered ProtocolOperationEntry[]
    disabled capability declarations
    schema bundle IDs
    privacy/permission defaults
    canonical digest
```

```text
ProtocolOperationEntry
    protocol method or MCP tool name/version
    protocol request/notification kind
    exact service operation ID/version
    input and output schema IDs/digests
    allowed session/binding/document states
    effect classification
    domain OperationId requirement
    cancellation/progress/continuation support
    authority ceiling and mandatory nonclaims
    privacy/source/credential/tool-permission profile
    timeout/resource maxima
    protocol error/result mapping profile
    canonical digest
```

No arbitrary callback or operation name.

## Client and server capabilities

```text
ClientCapabilityDeclaration
    protocol version/profile
    exact supported capability fields
    position encodings and document sync support where applicable
    progress/cancellation/partial-result support
    experimental fields under explicit allow-list
    unknown/ignored/unsupported field records
    canonical digest
```

```text
NegotiatedCapabilitySet
    exact client declaration
    exact server registry/profile
    enabled operation entries and lifecycle features
    selected position encoding/sync/framing/error profiles
    disabled/unsupported fields with reasons
    compatibility/coverage/conflict report
    canonical digest
```

Capabilities are immutable for one session.

## Protocol session

```text
ProtocolSession
    ProtocolSessionId
    protocol/profile/implementation/transport IDs
    client and server identity metadata under privacy profile
    negotiated capability set ID
    operation registry snapshot ID
    lifecycle state
    active SessionBindingGenerationId: optional
    request/notification/progress indexes
    durable operation/audit/transcript refs
    resource/retention/closure state
    canonical digest
```

Lifecycle states:

```text
Created
Initializing
InitializedAwaitingReadyNotification
ActiveUnbound
ActiveBound
ShuttingDown
Shutdown
Exiting
Exited
Failed
Quarantined
```

## Session binding generation

```text
SessionBindingGeneration
    SessionBindingGenerationId
    ProtocolSessionId
    sequence number
    exact expected prior binding ID/digest: optional
    workspace binding set
    primary project/profile/reference/source generation refs
    document overlay manifest
    server configuration/profile refs
    compatibility/capability/coverage/conflict state
    reason: Initialize | WorkspaceBind | Rebind | DocumentOpen | DocumentChange | DocumentSave | DocumentClose | ExplicitRefresh
    state: Candidate | Active | Superseded | Failed | Quarantined
    canonical digest
```

Only one binding is active for admission of future requests. Older bindings remain immutable for in-flight result/reconciliation.

## Workspace binding

```text
ProtocolWorkspaceBinding
    workspace binding ID
    normalized workspace URI/root identity
    workspace name metadata: optional
    exact project/source publication selectors resolved to exact IDs
    exact profile/reference/source generation compatibility
    permissions/privacy/license configuration
    document overlay namespace
    capability/coverage/conflict state
    canonical digest
```

No arbitrary host path escapes the configured root policy.

## Document overlay

```text
ProtocolDocumentOverlay
    document overlay ID/generation
    exact normalized document URI
    exact owning workspace binding
    language ID/profile
    monotonically validated document version
    negotiated position encoding
    exact content bytes/text digest and length
    origin: OpenFullText | ChangedFullText | ChangedIncremental | OwnerSourceBaseline
    applied change receipt IDs
    virtual/source-map refs where applicable
    privacy/license state
    canonical digest
```

Overlay text is session data, not a disk write.

## Document change

```text
ProtocolDocumentChangeRequest
    exact session and active binding
    exact document URI
    expected prior overlay/version/digest
    new document version
    ordered content changes
    position encoding and range-length policy
    resource limits
    canonical digest
```

```text
ProtocolDocumentChangeReceipt
    exact prior and target overlay/binding IDs
    validated ordered changes
    resulting content digest/length
    coordinate validation state
    target binding state
    canonical digest
```

## Protocol request

```text
ProtocolRequestRecord
    ProtocolRequestRecordId
    ProtocolSessionId
    protocol request ID preserving JSON type/value
    exact admitted SessionBindingGenerationId
    exact operation registry entry
    normalized params ID/digest
    optional domain OperationId + CanonicalRequestDigest
    admission sequence
    lifecycle/effect/cancellation/progress state
    service request/result/error refs
    response delivery/reconciliation state
    resource/retention/closure refs
    canonical digest
```

States:

```text
Admitted
Dispatching
InProgress
Cancelling
DomainCompleted
DomainOutcomeUnknown
ProtocolResponsePrepared
ResponseDelivered
ResponseDeliveryUnknown
Cancelled
Failed
Closed
```

## Notification receipt

```text
ProtocolNotificationReceipt
    receipt ID
    session and binding IDs
    method/kind
    sequence
    normalized params digest
    lifecycle/document/progress/cancel target refs
    processing status
    domain effect receipt when applicable
    canonical digest
```

No protocol response is emitted for a notification.

## Cancellation

```text
ProtocolCancellationRecord
    session ID
    exact target protocol request ID/record
    cancellation notification receipt
    requested/effective state
    owner cancellation receipt
    effect uncertainty and domain result state
    canonical digest
```

## Progress

```text
ProtocolProgressRecord
    session/request/binding IDs
    exact progress token and profile
    sequence
    begin/report/end kind
    bounded typed progress payload
    domain progress source receipt
    delivery state
    canonical digest
```

Progress does not enter domain result identity.

## Protocol response

```text
ProtocolResponseRecord
    session/request/binding IDs
    JSON-RPC response ID preserving request ID type/value
    result or error tagged union
    exact domain envelope/error ref
    authority/nonclaim preservation report
    canonical JSON payload digest
    framed bytes digest/length
    delivery state
    canonical digest
```

## Transcript

```text
ProtocolTranscriptManifest
    session/profile/registry/capability IDs
    ordered initialize/lifecycle/binding/document/request/notification/progress/response/close record IDs
    domain operation/effect reconciliation refs
    privacy/redaction report
    protocol conformance/closure report
    canonical digest
```

Transcript semantic identity excludes incidental timing, process/host, raw credentials, raw connection handles, and forbidden source content.

## Session result

```text
ProtocolSessionResult
    exact session and final binding state
    request/notification/progress/response counts and manifests
    unresolved/OutcomeUnknown requests
    document overlay closure
    shutdown/exit/transport EOF state
    retention/audit/transcript/closure reports
    canonical digest
```

## Canonical ordering

```text
registry entries by protocol name/version
workspace bindings by normalized exact URI/key
document overlays by workspace + normalized URI
binding generations by sequence + ID
requests/notifications by admission sequence + typed request ID
progress by request + token + sequence
transcript records by session sequence + record ID
```

Transport arrival order is recorded but cannot silently repair invalid protocol order.
