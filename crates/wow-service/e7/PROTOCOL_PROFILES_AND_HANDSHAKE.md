# E7-A protocol profiles, handshake, and capability negotiation

**Status:** normative.

## External specifications

E7-A implementations pin exact official protocol specifications and conformance vectors. Repository contracts define framework policy; they do not silently follow a mutable specification website.

```text
ProtocolProfile
    official specification revision/digest
    transport and framing
    JSON-RPC version and compatibility
    method/tool/resource allow-list
    initialization/capability rules
    cancellation/progress/error mapping
    message and resource limits
    source/privacy/authorization profile
```

Any spec revision or compatibility exception that changes observable behavior creates a new profile ID.

## Initial candidate profiles

```text
lsp-stdio-single-client-read-analysis-v1
mcp-stdio-single-client-fixed-tools-resources-v1
```

They remain documentation candidates until implementation probes and official-spec fixtures freeze.

Network listeners, remote HTTP transports, TLS, origin policy, multi-tenant authentication, session resumption across processes, and daemon discovery are deferred.

## Handshake sequence

```text
transport accepted
-> framing/profile identified from deployment configuration
-> bounded initialization request parsed
-> transport authentication receipt validated when configured
-> operation authorization context acquired
-> client capabilities normalized
-> server capabilities computed from implemented owner/service capabilities
-> exact protocol profile compatibility validated
-> session created in Initializing
-> optional explicit workspace bind performed
-> initialization response emitted
-> initialized/ready state entered according to protocol profile
```

No source/project/provider data controls the protocol profile or advertised tool/method set.

## Capability states

```text
ImplementedAndAvailable
ImplementedButUnavailableForCurrentSession
NotImplemented
DeniedByAuthorization
DeniedByPrivacyOrLicense
BlockedByMissingOwnerCapability
NotEvaluated
Failed
```

A server never advertises a capability merely because the protocol defines it. Runtime capability output is the intersection of:

```text
compiled implementation
frozen protocol profile
owner/service capability availability
authorization scope
privacy/license policy
session/workspace binding state
```

## LSP initialization

The adapter validates and normalizes:

- protocol/client capabilities used by the active profile;
- supported position encodings;
- workspace folder/root inputs under explicit policy;
- text document synchronization mode;
- diagnostic/navigation/code-action capabilities;
- cancellation/progress/partial-result support;
- initialization options under a closed schema.

Unknown initialization options fail or are ignored only where the frozen LSP profile explicitly requires forward-compatible ignoring. Ignored fields never alter service behavior.

Server capability output is deterministic for the same implementation/profile/owner capability set. Client name/version does not select special semantics.

## MCP initialization

The adapter validates and normalizes:

- protocol revision compatibility;
- client capabilities used by the active profile;
- server name/version as operational metadata;
- tools/resources/list-changed/progress/cancellation capabilities where supported;
- authorization context provided by deployment, not source content;
- request/notification schemas under the allow-list.

The E7-A server does not advertise sampling, elicitation, source-controlled prompts, arbitrary roots, dynamic executable tools, or generic operation passthrough.

## Version negotiation

- Exact supported revisions/profiles are enumerated.
- No “closest version” guessing.
- Unsupported version returns a protocol-compatible error and closes or remains inactive according to profile.
- Downgrade is allowed only to an explicitly implemented tested profile.
- The negotiated revision/profile is immutable for the session.
- Reconnect with another revision creates a new session.

## Authentication versus authorization

Transport authentication can identify the peer. `OperationAuthorizationPort` independently decides permitted workspace, source, owner, operation, artifact, privacy, and effect scopes.

```text
transport authenticated != operation authorized
client capability declared != operation authorized
editor workspace trusted != source/tool/edit authorized
MCP host approved != framework fact proven
```

## Static capability descriptors

LSP capabilities and MCP tools/resources are built from repository-owned descriptors that bind:

- stable ID and version;
- request/result schema;
- one service operation or immutable artifact class;
- side-effect class;
- authorization/privacy requirements;
- output limits;
- protocol projection/loss rules.

A source file, addon, provider, repository, client request, or model cannot register a new descriptor.

## Initialization failures

Classify separately:

```text
FramingFailure
MalformedInitialization
UnsupportedProtocolRevision
UnsupportedTransportProfile
CapabilityConflict
AuthenticationFailed
AuthorizationUnavailableOrDenied
WorkspaceBindingFailed
OwnerCapabilityUnavailable
ResourceLimitExceeded
InternalFailure
```

No owner resource or session operation remains open after failed initialization. No successful initialized response is emitted before mandatory acquisition and closure/retention gates.

## Shutdown and exit

- `session_shutdown` stops accepting new operations, cancels or drains according to profile, closes resources, and returns one terminal receipt.
- Protocol exit/transport close after shutdown releases application resources without a second service shutdown effect.
- Abrupt disconnect triggers bounded cancellation/reconciliation/closure; it does not blind-retry effecting operations.
- Shutdown does not publish project state, save documents, activate a pack, or terminate unrelated sessions.

## Conformance evidence

Before implementation activation, freeze:

- official spec revision references/digests;
- official and repository compatibility vectors;
- initialization request/response bytes;
- capability matrices;
- framing and error vectors;
- position encoding vectors;
- stdio behavior on supported platforms;
- unsupported-feature responses;
- cancellation/progress/partial-result vectors;
- message-size/depth/resource limits.
