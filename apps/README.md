# Applications

Applications are thin transports and host adapters over `wow-service`. They never reimplement reference, annotation, storage, analyzer, project, rule, graph, search, lineage, context, profile, migration, impact, or release semantics.

## Active contract routes

- [`wow/`](wow/README.md)
  - E0-F: `status`, `check` diagnostic CLI.
  - E3-C: context status/map/inspect/build/continue/validate/render.
  - E4-C: search index/query/explain/select/context, lineage build/review/query, migration candidate/validation, and static-impact commands.
- [`wow-reference-builder/`](wow-reference-builder/README.md)
  - E1-D: local Reference Pack `build`, `validate`, `rebuild-compare`.

## Dependency rule

Every application depends on `wow-service` only among framework crates. Host libraries may handle arguments, strict transport serialization, bounded explicit file/stdin I/O, signals, and output, but cannot absorb domain policy.

## `apps/wow` E4-C boundary

```text
strict CLI/config/query/review/artifact/continuation input
-> one typed wow-service request
-> one service invocation
-> canonical envelope JSON, faithful text, or exact returned artifact
-> frozen exit code
```

The app does not resolve current or artifact catalogs, open stores/views, build search shards, rank candidates, auto-select a result, accept lineage proof, infer reviewer authorization, apply a migration, infer runtime impact, inspect project source, build context artifacts, retry on another generation, authorize tools/edits, or start background work.

Exact candidate selection is mandatory before search-to-context handoff. Review input is transported as strict typed data and is independently authorized/validated by service and graph owners.

## Reference builder boundary

The builder executes typed staging/materialization/finalization and reviewed probe adapter plans issued by service. It does not download source, run repository scripts, execute Lua/generated files, mutate editors, sign, upload, publish, or activate a release.

## Later applications

```text
wow-emmy-ls   LSP frontend, E7
wow-mcp       MCP frontend, E7
```

E5 calibration and E6 external-candidate operations should extend `apps/wow` only through reviewed `wow-service` contracts.

## General prohibitions

- no direct lower framework dependency;
- no hidden current/latest profile/source/artifact;
- no arbitrary shell/network/repository execution;
- no implicit source/editor/client/config discovery;
- no source/project mutation, migration application or tool authorization;
- no automatic search/lineage candidate promotion;
- no semantic difference caused by terminal/environment;
- no empty success for deferred operations;
- no release/publishing/CI before its contract.
