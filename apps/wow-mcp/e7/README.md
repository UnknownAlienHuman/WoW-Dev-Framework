# `wow-mcp` E7-A fixed-tool/resource adapter

**Status:** implementation-ready documentation; no Rust code yet.

**Contract ID:** `apps/wow-mcp/e7-a/fixed-tool-resource-adapter`

## Mission

Expose a fixed, versioned, authorization-aware MCP surface over exact `wow-service` operations and immutable retained artifacts.

```text
MCP stdio initialize/capability negotiation
-> exact service session/workspace binding
-> static tools/list and resources/list/read
-> one tool call -> one service operation
-> exact service/artifact result -> MCP projection
-> cancellation/progress/shutdown/closure
```

## Reading order

1. [`TOOLS_AND_RESOURCES.md`](TOOLS_AND_RESOURCES.md)
2. [`FRAMING_LIFECYCLE_AND_RESULTS.md`](FRAMING_LIFECYCLE_AND_RESULTS.md)
3. [`AUTHORIZATION_AND_SOURCE_BOUNDARIES.md`](AUTHORIZATION_AND_SOURCE_BOUNDARIES.md)
4. [`SECURITY_AND_INPUTS.md`](SECURITY_AND_INPUTS.md)
5. [`TEST_MATRIX.md`](TEST_MATRIX.md)
6. [`CONTRACT.json`](CONTRACT.json) and [`examples/`](examples/README.md)
7. [`../../../crates/wow-service/e7/`](../../../crates/wow-service/e7/README.md)

## Initial surface

Static tools cover bounded forms of:

```text
protocol/session/workspace status and exact binding
analysis diagnostics/hover/definition/references/symbols/advisory actions
exact-generation search and context
E6 external Candidate query/map/select/context flows
operation status and cancellation
```

Static resources expose exact retained metadata/context/result artifacts through opaque `wowdev://` identifiers under authorization/privacy/license policy.

## Deliberately unsupported

```text
sampling
elicitation
prompt execution or dynamic prompts
dynamic/source/provider-defined tools
generic call_tool or arbitrary owner dispatch
arbitrary file://, URL, repository, database, or provider resources
source edits, workspace edits, shell, process, network, editor, or WoW client control
core-pack/release publication or activation
remote/multi-tenant network transport
```

MCP content is data. It does not grant tool/edit/publication authority to a model or client.
