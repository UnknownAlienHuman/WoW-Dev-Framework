# Project updates

Current code: `src/{update,publication,analyzer}.rs` and
[`wow-emmy/src/session.rs`](../wow-emmy/src/session.rs). This describes today's
update path and the replacement in [#109](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/issues/109),
not a requirement to retain old generation/receipt formats.

## Current path

`ProjectPublisher::derive_update_request` compares explicitly supplied final
inputs with the current publication. `apply_update_cancellable` validates the
expected base and add/update/remove operations before building the candidate.
It publishes only a successful result; failure/cancellation keeps the previous
immutable view. Equal configuration, inventory and Library inputs return
`NoChange` without analysis. No watcher, source scan or source execution occurs.

`ProjectLibraryOperation` expresses `Keep`, `Replace` or `Clear`. The current
analyzer binding requires a nonempty Library set, so Clear/empty replacement
rejects. `with_target_libraries([])` instead means Keep and `target_libraries()`
loses the Keep/Clear distinction: these legacy helpers are removal targets,
not behavior to reproduce in the replacement.

| Input / stage | Actual work |
| --- | --- |
| Physical Main change | `AnalyzerUpdateBatch::between` selects changed text; unchanged VFS syntax trees remain. |
| Semantic indexing | `session.rs::apply_delta` clears indexes and reindexes all remaining VFS files, currently in separate syntax and semantic engines. It does not reparse every retained text. |
| Reports | Diagnostics and facts are collected for the target input; `reused_files` counts unchanged Main parser inputs, not reused semantic facts or avoided indexing. |
| Library/configuration change | The cached owner reopens when inputs are incompatible. |
| TOC/XML/package input | Project uses its cold analysis branch. Service's `live_project/operations.rs::update` currently rejects selected TOC/package load plans. |
| Stored publication | Service rebuilds the selected graph before switching current; #131 replaces read-time replay/analysis reconstruction. |

## Replacement

#193 first removes the duplicate engine while preserving Main/Library diagnostics
and virtual mappings. Keep the conservative semantic rebuild until a narrower
update demonstrably covers its consumers. At the pinned donor, public bulk
updates remove touched file indexes and analyze only supplied replacement texts;
they do not enqueue unchanged consumers of a deleted global/type declaration.
Do not assume `require()` ordering supplies that dependency closure or build a
new incremental framework to claim reuse.

#109 connects the current index to add/edit/rename/delete and selected load-plan
changes. Reload/rebuild when the existing update cannot represent the change;
a TOC-only change cannot disappear because Lua text stayed equal. Update stored
facts in an ordinary transaction, remove stale results and switch after success.
Use one current format; remove displaced replay/version paths, legacy Library
helpers and their exclusive tests. Preserve user files and coherent results.

## Useful checks

Reuse `wow-emmy/tests/incremental.rs` for final-state comparison, especially
`removing_a_cross_file_global_definition_matches_cold_parity`. Update its cold
helper for the shared analyzer rather than retaining obsolete split-engine
expectations. `tests/project_e0.rs` already covers immutable old views,
Library replacement, NoChange and cancellation; select the affected behavior.

For #109, compare the relevant diagnostics/search hits from one changed addon
with a fresh build of the same final source, including its TOC/XML selection.
Check one failed/cancelled publication. No historical identity matrix, byte-proof
manifest, power-loss campaign or new benchmark system is needed.
