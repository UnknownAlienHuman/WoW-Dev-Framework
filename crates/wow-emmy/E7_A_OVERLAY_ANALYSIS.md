# E7-A EmmyLua overlay-analysis seam

**Status:** normative cross-crate seam; implementation has not started.

`wow-emmy` analyzes one exact project-owned overlay input view using the same pinned upstream EmmyLua adapter as saved project generations. It does not own sessions, workspace paths, document synchronization, transport positions, diagnostics policy, or editor protocols.

## Operations

```text
emmy_overlay_analyze
emmy_overlay_status
emmy_overlay_snapshot_validate
```

## Input

```text
exact ProjectDocumentOverlayAnalyzerInput ID/digest
exact persisted project/analyzer base generation
ordered virtual/physical Lua unit manifest
UTF-8 content/source handles and line maps
load/VFS/reference profile
analyzer implementation and configuration profile
requested capability subset
budgets and cancellation
```

The input is prepared by `wow-project`. `wow-emmy` does not read editor buffers or arbitrary filesystem paths.

## Output

```text
OverlayAnalyzerSnapshot
    exact overlay input and analyzer/profile IDs
    syntax/semantic facts for changed and required dependent units
    generic diagnostics
    source maps and UTF-8 byte ranges
    invalidated/reused unit manifest
    dependency/graph freshness boundary
    capabilities, coverage, conflicts, omissions
    cancellation/resource state
    canonical digest
```

## Freshness classes

```text
OverlayLocalComplete
OverlayLocalPlusPublishedDependencies
OverlayPartialDependencies
SavedGenerationOnly
NotEvaluated
Failed
```

A local syntax/semantic result can be complete for the changed unit while published cross-file graph/search results are stale. The output preserves these axes separately.

## Incremental reuse

Reuse requires exact analyzer implementation/profile, persisted base generation, unchanged unit bytes/digests, VFS/load/reference identity, and validated invalidation closure. Same URI or client version alone is insufficient.

Scheduling and cache layout are nonsemantic. Equivalent exact inputs produce identical canonical facts/diagnostics under 1/2/N workers.

## Position boundary

`wow-emmy` returns canonical UTF-8 byte source ranges. It never performs LSP position conversion or stores client position encoding. The E7-A service maps exact ranges to the negotiated transport encoding using the exact overlay line index.

## Hard boundaries

- no second Lua parser;
- no direct dependency on `wow-project`, `wow-service`, applications, LSP, MCP, or editor SDKs beyond owner-neutral input contracts permitted by the active crate slice;
- no filesystem/source acquisition, workspace registration, or overlay mutation;
- no WoW diagnostic rule ownership;
- no graph/search/context/provider operation;
- no source edit, command, or code action;
- no saved-generation completeness claim for overlay-partial analysis;
- no arbitrary analyzer configuration from source text.

## Tests

Cover exact full/partial overlay views, changed virtual units, load/reference changes, invalidation closure, stale dependency classification, malformed Lua, cancellation, resource bounds, cache equivalence, source-map parity, UTF-8 ranges over non-BMP text, and deterministic 1/2/N worker output.