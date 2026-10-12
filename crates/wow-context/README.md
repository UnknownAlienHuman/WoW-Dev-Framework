# `wow-context` contract router

**Status:** partial executable E3-B selected Primary maps, L0/L1 inspection,
semantic packs/rendering, one explicit native frontier round and finite ordered
progressive batches. Full E3-B and
launch acceptance remain open in
[#122](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/issues/122).

Selected contract routes preserve genuine retained project/graph owners, exact
configured Reference binding, original support and load/source custody:

- [Direct/project-root maps](e3/NATIVE_PRIMARY_MAP.md).
- [L0/L1 inspection](e3/NATIVE_PRIMARY_INSPECT.md), including selected native
  syntax/signature metadata, [original XML handler L1 metadata](e3/NATIVE_PRIMARY_INSPECT.md#native-xml-handler-l1-metadata)
  and ordinary TOC state-root L0 inspect/build.
- [Sealed semantic packs and canonical JSON/metadata-safe Markdown](e3/NATIVE_PRIMARY_PACK.md).
- [One explicit native frontier round](e3/NATIVE_PRIMARY_EXPANSION.md) and its
  [build/pack/JSON/Markdown consumer](e3/NATIVE_PRIMARY_FRONTIER_PACK.md).
- [Explicit ordered native progressive batches and inspect/pack/render selection](e3/NATIVE_PRIMARY_PROGRESSIVE.md).
- [Selected native state-detail L1 inspect/build](e3/NATIVE_PRIMARY_STATE_DETAIL.md),
  retaining literal state keys, plural TOC declarations, scope and ambiguity.
- [Selected native Reference-facts L1 inspect/build](e3/NATIVE_PRIMARY_REFERENCE_FACTS.md),
  retaining original records, normalized ordered source signatures and exact support;
  source type resolution and annotation display lowering remain unevaluated.
- [Exact Reference-root L0 navigation](e3/NATIVE_REFERENCE_ROOT_L0.md), followed
  by original ReferenceFacts L1 in combined Both Inspect/Build on one map/root/Budget.
- [Native platform universe/map/inspect through service and CLI](e3/NATIVE_PLATFORM_MAP.md),
  using one retained Skeleton input; broader platform and combined acceptance remain open.
- [Selected combined native map, L0/L1 inspection and semantic pack/JSON/Markdown](e3/NATIVE_COMBINED_MAP.md),
  retaining separate original owners/origins and exact shared Reference association
  under one continuing construction Budget through validation and final delivery.

Selected projections remain Partial or Truncated with absence_authoritative=false;
a bound Reference view does not imply Reference enrichment.

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

## Current implementation state

```text
Cargo.toml: active root workspace member
functional/build-verified: selected maps, L0/L1, native syntax/signature metadata,
  TOC state-root L0 inspect/build, packs/rendering and explicit native frontier
  and progressive batches
bounded functional lifecycle/CLI: selected state-detail L1 inspect/build/render,
  candidate-qualified native platform map/inspect
remaining: broader facets, automatic expansion/continuation, broader Reference/type enrichment,
  broader platform facets/combined views, excerpts/privacy/exact tokens/continuation,
  full fixtures, Tier A/X and the specified E6-B mapped-root handoff
```
