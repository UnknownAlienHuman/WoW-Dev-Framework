# E7-A host modes and commands

**Status:** normative grammar.

## Common host options

```text
--config <PATH>
--profile <ID>
--consumer-profile <ID>
--privacy-profile <ID>
--license-profile <ID>
--log-level <off|error|warn|info|debug>
--log-format <text|json>
```

Configuration is strict bounded JSON. Precedence is explicit CLI fields, explicit config, then named release-profile defaults. No environment/cwd/home/Git/editor/WoW discovery affects semantics. Secret material is not accepted in ordinary config.

## One-shot CLI

Existing E0–E6 commands remain governed by their own app/service contracts. E7-A may route them through the same immutable operation registry internally but must preserve their exact request, output, and exit behavior.

```text
wow transport capabilities
    [--transport-profile <ID>]
    [--output-mode <envelope-json|text>]
```

This invokes `frontend_capabilities` once and does not probe lower owners directly.

## Local daemon

```text
wow daemon run
    --profile wow-local-jsonrpc-v1
    [--endpoint <PIPE_OR_SOCKET>]

wow daemon status
    --endpoint <PIPE_OR_SOCKET>

wow daemon shutdown
    --endpoint <PIPE_OR_SOCKET>
    --operation-id <ID>
```

`run` is foreground. Endpoint defaults, if any, come only from the frozen release/platform profile and current-user scope. There is no `--listen`, TCP port, wildcard address, background/service install, arbitrary operation, plugin directory, or shell command.

The daemon wire operation `operation/call` accepts one operation name from the negotiated registry plus arguments matching its exact schema. The CLI parser cannot bypass registry validation.

## LSP

```text
wow lsp
    --transport stdio
    --profile lsp-3.18-stdio-v1
```

No socket/TCP/HTTP LSP transport is supported initially. Workspace folders and documents arrive through LSP after initialization; they remain untrusted explicit registration/change inputs.

## MCP stdio

```text
wow mcp
    --transport stdio
    --profile mcp-2025-11-25-stdio-v1
```

The process speaks MCP JSON-RPC on stdin/stdout only. No standalone query is accepted on the same invocation.

## MCP local Streamable HTTP

```text
wow mcp
    --transport streamable-http-local
    --profile mcp-2025-11-25-streamable-http-local-v1
    --bind 127.0.0.1:<PORT>
    --allowed-origin <ORIGIN>
```

This profile is disabled unless explicitly enabled by validated configuration. `--bind` must be loopback. Wildcard, LAN, public, Unix socket tunneling, reverse proxy, TLS termination, and remote-hosting flags are invalid in E7-A.

Session authentication material is acquired from a protected host adapter and is never passed through argv, URL, or ordinary config.

## Unsupported modes/options

```text
wow server --public
wow mcp --transport websocket
wow lsp --tcp
wow daemon install-service
wow daemon --background
--call-service
--tool
--rpc-method
--shell
--command
--plugin
--script
--model
--provider-database
--credential
--token
--password
--private-key
--remote
--auto-workspace
--git-root
--wow-installation
```

Unknown modes/options fail before session/service operation dispatch.