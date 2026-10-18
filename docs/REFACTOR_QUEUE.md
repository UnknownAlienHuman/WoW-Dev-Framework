# Implementation and reduction queue

**Default next code task: [#193](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/issues/193).**
The [manifesto](ARCHITECTURE.md) defines purpose and scope;
[implementation](IMPLEMENTATION.md) maps code, replacements and donors.
[ADR-058](DECISIONS.md) retires the broader platform
plan. Preparation is [PR #192](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/pull/192);
use its branch until integration. [#191](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/issues/191)
is the reduction tracker; [#125](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/issues/125)
tracks completion of this narrowed product.

## Take one task

Read root [AGENTS.md](../AGENTS.md) and the whole manifesto, then the Issue/current
claim and selected implementation route. Issues contain the change and check; this
file only orders work. Use one manager/worktree and nonoverlapping small tasks.
Do not reload historical package specifications or invent a parallel tracker.

Replace the old path in the same change. Remove its unused options, versions,
tests and docs. Keep source/configuration; derived data can rebuild. No legacy
fallback, new schema family, framework or archive. Run the relevant existing
check after the path works; broaden only for a real remaining risk.

## Reduce existing code

Order is a default. Independent tasks can proceed with distinct owned paths;
a dependency names required replacement behavior, not a whole package gate.

| Order | Task | Replacement / dependency |
| --- | --- | --- |
| 1 | [#193 — Emmy analysis](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/issues/193) | One actual analysis for syntax/member/flow. Existing AnalyzerSession still owns two. |
| Independent | [#202 — catalog parsing](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/issues/202) | Share one Emmy tree across existing validators; keep comment-group isolation. |
| 2 | [#194 — result envelopes](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/issues/194) | Plain typed results, one output serialization; remove self-hashes/size loops/read journals. |
| 3 | [#131 — current index](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/issues/131) | Store reader-ready facts; remove source rehydration and old replay formats. Coordinate #194. |
| 4 | [#169 — SQLite search](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/issues/169) | Direct source-content/FTS tables in #131's index; current FTS covers file labels/recorded names. Remove nested SQLite/hex/JSON. |
| 5 | [#111 — storage lifecycle](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/issues/111) | After #131, remove backup/quarantine/epoch migration, generic retention/GC/journals. |
| Native path | [#196 — annotation delivery](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/issues/196) | Supported native generation to a directory; remove pack administration. Coordinate #194; no index prerequisite. |
| Independent | [#203 — Wasm removal](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/issues/203) | Keep literal algorithm natively; remove guest/host, options, dependencies and exclusive CI. |
| Next graph cut | [#195 — typed recognizers](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/issues/195) | Remove fake pack JSON roundtrip; preserve useful real consumers. |
| After #131 | [#107 — useful relationships](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/issues/107) | Direct current-source records; remove graph proof/publication duplication. |
| Coordinate #107 | [#118 — removed product routes](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/issues/118) | Delete lineage/promotion/migration-impact/fingerprint routes and exclusive internals. |
| With #195/#107 | [#103 — recognizer scope](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/issues/103) | Remove unconsumed/default placeholder capabilities; keep actionable rules. |
| With touched owner | [#112 — legacy formats/docs](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/issues/112) | One current format and regeneration; remove old readers and duplicated requirements. |

[#120](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/issues/120)'s frozen
package-state validators are removed in #192. Normal file/JSON/native-only
checks remain. No replacement certification/checksum framework is planned.

## Finish the actual user workflows

| Task | Result / real dependency |
| --- | --- |
| [#200 — configuration](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/issues/200) | Root/flavor/TOC/library paths instead of evidence-heavy host inputs; uses #196. |
| [#197 — defaults](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/issues/197) | Search works without repeated internal budget flags; coordinate #200. |
| [#122 — context](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/issues/122) | Connect search hits to existing source-excerpt reading; plain located output. Rendering now, index via #131/#169. |
| [#106 — Blizzard UI](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/issues/106) | Real symbol/text/source-navigation CLI workflow using #200/#169. |
| [#117 — TOC/XML](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/issues/117) | Actual load/template/script links needed by #106; no classification/fingerprint platform. |
| [#109 — source updates](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/issues/109) | Changes/removals update useful results; compare one final state with cold indexing. Needs #131/#193. |
| [#113 — practical cost](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/issues/113) | Measure #106's necessary real run, fix its actual bottleneck and compare once. |
| [#198 — LSP](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/issues/198) | Ordinary empty results; correct document positions and session behavior. Independent. |
| [#199 — MCP](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/issues/199) | Standard metadata/notifications; no extra host profiles. Independent. |
| [#201 — complete CLI path](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/issues/201) | Generation → addon check; source index → later-process search/context. Uses selected setup/query tasks. |
| [#125 — product completion](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/issues/125) | Native Ketho behavior, these three workflows, useful stdio exposure and ordinary build/distribution. |

The former #108/#110/#116/#124 acceptance ledgers are retired; their useful
read/update/source checks are part of the replacement tasks. E5/E6/E7-B,
compatibility, cryptographic proof and full historical matrices do not re-enter
the queue. Completed behavior is checked in code before rebuilding it.
