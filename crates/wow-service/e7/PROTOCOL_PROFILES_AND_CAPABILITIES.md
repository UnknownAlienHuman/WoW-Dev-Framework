# E7-A protocol profiles and capability negotiation

**Status:** normative.

## Exact external specification pins

Before implementation, each protocol profile freezes:

```text
protocol kind: LSP | MCP
external specification revision/date/digest
JSON-RPC revision and interpretation profile
transport and framing profile
request-ID profile
lifecycle and ordering profile
cancellation/progress/partial-result profile
protocol error-code profile
URI/document/position profile where applicable
capability and experimental-field schema
conformance corpus and expected wire bytes
```

`latest`, a documentation landing page, or package version alone is not an exact profile.

## Initial E7-A profiles

```text
lsp-stdio-profile-v1
mcp-stdio-profile-v1
```

These are repository profile names, not claims about external spec versions. Their exact pins remain null until the implementation freeze gate.

## Separate framing

LSP and MCP framing parsers are independent. The profile defines exact:

- message boundaries;
- header or line rules;
- UTF-8/encoding behavior;
- maximum message/header/line bytes;
- empty/whitespace behavior;
- duplicate field/header handling;
- malformed/partial EOF behavior;
- stdout/stderr rules.

A byte stream valid for one profile is never assumed valid for the other.

## Server registry

The server starts with one immutable `AllowedServiceOperationRegistry`. Entries are repository-owned reviewed data compiled or loaded by exact digest. The client cannot add, rename, replace, or remove entries.

Registry validation checks:

- unique protocol method/tool name and version;
- unique exact service operation target;
- closed request/result schema;
- lifecycle/session/binding applicability;
- effect and domain operation identity requirements;
- cancellation/progress support;
- privacy/source/tool-permission limits;
- authority ceiling/nonclaims;
- protocol result/error mapping;
- no forbidden edit/command/provider/release effect;
- all referenced service operations implemented/frozen.

## Client capability declaration

Client capabilities are untrusted bounded data. Unknown fields are handled only by the exact profile:

```text
Reject
PreserveAsUnsupportedMetadata
IgnoreWithExplicitReceipt
```

Silent interpretation or dynamic execution is forbidden.

## Negotiation

```text
validate server protocol/profile/registry
-> validate client protocol revision and capabilities
-> compute exact intersection under profile rules
-> select one framing/request-ID/position/sync/progress/cancellation profile
-> record unsupported/disabled/conflicting capabilities
-> build immutable NegotiatedCapabilitySet
```

There is no “best effort” fallback to another spec revision without a new initialization/profile.

## Capability meanings

A negotiated capability means only that the transport/server/client can exchange that operation under the exact protocol schema. It does not imply:

- project/reference/source capability coverage;
- runtime WoW behavior;
- edit/tool/network/process authorization;
- provider credentials;
- model access;
- correctness of a Candidate/finding/context;
- dynamic registration permission.

## LSP initial capability boundary

The initial LSP application may advertise only entries frozen in its contract, including lifecycle, document synchronization, cancellation/progress, diagnostics, bounded hover/context, and bounded workspace symbol/search projections where implemented.

Explicitly disabled in E7-A:

```text
workspace/applyEdit
workspace/executeCommand
textDocument/rename
textDocument/codeAction
textDocument/formatting/rangeFormatting/onTypeFormatting
edit-producing completion/resolve behavior
server-initiated editor-settings mutation
dynamic client/registerCapability and unregisterCapability
arbitrary experimental method dispatch
```

A client advertising support does not enable these features.

## MCP initial capability boundary

The initial MCP application may expose:

```text
initialize lifecycle
ping
fixed tools/list
fixed allow-listed tools/call
cancellation/progress supported by the pinned profile
```

Disabled in E7-A unless a later contract activates them:

```text
prompts/list or prompts/get
resources/list/read/subscribe
dynamic tool registration
sampling/model invocation
roots mutation or arbitrary root discovery
elicitation
arbitrary completion
server-to-client tool calls
generic passthrough methods
```

The exact names/messages are determined by the pinned MCP revision; the service registry remains authoritative.

## Dynamic registration

No dynamic registration exists in E7-A. A registry/profile change requires:

- a new immutable registry ID;
- capability compatibility validation;
- a new protocol session;
- updated fixtures/tests/checksums.

## Experimental fields

Experimental capability fields are rejected by default. An allow-listed experimental extension requires:

```text
exact namespace/name/version/schema
owner and security review
no authority/permission upgrade
request/result/error/wire vectors
compatibility and downgrade behavior
```

Source/client-provided names cannot create extensions.

## Version mismatch

Outcomes:

```text
CompatibleExact
CompatibleUnderExplicitProfile
UnsupportedVersion
CapabilityConflict
NotEvaluated
Failed
```

No request is admitted until initialization reaches a compatible immutable set.

## Capability fingerprint

The negotiated set receives a canonical digest over exact server registry, client declaration, selected profiles, disabled features, and compatibility report. Client ordering and unknown ignored metadata do not alter enabled semantics unless the profile says they are canonical.

## Tests

- exact compatible LSP/MCP profiles;
- version mismatch/downgrade attempt;
- duplicate/unknown capabilities;
- dynamic registration attempt;
- client advertises forbidden edits/commands/sampling;
- arbitrary experimental method/tool;
- same capability set in shuffled input order;
- LSP bytes interpreted by MCP framing and vice versa;
- capability support misrepresented as analysis/tool authority.
