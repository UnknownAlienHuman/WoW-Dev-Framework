# Addon and load context

Delivery state: [CURRENT_STATUS.md](../../docs/CURRENT_STATUS.md). Project owns
bounded source selection, exact bytes, source projection and only the load/context
facts required by the active queue slice.

For Slice 1A:

- add `AddonLoadTarget` with flavor, optional positive target Interface and
  optional `TocLoadContext`;
- bind `ProjectLoadPlan` identity/validation directly to that target and an
  explicit projection scope; legacy profile callers adapt into the target;
- use `physical_lua` projection: retain TOC XML records/spans but do not open XML;
  selected XML produces a located incomplete-source reason while direct Lua
  remains analyzable; if no physical Lua exists, preserve the partial result
  through an empty Project snapshot rather than a dummy file;
- expose a standalone load-plan limitation projection for the requested scope;
  package-only mapping and global `external_files_complete()` are insufficient;
- transfer already admitted root-relative Lua bytes to compiled-pin snapshots
  without `ProjectConfiguration`, publication, replay or proof identities.

Later slices add dependency closure/order, full XML/chunk context, then channel,
triggers, bindings and secure/special cases. Preserve source-backed unknowns and
independent findings; never rewrite user source.

See [Implementation](../../docs/IMPLEMENTATION.md#addon-configuration-and-emmy-analysis)
and [WoW behavior](../../docs/WOW_LINTER.md).
