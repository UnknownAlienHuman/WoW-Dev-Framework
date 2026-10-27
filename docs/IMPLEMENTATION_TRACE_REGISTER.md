# Implementation and documentation trace register

**Baseline register:** [AUDIT_REGISTRY.md](AUDIT_REGISTRY.md), PR #244  
**Current audit base:** `main` after PR #265 (`231c15065563ee8443d4353282f395aa000e1a4d`)  
**Current traces:** T01–T31  
**Documentation corrections:** D01–D32

This file records post-baseline producer-to-consumer decisions and exact workflow
observations. It is evidence, not another product specification. Current delivery
state is summarized in [CURRENT_STATUS.md](CURRENT_STATUS.md); work order lives in
[REFACTOR_QUEUE.md](REFACTOR_QUEUE.md).

## Status language

- **traced** — named source producer, handoff and reader support the decision;
- **partially implemented** — useful code exists but the visible handoff is
  incomplete;
- **implemented** — code exists in `main` and passed the recorded code gate;
- **observed** — an ordinary command/protocol input and visible result were run;
- **unexecuted** — no end-to-end observation exists.

## Active traces

| ID | Chain | Code status | User observation |
| --- | --- | --- | --- |
| T01 | public `wow check` -> direct Service result | open; transitional generic route remains | unexecuted |
| T02 | ordinary config -> root/TOC/flavor/Library/dependency selection | traced; explicit first-slice adapter open | unexecuted |
| T03 | target/load context -> TOC/package admission | T30 predicates implemented; broader target/context open | unexecuted |
| T04 | bounded disk/Library bytes -> exact snapshots | lower readers exist; direct selected handoff open | unexecuted |
| T05 | selected XML -> mapped virtual Lua | extraction/maps exist; direct projection handoff open | unexecuted |
| T06 | Main/Library/virtual -> one configured Emmy owner | implemented by #247 | unexecuted |
| T07 | compiled Emmy pin -> runtime identity | implemented by #249 | unexecuted |
| T08 | unsaved text -> configured project diagnostics | overlay exists; shared owner open | unexecuted |
| T09 | source/config changes -> coherent diagnostics/index | traced; selected-load rebuild open | unexecuted |
| T10 | builder output -> ordinary Library metadata/maps | traced; manifest reader open | unexecuted |
| T11 | addon chunk arguments -> mapped analysis context | traced; open | unexecuted |
| T12 | upstream diagnostic -> framework/direct result | implemented by #246 | unexecuted |
| T13 | caller diagnostic config -> optional semantic collectors | implemented by #247 | unexecuted |
| T14 | exact snapshots -> direct findings/limitations | implemented by #248 | unexecuted |
| T15 | compiled identity vs report evidence | implemented by #249 | unexecuted |
| T16 | exact snapshots/config -> configured-owner reuse | implemented by #250; Library set is canonicalized before registration | unexecuted |
| T17 | TOC metadata/file clauses -> package/per-unit context | source-traced; metadata/context handoff open | unexecuted |
| T18 | selected load order -> snapshots plus validated context | source-traced; separate context open | unexecuted |
| T19 | product channel + `LoadWith` -> reachability/trigger | source-traced; open | unexecuted |
| T20 | implicit `Bindings*.xml` -> mapped callbacks | source-traced; open | unexecuted |
| T21 | source Interface zero + unsupported `Secure` -> admission | source-traced; open | unexecuted |
| T22 | ordinary chunk table + retrieval/suppression metadata | source-traced; open | unexecuted |
| T23 | TOC metadata -> issue class/completeness | correction designed; not implemented in `main` | unexecuted |
| T24 | virtual snapshot -> diagnostics without Project publication identity | implemented by #259 | unexecuted |
| T25 | direct result without snapshot/analysis receipts | implemented by #260 | unexecuted |
| T26 | selected-input limitation -> optional exact location | implemented by #261 | unexecuted |
| T27 | virtual diagnostic -> exact XML location | #262 hides unprojected findings; final projection open | unexecuted |
| T28 | Project issue -> finite direct limitation category | implemented by #263 | unexecuted |
| T29 | reachable retained load/package issues -> limitations | implemented by #264 | unexecuted |
| T30 | line/package predicates -> selected closure/completeness | implemented by #265; exact-head run #524 passed | unexecuted |
| T31 | requested `LuaTypes` facts -> completeness | current `check_addon` always adds chunk limitation and returns `Partial`; correction required | unexecuted |

