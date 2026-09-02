# E7-A normative fixture shapes

- `operation-registry.json` — reviewed registry generation, schema bindings, capability narrowing, and rejection of generic operation proxies.
- `lsp-session.json` — LSP 3.18 initialization, workspace/document overlays, position encoding, pull diagnostics, and feature projection.
- `mcp-session.json` — MCP 2025-11-25 initialization, fixed read-only tools/resources, structured results, and local Streamable HTTP restrictions.
- `reconnect-cancel.json` — progress, cancellation, disconnect, response delivery loss, reconnect, backpressure, and exact reconciliation.
- `CHECKSUMS.json` — prerequisite, protocol adapter, registry, profile, vector, client/platform, member, and bundle freeze gate.

Implementation-dependent values remain null only while `implementation_state` is `not-started`. Fixtures are verified inputs; test code never rewrites them automatically.