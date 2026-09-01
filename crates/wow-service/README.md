# `wow-service` contract router

**Status:** E0-F diagnostics, E1-D Reference Pack orchestration, E3-C context operations, and E4-C search/lineage/migration/static-impact operations are implementation-ready documentation; no Rust code exists.

`wow-service` is the only production crate allowed to coordinate multiple framework owners into one transport-independent user-facing operation. It never reimplements their domain algorithms.

## Contract routes

### E0-F — status and check

The original complete overview is preserved as [`E0_F_OVERVIEW.md`](E0_F_OVERVIEW.md). E0-specific agent instructions are preserved as [`E0_F_AGENTS.md`](E0_F_AGENTS.md). The root E0 contract package defines only:

```text
status
check
```

### E1-D — Reference Pack build and validation

Read [`e1/README.md`](e1/README.md). E1-D coordinates exact Reference Pack build, nonrepairing validation, and deterministic rebuild comparison without taking ownership from `wow-store`, `wow-reference`, or `wow-annotations`.

### E3-C — context acquisition and public use cases

Read [`e3/README.md`](e3/README.md). E3-C resolves symbolic publication selectors exactly once, acquires coherent retained project/graph/reference views, binds one E3-B `ContextUniverseSet`, invokes `wow-context`, validates/renders results, closes every resource, and returns one canonical service envelope.

```text
context_status
context_map
context_inspect
context_build
context_continue
context_validate
context_render
```

### E4-C — search, lineage, migration, impact, and explicit context handoff

Read [`e4/README.md`](e4/README.md). E4-C coordinates E4-A `wow-search`, E4-B `wow-graph` lineage overlays, project/reference producer inputs, review authorization, migration validation, bounded static impact, and explicit search-candidate-to-context-root handoff.

```text
search_index_status
search_index_build
search_index_validate
search_query
search_continue
search_explain
search_select
search_context

lineage_status
lineage_build
lineage_validate
lineage_review_validate
lineage_review_apply
lineage_compare
lineage_trace
lineage_explain

migration_candidates
migration_validate

impact_plan
impact_run
impact_continue
impact_explain
```

E4-C never selects the highest-ranked candidate automatically, applies a migration, edits source, infers reviewer authority from GitHub/OS/CLI identity, or converts static impact into runtime breakage/severity.

## Active E4-C dependency slice

```text
wow-core
wow-store
wow-reference
wow-project
wow-graph
wow-search
wow-context
```

Existing E0/E1/E3 operations retain their own narrower slices. `wow-emmy`, `wow-recognizers`, `wow-rules`, `wow-annotations`, and `wow-cbm` are not direct E4-C operation dependencies; their relevant immutable outputs arrive through owner publications.

## Service boundary

Service owns:

- request/configuration validation;
- one-time symbolic selector resolution;
- exact retained view/lease acquisition and reverse-order release;
- cross-owner compatibility validation;
- public use-case sequencing;
- search shard/catalog orchestration without hidden build;
- explicit search selection receipts;
- review authorization adapter orchestration without proof ownership;
- idempotency, response-loss recovery, continuation retention and closure;
- operation-level conservative status and canonical envelopes.

Service does not own:

- source/TOC/XML/Lua parsing;
- analyzer, recognizer, rule, graph, search, reference, context, renderer, or storage algorithms;
- search ranking, lineage proof ceilings, ambiguity resolution, migration semantics, or impact traversal;
- raw SQL/SQLite connections or mutable owner handles;
- review identity/credential policy itself;
- source/project mutation, migration application, runtime probes, severity or remediation;
- CLI parsing/stdout/stderr/exit codes;
- model/Codebase Memory, tool authorization, LSP/MCP, releases, or CI.

## Current implementation state

```text
documentation frontier: E4-C
implementation frontier: not-started
Cargo.toml: absent
Rust source: absent
CI/workflows: absent
```

Documentation presence does not bypass E0–E4 prerequisite implementations, exact owner-port contracts, fixtures, evaluations, authorization adapters, and SHA-256 freeze gates.
