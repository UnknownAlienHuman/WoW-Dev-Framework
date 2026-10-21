# Implementation and reduction queue

**Default next user-facing task: [#200](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/issues/200)
→ [#201](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/issues/201): normal configuration
and generated Library → addon check.** The parallel search route is #131/#169/#122.
Attach remaining reductions to these consumers; completing every cleanup Issue
is not a prerequisite for a usable workflow.
The [manifesto](ARCHITECTURE.md) defines purpose and scope;
[implementation](IMPLEMENTATION.md) maps code, replacements and donors.
[ADR-058](DECISIONS.md) retires the broader platform plan. `main` is the
source of truth; [#191](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/issues/191)
tracks reduction and [#125](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/issues/125)
tracks completion of the narrowed product.

The [rewrite basis](DOCUMENTATION_REWRITE_BASIS.md) records current findings,
including #232 rejection-check regressions. It is preparation material, not a
second queue or a new mandatory reading bundle.

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
| Completed | [#193 — Emmy analysis](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/issues/193) | One Main + Library analysis now supplies diagnostics, facts and flow; useful parameter-type errors reach Main locations. |
| Completed | [#202 — catalog parsing](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/issues/202) | One Emmy Lua 5.1 parse now feeds namespace, FunctionContainer and GlobalColors validators. |
| Completed | [#204 — single emission](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/issues/204) | Renderer-owned validation admits declarations; final mapped assembly is the only byte emission. |
| With the consuming workflow | [#194 — result envelopes](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/issues/194) | Core and local Service envelopes/registries are removed; simplify remaining Context/Search/graph self-identities only from real callers. |
| 3 | [#131 — current index](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/issues/131) | Eager all-partition payload reads are removed; store/read file text directly without Project/Emmy rehydration. |
| 4 | [#169 — SQLite search](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/issues/169) | Direct source-content/FTS tables in #131's index; current FTS covers file labels/recorded names. Remove nested SQLite/hex/JSON. |
| 5 | [#111 — storage lifecycle](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/issues/111) | Physical epoch migration is removed; after #131 delete remaining backup/quarantine/recovery and generic retention/GC/journals. |
| Completed | [#196 — annotation delivery](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/issues/196) | Supported generation writes ordinary Library files; builder/Service pack administration, distribution gate and second Lua copy are removed. |
| Completed | [#203 — Wasm removal](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/issues/203) | Native literal algorithm retained; guest/host, wire selection and exclusive CI removed. |
| Next graph cut | [#195 — typed recognizers](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/issues/195) | Start with typed State Core and remove its repeated pack handoff; continue through live adapters. |
| With #131/#196 | [#107 — useful relationships](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/issues/107) | Direct records; reuse Library/callable origins instead of regenerating them for graph proofs. |
| Coordinate #107 | [#118 — removed product routes](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/issues/118) | Source comparison and Graph lineage are removed; delete only remaining promotion/fingerprint machinery proven exclusive. |
| With #195/#107 | [#103 — recognizer scope](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/issues/103) | Remove unconsumed/default placeholder capabilities; keep actionable rules. |
| With touched owner | [#112 — legacy formats/docs](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/issues/112) | Historical catalogs, feature catalogs and replay tag ladders are removed; keep one rebuildable ProjectReplay. |

[#120](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/issues/120)'s frozen
package-state validators are removed in #192. Normal file/JSON/native-only
checks remain. No replacement certification/checksum framework is planned.

## Finish the actual user workflows

| Task | Result / real dependency |
| --- | --- |
| [#200 — configuration](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/issues/200) | Root/flavor/TOC/library paths instead of evidence-heavy host inputs; existing Lua files suffice, independent of #196. |
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
