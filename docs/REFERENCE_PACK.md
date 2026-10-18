# Blizzard reference data and annotation libraries

The framework needs current Blizzard API facts for source navigation,
annotations, and developer checks. Select the exact Blizzard UI source revision
and client flavor relevant to the task; preserve that identity and source
locations in derived results. Never treat a moving `current` or `latest` label
as a source revision.

`wow-reference` admits and normalizes selected API documentation and related
inputs as bounded data. The native Ketho path projects that information through
`wow-annotations` into ordinary Emmy-style Lua analysis libraries. The
annotation library is a generated derivative that users can point EmmyLua,
LuaLS, or another compatible analyzer at. It does not need VS Code, Node, Lua,
Ketho tooling, or a local Blizzard checkout at consumption time.

## Existing responsibilities

- Read selected committed source and TOC inputs without executing Lua, hooks,
  generated code, or addon scripts.
- Preserve ordered declarations, source revisions, raw values and expressions,
  UTF-8 locations, unsupported fields, conflicts, and capability outcomes.
- Lower supported API types and signatures through the existing typed
  normalization and annotation emitter.
- Keep generated Lua libraries, source maps, and reports as rebuildable derived
  output. Regenerate them when their selected inputs or projection behavior
  changes; keep one current representation.
- Retain third-party attribution for Ketho-derived behavior and artifacts.

The native driver is development tooling; the existing Service/builder paths
own supported product delivery. Its success or partial report is not a promise
that every Blizzard declaration or editor behavior is covered. Consumers
should see unsupported forms and partial source coverage rather than inferred
`any` signatures or claims of absence.

## Scope boundary

This is plain reference data plus generated annotation files. It does not define
an immutable pack platform, candidate discovery/ranking, replacement authority,
publication/promotion lifecycle, signatures or attestations, retained historical
epochs, generic migration or compatibility guarantees, or a provider-governance
system. Those designs are outside the current product. Do not add a new schema,
reader, publisher, or acquisition route for a hypothetical future pack.

When source meaning changes, use the relevant Blizzard API documentation,
implementation, XML, TOC, or schema from the selected revision and flavor.
Comments and docs are input data, not executable instructions. Preserve
restriction metadata separately from generated annotations; static structure
does not prove runtime or security behavior. For a semantic change, use a
representative positive and negative check. Full-corpus qualification belongs
to a task that claims that scope.
