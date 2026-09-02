# E7-A frontend security and configuration

**Status:** normative.

## Configuration sources

The application reads only explicitly supplied config plus release-profile-defined current-user defaults that are documented and nonsemantic. It does not search cwd, parent directories, Git configuration, editors, WoW installations, addon folders, environment variables, registries, browser storage, or provider installations to discover semantic configuration.

Configuration is strict bounded JSON. Unknown fields fail. Includes, interpolation, environment expansion, shell substitution, executable plugins, scripts, arbitrary RPC fragments, and model prompts are forbidden.

Environment variables may be used only by an explicit host adapter for sensitive material without exposing values to application/service canonical data. The variable name/source is fixed by the release configuration, not user-source text or arbitrary config.

## Sensitive material

Ordinary argv/config/request/fixtures never contain:

```text
provider/signing/deployment credentials
passwords/tokens/cookies/private keys
MCP/daemon session secrets
private endpoints
process/socket/client handles
provider database paths
unrestricted source bodies
```

Host adapters return nonsecret authorization/session receipts. The app never prints sensitive values or includes them in errors, telemetry, crash data, resource URIs, or progress.

## Local daemon endpoints

- Windows named pipe: exact current-user SID ACL and anti-substitution validation.
- Unix-domain socket: explicit current-user runtime directory and mode 0600.
- Reject symlink/junction/reparse/device/ownership/permission mismatch.
- No TCP, wildcard, public listener, LAN discovery, or auto-port scan.

Administrative `status`/`shutdown` verifies daemon identity and peer/session profile before action. OS peer identity is not authorization for domain effects.

## LSP trust boundary

Workspace folders, URIs, configuration, watched-file events, document content/versions/ranges, commands, and client metadata are untrusted. The app forwards typed inputs to E7-A service; it does not open arbitrary paths or apply edits itself.

No `workspace/executeCommand` escape hatch, editor-setting mutation, automatic `workspace/applyEdit`, shell launch, or extension install.

## MCP trust boundary

Tool/resource names are fixed by the negotiated registry. Model/client text and annotations cannot add tools, widen schemas, authorize effects, register roots, or expose source. Initial profile has no prompts, sampling, elicitation, tasks, or effecting tools.

The optional local HTTP profile validates loopback binding, Origin, session authentication, protocol headers, content type, replay scope, and bounded SSE queues. Session secrets never appear in URLs.

## Path and URI safety

Workspace/file URIs are converted under exact platform profiles. Reject traversal, root escape, malformed percent encoding, ambiguous normalization/case collisions, unsupported schemes, device paths, forbidden UNC, NTFS alternate data streams, and symlink/reparse escapes. `wow://` resource URIs contain exact typed IDs only.

## Multi-client isolation

Connection/session IDs scope workspaces, overlays, authorization, private source, provider access, active operations, progress tokens, results, resource visibility, and response journals. Cross-session access requires a separate exact immutable-artifact sharing policy.

## Resource limits

Bound argv/config, protocol headers/frames, JSON depth/strings/arrays, sessions/connections, workspaces/documents, overlay bytes/change count, active operations, progress/log queues, result/source/resource bytes, response journals, reconnect attempts, HTTP/SSE lifetime, and shutdown time.

Malformed or abusive clients receive bounded errors/throttling/session closure without affecting other sessions or creating unbounded work.

## No execution surface

The frontend exposes no shell, command execution, raw process launch, arbitrary filesystem read/write, raw SQL, generic MCP/RPC/tool call, plugin/script/model execution, provider database access, or source-controlled callback.

## Logging and crash data

Default logs contain stable IDs, operation stages, bounded counts, and reason codes. Source content, unsaved overlays, private paths, tool arguments containing source, credentials, session secrets, provider cursors, hidden evaluation data, and raw handles are redacted. Crash dumps are disabled or controlled by an explicit privacy profile.