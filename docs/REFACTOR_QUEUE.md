# Rewrite queue

The [manifesto](ARCHITECTURE.md) defines scope, [AUDIT.md](AUDIT.md) records the
source diagnosis, and [IMPLEMENTATION.md](IMPLEMENTATION.md) defines the target
owners. This file is the only ordered queue; Issues define bounded deliverables.

## Phase 0 — truthful instructions

1. Keep root `README`, `AGENTS`, Architecture, Audit, Implementation, Queue,
   Completion and Execution as the canonical set.
2. Reconcile #191/#125 and active workflow Issues with current `main`.
3. Move useful source/coordinate/TOC/XML facts from old package notes, then delete
   E1–E7/platform contracts, historical gates and broken links. Owner: #112.
4. Repair the #232 negative-test/validation-phase regressions while touching the
   Project replay path; do not restore removed schemas.

## Phase 1 — generated Library to useful addon check

| Order | Issue | Deliverable |
| --- | --- | --- |
| 1 | [#201](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/issues/201) | Align builder `/2` with the real Library reader; remove old producer/verifier and duplicate helpers; regular-file/batch/cancellation fixes. |
| 2 | [#200](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/issues/200) | Root/flavor/TOC/Library configuration, matching finite limits and upstream Lua 5.1 stdlib. |
| 3 | #201 / [#198](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/issues/198) | Preserve upstream diagnostic message/code/severity/range; configured Project path used by CLI/LSP. |
| 4 | #201 / [#103](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/issues/103) | Select checks before provider work; separate findings from requested-scope completeness. |
| 5 | [#109](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/issues/109) | Changed/deleted Lua and TOC/XML/load selection update the same owner without stale results. |

Acceptance: the supported builder output loads directly; CLI valid/invalid typed
Library calls differ at the addon line with the real message and severity; one
missing/ambiguous input is honest.

## Phase 2 — direct persisted source search

| Order | Issue | Deliverable |
| --- | --- | --- |
| 1 | [#131](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/issues/131) | One current SQLite schema for source identity, files/text, symbols/spans and useful relations; no replay hydration. |
| 2 | [#169](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/issues/169) | Direct body/symbol FTS/query; remove nested SQLite/hex/JSON and self-sizing pagination along this path. |
| 3 | [#122](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/issues/122) | Matching indexed text -> readable path/range/fenced excerpt; remove pack/render wrapping. |
| 4 | [#106](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/issues/106) / [#117](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/issues/117) | User CLI for body, symbol and useful TOC/XML navigation. |
| 5 | [#109](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/issues/109) / [#113](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/issues/113) | No stale rows after changes; measure and fix the first real bottleneck once. |

Acceptance: after restart, find a body-only term and symbol, return the matching
excerpt, and remove hits after change/delete.

## Phase 3 — remove general platforms from real consumers

| Issue | Replacement |
| --- | --- |
| [#195](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/issues/195) | Direct typed relations instead of generic fact/pack/plan/matcher handoff. |
| [#107](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/issues/107) | Reuse loaded Library/callable origins; remove graph requalification and proof DTOs. |
| [#111](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/issues/111) | Delete generic Store object/lifecycle, backup, retention, GC, quarantine, recovery and replacement after direct index readers move. |
| [#194](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/issues/194) | Plain results and one serialization; remove remaining self-identity/fixed-point paths from actual callers. |
| [#112](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/issues/112) | One current representation per retained workflow; delete exclusive formats/tests/docs. |

Do not finish every cleanup before a workflow. Attach each deletion to the caller
that makes it obsolete.

## Phase 4 — collapse frontends onto the same product operations

- [#197](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/issues/197):
  reduce the public CLI from the current internal Graph/Project/Search/Context
  platform to Library generation, configured addon check and direct index/search
  with readable context; keep finite user-meaningful defaults only.
- [#198](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/issues/198):
  one standard LSP stdio adapter to the configured check and retained indexed
  navigation owners; standard empty results and positions, no parallel Lua engine
  or profile family.
- [#199](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/issues/199):
  one standard MCP profile with useful read-only check/search/context tools;
  accept bounded extension metadata and ignore valid unknown notifications.
- [#125](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/issues/125):
  close only after all three advertised workflows run through their public
  entrypoints and the corresponding scoped evidence exists.

## Delivered components — do not rebuild

- #193: one Project Main+Library Emmy analysis.
- #202: one statement-catalog parse.
- #204: one final annotation emission.
- #196: supported native generation entrypoint.
- #203: native literals, no Wasm bridge.
- Core E0 envelopes, one-shot registries, Reference Pack administration,
  lineage/comparison and Project catalog/replay tag ladders are removed.

Delivered components may still have broken handoffs. Fix the handoff; do not
recreate their superseded implementations.

## Work rule

One manager/worktree, one bounded Issue. Read Architecture, the selected
Implementation route, actual callers and donor. Change code first, run formatting
and affected strict Clippy, then one proportional consumer check when the path is
available. Remove displaced options/tests/docs. No legacy fallback, new schema
family, certification framework or all-history campaign.
