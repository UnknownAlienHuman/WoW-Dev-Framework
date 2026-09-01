# `apps/wow` contract router

**Status:** E0-F diagnostics, E3-C context, and E4-C search/lineage/migration/static-impact CLI contracts are implementation-ready documentation; no Rust code exists.

`apps/wow` is a thin command-line adapter over `wow-service`. Its only framework dependency is `wow-service`.

## Contract routes

### E0-F — status and check

The original CLI overview is preserved as [`E0_F_CLI_OVERVIEW.md`](E0_F_CLI_OVERVIEW.md). Original E0 agent instructions are preserved as [`E0_F_AGENTS.md`](E0_F_AGENTS.md). The root [`CONTRACT.json`](CONTRACT.json) remains the E0-F machine contract.

```text
wow status
wow check
```

### E3-C — context operations

Read [`e3/README.md`](e3/README.md) and [`e3/CONTRACT.json`](e3/CONTRACT.json).

```text
wow context status
wow context map
wow context inspect
wow context build
wow context continue
wow context validate
wow context render
```

Context roots are exact typed IDs.

### E4-C — search, lineage, migration, and static impact

Read [`e4/README.md`](e4/README.md) and [`e4/CONTRACT.json`](e4/CONTRACT.json).

```text
wow search index status
wow search index build
wow search index validate
wow search query
wow search continue
wow search explain
wow search select
wow search context

wow lineage status
wow lineage build
wow lineage validate
wow lineage review validate
wow lineage review apply
wow lineage compare
wow lineage trace
wow lineage explain

wow migration candidates
wow migration validate

wow impact plan
wow impact run
wow impact continue
wow impact explain
```

There is no automatic top/first/sole candidate selection and no migration-apply command.

## Dependency boundary

```text
apps/wow -> wow-service
```

Every lower framework crate is a forbidden direct dependency, including `wow-search`, `wow-graph`, `wow-context`, `wow-store`, `wow-project`, and `wow-reference`.

## Application ownership

- parse strict bounded CLI/config/artifact/review/continuation transport input;
- construct one typed service request;
- pass symbolic current selectors unchanged to service;
- pass exact entity/result/candidate/shard/snapshot IDs mechanically;
- project signals to cancellation;
- invoke service exactly once;
- emit canonical envelope JSON, faithful text, or one exact validated artifact;
- map service outcomes to frozen command-specific exit codes;
- enforce stdout/stderr, atomic file output and broken-pipe behavior.

The app does not resolve current or catalogs, acquire views, inspect source, build shards, rank search results, select candidates, decide lineage/review proof, apply migrations, traverse impact, build context, retry on another generation, authorize tools/edits, or start background work.

## Current implementation state

```text
documentation frontier: E4-C
implementation frontier: not-started
Cargo.toml: absent
Rust source: absent
CI/workflows: absent
```
