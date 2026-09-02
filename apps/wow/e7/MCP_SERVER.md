# E7-A standalone MCP stdio server

**Status:** normative baseline MCP mapping.

## Invocation

```text
wow mcp serve --stdio --config <PATH>
```

The server hosts one `wow-service` runtime in-process for one stdio client. It does not auto-start/connect/fall back to the local daemon, discover other MCP servers/tools, or listen on a network endpoint.

The exact MCP protocol version, JSON-RPC/framing behavior, capability profile, tool/resource schemas and limits are pinned by the build compatibility manifest. `latest` is not accepted as a durable protocol profile.

## Initialization

Initialization negotiates only the exact intersection of:

```text
pinned MCP protocol profile
server compatibility manifest and operation registry
compiled transport/tool/resource mappings
host exposure/security/privacy/resource profile
client capabilities
```

The server returns stable implementation/build/registry identifiers and only capabilities it implements. Initialization is transport lifecycle and does not create project/source authority.

One `EphemeralStdioSession` is opened for the client after successful initialization.

## Baseline capabilities

Enabled:

```text
tools: static read-only developer tool registry
resources: exact retained resource reads and templates with exact IDs
logging/progress: bounded nonsemantic notifications where supported
```

Disabled unless a later exact profile explicitly adds them:

```text
prompts
sampling
elicitation
server-initiated model/tool calls
arbitrary completion APIs
generic service invocation
administrative effect tools
```

## Static tool registry

Each MCP tool has a fixed name, description, strict JSON schema, service operation ID/version, effect/authorization class, limits and result schema digest.

Baseline read-only tools may include implemented entries such as:

```text
wow_status
wow_check
wow_editor_hover
wow_editor_definition
wow_editor_references
wow_document_symbols
wow_workspace_symbols
wow_search
wow_lineage
wow_migration_validate
wow_static_impact
wow_context_build
wow_context_continue
wow_context_validate
wow_artifact_get
```

The exact list is generated from the approved `mcp-readonly-developer-v1` exposure profile. Missing implementations are absent, not stubs.

There is no:

```text
wow_invoke
wow_call_service
call_tool
execute
shell
sql
run_command
arbitrary_operation
```

Tool names are not accepted from client/source/provider text.

## Effecting tools

The baseline profile does not expose:

```text
calibration/review/holdout operations
core-pack signing/publication/activation/rollback
external provider query/session operations
release/build/publish/update operations
source-edit application
```

A later administrative MCP profile must be explicitly enabled at build/host/session levels and still requires exact service authorization, `OperationId + CanonicalRequestDigest`, response-loss reconciliation, privacy/license and audit. Transport access is not authorization.

## Project and profile registration

MCP roots are untrusted registration candidates. They do not automatically become project roots or source authority.

Project/profile/session binding uses static tools only when enabled:

```text
wow_session_project_bind
wow_session_profile_bind
wow_session_snapshot_get
```

These map exactly to service operations and require explicit strict arguments. The baseline may alternatively require project/profile binding in the explicit host config before tools are usable.

No cwd/home/Git/editor/WoW/provider discovery.

## Tool-call mapping

Each admitted `tools/call` invokes exactly one registered service operation. The server validates:

```text
tool name/version/schema digest
session/generation and exposure profile
argument JSON and limits
required operation ID/idempotency data
privacy/license/authorization scope
```

It passes one typed request to service. It does not compose search + context, query + mapping, or validation + effect locally; such workflows must already exist as one documented service operation.

## Tool results

Tool results preserve one canonical service envelope or a bounded protocol projection with an exact envelope/artifact reference and digest.

Text blocks are faithful and must preserve:

```text
exact profile/generation/session IDs
Candidate versus exact authority
coverage/conflict/partial/truncated/NotEvaluated
zero-result nonclaims
mapping/selection/context sidecar separation
OutcomeUnknown and unsafe-to-retry
privacy/license omissions
```

Structured content follows the exact result schema. The server never turns service failure into a successful text explanation.

## Resources

Resources expose exact retained artifacts only. A resource URI binds exact kind/ID/digest/schema/consumer profile; examples include exact project maps, context artifacts, diagnostic result envelopes, search results and manifests.

There is no floating `current`, `latest`, `best`, path traversal, raw database, arbitrary file, provider URL, or private source resource.

Resource templates are static and contain exact required identifiers. Reading a resource maps to one exact service artifact/get/stream operation.

Large resources use service-owned artifact streams. The MCP adapter does not broaden disclosure or reserialize canonical bytes.

## Roots

Client-provided roots are transport metadata only. They may be submitted through an explicit project-binding tool/request, where service/project owners validate them. A root does not grant source access, create a project, choose a profile, or authorize tools.

## Progress, logging, and cancellation

Progress notifications are bounded nonsemantic service progress. They do not constitute a tool result. Logging uses stable IDs/stages/status codes and excludes source, secrets, private paths, raw payloads and owner handles.

Cancellation identifies exact MCP request/service operation/session generation. Disconnect is not proof of no effect. Effecting requests, if ever enabled, reconcile by exact operation identity and may return `OutcomeUnknown`.

## Security

The server rejects:

```text
unknown tools/fields/schema versions
batch semantic requests in baseline
oversized/deep/duplicate-key JSON
raw SQL/RPC/MCP forwarding
shell/script/plugin/model prompt input
secret credentials/private endpoints/provider databases
arbitrary resource/file/URL access
source/provider text as tool instructions
cross-session resource/progress/cancel references
```

The server never calls client tools/models, requests sampling, or interprets resource text as instructions.

## Shutdown

Protocol shutdown/stdio close stops admission, cancels/drains according to profile, closes service session/resources synchronously and terminates. Abrupt disconnect preserves/reconciles effect state and never starts detached work.

## Conformance

Required tests cover initialization/version/capability intersection, exact static tool schemas, every tool-to-service mapping, absent admin/generic tools, resource URI exactness, roots-as-candidates only, structured result fidelity, Candidate/coverage/nonclaim preservation, artifact streaming, progress/cancellation, malformed/hostile messages, cross-session isolation, abrupt disconnect and canonical equivalence with direct service/CLI requests.