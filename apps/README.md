# Applications and frontend hosts

Applications are thin transports over `wow-service`. They never reimplement reference, analyzer, project, graph, recognizer, diagnostics, search, context, external-provider, calibration, publication, release or storage semantics.

## Active routes

- [`wow/`](wow/README.md)
  - E0-F: `status`, `check` one-shot CLI.
  - E3-C: context commands.
  - E4-C: search, lineage, migration validation and static impact.
  - E5-B/C: calibration and governed core-pack lifecycle commands.
  - E6-B: external provider/result/mapping/selection/context commands.
  - E7-A: one-shot CLI compatibility, foreground local daemon, LSP 3.18 stdio, MCP 2025-11-25 stdio and optional local-only MCP HTTP.
- [`wow-reference-builder/`](wow-reference-builder/README.md)
  - E1-D: local Reference Pack build/validate/rebuild-compare.

## Dependency rule

Every application/frontend depends on `wow-service` only among framework crates. Protocol/argument/file/signal/endpoint libraries may be host dependencies but cannot absorb domain policy.

## `apps/wow` E7-A boundary

```text
explicit mode and protocol profile
-> exact immutable operation registry/session
-> strict command/method/tool/resource input
-> one typed wow-service request
-> one service invocation
-> exact/lossless CLI, daemon, LSP or MCP result
-> bounded progress/cancellation/backpressure
-> explicit close/reconciliation
```

The app does not resolve current state, inspect owner stores, build analyzer/project/graph/context state, select external candidates, authorize effects, expose generic tools, infer workspaces, apply edits, change editor settings, read secrets/provider databases, retry unknown effects or publish/install releases.

## Initial host modes

```text
wow <one-shot command>
wow transport capabilities
wow daemon run|status|shutdown
wow lsp --transport stdio
wow mcp --transport stdio
wow mcp --transport streamable-http-local
```

The local HTTP profile is explicit and disabled by default. No remote listener is supported in E7-A.

## Frontend invariants

- exact registry, no reflection/generic operation proxy;
- one semantic request to one service operation;
- missing capabilities not advertised;
- explicit workspace/project/profile registration;
- exact versioned document overlays;
- disconnect is not cancellation and progress is not completion;
- response replay does not reexecute service;
- bounded queues and multi-client isolation;
- protocol stdout contains protocol frames/messages only;
- no source/editor/project mutation without an existing exact owner contract and authorization;
- no public package/update lifecycle before E7-B.

## Next

E7-B release tooling/packages consume built application artifacts only after implementation gates. Release tooling does not become a semantic application dependency and cannot package internal agent/architecture/TODO files by default.