# Static external GlobalColors overlay

`wow-annotations` can consume Ketho's `Annotations/Core/Type/GlobalColors.lua` through the existing explicit `--alias-catalog` input.

The reference boundary admits only top-level declarations of the exact form:

```lua
NAME = CreateColor(red, green, blue, alpha)
```

All four components must be numeric literals. Their source lexemes, the external resource revision, path, SHA-256 digest and half-open UTF-8 declaration range are retained. Lua is parsed but never executed. Locals, indexed targets, multiple assignments, alternate callees, nested expressions, missing or extra arguments, and any unrelated executable statement reject the resource.

The projection emits analysis-only declarations:

```lua
---@type colorRGBA
NAME = nil
```

It does not reproduce or execute `CreateColor`, does not claim exact runtime values, and does not transfer Blizzard authority to the external catalog. Output is deterministically ordered by name. Duplicate catalog names and conflicts with projected Blizzard declarations are explicit blocked outcomes; no first-wins or last-wins behavior is used.

If `colorRGBA` is not available from the selected native or external type set, the declarations remain present but the library is `partial` with source-bound `unresolved_global_color_type` evidence. The selected projection retains its outcomes, issues and source maps. The obsolete `verify-library` command is removed; public builder cases inspect emitted bytes and mappings, while the actual addon Library handoff remains separate acceptance.

Generated-to-source and source-to-generated navigation use the normal `annotation_alias_catalog` scope and the exact external revision. Rebind after changing the catalog buffer; stale text or digest identities reject rather than being repaired.
