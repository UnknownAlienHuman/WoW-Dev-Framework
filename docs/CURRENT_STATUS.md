# Current implementation status

**Documentation audit base:** `main` after PR #265 (`231c15065563ee8443d4353282f395aa000e1a4d`)  
**Baseline documentation audit:** PR #244 / `c36a058` source baseline  
**Active source decisions:** T01–T31 in
[IMPLEMENTATION_TRACE_REGISTER.md](IMPLEMENTATION_TRACE_REGISTER.md)

This is the only live delivery checkpoint. Domain guides define durable behavior;
the queue orders work; the trace register carries evidence. README files and
Issue summaries must link here instead of copying a moving implementation list.

## Status vocabulary

- **baseline reviewed** — the old document requirement was read and reconciled;
- **traced** — named producer, handoff and consumer support the recorded decision;
- **implemented** — code exists in `main` and passed its recorded code gate;
- **observed** — an ordinary command or protocol sequence was executed and its
  visible result recorded;
- **open** — the handoff is absent, incomplete or transitional.

Compilation, Clippy and source review never make a workflow **observed**.

## Documentation audit

The PR #244 baseline reconciliation remains valid:

- 329 baseline Markdown files were read;
- 287 applicable requirements were traced to source owners/readers;
- 42 policy, notice, descriptive-data or retired-governance rows were marked not
  applicable;
- no baseline row remains `partial` or `not_checked`;
- obsolete E1/E2/E3/platform specifications were removed from the active tree.

The supplied Claude audit was useful diagnosis, not a complete final audit. Its
named source defects were checked; its blanket recommendation to delete eight
failing tests was rejected because several still encode useful admission,
identity, bounds, privacy or lock-lifetime behavior. Its reported
`1085 passed / 13 failed / 23 ignored` run had no attached raw log and remains a
reported observation, not a reproduced one.

The follow-up audit found four current-document defects:

1. live implementation status had been copied into README, AGENTS, owner guides,
   package summaries, queue prose and Issues; several copies stopped at PR #250;
2. the first `wow check` slice had expanded back into almost the complete Blizzard
   load model, blocking a useful vertical result;
3. `ConfiguredAnalyzerSession` documentation still tied virtual owner reuse to a
   Project generation after PR #259 removed that dependency;
4. the active Issue audit still called Issue #194 reconciled although its body
   required snapshot/analysis IDs removed by PR #260, and #200/#201 still listed
   the T30 predicate fix as unfinished after PR #265.

Those are documentation errors, not additional product requirements. The active
docs now keep moving status here and keep owner guides stable.

## Implemented internal owners

| Trace | Delivered behavior | User workflow |
| --- | --- | --- |
| T12 / #246 | configured Emmy diagnostics preserve code, exact message, severity and byte range; standalone Lua 5.1 STD is initialized | unexecuted |
| T06/T13 / #247 | one exact Main + explicit Library + optional virtual configured Emmy owner; diagnostics and optional semantic collection share it | unexecuted |
| T14 / #248 | direct `wow-service::addon_check` result without Reference, Rules, Project publication, graph, store or presentation construction | unexecuted |
| T07/T15 / #249 | compiled Emmy identity is separated from compatibility-report evidence | unexecuted |
| T16 / #250 | immutable owner reuse requires exact Main plus canonical Library/virtual/config compatibility | unexecuted |
| T24 / #259 | direct virtual diagnostics are snapshot-owned; Project generation is required only by explicit virtual semantic reports | unexecuted |
| T25 / #260 | direct public result contains diagnostics and limitations, not snapshot or upstream-analysis receipts | unexecuted |
| T26 / #261 | exact input limitations can carry an optional validated source location | unexecuted |
| T27 / #262 | unprojected virtual findings remain internal rather than exposing synthetic workspace locations | unexecuted |
| T28 / #263 | one finite direct limitation-code surface covers Project outcomes | unexecuted |
| T29 / #264 | reachable retained Project load/package issues map to that surface without raw-root rebinding or unreachable-package false partial status | unexecuted |
| T30 / #265 | TOC line/package predicates use one three-valued reduction; definitely excluded source no longer poisons completeness | unexecuted |

The canonical Library order concern was rechecked and is **not** an open defect:
`RegisteredMemberAnalysis::new` sorts Library snapshots by exact snapshot ID before
workspace-root assignment and registration; `ConfiguredAnalyzerSession::compatible`
compares the same order-insensitive set.

## Open product boundary

There is still no ordinary public addon checker. The application uses the
transitional `LocalCommand -> LocalProjectBackend -> generic Service` route.

The existing direct result implements only `AddonCheckScope::LuaTypes`, but
`check_addon` currently adds `AddonChunkContextUnavailable` and sets
`completeness = Partial` unconditionally. That is conservative internal behavior,
not the target contract. Completeness must depend on facts required by the
selected scope and selected units; unused chunk arguments or unavailable
unrequested WoW features cannot make every type-only run partial.

The next work is deliberately split:

1. **First usable `wow check`:** explicit root, explicit TOC, flavor, one or more
   Library directories, bounded selected physical Lua, exact Main/Library
   snapshots, existing `check_addon`, exact path projection, text/JSON output and
   exit policy. No WoW rule by default.
2. **Selected-context expansion:** dependencies, source-order/context receipt,
   source-projectable XML units, addon chunk arguments where used, XML handler
   adapters and coherent updates. Unsupported selected context remains a located
   limitation without suppressing independent diagnostics.
3. **Special loading:** product channel, `LoadWith`, implicit bindings,
   zero-Interface compatibility, secure/special contexts and remaining metadata.
   These must not block the first type-check command when the selected input does
   not require them.
4. **WoW checks:** generated-Library metadata consumption, corrected Blizzard
   projection semantics and direct selected API/Secret rules.
5. **Adapters and source tooling:** project-aware LSP/MCP, then the direct SQLite
   body/symbol index and readable context.

The exact order and acceptance conditions are in
[REFACTOR_QUEUE.md](REFACTOR_QUEUE.md).

## Readiness verdict

- **Baseline document reconciliation:** complete.
- **Current documentation consistency audit:** complete for the active canonical
  docs and 21 open implementation Issues at the named base; corrections are
  recorded in the trace and Issue audit.
- **Internal language-check owners:** partially implemented.
- **First ordinary `wow check`:** open.
- **Any addon/editor/search workflow observed:** no.
- **Project ready for blind implementation from an abbreviated README/Issue:** no;
  use the queue, owner guide and trace together.