## Durable decisions

### T01–T05 — first public path

The first command is a small explicit `LuaTypes` vertical slice:

```text
root + explicit TOC + flavor + Library
  -> bounded selected physical Lua
  -> exact Main/Library snapshots
  -> existing ConfiguredAnalyzerSession
  -> existing addon_check
  -> exact user locations and text/JSON output
```

Do not wait for product channel, `LoadWith`, implicit bindings, every XML handler
or a WoW rule. Add those in later slices. Unsupported selected input is a scoped
limitation; unavailable unrequested features are not global incompleteness.

### T06/T12–T16 — delivered lower owners

- #246 preserves configured code, exact message, severity and byte range.
- #247 delivers one Lua 5.1 Main/Library/virtual owner with STD and caller config.
- #248 delivers the direct Service result.
- #249 separates compiled provenance from report evidence.
- #250 requires exact Main compatibility.

`RegisteredMemberAnalysis::new` sorts Library snapshots by exact snapshot ID
before assigning roots and registering files. `ConfiguredAnalyzerSession::compatible`
compares the same order-insensitive snapshot-ID set. Library argument order is
therefore not an unresolved reuse defect.

### T08/T09 — updates

Use one configured owner. A physical delta is valid only while selected
membership, relevant context, Library set, virtual input and configuration remain
exact. TOC/XML/dependency/channel/context/Library changes rebuild selection and
owner cold. Failed/cancelled/stale work never publishes over newer input.

### T10 — generated Library manifest

Keep one optional typed `source-report.json` beside ordinary Lua. Plain Lua remains
valid for typing. Read only maps/facts consumed by implemented checks; do not add
another artifact envelope or converter.

### T11/T22/T31 — addon chunk context and completeness

Pinned Blizzard documentation supports ordinary `(addonName, addonTable)`.
`AllowAddOnTableAccess` is separate retrieval permission.
`SuppressLocalTableRef` remains unsupported special context; donor `nil` behavior
is not client authority.

Current `check_addon` has scope `LuaTypes` but unconditionally adds
`AddonChunkContextUnavailable` and sets `Partial`. Target completeness is based on
facts actually required by selected units. A file that never uses top-level `...`
must not force every type-only result partial. Context-dependent files remain
explicitly limited until the mapped helper exists.

### T17/T18 — load facts and order

Use finite dependency aliases and three-valued predicates. Keep exact selected
package/TOC/XML/callback order in a validated Project value beside path-sorted
snapshots. Emmy registration order is not a WoW runtime emulator.

### T19–T23 — special source context

- product channel is distinct from flavor/Interface/environment;
- `LoadWith` is a reverse trigger, not a dependency;
- binding callbacks are mapped callback units with reviewed `keystate`, not addon
  chunks;
- source `Interface: 0` is unknown compatibility, not a positive target;
- TOC `Secure` remains raw unsupported semantics;
- known lifecycle/persistence/error/retrieval/display/custom metadata does not
  make independent Lua enumeration partial;
- unknown metadata remains located Project information and limits only checks that
  consume it.

Focused evidence lives in [TOC_LOAD_CONTEXT_AUDIT.md](TOC_LOAD_CONTEXT_AUDIT.md)
and [TOC_TRIGGER_BINDINGS_AUDIT.md](TOC_TRIGGER_BINDINGS_AUDIT.md).

