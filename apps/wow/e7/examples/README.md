# E7-A application fixture shapes

- `mode-cases.json` — explicit one-shot/daemon/LSP/MCP mode grammar and forbidden discovery, remote, secret, and generic-tool options.
- `lsp-method-map.json` — exact LSP 3.18 method-to-service-operation mappings and unadvertised method cases.
- `mcp-tool-map.json` — fixed MCP 2025-11-25 read-only tool/resource mappings and forbidden capability/effecting-tool cases.
- `stdout-exit-cases.json` — stdout/stderr/framing, graceful/abnormal exit, delivery failure, reconnect, and no-double-dispatch behavior.
- `CHECKSUMS.json` — service, protocol adapter, command/mapping/profile/client/platform/vector/member freeze gate.