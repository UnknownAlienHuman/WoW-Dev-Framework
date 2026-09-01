# Legacy context milestone label

**Status:** migration note only; not an implementation contract.

The first `wow-context` documentation package was written before the Blizzard UI source-index producer received the E3-A milestone. The authoritative milestone assignment is now:

```text
E3-A = wow-project Blizzard UI source universe and SkeletonInputView producer
E3-B = wow-context Project Map, L0/L1 skeletons, semantic context packs, and renderers
```

Historical references to:

```text
wow-context/e3-a/project-map-skeleton-progressive-context
ContextInputSnapshot
ContextBundleCore
build_context_bundle
continue_context_bundle
```

map to the single current E3-B model:

```text
wow-context/e3-b/project-map-l0-l1-context-pack
ContextUniverseSet
ContextSemanticPack
build_context_semantic_pack
continue_context_semantic_pack
```

These mappings are documentation migration aids. They do not authorize duplicate Rust types, traits, modules, operations, serialization tags, code paths, or public compatibility aliases. Git history preserves the retired draft in full.
