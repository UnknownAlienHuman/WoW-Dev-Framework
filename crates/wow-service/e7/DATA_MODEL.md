# E7-A data model

**Status:** normative.

## Protocol profile

```text
FrontendProtocolProfile
    profile ID/version/digest
    transport kind: cli | local_daemon | lsp_stdio | mcp_stdio | mcp_streamable_http_local
    wire protocol/revision
    framing and canonicalization profile
    request/response/progress/error limits
    lifecycle and timeout policy
    security/authentication/peer policy
    supported position encodings where applicable
    operation registry ID/digest
```

## Operation descriptor

```text
FrontendOperationDescriptor
    descriptor ID/version
    stable service operation ID
    request/result/error schema IDs and digests
    required owner capabilities/profiles
    effect class:
        PureRead
        SessionLocalMutation
        DurableLocalEffect
        ExternalEffect
    idempotency/operation-ID requirements
    authorization scopes
    privacy/license/consumer restrictions
    continuation/cancellation/progress support
    allowed frontend profiles and method/tool/resource names
    output classes and nonclaims
    canonical digest
```

## Registry

```text
FrontendOperationRegistry
    registry ID/version
    exact descriptor set and deterministic order
    service implementation/capability manifest
    protocol projection profiles
    disabled/experimental entries and reasons
    canonical digest
```

## Client and session

```text
FrontendClientIdentity
    transport-scoped client ID
    implementation name/version
    protocol version/capabilities
    consumer/privacy/license profile
    peer/process/user evidence permitted by profile
    no semantic authorization by identity alone

FrontendSession
    session ID/version
    protocol profile and negotiated capabilities
    client identity
    exact operation registry
    workspace registrations
    document overlay heads
    active/durable operation tickets
    leases/retention/output policy
    state: Opening | Ready | Draining | Closing | Closed | Failed
    canonical digest
```

## Workspace registration

```text
FrontendWorkspaceRegistration
    registration ID
    session/consumer
    explicit root URI/path
    normalized platform path identity
    source/project profile selector and exact resolution receipt
    project publication/generation when available
    trust/privacy/license/access policy
    include/exclude boundaries
    watcher/input policy
    state and canonical digest
```

A root URI is transport input, not proof that the path exists, is trusted, or is an addon.

## Document overlay

```text
FrontendDocumentOverlay
    overlay snapshot ID
    session/workspace/document URI
    canonical owner-relative path
    language/profile
    negotiated position encoding
    client document version
    exact prior overlay snapshot
    full content digest/length/line index digest
    applied change manifest
    persisted project/source base identity
    saved/dirty/conflict state
    privacy/retention policy
    canonical digest
```

The canonical owner representation uses exact UTF-8 bytes and byte ranges. Transport positions are projections validated against this snapshot.

## Operation ticket

```text
FrontendOperationTicket
    transport request ID
    session ID
    operation descriptor
    exact service request ID
    optional durable OperationId + CanonicalRequestDigest
    workspace/document/overlay/generation bindings
    authorization and budget profiles
    progress token/stream binding
    cancellation state
    owner dispatch/result/reconciliation refs
    delivery state
    canonical digest
```

## Progress

```text
FrontendProgressEvent
    session/operation/progress token
    monotonic event sequence within stream
    stage and bounded counters
    optional message under redaction policy
    nonauthoritative = true
    coalescible class
```

Progress excludes source, credentials, hidden holdout/cohort data, private provider fields, and owner handles by default.

## Delivery journal

```text
FrontendResponseJournalEntry
    exact operation/service result or error digest
    transport response ID
    prepared/sent/acknowledged/connection-lost state
    replay eligibility and consumer scope
    durable result lookup/reconciliation reference
    expiry/retention/close state
```

A journal records delivery, not semantic completion.

## LSP projection

```text
LspSessionProjection
    LSP version = 3.18
    negotiated position encoding
    text synchronization mode
    advertised capability set
    workspace/document registrations
    diagnostic result IDs
    work-done/partial-result tokens
    request-to-operation mapping registry
```

## MCP projection

```text
McpSessionProjection
    protocol revision = 2025-11-25
    transport: stdio | local Streamable HTTP
    negotiated capabilities
    fixed tool/resource registry projection
    MCP session ID where applicable
    request/progress/cancellation mappings
    structured output schema refs
    authorization/consent profile
```

## Exact resource URI

```text
wow://profiles/<profile-id>
wow://projects/<project-generation-id>/map
wow://results/<result-id>
wow://context/<context-artifact-id>
wow://publications/<publication-id>
```

No exact resource URI contains an unresolved `current`, `latest`, `best`, filesystem traversal, credential, or private endpoint.

## Status axes

Keep protocol, session, workspace, document/overlay, operation, owner result, durable effect, delivery, privacy/license, and close states separate.