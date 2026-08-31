# `wow-context` contract router

**Status:** E3-B Project Map, L0/L1 skeleton, and context-pack contract is implementation-ready documentation; no Rust code exists.

`wow-context` converts exact, already-published project, Blizzard UI source, graph, analyzer-derived, and reference views into deterministic bounded context artifacts. It does not parse source, own graph/search/storage, infer facts with a model, or mutate projects.

## Contract routes

The initial crate brief is preserved as [`PRE_E3_OVERVIEW.md`](PRE_E3_OVERVIEW.md).

Read the current package in this order:

1. [`e3/MILESTONE_RENUMBERING.md`](e3/MILESTONE_RENUMBERING.md)
2. [`e3/README.md`](e3/README.md)
3. the remaining route declared by the E3-B README and machine contract.

The renumbering document is mandatory because the first context draft used the label `E3-A` before the Blizzard UI source-index package was assigned that milestone. The current authoritative assignment is:

```text
E3-A = wow-project Blizzard UI source index and SkeletonInputView producer
E3-B = wow-context Project Map, L0/L1, semantic context pack, and renderers
```

There is one context architecture, not two.

## Direct dependencies

```text
wow-core
wow-graph
wow-project
wow-reference
```

`wow-context` has no direct dependency on `wow-store`, `wow-emmy`, `wow-recognizers`, `wow-rules`, `wow-search`, `wow-cbm`, `wow-service`, or applications. It receives relevant data only through exact public project, graph, and reference views.

## E3-B output hierarchy

```text
ContextUniverseSet
-> ProjectMap
-> L0Skeleton(s)
-> L1Skeleton(s)
-> ContextSemanticPack
-> RenderedContextArtifact(s)
```

Every layer is immutable, exact-generation-bound, evidence-preserving, budgeted, and independently identified. Rendering never becomes semantic truth.

## Hard boundaries

- exact roots only; natural-language/fuzzy root resolution is not E3-B;
- no second TOC/XML/Lua parser and no raw-source semantic inference;
- no LLM/model inference, summarization, ranking, or tool calls in the canonical path;
- no current-generation switching during a request;
- no merged user-project, Blizzard UI source, reference, external, runtime, or historical identities;
- no source text treated as framework or agent instructions;
- no context claim without exact fact/assertion/evidence/coverage closure;
- no hidden omission, silent truncation, or token-count claim without a pinned estimator/tokenizer profile;
- no unbounded source/graph export;
- no persistence or cache storage implementation in this crate;
- no search, lineage, patch impact, diagnostics, remediation, or runtime truth;
- no Cargo/Rust/CI activation during this documentation phase.

## Current implementation state

```text
documentation frontier: E3-B
implementation frontier: not started
Cargo.toml: absent
Rust source: absent
CI/workflows: absent
```
