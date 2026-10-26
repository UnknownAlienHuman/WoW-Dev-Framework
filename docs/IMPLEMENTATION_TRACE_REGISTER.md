# Post-merge implementation trace register

**Documentation baseline:** PR #244, merge `0fdc34c4a8afb1af51ec47d2038734d02500af42`
**Consistency review base:** `0784d0cb5f2c709fa7d6778f6ad7448df7a8cb77` (`main` after PR #256); PR #257 owns this review
**Runtime implementation reviewed through:** PR #250, `e40b64ffa1c16851fa657c0bd7fa0073e3cc33d9`
**Implementation work started:** 2026-10-24

This is the active addendum to [AUDIT_REGISTRY.md](AUDIT_REGISTRY.md). The old
register accounts for every baseline document and retained idea. This file tracks
only post-merge producer-to-consumer traces, implementation status and exact
observations. [CURRENT_STATUS.md](CURRENT_STATUS.md) is the concise checkpoint.

A source trace is not a user workflow. Documentation, compilation and Clippy can
move a row to **implemented**; only a recorded ordinary command or protocol
sequence can move it to **observed**.

## Status language

- **traced** — the named source chain supports the recorded decision;
- **partially implemented** — useful code exists, but its advertised handoff or
  visible result is incomplete;
- **implemented** — code exists in `main` and passed the named exact-head gate;
- **observed** — the ordinary workflow was exercised with exact input/result;
- **unexecuted** — no end-to-end observation exists.

## Active traces

| ID | Chain | Code status | User observation |
| --- | --- | --- | --- |
| T01 | public CLI check -> ordinary Service result | transitional generic route remains; replacement adapter open | unexecuted |
| T02 | ordinary config -> root/flavor/channel/TOC/Library/dependency selection | traced design; open | unexecuted |
| T03 | optional Interface + load context -> TOC/package admission | traced design; open | unexecuted |
| T04 | bounded disk/Library bytes -> exact snapshots | safe lower readers exist; direct selected acquisition open | unexecuted |
| T05 | selected XML facts -> mapped virtual Lua units | lower extraction/maps exist; pure direct handoff open | unexecuted |
| T06 | Main/Library/virtual -> one configured Emmy owner | implemented by PR #247; run #422 passed | unexecuted |
| T07 | compiled Emmy pin -> runtime identity | implemented by PR #249; run #440 passed | unexecuted |
| T08 | unsaved editor text -> configured project diagnostics | overlay exists; project reuse open | unexecuted |
| T09 | source/config changes -> coherent diagnostics/index | traced design; selected-load rebuild open | unexecuted |
| T10 | builder output -> ordinary Library metadata/maps | traced design; manifest reader open | unexecuted |
| T11 | addon chunk arguments -> mapped analysis context | traced design; open | unexecuted |
| T12 | upstream diagnostic -> framework DTO -> transitional LSP payload | implemented by PR #246; run #418 passed | unexecuted |
| T13 | caller diagnostic config -> optional semantic collectors | implemented by PR #247; run #422 passed | unexecuted |
| T14 | exact snapshots -> direct Service findings/limitations | implemented by PR #248; run #425 passed | unexecuted |
| T15 | compiled pin/report evidence -> legacy admission | implemented by PR #249; run #440 passed | unexecuted |
| T16 | requested Main snapshot -> configured-owner reuse | implemented by PR #250; run #445 passed | unexecuted |
| T17 | TOC metadata/file clauses -> typed package/per-unit execution context | source-traced; aliases/context handoff open | unexecuted |
| T18 | dependency/TOC/XML order -> snapshots + context-sensitive checks | source-traced; order lost at snapshot boundary | unexecuted |
| T19 | product channel + `LoadWith` -> package reachability/load trigger | source-traced; channel/trigger model open | unexecuted |
| T20 | implicit `Bindings*.xml` -> mapped binding callback diagnostics | source-traced; discovery/body/context adapter open | unexecuted |
| T21 | official zero Interface + unresolved `Secure` metadata -> compatibility/context admission | source-traced; current parser hard-rejects/blocks | unexecuted |
| T22 | ordinary chunk namespace + `AllowAddOnTableAccess` / `SuppressLocalTableRef` | official normal contract traced; retrieval permission missing; restricted suppression semantics unsupported | unexecuted |
| T23 | current TOC metadata -> issue classification/completeness | current known directives become blocking `UnknownDirective`; correction designed | unexecuted |

## Decisions and evidence

### T01–T05 — connect the ordinary path, not the old platform

The public path is:

```text
ordinary config
  -> selected TOC/dependencies/context and exact bounded source
  -> exact path-sorted Main/Library/virtual snapshots
  + validated selected-order/context handoff
  -> existing ConfiguredAnalyzerSession
  -> existing wow-service::addon_check
  -> CLI/LSP/MCP result adapters
```

Do not extend `LocalCommand -> LocalProjectBackend -> generic Service::check`,
fabricate Reference/Rules/publication objects, or wrap the direct result inside
the old component/capability/presentation result.

Ordinary configuration contains root, optional TOC, supported flavor, explicit
product channel where needed, optional target Interface, Library directories,
dependency roots, selected checks, Emmy diagnostic settings and failure
threshold. It never exposes Project IDs, generations, profile/report digests,
receipts or graph budgets.

Reuse `ProjectInputDirectory`, `ProjectInputFile`, Library discovery, existing
TOC/XML records and exact maps. Do not build `ProjectInputInventory` for the
direct route. Split pure selected XML preparation from publication receipts.
Preserve independent captured units when one included XML file fails.

### T06/T12–T16 — delivered lower owners

- PR #246 preserves configured diagnostic code, exact bounded message, upstream
  severity and byte range; standalone analysis initializes Lua 5.1 STD.
- PR #247 delivers `ConfiguredAnalyzerSession` over exact Main, Library and
  optional virtual snapshots with caller diagnostics and explicit semantic
  collection.
- PR #248 delivers `wow-service::addon_check` without Reference, Rules,
  publication, graph, store or presentation construction.
- PR #249 separates compiled Emmy provenance from compatibility-report evidence.
- PR #250 requires exact Main snapshot identity for immutable configured-owner
  reuse.

Their successful exact-head runs are #418, #422, #425, #440 and #445. These are
build gates, not addon/editor observations. Do not recreate these owners.

### T08/T09 — updates and editor state

Keep exact overlay text, client version and coordinate conversion. Pass exact
text overrides to the same configured project owner. Increasing LSP versions may
have gaps. A stale/rejected/cancelled candidate never publishes over newer text.

Use physical delta only while selected membership, ordered load/context,
Library identity and analyzer configuration remain exact. TOC/XML/dependency/
Library/config changes rebuild selection and owner cold. Removal deletes
associated virtual units, maps, diagnostics and index rows. Publish atomically.

### T10 — one optional generated-Library manifest

Keep `source-report.json` as the single optional producer/consumer DTO. Project
returns exact bounded bytes; Service validates the typed facts/maps needed by
implemented checks. Plain Lua Libraries remain valid for typing and limit only
checks that require the manifest. Do not add a converter, original-source
requirement or old native-artifact envelope.

### T11 — addon chunk context

Use one analysis-only helper/private table per supported ordinary addon and
replace only chunk-owned ellipsis expressions in memory. Compose exact source
maps and keep synthetic declarations/calls out of user diagnostics/API/flow
facts. External chunks, free inline scripts, reviewed handler adapters, suppressed
private tables, secure overrides and bootstrap contexts remain distinct.

### T17 — actual TOC execution facts

The reviewed finite dependency aliases are:

- required: `Dep`, `Dependencies`, `DependsOn`, `RequiredDep`, `RequiredDeps`;
- optional: `OptionalDep`, `OptionalDeps`.

Blizzard source establishes the ordinary `(addonName, addonTable)` chunk
contract, `AllowAddOnTableAccess` retrieval permission, and the occurrence of
`UseSecureEnvironment`, `LoadIntoEnvironment`, `SuppressLocalTableRef` and phase
directives. The reviewed Wowless implementation supplies only comparative
secure/bootstrap/load behavior. Carry exact source records and the narrow typed
facts consumed by the checker. `SuppressLocalTableRef` remains unsupported
restricted/internal context; donor `nil` behavior is not a client guarantee. If
a result depends on unverified parity, attach a scoped limitation. Do not add a
generic evidence/proof field. Cosmetic metadata does not make unrelated type
checking partial; an unknown context-changing directive remains located.

Reduce predicates by their operators: one known allow match includes; one known
exclude match excludes; across conjunction, known exclusion dominates unknown.
Retain unknown source evidence without poisoning a definitely excluded unit.
See [TOC_LOAD_CONTEXT_AUDIT.md](TOC_LOAD_CONTEXT_AUDIT.md).

### T18 — load order is not snapshot order

Package closure computes reachability/phases/source ordinals, but package/Main
projection and `LuaWorkspaceSnapshot` sort files by path. `addon_check` currently
receives no exact load-plan/context receipt.

Keep snapshots path-sorted and content-addressed for identity and analyzer
ownership. Pass a separate immutable validated value binding selected
package/TOC/XML order to snapshot role/path/digest, originating load record/unit,
addon identity, effective environment, ordinary or unsupported chunk-table
context, retrieval permission and phase. Reject missing, foreign, duplicate or
mismatched entries. Never call Emmy registration order a WoW runtime model.

### T19 — product channel and reverse load triggers

The pinned Blizzard source uses `OnlyBetaAndPTR: 1` and `LoadWith`. Current
`TocLoadContext` has no Stable/PTR/Beta channel; Project has no `LoadWith`
relation. Both directives become blocking `UnknownDirective` issues.

Add explicit product channel separate from flavor, Interface, game/glue and
secure/global context. The reviewed Wowless model treats
`OnlyBetaAndPTR: 1` as PTR/Beta included, Stable excluded and missing channel
unresolved. Use this as a provisional adapter and scope a limitation to dependent
results until client-authoritative semantics are established.

Model `LoadWith` as the reviewed donor-backed source-located reverse trigger, not
a required or optional dependency. It does not establish prior symbol
availability in the trigger package. Preserve source package, target package,
resolution state and order; scope a limitation to checks that depend on
unverified lifecycle timing. Explicitly selected source remains analyzable when
automatic reachability is unresolved.

Recognize `DefaultState` as bounded lifecycle metadata rather than a blocking
unknown directive. For an explicitly selected root it does not suppress static
analysis.

Current SavedVariables declarations already support account/per-character scope
and never open user data. Add the source-backed `Machine` declaration scope for
`SavedVariablesMachine`; do not infer schemas or initialize contents.

See [TOC_TRIGGER_BINDINGS_AUDIT.md](TOC_TRIGGER_BINDINGS_AUDIT.md).

### T20 — implicit binding callback source

A Blizzard addon can contain `Bindings.xml` without listing it in the TOC.
Blizzard source proves that `<Binding>` bodies contain Lua and use `keystate`;
the reviewed Wowless donor supplies the callback wrapper and candidate-selection
model. Current selected acquisition does not discover the sibling file, and the
XML index only extracts `Ui/Script` and `Ui/.../Scripts/*` bodies.

Use a bounded exact `Bindings<suffix>.xml` candidate list derived from reviewed
flavor/product donor rules. Select at most one; absence is normal and case or
same-precedence ambiguity is explicit. Capture through the held root and existing
file bounds. Until client-authoritative loader evidence exists, do not claim exact
discovery precedence or timing; scope a limitation to results that depend on it.

Parse binding bodies with the existing quick-xml/source-map ownership into mapped
callback units. A binding callback is neither a top-level TOC chunk nor an addon
ellipsis chunk. Bind it to addon identity, supported effective environment,
exact source and the reviewed source-and-donor-backed `keystate` adapter in a
separate callback phase. Malformed/unsupported binding scope is a located
limitation and preserves independent addon diagnostics.

See [TOC_TRIGGER_BINDINGS_AUDIT.md](TOC_TRIGGER_BINDINGS_AUDIT.md).

### T21 — official sentinel and unresolved security metadata

Pinned `Blizzard_ReforgingUI_Classic.toc` contains both `Interface: 0` and
`Secure: 1`. Current TOC parsing rejects zero as malformed and records `Secure`
as an unknown blocking directive. The reviewed Wowless parser retains the zero,
but no reviewed Project or donor source establishes that `Secure` means
`UseSecureEnvironment`, signing, taint mode or private-table policy.

**Decision:**

- retain zero as an explicit unknown/sentinel compatibility value rather than a
  syntax error. It never proves compatibility with a positive target and must not
  win automatic selection over a positive compatible candidate without a
  documented product rule. Explicit source analysis survives with a scoped
  compatibility limitation;
- retain `Secure` as typed source-backed metadata with raw value/span and an
  unsupported semantic state. Do not alias it by name to another directive;
- context-dependent checks remain limited where unresolved `Secure` semantics
  matter, while independent syntax/type diagnostics survive;
- bind both facts into the validated Project context/rebuild identity.

The focused evidence is in
[TOC_TRIGGER_BINDINGS_AUDIT.md](TOC_TRIGGER_BINDINGS_AUDIT.md).

### T22 — ordinary addon-table contract and unsupported legacy suppression

Pinned Blizzard `AddOnsDocumentation.lua` states that addon files receive the
addon table as the second result of `...`; the normal first result is the addon
name. It also states that `AllowAddOnTableAccess: 1` only opts this table into
retrieval through `C_AddOns.GetAddOnLocalTable`. Pinned
`Blizzard_CombatAudioAlerts.toc` uses that directive, but current Project code does
not recognize it.

Pinned `Blizzard_RecruitAFriend.toc` still contains restricted/internal
`SuppressLocalTableRef`. The reviewed Wowless loader responds by passing a `nil`
second value, but this comparative behavior does not establish current client
semantics and must not override the official ordinary-chunk contract.

**Decision:** ordinary external chunks use `(addonName, addonTable)`.
`AllowAddOnTableAccess` is a separate retrieval-permission fact. A selected chunk
under `SuppressLocalTableRef` remains unsupported special context: preserve the
raw directive, do not apply the ordinary helper or fabricate a table, and scope
only ellipsis/private-table-dependent conclusions as limited. No generic proof
DTO is introduced.

### T23 — known current metadata must not become a global blocking issue

Pinned source uses `LoadSavedVariablesFirst`, `AllowAddOnTableAccess` and
`EscalateErrorDuringLoad`. Current `toc.rs` recognizes none of them and classifies
all other included metadata as `UnknownDirective`; current issue projection marks
that blocking except for unresolved optional dependencies.

**Decision:** use one semantic metadata table and explicit nonblocking metadata
category. `LoadSavedVariablesFirst` is load-order metadata without user-data
access; `AllowAddOnTableAccess` is namespace retrieval permission;
`EscalateErrorDuringLoad` is known error-policy metadata. Localized display,
icon/category/group and `X-*` custom metadata stay nonblocking. Unknown package
metadata produces a located scoped limitation instead of invalidating enumerated
files; unknown inline/file directives remain unit-selection limitations.

## Documentation reconciliation ledger

| ID | Drift found after PR #244 | Resolution |
| --- | --- | --- |
| D01–D08 | delivered #246–#250 owners, stale links/run numbers and active instructions disagreed | corrected by PRs #251–#254 |
| D09 | ordinary-load guidance named only missing `Dep` and generic secure/bootstrap context | PR #255 added full aliases, tri-state conditions, typed context and selected-order handoff |
| D10 | product channel, `LoadWith`, implicit binding callback source and machine SavedVariables scope were absent | recorded as T19/T20 and in the focused trigger/bindings audit |
| D11 | official `Interface: 0` was still described as invalid and `Secure: 1` had no explicit uncertainty boundary | recorded as T21; preserve sentinel/raw metadata and scoped limitations |
| D12 | PR #256 updated the canonical Project boundary, but live summaries/status headers still pointed at PR #255 or omitted channel, triggers, bindings and validated order/context | post-#256 consistency pass: queue item 1 is the sole complete implementation checklist; CURRENT_STATUS reports delivery, the trace register records evidence, and README/AGENTS/package summaries may not narrow either |
| D13 | several special-load rules were written as client-authoritative source facts even though Blizzard source proves occurrence while Wowless supplies product-channel, secure/bootstrap, `LoadWith` and binding adapter behavior | preserve exact raw source and state donor assumptions directly; use scoped limitations instead of stronger client guarantees |
| D14 | status files embedded the exact “current main” SHA, so their own merge immediately made the value stale | record a named review base/PR, not the branch head containing the status document |
| D15 | the correction proposed propagating a generic “evidence class” through Project, Service, frontends and Issues | reject the new proof layer; keep raw source, narrow adapter behavior and ordinary scoped limitations only |
| D16 | `.ci/wow-emmy-syntax-bootstrap-passed` contained only `verified-bootstrap` and had no reader | remove the orphan ceremony marker; real repository checks remain in the workflow/xtask owners |
| D17 | chunk context was derived mainly from Wowless and omitted Blizzard's current `AllowAddOnTableAccess`/second-result contract | use pinned `AddOnsDocumentation.lua` for ordinary chunks; keep retrieval permission separate and `SuppressLocalTableRef` unsupported |
| D18 | `LoadSavedVariablesFirst`, `AllowAddOnTableAccess` and `EscalateErrorDuringLoad` were absent from the canonical plan although selected source uses them and current parsing makes them blocking unknowns | add T23 and effect-scoped metadata classification; do not make known/nonsemantic metadata globally partial |

Documentation/source inspection does not change runtime observation status.
The authority split in D12 is a maintenance decision, not another product layer.

## Observations still required

Record exact command/protocol input, source identities and result before marking
anything observed:

- public `wow check` with valid/invalid typed calls preserving code, message,
  severity and addon location;
- complete dependency aliases, missing required and absent optional dependency;
- known-determined predicates containing another unknown value;
- secure package/file overrides and exact selected order beside snapshot order;
- ordinary `(addonName, addonTable)` plus `AllowAddOnTableAccess` retrieval
  permission, and `SuppressLocalTableRef` remaining a scoped unsupported context;
- `LoadSavedVariablesFirst`/`EscalateErrorDuringLoad` without user-data reads or
  false global partial status;
- Stable exclusion plus PTR/Beta inclusion for `OnlyBetaAndPTR`;
- `LoadWith` trigger behavior without dependency semantics;
- reviewed-donor binding selection, mapped `keystate` callback diagnostics,
  normal no-bindings absence and malformed bindings preserving independent
  findings, without claiming unverified exact client timing;
- `DefaultState`/`SavedVariablesMachine` without false global partial or user-data
  access;
- explicit `Interface: 0` source preserving analysis with unknown compatibility;
- `Secure: 1` yielding a scoped unsupported-context limitation, not guessed
  execution semantics;
- generated manifest and plain Lua Library;
- physical, mapped XML, XML-only and malformed included XML;
- shared addon private table and one reviewed UI handler adapter with any scoped donor-parity limitation;
- coherent updates after source/TOC/XML/dependency/trigger/Library/channel/context
  change;
- unsaved LSP edit, version gap and stale-result rejection;
- MCP metadata/unknown notification plus useful result;
- later-process body/symbol search and readable exact source context;
- truthful cancellation at the granularity actually supported.

Update this register only when evidence changes a boundary or an exact workflow
changes status. Do not turn it into another product specification, percentage
dashboard or acceptance-tier platform.
