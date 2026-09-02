# E7-A security, multi-client isolation, and credentials

**Status:** normative.

## Threat model

Untrusted inputs include CLI arguments/config, local daemon frames, LSP/MCP messages, workspace folders, document URIs/text/versions, MCP roots/resources/tool arguments, client metadata, progress labels, stored session records, and all source/provider text.

The host and service assume malformed, oversized, stale, replayed, cross-session, injection-bearing, or deliberately ambiguous input.

## Transport boundary

Baseline supported transports are:

```text
one-shot local CLI
explicit local named pipe on Windows
explicit Unix domain socket on Unix-like systems
stdio LSP
stdio MCP
```

No TCP, HTTP, WebSocket, browser origin, remote host, port discovery, multicast, or public listener is enabled by the baseline E7-A profile.

A later remote profile requires a separate ADR, authentication/encryption/threat model, replay protection, rate limits, audit, and release compatibility contract.

## Local endpoint policy

The daemon endpoint is created under an explicit data-root/endpoint profile with restrictive owner permissions. The host validates peer transport scope where the platform supports it.

Peer identity is used only to admit a transport connection and isolate clients. It is not service authorization and cannot grant source access, provider use, review/signing/publication, activation, edits, or release effects.

Endpoint paths, pipe names, process IDs, user names, and host names are operational data and never semantic identities.

## Session capability material

Reconnectable daemon sessions may use an opaque high-entropy transport capability bound to one session/client/host profile. It is:

```text
kept in memory or an OS-protected local transport store
never committed to repository/config examples
never logged or returned in semantic envelopes
redacted from errors/audit views
rotated/revoked on exact session events
not accepted as service authorization
```

Stdio sessions do not need a reconnect token.

## Multi-client isolation

Every request/resource is scoped by exact:

```text
host compatibility profile
client/tenant ID
SessionId and SessionGenerationId
project/profile/data-root bindings
overlay generations
authorization and consumer profiles
privacy/license policy
in-flight/queue/stream budgets
```

Cross-session project aliases, overlays, operation receipts, progress tokens, artifact streams, cancellation IDs, or retained outputs are rejected unless an explicit share/grant artifact exists. Baseline profile defines no sharing.

## Sensitive material

The E7-A host/service public surface never accepts or exposes raw:

```text
provider API tokens, cookies, passwords or private endpoints
signing keys, KMS/HSM/vault credentials or recovery material
GitHub tokens or SSH keys
arbitrary environment blocks
process/socket/client handles
provider/store database paths or SQL
private cohort/holdout material
unrestricted source trees or SavedVariables/log data
```

Narrow adapters may use sensitive material outside canonical requests/results. The service receives only nonsecret references and authorization/effect receipts.

## Configuration

Configuration is explicit, strict, versioned, bounded, and separated into:

```text
public host configuration
sensitive adapter configuration outside repository and normal CLI payloads
project registration requests
transport compatibility/exposure profiles
```

No implicit cwd/home/environment/registry/editor/Git/WoW/provider discovery. Unknown fields fail. No include, interpolation, command substitution, template execution, script, plugin, dynamic library, arbitrary URL, or remote configuration fetch.

## URI/path handling

Transports parse URIs as bounded identifiers. Only owner crates resolve approved file URIs/paths into exact project/reference records. App/service never follow provider locators or open arbitrary client-supplied paths.

Explicit config/data/output paths use platform path policy and reject forbidden traversal, symlink/reparse/device/UNC/ADS or cross-root behavior as selected by the exact platform profile.

## Message security

- strict framing and maximum frame size;
- strict JSON and duplicate-key rejection;
- bounded nesting, strings, arrays, maps, numeric ranges and batch size;
- exact protocol/schema/method/tool versions;
- no unknown method fallback;
- no deserialization into executable objects;
- canonical output escaping;
- bounded errors without raw payload reflection;
- replay checks for effecting requests;
- per-client/session rate and resource limits.

Batch semantic requests are not accepted in the baseline local-daemon/MCP profile. Each request maps to one service operation.

## Injection isolation

Document/source/provider/client text remains data. It cannot:

```text
create or rename tools/methods
select another service operation
change project/profile/owner generation
set authorization or exposure profile
open files/URLs/processes
execute shell/scripts/models/plugins
alter output path
become system/agent instructions
```

Structural typed separation is required; lexical prompt-injection detection is not a correctness mechanism.

## LSP workspace edits

Code actions may return exact guarded edits as data. The host never sends `workspace/applyEdit`, runs commands, saves files, or writes source automatically in the baseline profile. The editor/user owns applying an advertised edit and must satisfy version/digest guards.

## MCP safety

Baseline MCP profile:

```text
static tool registry
read-only exact resources
no prompts capability
no sampling
no elicitation
no arbitrary roots-to-project trust
no generic execute/invoke tool
no server-initiated model/tool request
```

Effecting tools, if a later explicit admin exposure profile enables them, still require exact service authorization and idempotency.

## Logging and telemetry

Default logs include stable operation/session/build/profile IDs, stages, bounded counts, status and reason codes. They exclude source text, document bodies, snippets, secrets, endpoint capabilities, private paths, raw client payloads, authorization envelopes, and owner handles.

Telemetry is disabled unless an explicit privacy-reviewed release profile defines collection, consent, retention, schema and deletion. E7-A does not silently add network telemetry.

## Resource abuse

Bound connections, sessions, projects, overlays, open documents, content bytes, incremental edits, owner calls, graph/search/context budgets, queued requests, progress, streams, logs and shutdown time. Resource exhaustion yields explicit errors/cancellation, never silent data mixing or unbounded allocation.

## Security tests

Required tests include malformed framing, duplicate keys, oversized/deep/batch messages, method/tool confusion, cross-session replay, stale generation, URI/path attacks, document-version attacks, progress/stream substitution, credential reflection, source prompt/control injection, output-path attacks, slow reader/backpressure, abrupt disconnect, and crash recovery.