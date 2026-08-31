# Superseded context-package numbering and terminology

**Status:** historical routing note; not an active implementation contract.

Before the exact Blizzard UI source producer was formalized, the first detailed `wow-context` draft used work-package label `E3-A` and contract ID:

```text
wow-context/e3-a/project-map-skeleton-progressive-context
```

The project sequence is now corrected:

```text
E3-A = wow-project Blizzard UI source universe/index
E3-B = wow-context Project Map, L0/L1, and context pack
```

The old context draft remains available through Git history, including commit `1ecd2b5b6f1ee068b84bef5bafa2a6dea7f694d7`. It must not be restored as a second active contract.

## Terminology migration

These names describe the same conceptual boundary, but only the right-hand names are current E3-B vocabulary:

```text
ContextInputSnapshot        -> ContextUniverseSet
ContextBundleCore           -> ContextSemanticPack
ContextRendererArtifact     -> RenderedContextArtifact
build_context_bundle        -> build_context_semantic_pack
continue_context_bundle     -> continue_context_semantic_pack
pinned_platform_ui_source   -> blizzard_ui_source universe binding
```

This mapping is documentation migration guidance, not an instruction to expose duplicate Rust types or API aliases. Implement exactly one public type and operation set from the active E3-B contract.

## Preserved design substance

The active E3-B package retains the valuable constraints from the draft:

- exact coherent project/graph/reference/source identities;
- Project Map and L0/L1 progressive detail;
- explicit control/effect/source limitations;
- evidence, provenance, coverage, conflict, loss, omission, and stopping records;
- faithful bounded source excerpts;
- strict budgets and exact-tokenizer requirements;
- noncyclic semantic/render/metrics identity order;
- deterministic output and continuation;
- no parser, search, storage, model, runtime, editor, or mutation authority.

When historical wording conflicts with the active package, the active E3-B files under `crates/wow-context/e3/` control implementation.
