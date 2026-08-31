# `wow-context` contract router

**Status:** E3-B Project Map, L0/L1 skeleton, and context-pack contract is implementation-ready documentation; no Rust code exists.

`wow-context` converts exact, already-published project, Blizzard UI source, graph, analyzer-derived, and reference views into deterministic bounded context artifacts. It does not parse source, own graph/search/storage, infer facts with a model, or mutate projects.

## Canonical route

1. [`e3/README.md`](e3/README.md)
2. [`e3/AGENTS.md`](e3/AGENTS.md)
3. [`e3/DECISIONS.md`](e3/DECISIONS.md)
4. [`e3/DATA_MODEL.md`](e3/DATA_MODEL.md)
5. [`e3/INPUTS_AND_AUTHORITY.md`](e3/INPUTS_AND_AUTHORITY.md)
6. [`e3/OPERATIONS.md`](e3/OPERATIONS.md)
7. [`e3/PROJECT_MAP.md`](e3/PROJECT_MAP.md)
8. [`e3/SKELETONS.md`](e3/SKELETONS.md)
9. [`e3/EXPANSION_AND_CONTINUATION.md`](e3/EXPANSION_AND_CONTINUATION.md)
10. [`e3/BUDGETS_AND_TOKENIZATION.md`](e3/BUDGETS_AND_TOKENIZATION.md)
11. [`e3/SOURCE_BOUNDARIES_PRIVACY_AND_SECURITY.md`](e3/SOURCE_BOUNDARIES_PRIVACY_AND_SECURITY.md)
12. [`e3/CONTEXT_PACK_RENDERING_AND_CACHE.md`](e3/CONTEXT_PACK_RENDERING_AND_CACHE.md)
13. [`e3/ERROR_MODEL.md`](e3/ERROR_MODEL.md)
14. [`e3/TEST_MATRIX.md`](e3/TEST_MATRIX.md)
15. [`e3/IMPLEMENTATION_PLAN.md`](e3/IMPLEMENTATION_PLAN.md)
16. [`e3/CONTRACT.json`](e3/CONTRACT.json)
17. [`e3/examples/`](e3/examples/README.md)

The original pre-E3 crate brief remains [`PRE_E3_OVERVIEW.md`](PRE_E3_OVERVIEW.md). The superseded context-package numbering and terminology are documented in [`LEGACY_E3_A_CONTEXT_DRAFT.md`](LEGACY_E3_A_CONTEXT_DRAFT.md); that file is not an active implementation contract.

## Direct dependencies

```text
wow-core
wow-graph
wow-project
wow-reference
```

`wow-context` has no direct dependency on `wow-store`, `wow-emmy`, `wow-recognizers`, `wow-rules`, `wow-search`, `wow-cbm`, `wow-service`, or applications. Relevant analyzer and storage identities arrive through exact public project/graph views.

## Frontier order

```text
E3-A  wow-project: exact Blizzard UI source universe and structural graph
E3-B  wow-context: Project Map, L0/L1, bounded context packs
E3-C  wow-service/apps: context acquisition and public use cases, if kept separate
E4    wow-search plus lineage and impact
```

## Hard boundaries

- exact roots only; fuzzy/natural-language resolution belongs to `wow-search` or a higher layer;
- no second TOC/XML/Lua parser and no raw-source semantic inference;
- no LLM/model inference, summarization, ranking, or tool calls in the canonical path;
- no current-generation switch during a request;
- no merged user-project, Blizzard UI source, reference, external, runtime, or historical identities;
- no hidden omission, silent truncation, or unpinned exact token claim;
- no unbounded source/graph export;
- no persistence/cache storage implementation in this crate;
- no diagnostics, remediation, lineage, patch impact, or runtime truth;
- no Cargo/Rust/CI activation during this documentation phase.

## Current implementation state

```text
documentation frontier: E3-B
implementation frontier: not started
Cargo.toml: absent
Rust source: absent
CI/workflows: absent
```
