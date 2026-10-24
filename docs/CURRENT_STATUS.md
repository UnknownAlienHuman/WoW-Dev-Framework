# Current implementation status

**Checked:** 2026-10-24  
**Implementation code reviewed through:** PR #250, `e40b64ffa1c16851fa657c0bd7fa0073e3cc33d9`  
**Documentation/source reconciliation reviewed through:** PR #255, `337b9b6e676af6735f416e8527176ed17e865172`  
**Documentation baseline:** PR #244, merge `0fdc34c4a8afb1af51ec47d2038734d02500af42`

This is the authoritative current-state checkpoint. It does not replace
[ARCHITECTURE.md](ARCHITECTURE.md), [IMPLEMENTATION.md](IMPLEMENTATION.md), or
the baseline [AUDIT.md](AUDIT.md). Runtime status remains the code delivered
through PR #250; later source-trace/documentation work changes the implementation
instructions, not delivered product behavior.

## Verification vocabulary

- **baseline reviewed** — the source/document claim was checked at the baseline;
- **traced** — a named producer, handoff and consumer support the recorded
  decision;
- **implemented** — code is in `main` and passed the named exact-head
  repository/format/compile/Clippy gate;
- **observed** — an ordinary command or protocol sequence was actually run with
  recorded input/result;
- **open** — the product handoff is absent, incomplete or transitional.

Build and Clippy evidence never promote a workflow to **observed**.

## Documentation and audit status

PR #244 completed the old-document reconciliation:

- all 329 baseline Markdown files were read;
- 287 applicable document requirements were traced to source owners/readers;
- 42 policy, notice, descriptive-data or retired-governance rows were marked not
  applicable;
- no baseline row remains `partial` or `not_checked`;
- superseded E1/E2/E3/platform specifications were removed from the active tree;
- useful behavior, donors, defects and removal boundaries were retained.

These figures describe document requirements, not Rust code coverage or runtime
qualification.

The supplied Claude review independently reported a full baseline test run of
1,085 passed, 13 failed and 23 ignored. Its diagnosis was used as input, but no
raw run log was attached to the repository. Every named failure was separately
traced at source level. The blanket recommendation to delete eight tests was not
accepted because several preserve useful admission, identity, bounds, privacy or
lock-lifetime behavior.

PRs #251–#254 reconciled the active guides and all 21 open Issue bodies with the
post-#250 code, repaired renamed local anchors and added repository validation for
local Markdown paths/anchors. PR #255 then closed deeper T02–T05/T11 gaps:
complete dependency aliases, secure/private-table execution facts, proper
three-valued selection and a separate exact selected-order/context handoff beside
path-sorted snapshots.

The follow-up trace found the next missing special-load layer:

- explicit stable/PTR/beta product channel for `OnlyBetaAndPTR`;
- typed `LoadWith` reverse triggers, distinct from dependencies;
- automatically selected `Bindings*.xml` and mapped `<Binding>` callback bodies;
- known nonblocking `DefaultState` metadata;
- `SavedVariablesMachine` declaration scope without reading user data.

These findings are recorded as T19/T20 in
[IMPLEMENTATION_TRACE_REGISTER.md](IMPLEMENTATION_TRACE_REGISTER.md) and in
[TOC_TRIGGER_BINDINGS_AUDIT.md](TOC_TRIGGER_BINDINGS_AUDIT.md).

## Implemented after the baseline

| Slice | Delivered behavior | Evidence | Runtime observation |
| --- | --- | --- | --- |
| T12 / PR #246 | configured Emmy diagnostics retain code, exact bounded message, upstream severity and byte range; standalone path initializes Lua 5.1 STD | exact-head run #418 passed | unexecuted as addon/editor workflow |
| T06/T13 / PR #247 | `ConfiguredAnalyzerSession` owns exact Main + Library + optional virtual analysis, caller diagnostics, one embedded STD and explicit semantic collection | run #422 passed | unexecuted |
| T14 / PR #248 | direct `wow-service::addon_check` result without Reference, Rules, Project publication, graph, store or presentation platform | run #425 passed | unexecuted |
| T07/T15 / PR #249 | compiled provenance is distinct from compatibility-report evidence | run #440 passed | unexecuted |
| T16 / PR #250 | configured-owner reuse requires exact Main plus Library/virtual/config compatibility | run #445 passed | unexecuted |

