# CLI, editors and agents

CLI is the primary complete user path. Current commands are in
[apps/wow](../apps/wow/README.md); normal root/TOC/library configuration remains
#200. The workflows are in the [manifesto](ARCHITECTURE.md#what-the-product-must-deliver);
their code routes are in [implementation](IMPLEMENTATION.md).

## EmmyLua

Use the upstream analysis engine and standard language server for general Lua
features. Its LSP handlers are private to `emmylua_ls`, while the semantic API
is exposed by `emmylua_code_analysis`. See [EMMYLUA_UPSTREAM.md](EMMYLUA_UPSTREAM.md).
Generated Ketho-style libraries are ordinary `.lua` analysis files that can be
loaded independently of VS Code. Do not copy a complete LSP implementation.

## Framework stdio surfaces

| Surface | Current recipe | Required behavior |
| --- | --- | --- |
| LSP | [LSP.md](../apps/wow/LSP.md) | Correct document versions/positions, real diagnostics/navigation, supported capabilities only. |
| MCP | `wow mcp --transport stdio --profile mcp-2025-11-25-stdio-v1` | Fixed useful read-only tools over the same local data; no generic invocation API. |

Selected standards are [LSP 3.18](https://microsoft.github.io/language-server-protocol/specifications/lsp/3.18/specification/),
[MCP 2025-11-25](https://modelcontextprotocol.io/specification/2025-11-25/basic/lifecycle)
and [JSON-RPC 2.0](https://www.jsonrpc.org/specification). Ordinary LSP no-match
results use the method's null/empty shape. Valid unknown MCP notifications get
no response and do not end the session; initialization metadata is extensible
within input bounds. #198/#199 correct current deviations.

Keep unsaved buffers in memory, source locations tied to the actual document
version, cancellation and bounded framing/output. Do not expose source writes or
claim unsupported features. Runtime capability negotiation remains useful;
content-addressed registries, a host schema per feature combination, response
journals and replay streams are being removed. Internal old inputs can reject
with a current example instead of a backward-compatibility adapter.

## Removed hosting scope

No daemon, HTTP transport, background service, provider/plugin host, installation
or update client is required. The old E7 protocol/release plans are superseded
by ADR-058. Current stdio code still contains extra lifecycle machinery; remove
it with its callers under #194/#112/#200. Follow [TEST_STRATEGY.md](TEST_STRATEGY.md)
for one real request sequence when a protocol behavior changes.
