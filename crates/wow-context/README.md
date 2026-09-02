# `wow-context` contract router

**Status:** E3-B Project Map/L0/L1/context-pack contract and the E6-B exact mapped-root handoff seam are implementation-ready documentation; no Rust code exists.

`wow-context` converts exact published project, Blizzard UI source, graph, analyzer-derived, and reference views into deterministic bounded context artifacts. It does not parse source, own graph/search/storage, infer facts with a model, consume external-provider semantics, or mutate projects.

## Contract history

- [`PRE_E3_OVERVIEW.md`](PRE_E3_OVERVIEW.md) preserves the initial crate brief.
- [`LEGACY_E3_A_CONTEXT_DRAFT.md`](LEGACY_E3_A_CONTEXT_DRAFT.md) maps the retired context milestone label to the single active E3-B model.

## Active E3-B route

Read [`e3/README.md`](e3/README.md) and its complete normative package. It defines:

```text
ContextUniverseSet
-> ProjectMap
-> L0Skeleton(s)
-> L1Skeleton(s)
-> ContextSemanticPack
-> RenderedContextArtifact(s)
```

Every layer is immutable, exact-generation-bound, evidence-preserving, budgeted, and independently identified. Rendering never becomes semantic truth.

## E6-B exact mapped-root handoff

Read [`E6_B_EXTERNAL_CONTEXT_HANDOFF.md`](E6_B_EXTERNAL_CONTEXT_HANDOFF.md).

E6-B does not add an external-provider dependency or new semantic lane inside `wow-context`. After `wow-service` validates one `ExactMapped` owner record and one explicit `Selected` receipt, it supplies the normal exact E3 universe plus the exact mapped root. Existing E3 operations are reused.

Provider labels, scores, snippets, summaries, traces, locators, mapping records, and selection receipts remain outside `ContextSemanticPack`. A separate external Candidate sidecar is composed by `wow-service`/the application, not by `wow-context`.

## Direct dependencies

```text
wow-core
wow-graph
wow-project
wow-reference
```

No direct dependency on `wow-store`, `wow-emmy`, `wow-recognizers`, `wow-rules`, `wow-search`, `wow-cbm`, `wow-service`, or applications. Relevant analyzer, recognizer, persistence, mapping, and selection state arrives only through exact public owner views/roots validated by service.

## Hard boundaries

- exact roots only; search/fuzzy/natural-language/external resolution is outside the crate;
- no second TOC/XML/Lua parser or raw-source semantic inference;
- no model inference, summarization, ranking, embedding, or tool call in the canonical path;
- no current-generation switch after binding;
- no user-project, Blizzard UI, reference, external, runtime, or historical identity merge;
- no provider/source text interpreted as framework or agent instruction;
- no claim without exact origin/evidence/coverage closure;
- no hidden omission, silent truncation, or exact-token claim without a frozen tokenizer;
- no unbounded source/graph export;
- no physical cache/persistence implementation;
- no diagnostics, remediation, edits, lineage, impact, external-candidate verification, or runtime truth;
- no Cargo/Rust/CI activation during this documentation phase.

## Current implementation state

```text
documentation frontier: E6-B exact mapped-root handoff seam
implementation frontier: not started
Cargo.toml: absent
Rust source: absent
CI/workflows: absent
```
