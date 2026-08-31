# Blizzard UI source input handoff — `wow-context` owner seam

**Status:** normative E3-B/E3-A integration contract; `wow-context` consumes exact views and never ingests source.

## Inputs

A context request may add one optional exact UI source input:

```text
BlizzardUiContextInput
    CurrentBlizzardUiSourcePublicationRecord or exact selected publication ID
    source profile/generation/snapshot IDs
    source GraphGenerationId / GraphSnapshotId
    reference profile/generation used by bridge partitions
    build-binding state
    source/license/redistribution/capability summaries with exact record refs
    source-reader capability manifest
    source graph query profile
    compatibility decision with the selected user project/reference inputs
    canonical digest
```

This input joins the existing exact user `ProjectSnapshot`, user `GraphSnapshot` and reference inputs only after compatibility validation.

## No ingestion

`wow-context` does not:

- clone, fetch, materialize, scan, parse or analyze Blizzard UI source;
- open a source ProjectStore or SQLite connection directly;
- infer source entities from names/paths/text;
- run source recognizers or bridge builders;
- change source current selection;
- redistribute source bytes by default.

It receives public immutable source/graph/source-reader views from the owning service/project boundary.

## Universe preservation

Rendered and machine context retains exact endpoint scopes:

```text
project:function:X
reference_api:api:Y
blizzard_ui_source:function:Z
```

A bridge fragment displays the relation and both endpoint generations. Same labels never collapse entities.

## L0 source skeleton input

L0 can include source entities under the E3-A structural rules:

- exact source entity key/kind/generation;
- root/package/load/source handles;
- direct axis/relation summaries;
- build-binding/license/capability state;
- source/reference bridge handles;
- conflicts/coverage and available detail.

No implementation body or source prose.

## L1 source skeleton input

L1 can include selected exact source semantics:

- signature/types from analyzer facts;
- direct source calls/registrations/hooks/state relations;
- XML/template/object/mixin/factory facts;
- exact `uses_api` and event reference bridges;
- bounded reason paths;
- source handles and optional bounded excerpts;
- confidence/provenance/coverage/conflicts.

Static source remains implementation evidence only. L1 cannot convert it into API/runtime/security authority.

## Project Map integration

The user project Project Map may include a bounded `Blizzard UI source bridges` section only when exact project/source/reference compatibility and bridge facts exist.

Examples:

```text
project hook -> exact UI source function/method
project XML inheritance -> exact UI source template
project entity -> exact reference API symbol <- used by UI source entity
```

E3-B does not precompute project-specific bridge truth. If no exact project bridge partition exists, the map may show only available source/reference views and query recipes, not guessed matches.

## Source excerpts

Source excerpts require:

- exact source publication/generation/handle/span;
- compatible source-reader capability;
- exact artifact-specific redistribution decision for the requested channel;
- E3-A byte/token/redaction/license budgets;
- source-map and content digest validation.

Default external context is handle/fact-only. `LocalAnalysisOnly` bytes cannot enter a remotely transmitted or packaged context bundle.

## Build-binding display

Every source-derived fragment that could be interpreted as build-specific carries the source profile/build-binding state. `ProviderDeclared`, `ContentCorrelated` or `Unverified` cannot be rendered as exact client-build truth.

If source and user project/reference profiles are incompatible, source sections become unavailable/NotEvaluated rather than silently falling back to another source generation.

## Authority markers

Context must distinguish:

```text
Reference contract
Source implementation structure
User project structure
Runtime observation
Derived cross-universe bridge
```

Source call/absence/comment does not become public API or runtime claim. A path containing `Possible` remains possible.

## Selection and budgets

UI source fragments compete under the same E3-A deterministic priority/budget system. Required user-root identity and uncertainty closure outrank optional source neighborhoods.

Source expansion always names exact source graph axes/relations and hard bounds. No whole UI source graph/tree dump.

## Invalidation

Invalidate source-dependent context partitions when any exact source publication/profile/build binding/source entity/relation/bridge/license/coverage/conflict/source-handle dependency changes.

A source current update does not automatically rewrite context for a user project pinned to an older exact source publication; policy must select the new compatible input and build a new context artifact set.

## Failure and fallback

If the requested source publication is unavailable:

- user project/reference context can remain usable if source context is optional;
- source-dependent sections are explicit `NotEvaluated`/unavailable;
- a last-known-good source publication can be used only under explicit fallback with requested/actual mismatch disclosure;
- no source fragment is relabeled to the requested build/profile.

## Tests

- exact source input with compatible user/reference publication;
- incompatible source/reference/project profiles;
- same symbol label across three universes;
- bounded L0/L1 source skeletons with no bodies;
- exact project/source bridge and absent project bridge;
- source implementation rendered as API/runtime authority mutation;
- local-only excerpt blocked from external bundle;
- source prompt/comment fenced as data;
- source current update invalidates exact dependent context only;
- optional source failure degrades only source lane;
- last-known-good fallback shows mismatch;
- high-fanout source graph remains bounded and progressively expandable.
