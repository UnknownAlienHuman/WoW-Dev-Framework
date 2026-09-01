# E3 milestone assignment and migration

**Status:** normative routing note; not a runtime compatibility contract.

## Authoritative assignment

```text
E3-A = wow-project Blizzard UI source universe, structural graph, and SkeletonInputView producer
E3-B = wow-context Project Map, L0/L1 skeletons, context expansion, semantic pack, and renderers
E3-C = wow-service/apps context acquisition and public use-case orchestration, if retained as a separate package
```

The earlier context draft used E3-A before the source-index package was assigned. That label is retired. The active context contract ID is:

```text
wow-context/e3-b/project-map-l0-l1-context-pack
```

## Documentation mapping

```text
old ContextInputSnapshot  -> ContextUniverseSet
old ContextBundleCore     -> ContextSemanticPack
old build_context_bundle  -> build_context_semantic_pack
old continue_context_bundle -> continue_context_semantic_pack
```

This mapping helps reviewers read Git history. It does not authorize:

- duplicate Rust types, traits, modules, enums, serialization tags, or operations;
- a legacy implementation path;
- automatic deserialization aliases;
- long-lived public API compatibility obligations;
- different identities for the same semantic artifact.

Any real compatibility layer requires a future reviewed migration contract with versioning, deprecation, test, and removal gates.

## Source-index dependency

A user-project-only E3-B request may explicitly omit the Blizzard UI universe. Any profile requiring platform implementation context binds an exact E3-A `wow-project` publication and `SkeletonInputView`; it cannot download, parse, or synthesize that universe inside `wow-context`.

## Freeze rule

The first E3-B Rust commit must contain only the current names. Tests must fail if both retired and current type/operation models are implemented as independent APIs.
