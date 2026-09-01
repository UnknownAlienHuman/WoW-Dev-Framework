# Applications

Applications are thin transports and host adapters over `wow-service`. They never reimplement reference, annotation, storage, analyzer, project, rule, graph, search, context, profile, or release semantics.

## Active contract routes

- [`wow/`](wow/README.md)
  - E0-F: `status`, `check` diagnostic CLI.
  - E3-C: `context status`, `map`, `inspect`, `build`, `continue`, `validate`, `render`.
- [`wow-reference-builder/`](wow-reference-builder/README.md)
  - E1-D: local Reference Pack `build`, `validate`, `rebuild-compare`.

## Dependency rule

Every application depends on `wow-service` only among framework crates. Host libraries may handle arguments, strict transport serialization, bounded explicit file/stdin I/O, signals, and output, but cannot absorb domain policy.

## `apps/wow` E3-C boundary

```text
strict CLI/config/artifact input
-> one typed wow-service request
-> one service invocation
-> canonical envelope JSON, faithful text, or exact returned artifact
-> frozen exit code
```

It does not resolve current, open stores/views, search roots, inspect project source, build maps/skeletons/packs, render context, retry on another generation, authorize tools/edits, or start background work.

## Reference builder boundary

The builder executes typed staging/materialization/finalization and reviewed probe adapter plans issued by service. It does not download source, run repository scripts, execute Lua/generated files, mutate editors, sign, upload, publish, or activate a release.

## Later applications

```text
wow-emmy-ls   LSP frontend, E7
wow-mcp       MCP frontend, E7
```

Search is expected to extend `apps/wow` through service in E4-A rather than create a domain-rich application.

## General prohibitions

- no direct lower framework dependency;
- no hidden current/latest profile/source;
- no arbitrary shell/network/repository execution;
- no implicit source/editor/client/config discovery;
- no source/project mutation or tool authorization;
- no semantic difference caused by terminal/environment;
- no empty success for deferred operations;
- no release/publishing/CI before its contract.
