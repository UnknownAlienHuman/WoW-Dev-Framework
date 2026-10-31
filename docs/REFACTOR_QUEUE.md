# Implementation work register

This is the single execution queue. [Status](CURRENT_STATUS.md) records delivery;
[the map](REPOSITORY_MAP.md) locates producers/consumers; [traces](IMPLEMENTATION_TRACE_REGISTER.md)
retain evidence; owner guides define behavior. Issue bodies route here instead of
maintaining another plan. All work below remains open unless its row links an
implemented change. A prerequisite is code the row consumes, not another Issue
that must be closed in its entirety.

## Working one step

Use the assigned row, or the first unfinished unblocked row in the active slice.
Read its owner contract and actual producer/consumer at one revision. Check an
existing draft before duplicating work. Take one connected change, not an entire
multi-row Issue. Put input -> output, owner/caller, exclusions and verification
in the existing task/PR; do not create another planning file.

The manager resolves shared signatures before delegating disjoint changes in its
single worktree. An implementation task must change its real caller, not stop at
unused types, a new report or another audit. If a required input/decision is absent,
record that specific blocker in the row and choose an independent assigned row;
do not invent metadata, defaults, source identities or a parallel implementation.

Before publication follow [CONTRIBUTING](../CONTRIBUTING.md#choose-the-checks).
Update the affected row with code/evidence and the next remaining boundary.
Implemented, linted and observed are different states. Tests remain deferred under
[AGENTS](../AGENTS.md#verification-and-handoff); completion cases below are review
questions until execution is authorized. No documentation-correction numbering,
completion percentage or full-repository audit is required for a normal patch.

<a id="1-first-usable-ordinary-addon-check"></a>

## 1. First usable ordinary addon check

### 1A. Explicit physical-Lua `wow check`

**Result:** explicit addon root, TOC, retail and ordinary Lua Libraries reach one
Service operation, one configured Emmy owner and ordinary diagnostics/limitations.
No dependency expansion, XML/chunk injection, WoW rules or search prerequisite.
Read [the contract](IMPLEMENTATION.md#first-implementation-slice), then only the
row being implemented. Detailed file routes are in [the map](REPOSITORY_MAP.md#ordinary-check).

| Work / owner | Needs first | Connected result and cleanup | Task / evidence |
| --- | --- | --- | --- |
| TOC metadata: Project parser -> package admission | Existing draft review | Reconcile the classifier/aliases in draft #266 with current guidance; one classification reaches package and limitation consumers. No parallel metadata table. | [#200][i200], [draft #266][p266]; T23 |
| Physical plan: `ProjectLoadPlan` / TOC reader | Metadata classification | Bind ordinary target and physical projection without profile identity; retain deferred XML locations without opening XML. Change both Main nonempty gates; keep Library admission strict. Update retained real callers rather than fabricate profiles. | [#200][i200]; T32/T33/T35 |
| Safe preparation: Project disk/Library -> Service | Physical plan for the combined handoff | Return captured Main/Library bytes, admitted authority and structured origins. Distinguish missing selected children from unsafe/I/O failure; enforce aggregate bounds and collisions before analysis. No source reread or ambient opener. | [#200][i200]; T04/T34/T36/T38 |
| Diagnostic settings: Emmy configuration -> configured session | Existing owner; independent of TOC work | One strict admission/canonical-equivalence boundary; handle global-off without skipping health or context. CLI later calls it, not a second validator. | [#273][i273]; T13/T16 |
| Engine failure origins: Emmy registration/collection -> Service | Existing owner/checker | Attach Library snapshot/path and resource cause at the producer; preserve them through conversions. Input/resource refusal is not native failure or session poisoning. | [#273][i273]; T38 |
| Chunk requirements: registered Main AST -> lower checker | Existing configured owner | Same-session collection identifies affected chunks; the checker consumes it and removes unconditional chunk fallback/hard-coded Partial. No second parse/session or helper injection. | [#273][i273]; T31 |
| Public operation: Service preparation -> internal checker | Combined Project preparation and the three Emmy rows | Consume admitted bytes once, map requested blockers, build exact snapshots and return one result/error. Keep one analyzer construction site; internalize lower requests after callers move. | [#194][i194]; T14/T37/T38 |
| Ordinary command: CLI args/config -> Service -> output | Public operation; parsing/encoding can be prepared independently | Connect actual command/help to Service. Apply safe config acquisition and bounded typed rendering from [Frontends](FRONTENDS.md); replace the ordinary legacy dispatch and its exclusive options. | [#197][i197]; T01/T02/T39 |

Engine fixes and small acquisition helpers need not wait for unrelated TOC work,
but unused helper construction is not completion. Do not advertise the ordinary
command until preparation, analysis and rendering are connected.

<a id="source-traced-replacement-boundary"></a>
<a id="exact-owner-split"></a>

### Replacement boundary

Replace `LocalCommand -> LocalProjectBackend -> generic Service::check` for this
operation. Do not emulate its Reference/Rules/publication prerequisites. Preserve
other still-used routes until their own callers move. [Ownership](IMPLEMENTATION.md#code-ownership)
and [selective reuse](IMPLEMENTATION.md#small-implementation-decisions) govern the
change, not the current crate count or legacy fixture shapes.

### Completion cases

Review these now; record actual inputs/results under existing traces only after
the path is ready and execution is authorized:

- Clean and invalid typed calls keep exact code/message/severity/root-relative range.
- Chunk ellipsis limits affected files, not function varargs, strings or comments.
  Missing child Lua/parents, unresolved required dependencies and deferred XML
  preserve independent diagnostics; unsafe/permission/I/O errors are not absence.
- Empty Main with no blocker is complete; deferred-only Main is partial. Library
  failure retains its root/path. Duplicate authority is refused; disjoint nested
  roots and equal relative paths in distinct roots remain valid.
- Invalid diagnostic keys/regex/interval are refused; equivalent settings reuse
  the owner, real semantic changes rebuild. Global-off preserves health/blockers.
- Config origins and overrides, deterministic text/JSON, resource refusals,
  cancellation, stdout/stderr and exits follow [Frontends](FRONTENDS.md#check-results-and-command-status).

### 1B. Dependencies, XML and chunk injection

| Work / owner | Needs first | Connected result and cleanup | Task / evidence |
| --- | --- | --- | --- |
| Dependencies and selected order: Project -> Service/Emmy | Working 1A | Required/available optional closure and exact order reach analysis; addons keep distinct chunk context. Do not turn path sorting into execution order or require the source index. | [#200][i200]; T03/T17/T18 |
| XML: existing load/maps -> virtual Main -> diagnostics | Working 1A; closure only for selected dependencies | Reuse selected XML parsing, preserve independent units on malformed input, support XML-only/empty input and map diagnostics to exact source pieces. Remove publication-dependent handoff for moved callers. | [#117][i117]; T05/T24/T27 |
| Ordinary chunk and reviewed handlers: Project context -> same Emmy owner | Requirement collector; XML maps for virtual handlers | Supply mapped chunk arguments only to supported affected external chunks; keep free inline scripts and reviewed handlers separate. Hide helpers; do not build another interpreter or type engine. | [#117][i117], [#273][i273]; T11/T22 |

### 1C. Special loading

| Work / owner | Needs first | Connected result and cleanup | Task / evidence |
| --- | --- | --- | --- |
| Channel, Interface, triggers and environments: Project -> requested checks | Selected-order/context handoff | Carry source Interface zero, positive target compatibility, channel gates, reverse LoadWith and unsupported security/private-table evidence. Preserve lifecycle and SavedVariables declarations without reading user data. Add only [source-backed semantics](WOW_LINTER.md#4-toc-and-xml-form-the-load-model). | [#200][i200]; T19/T21/T22 |
| Implicit bindings: Project discovery/maps -> mapped callbacks | XML mapping and selected flavor/context | Select an unambiguous bindings candidate; preserve absence, malformed/unsupported units and exact keystate callback mapping. No addon chunk arguments or invented runtime timing. | [#117][i117]; T20 |

## 2. Generated Library semantics and selected WoW checks

Generation safety/projection can progress independently of 1A. Ordinary type-only
checking accepts plain Lua and must not wait for this section. Use the existing
[builder and projection](KETHO_RUST_PORT.md), not another producer.

| Work / owner | Needs first | Connected result and cleanup | Task / evidence |
| --- | --- | --- | --- |
| Bounded complete output: builder I/O/report | Existing builder | Bound report serialization while encoding; stage tool-owned output and publish the finished directory without replacing existing data. Preserve primary errors and actual cancellation limits. | [#274][i274]; T10 |
| Missing source semantics: Reference -> Annotations | Selected source and supported Emmy lowering | Take one callable Environment/return/nilable-content/stride/mixin/restriction fact through normalization and emission. Preserve exact meaning or visible conservative loss; do not invent annotation syntax. | [#274][i274]; T10 |
| Ordinary metadata/maps: emitted report -> Library reader | Emitted consumed fields and ordinary preparation | Read useful typed metadata/maps without regeneration or legacy cache envelopes. Source counts are not source bodies; plain Lua needs no report. A producer-only field does not complete this row. | [#274][i274]; T10 |
| Selected WoW check: semantic binding + typed metadata -> Service | Working direct checker and facts used by that rule | Implement one requested API-context/local-Secret/relationship check through its real consumer. Preserve uncertainty and locations; no spelling-based API identity or broad new flow engine. Retire unused rule slots with their last caller. | [#103][i103]; [domain contract](WOW_LINTER.md#5-checks-to-deliver) |

## 3. Thin LSP/MCP adapters

| Work / owner | Needs first | Connected result and cleanup | Task / evidence |
| --- | --- | --- | --- |
| Coherent candidates: Project/Service -> retained result | Working 1A; each later context only when implemented | Source/Library/config changes use conservative rebuild and exact binding; failed/cancelled/stale candidates do not publish as current. Remove obsolete replay helpers with their readers. Index integration is later, not a diagnostic prerequisite. | [#109][i109]; T08/T09/T16 |
| LSP wire/overlay -> same Service operation | Public check and candidate/version handling | Correct signed versions, ordered/empty changes and wire-only rangeLength; preserve encoding/root isolation and reject stale publication. General Lua intelligence stays upstream; later indexed features wait only for their own reader. | [#198][i198]; T08; [protocol](FRONTENDS.md#lsp-adapter) |
| MCP wire -> implemented Service operations | Public check; search/context only before advertising those tools | Preserve metadata, valid unknown notifications, bounded framing, cancellation and cleanup. No lower-request assembly, receipts or adapter-owned analyzer. | [#199][i199]; [protocol](FRONTENDS.md#mcp-adapter) |

<a id="direct-source-lookup-and-remaining-platform-removal"></a>

## 4. Direct source index, search and context

The dependency direction is **index -> search -> context -> public workflow**.
A downstream integration task is not a prerequisite for its producer. Priority is
after the first checker; this index is never required to check an addon.

| Work / owner | Needs first | Connected result and cleanup | Task |
| --- | --- | --- | --- |
| Direct current rows: Store -> later-process reader | Admitted exact source/symbol data | Store exact bodies, spans and consumed relations in one rebuildable SQLite DB. Switch a real reader off AcquiredProjectPair/ProjectReplay hydration; retain useful lazy SQL reads. Rebuild obsolete derived data, not user source. | [#131][i131] |
| Body/exact-symbol query: Search -> located hits | Direct rows from #131 | Query actual body content, not just labels/names; update/delete transactionally. Replace nested SQLite -> hex -> JSON and displaced federation/response journals when their readers move. | [#169][i169] |
| Readable context: hit -> exact excerpt | Located hit and matching indexed body | Render bounded located source; disk rereads must match indexed identity. Keep complete items plus explicit truncation. Remove displaced Context pack/custody/continuation machinery. Does not wait for #106 or XML features. | [#122][i122] |
| Public source workflow: CLI -> index/search/context | The preceding three connected owners | Expose a real selected-source -> query -> excerpt path after restart. Add TOC/XML relations only when implemented and requested; do not make them a basic search dependency. Remove obsolete help/options with moved routes. | [#106][i106] |

## 5. Remove displaced platform code

Cleanup accompanies each replacement; it is not one final rewrite. A name, old
caller census or dead-code lint is insufficient deletion evidence. Search current
calls/re-exports, then remove the exclusive module, flags, fixtures and prose in
the same patch. Keep notices, source files, unpublished work and useful behavior.

| Family / entry point | Keep / replace first | Remove when that consumer no longer needs it | Task |
| --- | --- | --- | --- |
| Recognizers / Service graph-build adapters | One named diagnostic/navigation consumer receives typed located facts, including exact keys, ambiguity and incomplete coverage | Pack/plan/matcher JSON, witness requalification, synthetic ID inventories and unconsumed operators/adapters. No replacement DSL or plugin registry. | [#195][i195] |
| Native Graph / Library origins | Useful callable/load/source mapping reads already available Library facts | Duplicate generation/requalification, proof/crosswalk/publication seals and redundant graph forms. No second universal graph. | [#107][i107] |
| Fingerprint product routes / CLI graph/project | Retained exact symbol/load navigation still has its input | Function/XML fingerprint flags, manifests, exclusive implementation/fixtures. Already absent lineage/source-compare routes must not be recreated. | [#118][i118] |
| Store/Search administration | Surviving readers use current direct data; preserve transactions, bounds and failure atomicity | Orphan backup/quarantine/replacement APIs immediately after caller verification; used replay/recovery/retention/leases/journals only after their readers move. No destructive recovery over user data. | [#111][i111] |

Internal-format cleanup belongs to the affected row, not a separate versioning
project. Ordinary errors/results stay in their existing owners. Do not retain
unused compatibility signatures or delete tests merely to hide a failure.

## 6. Measure a real workflow

[#113][i113] measures the runnable source workflow from #106: selected revision,
file count, elapsed time, database size and the first demonstrated bottleneck.
Compare the same input after the fix; no new benchmark/telemetry platform.

[#125][i125] is the single final workflow-acceptance task: generated Library,
ordinary checker, editor/agent adapters and later-process source search/context.
Record exact command/framed input, revision and visible result under existing
traces. A compiler pass or closed planning Issue is not product acceptance.
After functional completion, add only the proportional coverage authorized by
the owner; do not revive old certification programmes.

## Issue routing and maintenance

Each open implementation Issue has a row above. #112 (cross-cutting formats),
#191 (umbrella plan) and #201 (duplicate workflow acceptance) are superseded by
this register, the per-owner cleanup rows and #125. Their closure means duplicate
planning was removed, not that their former runtime requirements were delivered.
History and source evidence stay available; do not reopen a parallel queue.

When adding a genuinely missing task, give it a consuming operation and a row
here. Change that row's prerequisite/outcome only when the handoff changes. Do not
copy numeric limits, configuration keys or error taxonomies out of owner guides.

[i103]: https://github.com/UnknownAlienHuman/WoW-Dev-Framework/issues/103
[i106]: https://github.com/UnknownAlienHuman/WoW-Dev-Framework/issues/106
[i107]: https://github.com/UnknownAlienHuman/WoW-Dev-Framework/issues/107
[i109]: https://github.com/UnknownAlienHuman/WoW-Dev-Framework/issues/109
[i111]: https://github.com/UnknownAlienHuman/WoW-Dev-Framework/issues/111
[i113]: https://github.com/UnknownAlienHuman/WoW-Dev-Framework/issues/113
[i117]: https://github.com/UnknownAlienHuman/WoW-Dev-Framework/issues/117
[i118]: https://github.com/UnknownAlienHuman/WoW-Dev-Framework/issues/118
[i122]: https://github.com/UnknownAlienHuman/WoW-Dev-Framework/issues/122
[i125]: https://github.com/UnknownAlienHuman/WoW-Dev-Framework/issues/125
[i131]: https://github.com/UnknownAlienHuman/WoW-Dev-Framework/issues/131
[i169]: https://github.com/UnknownAlienHuman/WoW-Dev-Framework/issues/169
[i194]: https://github.com/UnknownAlienHuman/WoW-Dev-Framework/issues/194
[i195]: https://github.com/UnknownAlienHuman/WoW-Dev-Framework/issues/195
[i197]: https://github.com/UnknownAlienHuman/WoW-Dev-Framework/issues/197
[i198]: https://github.com/UnknownAlienHuman/WoW-Dev-Framework/issues/198
[i199]: https://github.com/UnknownAlienHuman/WoW-Dev-Framework/issues/199
[i200]: https://github.com/UnknownAlienHuman/WoW-Dev-Framework/issues/200
[i273]: https://github.com/UnknownAlienHuman/WoW-Dev-Framework/issues/273
[i274]: https://github.com/UnknownAlienHuman/WoW-Dev-Framework/issues/274
[p266]: https://github.com/UnknownAlienHuman/WoW-Dev-Framework/pull/266
