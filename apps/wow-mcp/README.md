# `apps/wow-mcp` contract router

**Status:** E7-A MCP adapter contract is implementation-ready documentation; no Rust code exists.

`wow-mcp` is a thin Model Context Protocol adapter over `wow-service`. Its only framework dependency is `wow-service`.

Read:

1. [`AGENTS.md`](AGENTS.md)
2. [`e7/README.md`](e7/README.md)
3. [`e7/CONTRACT.json`](e7/CONTRACT.json)
4. [`../../crates/wow-service/e7/`](../../crates/wow-service/e7/README.md)
5. official [Model Context Protocol specification](https://modelcontextprotocol.io/specification/)

Initial profile candidate:

```text
mcp-stdio-single-client-fixed-tools-resources-v1
```

The server exposes a static allow-listed set of exact framework tools/resources for status, workspace/session binding, diagnostics/navigation, search/context, external Candidate workflows, operation status, and cancellation. It does not expose generic tool dispatch, arbitrary files/URLs, dynamic source/provider tools, sampling, elicitation, prompts, model calls, source edits, publication effects, or direct lower-crate access.
