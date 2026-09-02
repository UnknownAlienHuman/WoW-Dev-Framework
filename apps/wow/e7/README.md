# `apps/wow` E7-A frontend transports

**Status:** implementation-ready documentation; no Rust code exists.

**Contract ID:** `apps/wow/e7-a/frontend-transports`

`apps/wow` hosts explicit one-shot CLI, local daemon, LSP 3.18 stdio, MCP 2025-11-25 stdio, and optional local-only MCP Streamable HTTP modes over `wow-service`. Its only framework dependency is `wow-service`.

## Host modes

```text
wow <existing one-shot command>
wow transport capabilities
wow daemon run
wow daemon status
wow daemon shutdown
wow lsp --transport stdio
wow mcp --transport stdio
wow mcp --transport streamable-http-local
```

The last mode is disabled unless an explicit local-only profile is configured. No remote listener is supported by E7-A.

## Reading order

1. `AGENTS.md`
2. `MODES_AND_COMMANDS.md`
3. `LSP_METHOD_MAPPING.md`
4. `MCP_TOOL_RESOURCE_MAPPING.md`
5. `OUTPUT_STREAMS_AND_EXIT.md`
6. `SECURITY_AND_CONFIG.md`
7. `TEST_MATRIX.md`
8. `CONTRACT.json` and `examples/`
9. `../../../crates/wow-service/e7/README.md`

## Responsibilities

- parse one explicit host mode and strict bounded configuration;
- initialize one exact E7-A protocol/operation-registry profile;
- implement protocol framing and lifecycle only;
- map each valid command/method/tool/resource operation to exactly one service request;
- preserve session/workspace/document/operation identities;
- project bounded progress/cancellation/reconnect/backpressure;
- emit exact canonical or lossless protocol results;
- keep stdout/stderr/endpoints/file output protocol-correct;
- close the frontend session and transport synchronously.

## Forbidden responsibilities

- importing lower framework crates;
- implementing project overlays, diagnostics, graph/search/context, external provider, calibration, publication, or storage semantics;
- dynamically discovering service methods/tools/plugins;
- exposing arbitrary RPC, MCP tool, shell, command, script, model, or filesystem operations;
- inferring workspace/project/profile/provider from cwd, Git, editor, or WoW installation;
- accepting private signing/provider/deployment credentials through ordinary flags/config;
- treating client/model/editor identity as authorization;
- applying edits or changing editor settings automatically;
- remote network hosting;
- release packaging/updating, which belongs to E7-B.

## Current state

```text
documentation frontier: E7-A
implementation frontier: not-started
Cargo.toml: absent
Rust source: absent
CI/workflows: absent
```