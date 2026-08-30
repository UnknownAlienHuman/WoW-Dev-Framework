# Applications

Applications are thin transports over the shared `wow-service` use-case layer. They must not reimplement reference, analyzer, project, rule, graph, search, context, or profile logic.

## E0 active application

- [`wow/`](wow/README.md) — minimal CLI projection for `status` and `check` only.

The E0 CLI:

```text
parses explicit typed arguments
constructs wow-service requests
serializes exact service result types
maps service state to frozen exit codes
```

Its only framework dependency is `wow-service`. See [`wow/AGENTS.md`](wow/AGENTS.md) and [`wow/CONTRACT.json`](wow/CONTRACT.json).

## Planned applications

```text
wow                 primary CLI router; E0 status/check only
wow-emmy-check      possible compatibility/batch frontend, not active separately in E0
wow-emmy-ls         Language Server Protocol frontend, deferred to E7
wow-mcp             Model Context Protocol frontend, deferred to E7
wow-reference-builder
                    Reference Pack build/validation entry point, deferred to E1
```

Frontends translate transport requests into service use cases and serialize the same versioned result contracts. A transport-specific convenience is not a reason to add a domain operation or bypass `wow-service`.

## E0 prohibitions

- no source filesystem scan or stdin source ingestion;
- no direct dependencies on lower framework crates;
- no analyzer/project/reference/rule orchestration;
- no search, graph, LSP, MCP, daemon, runtime probe, edit/apply, release, or publishing command;
- no empty/default success for deferred commands;
- no semantic changes between JSON/text projections;
- no CI or release automation.

The application directory does not activate every planned binary. Create only the executable required by the active milestone and its executable tests.
