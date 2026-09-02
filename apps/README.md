# Applications

Applications are thin transports and host adapters over `wow-service`. They never reimplement reference, annotation, storage, analyzer, project, rule, graph, search, lineage, context, profile, migration, impact, external-provider, mapping, selection, or release semantics.

## Active contract routes

- [`wow/`](wow/README.md)
  - E0-F: `status`, `check` diagnostic CLI.
  - E3-C: context status/map/inspect/build/continue/validate/render.
  - E4-C: search, lineage, migration validation, static impact, and search-to-context commands.
  - E5-B: calibration run/review/holdout/submission commands.
  - E5-C: core-pack artifact/signing/publication/canary/rollout/activation/LKG/rollback/revocation/closure commands.
  - E6-B: external provider/query/result/mapping/selection/context commands.
- [`wow-reference-builder/`](wow-reference-builder/README.md)
  - E1-D: local Reference Pack `build`, `validate`, `rebuild-compare`.

## Dependency rule

Every application depends on `wow-service` only among framework crates. Host libraries may handle arguments, strict transport serialization, bounded explicit file/stdin I/O, signals, and output, but cannot absorb domain policy.

## `apps/wow` boundary

```text
strict command/config/request/selector input
-> one typed wow-service request
-> one service invocation
-> canonical envelope JSON, faithful text, or exact returned artifact
-> frozen exit code
```

The app does not resolve current/catalog/provider state, open owner stores/views, build indexes/graphs/context, rank or select candidates, authorize reviews/signing/provider use, map external locators, inspect source, access credentials/provider databases, retry unknown effects, apply migrations/edits, or publish releases.

Exact caller-supplied selection is mandatory before search-to-context or external-candidate-to-context handoff. Review, signing, provider-use, mapping, selection, activation, and distribution authorization remain independent service/owner concerns.

## Reference builder boundary

The builder executes typed staging/materialization/finalization and reviewed probe adapter plans issued by service. It does not download source, run repository scripts, execute Lua/generated files, mutate editors, sign, upload, publish, or activate a release.

## Next applications and transports

```text
E7-A
    supported CLI-daemon/session host
    thin LSP frontend
    thin MCP frontend
    one transport request -> one wow-service operation

E7-B
    public packaging/install/update/distribution frontends
```

Transport packages must not import lower framework crates, expose arbitrary shell/tool/RPC calls, or change evidence/authorization semantics.

## General prohibitions

- no direct lower framework dependency;
- no hidden current/latest/default profile/source/artifact/provider;
- no arbitrary shell/network/repository/tool execution;
- no implicit source/editor/client/config/provider discovery;
- no source/project mutation, migration application, or tool authorization;
- no automatic search/external/lineage candidate promotion or mapping selection;
- no semantic difference caused by terminal/environment;
- no empty success for deferred operations;
- no retry after `OutcomeUnknown` without exact service reconciliation;
- no release/publishing/CI before its contract and executable evidence.