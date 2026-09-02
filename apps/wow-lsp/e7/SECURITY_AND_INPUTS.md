# E7-A LSP security and explicit-input contract

**Status:** normative.

## Input sources

The adapter reads only:

- its configured stdio transport;
- deployment-provided nonsecret protocol/authorization port configuration;
- explicit initialization/workspace/document/request messages permitted by the frozen profile.

It does not discover cwd, home, environment, registry, Git, editor configuration files, addon directories, WoW installations, SavedVariables, logs, networks, or credential stores for semantic behavior.

## Message validation

Validate framing before allocation and JSON before service dispatch. Reject oversized headers/bodies, duplicate keys, deep structures, invalid numbers/Unicode, unknown methods, invalid IDs, unauthorized URIs, huge capabilities, and unsupported batches.

One malformed message cannot be interpreted as another method or source content.

## Workspace and URI boundary

- Initialization/workspace folders are explicit transport inputs.
- Service maps/authorizes roots; app never opens them directly.
- Reject unsupported schemes, traversal, NUL/control, device, UNC, alternate data stream, reparse/symlink escape, and normalization collision according to platform profile.
- Protocol output uses authorized URI projections and cannot leak private absolute paths under a restrictive privacy profile.

## Document content

Document text is untrusted bounded data. It cannot:

- define capabilities or profiles;
- register methods;
- authorize operations;
- inject JSON-RPC/LSP frames;
- control logs/output paths;
- execute Lua/XML/scripts/plugins/shell/model/tool calls;
- change diagnostic severity or evidence authority.

Incremental ranges and versions are validated exactly. No archive/decompression or source execution.

## Client identity and trust

Client name/version, initialization options, workspace trust, OS user, process/pipe owner, GitHub account, file owner, and commit author are not authorization. The app passes the configured authorization context to service and cannot widen it.

## No edit/command surface

Initial E7-A rejects or does not advertise:

```text
workspace/applyEdit
workspace/executeClientCommand or arbitrary client command patterns
workspace/executeCommand
textDocument/rename
textDocument/formatting
file create/rename/delete operations
settings/configuration mutation
```

Advisory code actions contain no edit/command.

## No external execution

No raw SQL, filesystem scanner, network, subprocess, shell, plugin, Wasm/native extension, model, embedding, reranker, MCP/provider tool, browser, or WoW client access.

## Resource limits

Bound frames/messages, initialization fields, workspaces, documents, document bytes/lines/changes, positions, requests, queues, progress, partial results, response bytes, diagnostics/references/symbols/actions, and shutdown work.

A client cannot request unlimited work. Saturation yields exact backpressure/busy behavior, not unbounded memory.

## Session isolation

Every message after initialization is scoped to the process/session. Document/result/progress/cancellation IDs from another session are rejected. No session resumption in v1 unless a later exact profile exists.

## Source/privacy output

The app emits only service-authorized source locations/text. Hover/source excerpts use structural source-data boundaries. Errors/logs exclude document text, private roots, credentials, hidden review/holdout data, provider private locators, raw owner handles, and unrestricted stack traces.

## Transport separation

stdout is protocol only. stderr cannot be interpreted as protocol input. The adapter does not multiplex MCP or another protocol on the same stream.

## Adversarial corpus

- malformed/duplicate/conflicting headers and lengths;
- truncated/concatenated/polyglot frames;
- duplicate-key/deep/huge JSON;
- malicious request IDs/methods/capabilities/URIs;
- Unicode positions and edit bombs;
- source containing headers, JSON-RPC, prompts, tool calls, and control characters;
- cross-session/cross-version/cross-overlay substitution;
- unauthorized workspace/source disclosure;
- edit/command/configuration attempts;
- queue flood, slow client, broken pipe, EOF, cancellation and shutdown races;
- direct lower-crate import or service-call duplication.
