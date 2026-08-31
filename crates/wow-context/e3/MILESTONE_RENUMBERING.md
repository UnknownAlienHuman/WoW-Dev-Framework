# E3 context milestone renumbering and compatibility

**Status:** normative routing and terminology correction.

## Why this file exists

The first `wow-context` documentation package was written under the label `E3-A`, while the pinned Blizzard UI source producer was described as future `E3-B`.

The repository sequence was later corrected so the source universe exists before the context projection that consumes it:

```text
E3-A
    wow-project Blizzard UI source acquisition-input contract,
    exact source index, analyzer/recognizer/graph publication,
    and SkeletonInputView

E3-B
    wow-context Project Map, L0/L1 skeletons,
    bounded context construction, and rendering
```

The current assignment is authoritative.

## No second context implementation

The inherited files under `crates/wow-context/e3/` are the foundation of the current E3-B contract. They are not a separate legacy implementation and must not produce an additional crate, feature flag, schema family, or runtime path.

Any inherited prose that says `E3-A` while discussing `wow-context` is read as `E3-B`.

Any inherited prose that says a future `E3-B` source producer is required is superseded by:

```text
crates/wow-project/e3/CONTRACT.json
contract_id = wow-project/e3-a/blizzard-ui-source-index
```

## Active machine identity

```text
work package: E3-B
contract ID: wow-context/e3-b/project-map-l0-l1-context-pack
manifest: crates/wow-context/e3/CONTRACT.json
```

The prior ID:

```text
wow-context/e3-a/project-map-skeleton-progressive-context
```

is a documentation-history alias only. It is not an active implementation target.

## Terminology consolidation

| Earlier term | Current E3-B term | Rule |
|---|---|---|
| `ContextInputSnapshot` | `ContextUniverseSet` | One exact immutable multi-universe binding, not two objects |
| `ContextBundleCore` | `ContextSemanticPack` | One canonical semantic artifact, not two schemas |
| `build_context_bundle` | `build_context_semantic_pack` | Historical operation alias only |
| `continue_context_bundle` | `continue_context_semantic_pack` | Historical operation alias only |
| E3-A context profile | E3-B context profile | Same conceptual profile family after renumbering |
| future E3-B platform source | E3-A `wow-project` platform-source publication | Current producer boundary |

Existing fixture field names can remain temporarily as compatibility aliases only when `CONTRACT.json` and the fixture explicitly declare the mapping. First Rust implementation must use the current names or a reviewed migration layer; it must not expose both as independent public models.

## Precedence inside the context package

1. repository and crate `AGENTS.md`;
2. this renumbering contract;
3. current `README.md` and `CONTRACT.json`;
4. current E3-B decisions, data model, operations, tests, and implementation plan;
5. inherited specialized documents, interpreted through this file;
6. examples and frozen fixtures.

A conflict with the current machine contract is resolved in favor of the current machine contract and must be fixed before implementation.

## Required mutation test

Rename all historical labels in a synthetic copy:

```text
E3-A context -> E3-B context
ContextInputSnapshot -> ContextUniverseSet
ContextBundleCore -> ContextSemanticPack
```

Canonical semantic behavior must remain unchanged except for explicitly versioned schema/contract IDs. If the implementation creates duplicate artifacts or code paths, the test fails.
