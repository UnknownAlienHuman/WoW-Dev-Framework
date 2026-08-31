# Applications

Applications are thin transports and host adapters over `wow-service`. They must not reimplement reference, annotation, storage, analyzer, project, rule, graph, search, context, or profile logic.

## Contract routes

- [`wow/`](wow/README.md) — E0 `status` and `check` CLI projection.
- [`wow-reference-builder/`](wow-reference-builder/README.md) — E1-D local Reference Pack `build`, `validate`, and `rebuild-compare` frontend.

## Dependency rule

Each application depends on `wow-service` only among framework crates. Host libraries for arguments, JSON, filesystem, process isolation, or transport may be used only at the application boundary and must not absorb domain policy.

## E0 `wow`

```text
parses explicit typed status/check arguments
constructs wow-service requests
serializes exact service results
maps semantic state to frozen exit codes
```

No source scan, lower-crate orchestration, deferred-operation fake success, LSP/MCP, or editor mutation.

## E1 `wow-reference-builder`

```text
build
    explicit request + materialized source root + output root

validate
    read-only nonrepairing candidate validation

rebuild-compare
    isolated repeated builds under frozen execution profiles
```

The application executes only typed staging/materialization/finalization and reviewed external probe adapter plans issued by `wow-service`. It does not download source, run repository scripts, execute Lua/generated files, mutate editors, sign, upload, publish, or activate a release.

## Later planned applications

```text
wow-emmy-check      optional batch compatibility frontend, not separately active
wow-emmy-ls         LSP frontend, E7
wow-mcp             MCP frontend, E7
```

A transport convenience is not a reason to add a domain operation or bypass `wow-service`.

## General prohibitions

- no direct lower framework crate dependencies;
- no hidden current/latest profile or source;
- no arbitrary shell/network/repository execution;
- no source/editor/client mutation;
- no semantic difference between JSON and text projections;
- no final release/publishing/CI before the owning milestone contract.
