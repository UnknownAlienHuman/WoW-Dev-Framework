# Disk inputs and Library directory

`wow-service/local-project-files/1` selects Main files and a Lua Library below
configured roots. This remains the internal disk input; ordinary addon setup in
#200 still needs to remove the profile/analyzer/evidence fields described below.
The reader does not execute Lua, write files or require a generator checkout.

## Main and Library

`main` has a relative `root` and a nonempty `files` list. The separate
[TOC input](TOC_INPUT.md) selects `toc` and optional `load_context` instead.

For a generated or existing Library, supply only its directory:

```json
"library": { "root": "generated-library" }
```

This is a fragment of the input object, not a complete configuration. The reader
collects `.lua` files recursively, sorts relative paths, and ignores ordinary
non-Lua files such as `source-report.json`. That report is never needed to load
Lua. Empty Libraries, case-colliding Lua paths, links and special files reject.
Discovery visits at most 16,384 entries and 32 directory levels; file reads use
the existing bounded, no-follow Project reader and cancellation checkpoints.
It is not an atomic filesystem snapshot; concurrent tree edits can fail a read.

An optional nonempty `library.files` list selects an exact subset instead. This
is used by current combined/platform inputs whose physical tree contains both
Main and Library files; it is not a fallback after failed directory acquisition.
Main and Library keep distinct roles. An explicitly empty list is an error.

The config's parent is the physical base. Roots and file paths are canonical
relative paths; file paths are relative to their selected root. Explicit file
entries have `path` and may include `content_digest` plus `byte_length` together.
Without them, the reader captures current bytes. Directory selection requires
neither per-file identities nor a manually maintained inventory.

## Bounds and remaining internal metadata

Library generation, disk acquisition and the Service Library workspace share
4,096 files / 8 MiB per file / 64 MiB total ceilings in
`wow-core::library_limits`. Main acquisition stays at 1,024 files / 1 MiB per
file / 16 MiB total. Analysis-output and existing replay/publication budgets are
separate; this alignment is not a 64 MiB end-to-end capacity measurement.

The current disk form still requires `schema`, project/workspace/source IDs,
`logical_root`, `profile`, `reference_view`, `analyzer`, `main` and `library`.
Profile, ReferenceView and compatibility-report JSON artifacts still require
exact digest/length declarations; their ceiling is 8 MiB, configuration 32 MiB.
Removing those setup requirements remains #200. See
[local checking](LOCAL_INPUT.md#execution-and-output) for command behavior.

The [Primary callable input](PRIMARY_CALLABLE_INPUT.md) adds its current recipes.
No new host schema, compatibility reader, annotation conversion or automatic
source/configuration rewrite is introduced by directory loading.
