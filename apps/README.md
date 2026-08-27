# Applications

This directory will contain thin binaries over the shared service layer.

Planned applications:

```text
wow                 primary CLI router
wow-emmy-check      batch diagnostics entry point
wow-emmy-ls         Language Server Protocol frontend
wow-mcp             Model Context Protocol frontend
wow-reference-builder
                    Reference Pack build/validation entry point
```

Frontends must not reimplement query, diagnostic, graph, or profile logic. They translate transport requests into service use cases and serialize the same versioned result contracts.

E0 requires only the smallest CLI path needed to produce and golden-test one deterministic `wow check` result.