### T24–T30 — corrected direct boundary

- #259 removes Project generation from diagnostics-owner construction/reuse;
  generation remains only for explicit virtual semantic reports.
- #260 removes public snapshot/analysis receipts.
- #261 carries located input limitations.
- #262 withholds synthetic virtual locations.
- #263 provides finite limitation codes.
- #264 maps only the analyzed reachable package closure and retains Project-owned
  package-qualified locations where exact spans exist.
- #265 applies one three-valued predicate reduction and preserves excluded source
  evidence without false blocking completeness.

No item above is an observed addon workflow.

## Documentation reconciliation ledger

| ID | Drift/error | Resolution |
| --- | --- | --- |
| D01–D08 | delivered #246–#250 owners, stale links/run numbers and active instructions disagreed | reconciled by #251–#254 |
| D09 | dependency aliases, execution context and selected order were incomplete | source-traced as T17/T18 |
| D10 | product channel, reverse triggers, implicit bindings and machine SavedVariables were absent | source-traced as T19/T20 |
| D11 | official zero Interface and unresolved `Secure` were mishandled | source-traced as T21 |
| D12 | summaries omitted the post-#255 load handoff | corrected in post-#256 review |
| D13 | donor comparison was written as client authority | authority boundaries made explicit |
| D14 | status embedded an exact “current main” SHA and became stale by merging | use immutable review bases, not self-referential head claims |
| D15 | proposed generic evidence class would recreate a proof layer | rejected; keep raw source and scoped limitations |
| D16 | orphan CI marker had no reader | removed |
| D17 | chunk contract omitted Blizzard `AllowAddOnTableAccess` distinction | recorded as T22 |
| D18 | current known TOC metadata was globally blocking | recorded as T23 |
| D19 | virtual diagnostics required unused Project generation | corrected by #259 |
| D20 | direct result exposed internal receipts | corrected by #260 |
| D21 | input limitations lost exact location | corrected by #261 |
| D22 | public result leaked synthetic virtual paths | corrected by #262 |
| D23 | direct limitation codes could not classify Project outcomes | corrected by #263 |
| D24 | first mapper re-trusted roots and included unreachable packages | corrected by #264 |
| D25 | parser placeholders survived after semantic owners had newer outcomes | corrected by #264 |
| D26 | unknown predicates overrode known inclusion/exclusion | corrected by #265 |
| D27 | README, AGENTS, owner guides, package summaries and Issues copied live PR status and diverged | only current status/trace may enumerate delivered slices |
| D28 | first implementation slice again required nearly the whole Blizzard loader before one command | queue split into explicit `LuaTypes`, context and special-load slices |
| D29 | Emmy guide still tied configured virtual-owner compatibility to Project generation after #259 | generation restricted to explicit virtual semantic reports |
| D30 | active Issue audit called #194 reconciled although it required fields removed by #260; #200/#201 still listed T30 as open | affected Issues and audit corrected |
| D31 | a proposed Library-order defect ignored existing snapshot-ID sorting before registration | correction recorded; no new work item |
| D32 | docs accepted unconditional addon-chunk limitation and hard-coded `Partial` for every `LuaTypes` result | T31 added; completeness must be dependency-scoped |

## Observations still required

Do not mark a trace observed without exact input and visible result. Minimum
observations are:

- explicit physical-Lua `wow check`: valid and invalid typed call, exact
  code/message/severity/user path;
- relevant limitation without losing independent diagnostics;
- dependency and XML/chunk slices after they are implemented;
- generated and plain-Lua Library through the same check;
- coherent source/config updates;
- LSP unsaved/version/stale-result behavior;
- MCP metadata/unknown-notification behavior;
- later-process body/symbol search and readable exact context;
- truthful cancellation at supported checkpoints.

Update this register only when source evidence changes a boundary or an exact
workflow changes status. Do not turn it into a percentage dashboard or PR diary.