The lower language-check core exists. The public product path does not.

## Open path to the first usable command

```text
ordinary CLI/config
  -> root + flavor + product channel + TOC/dependencies/triggers
  -> exact selected package/unit context and order
  -> implicit binding documents + bounded Main/Library/XML acquisition
  -> mapped virtual Lua preparation
  -> existing ConfiguredAnalyzerSession
  -> existing wow-service::addon_check
  -> text/JSON diagnostics, limitations and exit status
```

The following remain open and must not be described as delivered:

- normal root/TOC/flavor/product-channel/Library configuration and ambiguity;
- small load target independent of `ProfileIdentity` and publication receipts;
- dependency aliases `Dep`, `Dependencies`, `DependsOn`, `RequiredDep`,
  `RequiredDeps`, `OptionalDep`, `OptionalDeps`;
- package/per-unit `UseSecureEnvironment`, `LoadIntoEnvironment`,
  `SuppressLocalTableRef`, bootstrap/load-first and supported phases;
- correct three-valued allow/exclude reduction;
- exact package/TOC/XML order/context receipt beside path-sorted snapshots;
- stable/PTR/beta admission and `LoadWith` trigger relations;
- bounded exact `Bindings*.xml` discovery and mapped binding callback context;
- known nonblocking `DefaultState` and machine SavedVariables declaration scope;
- preservation of independent units after one malformed included/bindings XML;
- public `wow check` wiring, rendering, exit policy and cancellation handoff;
- XML-only Main and exact virtual diagnostic projection;
- addon chunk `(addonName, privateTable)` and verified handler signatures;
- generated-Library manifest consumption;
- corrected `StrideIndex`, `NilableContents`, `MayReturnNothing`, mixins and
  member environment;
- semantic API binding and Secret uncertainty fixes before WoW rules;
- coherent updates after TOC/XML/dependency/trigger/Library/channel/context
  change;
- project-aware LSP, valid version gaps, stale-result rejection and standard empty
  results;
- MCP metadata/unknown notifications and common Service reuse;
- direct body/symbol SQLite index, later-process query, readable context and real
  SQL interruption.

[TOC_LOAD_CONTEXT_AUDIT.md](TOC_LOAD_CONTEXT_AUDIT.md) covers dependency aliases,
execution context, predicates and selected order. The follow-up
[TOC_TRIGGER_BINDINGS_AUDIT.md](TOC_TRIGGER_BINDINGS_AUDIT.md) covers product
channel, reverse triggers, implicit binding callbacks and known lifecycle/
persistence metadata.

`LuaWorkspaceSnapshot` remains correctly path-sorted for exact identity. It is
not a runtime order owner. `check_addon` currently consumes exact snapshots and
truthfully returns `Partial` because addon chunk context is missing; virtual input
also lacks final XML projection/handler context.

## Readiness verdict

- **Baseline documentation reconciliation:** complete.
- **Current-guide and active-Issue reconciliation:** complete through PR #255;
  local Markdown links/anchors are mechanically checked.
- **Baseline source architecture/error register:** complete for retained baseline
  requirements.
- **Current implementation tracing:** active; T17–T20 are source-traced and open.
- **Lower first-slice owners:** defined and partially implemented.
- **First ordinary user workflow:** not implemented or observed.
- **Ready for blind execution without producer-to-reader tracing:** no.

Do not reopen the old platform. Do not repeat T06/T12–T16. Continue from the
ordinary acquisition/context/CLI path and update the trace register whenever a
boundary or observed result changes.