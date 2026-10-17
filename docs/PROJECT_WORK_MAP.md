# Current project work map

The current refactoring preparation and integration claim belong to
[Issue #191](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/issues/191).
Read root [AGENTS.md](../AGENTS.md), the [reviewed audit and bounded plan](AUDIT_2026-10-17.md)
and [execution model](EXECUTION_MODEL.md). The audit records verified defects,
corrections to Gemini's deletion recommendations, and the next code slices.

The [completion matrix](PROJECT_COMPLETION_MATRIX.md) records package/launch
acceptance. Existing owners and applications must not be recreated from stale
bootstrap prose. A pending acceptance issue blocks its named claim, not all
independent implementation work.

## Active routing

| Work | Canonical tracker | Next bounded direction |
|---|---|---|
| Refactoring preparation, analyzer/canonical cost and Service ownership | [#191](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/issues/191) | Follow the reviewed plan; one owner/consumer path per claim. |
| Machine-state and checksum consistency | [#120](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/issues/120) | Replace one owner's pending-state snapshot checks with evidence/consistency validation. Keep acceptance distinct from checksums. |
| Retained Project read cost | [#131](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/issues/131) | Persist/hydrate the minimum exact read model instead of rebuilding source analysis during normal acquisition. |
| Context | [#122](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/issues/122) | Shared payload shapes and readable Markdown, preserving external admission and owner evidence. |
| Search | [#169](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/issues/169) | Simplify retained FTS storage and pagination without replacing the existing search owners. |
| CLI and LSP/MCP usability | [#125](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/issues/125) | Finite defaults; standard empty results, notifications and metadata; then one missing owner-backed operation. |

Lifecycle, context and frontend repairs can proceed independently when their
owned paths do not overlap. They do not wait for optional E5/E6 or release work.
Use the existing bounded Clippy/Quick cadence; keep expanded tests for a coherent
functional milestone under the operator's instructions.

## Existing application routes

- [Local status/check](../apps/wow/LOCAL_INPUT.md),
  [Project publication and retained lifecycle](../apps/wow/LIVE_PROJECT.md).
- [Map](../apps/wow/CONTEXT_MAP.md), [Inspect](../apps/wow/CONTEXT_INSPECT.md),
  [Build/render/continuation](../apps/wow/CONTEXT_BUILD.md).
- [Exact-generation Search build/query](../apps/wow/SEARCH_BUILD_QUERY.md).
- [Manifest-bound platform input](../apps/wow/PLATFORM_INPUT.md),
  [source-only Reference](../apps/wow/SOURCE_REFERENCE.md),
  [source census](../apps/wow/SOURCE_CENSUS.md).
- [Frontend Service](../crates/wow-service/e7/CONTRACT.json) and
  [application](../apps/wow/e7/CONTRACT.json) contracts: selected LSP routes,
  MCP status and opt-in context. Partial implementations are not full conformance.

These routes preserve exact generations, source/owner identities and coverage.
Partial output never establishes complete corpus coverage or runtime authority.

## Acceptance and historical work

| Trackers | Responsibility |
|---|---|
| #106, #113, #116–#118 | W17 corpus, capacity, candidate/Skeleton, classification and fingerprint qualification |
| #103, #107–#111 | W11–W16 structural, graph, update, retention and recovery acceptance |
| #112, #120 | Documentation/version-consumer inventory and machine-state census |
| #124, #125 | Residual hardening and future implementation scope |

Closed #159, #161, #162, #166 and #170 are delivered selected scopes, not the next
work queue. Likewise #114/#115/#119/#121/#123 are historical evidence. Full
acceptance remains in the open trackers above; do not reopen or recreate those
closed implementations merely because a previous work map listed them.

Publication uses one manager/worktree/claim and a reviewable branch/PR, without
force. Record the exact commit and Quick result once in the canonical issue/PR;
keep detailed historical evidence there instead of duplicating it in hot routing.
