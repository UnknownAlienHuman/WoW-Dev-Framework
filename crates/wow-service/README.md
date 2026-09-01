# `wow-service` contract router

**Status:** E0-F diagnostics, E1-D Reference Pack orchestration, and E3-C context-service contracts are implementation-ready documentation; no Rust code exists.

`wow-service` is the only production crate allowed to coordinate multiple framework owners into one transport-independent user-facing operation. It never reimplements their domain algorithms.

## Contract routes

### E0-F — status and check

The original complete overview is preserved as [`E0_F_OVERVIEW.md`](E0_F_OVERVIEW.md). E0-specific agent instructions are preserved as [`E0_F_AGENTS.md`](E0_F_AGENTS.md). The root E0 contract package remains:

```text
CONTRACT.json
DECISIONS.md
DATA_MODEL.md
CONTEXT_ACQUISITION.md
STATUS_OPERATION.md
CHECK_OPERATION.md
ROOT_CAUSE_FOLDING.md
RESULT_ENVELOPE.md
ERROR_MODEL.md
TEST_MATRIX.md
IMPLEMENTATION_PLAN.md
examples/
```

Only `status` and `check` activate in E0-F.

### E1-D — Reference Pack build and validation

Read [`e1/README.md`](e1/README.md). E1-D coordinates exact Reference Pack build, nonrepairing validation, and deterministic rebuild comparison without taking ownership from `wow-store`, `wow-reference`, or `wow-annotations`.

### E3-C — context acquisition and public use cases

Read [`e3/README.md`](e3/README.md). E3-C resolves symbolic publication selectors exactly once, acquires coherent retained project/graph/reference views, binds one E3-B `ContextUniverseSet`, invokes `wow-context`, validates/render results, closes every resource, and returns one canonical service envelope.

E3-C public operations:

```text
context_status
context_map
context_inspect
context_build
context_continue
context_validate
context_render
```

Exact roots are required. Search, fuzzy names, natural-language root resolution, lineage, migration, and impact remain E4.

## E3-C context-path dependency slice

```text
wow-core
wow-store
wow-reference
wow-project
wow-graph
wow-context
```

Existing E0/E1 operations retain their own narrower dependencies. The E3-C path does not directly invoke `wow-emmy`, `wow-recognizers`, `wow-rules`, `wow-search`, `wow-cbm`, or applications. Their relevant published records arrive through owner views when explicitly requested.

## Service boundary

Service owns:

- request/configuration validation;
- one-time selector resolution;
- coherent exact view/lease acquisition and release;
- cross-owner compatibility validation;
- use-case sequencing;
- explicit profile/renderer selection;
- operation-level status and canonical envelopes;
- cancellation, failure isolation, and resource closure;
- typed deferred-operation reporting.

Service does not own:

- source/TOC/XML/Lua parsing;
- analyzer, recognizer, rule, graph, reference, context, renderer, or storage algorithms;
- raw SQL/SQLite connections or mutable owner handles;
- project/source mutation;
- search/ranking or model inference;
- CLI parsing/stdout/stderr/exit codes;
- tool authorization, LSP/MCP, runtime probes, or releases.

## Current implementation state

```text
documentation frontier: E3-C
implementation frontier: not started
Cargo.toml: absent
Rust source: absent
CI/workflows: absent
```

Documentation presence does not bypass E0-E3 prerequisite implementations, exact owner-port contracts, fixtures, evaluations, and SHA-256 freeze gates.
