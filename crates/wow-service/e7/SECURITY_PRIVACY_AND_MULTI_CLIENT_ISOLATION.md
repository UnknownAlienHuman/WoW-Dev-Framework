# E7-A security, privacy, and multi-client isolation

**Status:** normative.

## Untrusted inputs

Treat protocol frames, client metadata, workspace roots, document text/versions/ranges, configuration, LSP/MCP capability objects, tool arguments, resource URIs, progress/cancellation tokens, daemon endpoints, and reconnect data as untrusted bounded inputs.

## Default network posture

```text
CLI:        no listener
LSP:        stdio only
MCP:        stdio only
local daemon: named pipe or Unix-domain socket, current-user access
MCP HTTP:   disabled unless explicit local-only profile
remote HTTP/TCP: unsupported in E7-A
```

No default port is opened.

## Workspace trust

A workspace root is never trusted because an editor or MCP client supplied it. Registration validates path policy and project access. The framework does not execute workspace hooks, tasks, binaries, package managers, Lua, XML scripts, generated code, or repository tools.

Path safety includes exact normalization and platform-specific rejection/handling of traversal, symlink/junction/reparse escape, device paths, UNC policy, NTFS alternate streams, case/normalization collisions, and root changes.

## Session isolation

Each session has independent:

```text
client/consumer identity
operation registry and protocol profile
workspace registrations and access roots
document overlays and unsaved bytes
authorization scopes
active/durable operation visibility
provider/session access
privacy/license/output profile
progress/log/response queues
leases and close state
```

No session can enumerate another session's private workspaces, overlays, source, provider data, results, operation IDs, progress tokens, or response journals unless a separate explicit sharing policy authorizes exact immutable artifacts.

## Secret and credential isolation

Transport/application requests never contain private signing/provider/deployment credentials, tokens, passwords, cookies, private endpoints, raw OS handles, or unrestricted environment blocks. Host adapters own secret access and return nonsecret receipts.

MCP/LSP initialization metadata, tool annotations, daemon peer identity, OS user, Git identity, editor identity, and process ancestry are not semantic or effect authorization.

## Source privacy

Unsaved source is session-private and memory-only by default. Saved source, context L2, provider snippets, diagnostics, logs, crash reports, resources, and response journals each obey source-owner/consumer/privacy/license policy.

No full source body appears in progress or default error output. Resource URIs are opaque stable identifiers rather than filesystem paths when source disclosure is restricted.

## MCP trust boundary

MCP tool descriptions and annotations are server-owned but remain descriptive; the client/model cannot widen them. Model-provided arguments are untrusted. Default tools are read-only. The server makes no sampling/elicitation request and never treats model intent as user consent.

Local Streamable HTTP validates Origin, loopback binding, session authentication, request size/content type, and replay scope. DNS rebinding and cross-session replay tests are mandatory.

## LSP trust boundary

Editor configuration, commands, workspace folders, watched-file events, and text changes cannot execute code or bypass service validation. The server never sends arbitrary `workspace/executeCommand`, modifies editor settings, or automatically applies edits in the initial profile.

## Resource limits

Bound frames, headers, JSON depth, strings, arrays, workspaces, documents, overlay bytes, change count/ranges, active operations, progress/log queues, source/resource bytes, search/context results, sessions, connections, reconnect attempts, journals, and shutdown time.

Overflow/unlimited values fail. Truncation is explicit and cannot become complete or clean negative.

## Logging and crash data

Default logs contain stable IDs, operation stages, counts, statuses, durations under operational policy, and bounded reason codes. They exclude document/source content, paths when private, credentials, client secrets, MCP arguments containing source, provider cursor bytes, hidden holdout/cohort data, and raw owner handles.

Crash dumps are disabled or privacy-profile-controlled. A crash does not authorize uploading source or secrets.

## Denial of service and abusive clients

Malformed framing, repeated invalid requests, oversized changes, cancellation storms, progress-token collisions, reconnect abuse, resource enumeration, and expensive-request floods trigger bounded errors, throttling, session closure, and audit—not unbounded allocation or global service failure.

## No generic execution

No frontend exposes shell, process execution, arbitrary filesystem read/write, raw SQL, arbitrary MCP/RPC, plugin/script/model execution, editor command execution, or provider database access. Any future effecting tool requires its own service contract and authorization.