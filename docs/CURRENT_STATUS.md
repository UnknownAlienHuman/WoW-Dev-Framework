# Current implementation status

**Consistency review base:** `0784d0cb5f2c709fa7d6778f6ad7448df7a8cb77` (`main` after PR #256)
**Runtime implementation review:** PR #259 owns T24; previous delivered slices are PRs #246–#250
**Documentation/source reconciliation:** PR #257, based on `main` after PR #256
**Documentation baseline:** PR #244, merge `0fdc34c4a8afb1af51ec47d2038734d02500af42`

This is the authoritative current-state checkpoint. It does not replace
[ARCHITECTURE.md](ARCHITECTURE.md), [IMPLEMENTATION.md](IMPLEMENTATION.md), or
the baseline [AUDIT.md](AUDIT.md). Runtime product wiring remains incomplete;
source-trace/documentation work changes implementation instructions but not
workflow observation status.

## Verification vocabulary

- **baseline reviewed** — the source/document claim was checked at the baseline;
- **traced** — a named producer, handoff and consumer support the recorded
  decision;
- **implemented** — code is in `main` and passed the named exact-head
  repository/format/compile/Clippy gate;
- **observed** — an ordinary command or protocol sequence was actually run with
  recorded input/result;
- **open** — the product handoff is absent, incomplete or transitional.

Build and Clippy never promote a workflow to **observed**.

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
raw run log was attached. Every named failure was separately traced at source
level. The blanket recommendation to delete eight tests was rejected because
several preserve useful admission, identity, bounds, privacy or lock-lifetime
behavior.

PRs #251–#254 reconciled active guides and all 21 open Issues with post-#250 code,
repaired local anchors and added repository validation for local Markdown
paths/anchors. PR #255 then closed deeper T02–T05/T11 specification gaps:
complete dependency aliases, selected load-context facts, proper three-valued
selection and a separate exact selected-order/context handoff beside path-sorted
snapshots.

PR #256 recorded the next special-load layer:

- explicit Stable/PTR/Beta channel for `OnlyBetaAndPTR`;
- typed `LoadWith` reverse triggers, distinct from dependencies;
- unlisted `Bindings*.xml` source plus the reviewed donor selection/callback
  model;
- known nonblocking `DefaultState` metadata;
- `SavedVariablesMachine` declaration scope without user-data reads;
- official `Interface: 0` source that current parsing incorrectly hard-rejects;
- official `Secure: 1` metadata whose exact semantics are not established by the
  reviewed Project or donor source and therefore must remain a scoped unsupported
  context rather than a guessed alias.

These findings are T19–T21 in
[IMPLEMENTATION_TRACE_REGISTER.md](IMPLEMENTATION_TRACE_REGISTER.md) and
[TOC_TRIGGER_BINDINGS_AUDIT.md](TOC_TRIGGER_BINDINGS_AUDIT.md).

PR #257 records seven documentation/maintenance defects after #256: active
summaries still exposed the pre-T17/T19 handoff; donor models had been promoted
into client claims; exact-current-head status fields became stale through their
own merge; a proposed generic “evidence class” handoff would have recreated a
proof layer; an orphan CI marker had no reader; the ordinary addon-table contract
and `AllowAddOnTableAccess` were omitted while `SuppressLocalTableRef` was
overclaimed; and current `LoadSavedVariablesFirst`/`EscalateErrorDuringLoad`
metadata was absent even though the parser turns it into blocking unknown issues.
T22–T23 and D12–D18 record the corrections. Exact raw source, narrow adapters and
scoped limitations remain; no runtime status changes from this documentation work.

PR #259 fixes the first post-#250 runtime boundary defect found during continued
review: direct mapped virtual diagnostics required `ProjectGenerationId` even
though the diagnostic collector used only the exact virtual snapshot. T24 keeps
configured-owner construction/reuse snapshot-owned and moves generation to the
optional virtual semantic-report call that actually consumes it. This does not
wire or observe the ordinary addon workflow.

## Implemented after the baseline

| Slice | Delivered behavior | Evidence | Runtime observation |
| --- | --- | --- | --- |
| T12 / PR #246 | configured Emmy diagnostics retain code, exact message, severity and byte range; standalone path initializes Lua 5.1 STD | run #418 passed | unexecuted |
| T06/T13 / PR #247 | one exact Main + Library + optional virtual configured owner with caller diagnostics, STD and explicit semantic collection | run #422 passed | unexecuted |
| T14 / PR #248 | direct `wow-service::addon_check` result without Reference, Rules, publication, graph, store or presentation platform | run #425 passed | unexecuted |
| T07/T15 / PR #249 | compiled provenance separated from compatibility-report evidence | run #440 passed | unexecuted |
| T16 / PR #250 | configured-owner reuse requires exact Main plus Library/virtual/config compatibility | run #445 passed | unexecuted |
| T24 / PR #259 | direct virtual diagnostics require only the exact snapshot; Project generation remains only for explicit virtual semantic facts | exact-head compile/Clippy gate required before merge | unexecuted |

These six internal building blocks exist after PR #259. No ordinary addon input
reaches them through the public command yet.

## Open path to the first usable command

```text
ordinary CLI/config
  -> root + flavor + channel + TOC/dependencies/triggers
  -> exact selected package/unit context and order
  -> implicit binding documents + bounded Main/Library/XML acquisition
  -> mapped virtual Lua preparation
  -> existing ConfiguredAnalyzerSession
  -> existing wow-service::addon_check
  -> text/JSON diagnostics, limitations and exit status
```

Open work includes:

- normal root/TOC/flavor/channel/Library configuration and ambiguity;
- a small load target independent of `ProfileIdentity`/publication receipts;
- complete dependency aliases and typed `LoadWith` triggers;
- `UseSecureEnvironment`, per-unit `LoadIntoEnvironment`, ordinary
  `(addonName, addonTable)` chunk context and `AllowAddOnTableAccess`;
  `SuppressLocalTableRef` remains unsupported legacy/internal metadata rather
  than a guessed suppression adapter;
- `LoadSavedVariablesFirst`, `EscalateErrorDuringLoad` and nonblocking display/
  localized/custom metadata without false global partial status;
- correct three-valued allow/exclude reduction;
- exact selected order/context beside path-sorted snapshots;
- Stable/PTR/Beta admission, official zero-Interface compatibility uncertainty,
  and unresolved `Secure` metadata;
- bounded exact `Bindings*.xml` discovery and callback context;
- known nonblocking `DefaultState` and machine SavedVariables declarations;
- independent-unit preservation after malformed included/bindings XML;
- public `wow check` wiring, rendering, exit and cancellation;
- XML-only Main and exact virtual projection;
- addon chunk/private table plus reviewed handler/binding adapters and scoped
  limitations where client parity is unverified;
- generated-Library manifest consumption;
- corrected `StrideIndex`, `NilableContents`, `MayReturnNothing`, mixins and
  member environment;
- semantic API binding and Secret uncertainty fixes before WoW rules;
- coherent updates after TOC/XML/bindings/dependency/trigger/Library/channel/
  context changes;
- project-aware LSP, standard empty results, MCP protocol behavior and direct
  current source search/context with real SQLite interruption.

[TOC_LOAD_CONTEXT_AUDIT.md](TOC_LOAD_CONTEXT_AUDIT.md) covers dependency aliases,
execution context, predicates and selected order.
[TOC_TRIGGER_BINDINGS_AUDIT.md](TOC_TRIGGER_BINDINGS_AUDIT.md) covers product
channel, reverse triggers, implicit bindings, known lifecycle/persistence
metadata and official unsupported compatibility/security facts.

`LuaWorkspaceSnapshot` remains path-sorted for exact identity; it is not runtime
order. `check_addon` consumes exact snapshots and truthfully returns `Partial`
because addon chunk context is missing; virtual input also lacks final source
projection/handler context. It no longer requires a Project generation merely to
collect virtual diagnostics.

## Readiness verdict

- **Baseline document reconciliation:** complete.
- **Current-guide/Issue reconciliation:** complete through PR #257; active
  summaries and Issue instructions use the canonical queue without a generic
  proof/evidence handoff. Local links and anchors are mechanically checked.
- **Baseline source architecture/error register:** complete for retained baseline
  requirements.
- **Current implementation tracing:** active; T17–T24 are traced, with T24
  implemented by PR #259 and the ordinary route still open.
- **Lower first-slice owners:** defined and partially implemented.
- **First ordinary user workflow:** not implemented or observed.
- **Ready for blind execution without producer-to-reader tracing:** no.

Do not reopen the old platform or repeat T06/T12–T16/T24. Continue from ordinary
acquisition/context/CLI and update the trace register whenever a boundary or
observed result changes.
