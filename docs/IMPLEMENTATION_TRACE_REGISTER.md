# Implementation and documentation trace register

**Baseline register:** [AUDIT_REGISTRY.md](AUDIT_REGISTRY.md), PR #244
**Current audit base:** `main` after PR #268 (`28302632599bed087394dee83751815d13041579`)
**Current traces:** T01–T35
**Documentation corrections:** D01–D46

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
| T32 | ordinary root/TOC/flavor -> profile-free target -> target-bound plan identity/validation | source-traced; current plan still stores mandatory Interface/profile digest | unexecuted |
| T33 | physical TOC plan -> scope-aware standalone limitations + preserved physical Main | source-traced; current loader eagerly parses XML and Service maps package plans only | unexecuted |
| T34 | admitted Main/Library -> compiled-pin snapshots + canonical result paths/input errors | source-traced; direct construction/error mapping open | unexecuted |
| T35 | empty physical Main + deferred selected units -> partial direct result without dummy source | source-traced; workspace/session accept empty Main, Project route open | unexecuted |

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

### T23/T32–T35 — first-slice Project admission

Current `read_toc_project*` and `ProjectLoadPlan` are both profile-bound. The plan
stores mandatory `target_interface` and `target_profile_digest`, and
`validate_profile` requires exact equality. T32 therefore replaces plan identity
and validation, not merely the function argument, with `AddonLoadTarget` plus an
explicit projection scope. Legacy callers adapt into that target; the direct
route never fabricates a profile.

T33 separates Slice 1A from recursive XML. Current `Loader::expand` opens and
parses every selected XML before returning Main; one malformed XML aborts physical
Lua. A `physical_lua` plan retains TOC XML records/spans but defers opening them,
marks selected XML scope incomplete and preserves direct Lua diagnostics. The
scope is part of plan identity.

The only public Project limitation mapper currently accepts
`ProjectPackageLoadPlan`; its private load mapper drops dependency placeholders
because package closure normally owns them. Slice 1A needs a standalone,
requested-scope mapper over `ProjectLoadPlan`. It must not equate generic
`blocks_complete`/`external_files_complete` with `LuaTypes` completeness.
Nonblocking Project observations never become `AddonCheckLimitation`.

T23 metadata-by-effect admission remains a prerequisite: cosmetic/custom and
known non-selection metadata does not block independent typing; malformed or
selection/context-changing directives remain exact errors/limitations.

T34 fixes the final direct handoff: compiled-pin backend; Main `Project` snapshot;
one `BlizzardUi` snapshot per Library directory under existing ceilings; reject
duplicate resolved roots/snapshot IDs; canonical root-relative result paths; and
classify caller Library parse failure as invalid input with its configured root
and relative file rather than `ComponentUnavailable`.

T35 records the source-supported empty-Main boundary. `LuaWorkspaceSnapshot::build`
and the configured owner do not require a Main file. When selected deferred XML
exists but Slice 1A has no physical Lua, use an empty `Project` snapshot plus the
located blocker and return `Partial`; never fabricate a dummy source or call the
result clean.

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
| D33 | the staged first slice omitted that all current TOC readers require legacy `ProfileIdentity` and a positive Interface | T32 added; extract one profile-free `AddonLoadTarget` and adapt legacy callers into it |
| D34 | metadata-by-effect classification was deferred behind the first command, so common display/`X-*` metadata could still cause false `Partial` | T23 is a Slice 1A prerequisite; special metadata semantics remain later |
| D35 | active #109/#112/#117/#197/#198/#199 bodies still copied PR progress or widened the immediate slice | bodies rewritten as durable outcomes and staged work |
| D36 | four baseline-register rows lacked the promised current `Home:` | errata recorded below; the immutable baseline register remains tied to its reviewed commit |
| D37 | `AddonLoadTarget` was documented only as a reader argument while `ProjectLoadPlan` identity and validation remained profile-digest/mandatory-Interface bound | T32 now replaces the retained plan identity boundary itself |
| D38 | Slice 1A claimed physical-only work but the reused loader eagerly opened/parsed XML and could abort before returning physical Main | T33 adds an identity-bound `physical_lua` projection with located deferred-XML limitation |
| D39 | Service exposed only package-plan limitations and dropped dependency placeholders; a standalone TOC check could lose required-dependency/load issues | T33 adds a scope-aware standalone plan mapper |
| D40 | generic `blocks_complete`/`external_files_complete` and direct limitations were not distinguished from nonblocking Project observations | only requested-scope blockers enter `AddonCheckLimitation` |
| D41 | snapshot backend/universe/per-directory Library/limit/path/error mapping was unspecified, inviting profile reuse or wrong internal-failure classification | T34 fixes the exact direct construction and error boundary |
| D42 | current status had become an audit diary and active Issues still contained stale dependency/checkpoint prose | status reduced to current truth; affected Issues reconciled |
| D43 | Slice 1A named a supported flavor without one canonical accepted value | accept exactly `retail`; add other flavors only in later explicit slices |
| D44 | empty physical Main under deferred XML had no exact outcome and invited dummy files or hard failure | T35 uses an empty Project snapshot plus a located partial limitation |
| D45 | multiple Library roots could produce duplicate snapshot IDs and ambiguous parse-error paths | reject duplicate roots/IDs and retain configured root + relative offending file |
| D46 | CLI listed exit codes but not precedence when partial results also contained findings | invalid/cancel/internal precede result; partial precedes threshold finding, then clean |

### Baseline register errata

The baseline register remains frozen as historical evidence. Three applicable
rows omitted the promised current-home pointer; one descriptive/not-applicable
row also benefits from an explicit navigation home. Their read/source conclusions
remain unchanged:

| Baseline row | Current home |
| --- | --- |
| `crates/wow-project/DATA_MODEL.md` | [updates](IMPLEMENTATION.md#updates), [results and interfaces](IMPLEMENTATION.md#results-and-interfaces), [useful output](WOW_LINTER.md#7-useful-output-and-updates) |
| `crates/wow-project/e2/REAL_ADDON_FIXTURE.md` | [completion and verification](IMPLEMENTATION.md#completion-and-verification), [queue](REFACTOR_QUEUE.md) |
| `crates/wow-project/e3/DATA_MODEL.md` | [addon configuration](IMPLEMENTATION.md#addon-configuration-and-emmy-analysis), [updates](IMPLEMENTATION.md#updates), [typed relationships](IMPLEMENTATION.md#typed-source-relationships) |
| `crates/wow-project/e3/SKELETON_INPUT_VIEW.md` | [source index/search/context](IMPLEMENTATION.md#blizzard-ui-index-search-and-context), [typed relationships](IMPLEMENTATION.md#typed-source-relationships) |

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
