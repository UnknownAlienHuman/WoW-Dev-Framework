# Implementation and documentation trace register

**Baseline register:** [AUDIT_REGISTRY.md](AUDIT_REGISTRY.md), PR #244  
**Current audit base:** `main` after PR #269
(`f674ccc7615d63c76c0781aef2150e49ea442bf8`)  
**Current traces:** T01–T39  
**Documentation corrections:** D01–D54

This file records post-baseline producer-to-consumer decisions and exact workflow
observations. It is evidence, not another product specification. Current delivery
state is summarized in [CURRENT_STATUS.md](CURRENT_STATUS.md); work order lives in
[REFACTOR_QUEUE.md](REFACTOR_QUEUE.md).

## Status language

- **traced** — named producer, handoff and reader support the decision;
- **partially implemented** — useful code exists but the visible handoff is incomplete;
- **implemented** — code exists in `main` and passed its recorded code gate;
- **observed** — an ordinary command/protocol input and visible result were run;
- **unexecuted** — no end-to-end observation exists.

## Active traces

| ID | Chain | Code status | Observation |
| --- | --- | --- | --- |
| T01 | public `wow check` -> direct Service result | open; transitional generic route remains | unexecuted |
| T02 | ordinary config -> root/TOC/flavor/Library selection | traced; strict adapter open | unexecuted |
| T03 | target/load context -> TOC/package admission | predicates implemented; broader context open | unexecuted |
| T04 | bounded disk/Library bytes -> exact snapshots | lower readers exist; direct handoff open | unexecuted |
| T05 | selected XML -> mapped virtual Lua | extraction/maps exist; direct projection open | unexecuted |
| T06 | Main/Library/virtual -> one configured Emmy owner | implemented by #247 | unexecuted |
| T07 | compiled Emmy pin -> runtime identity | implemented by #249 | unexecuted |
| T08 | unsaved text -> configured project diagnostics | overlay exists; shared owner open | unexecuted |
| T09 | source/config changes -> coherent diagnostics/index | traced; selected-load rebuild open | unexecuted |
| T10 | builder output -> ordinary Library metadata/maps | traced; manifest reader open | unexecuted |
| T11 | addon chunk arguments -> mapped analysis context | traced; open | unexecuted |
| T12 | upstream diagnostic -> direct result | implemented by #246 | unexecuted |
| T13 | diagnostic config -> optional semantic collection | implemented by #247 | unexecuted |
| T14 | exact snapshots -> direct findings/limitations | implemented by #248 | unexecuted |
| T15 | compiled identity vs report evidence | implemented by #249 | unexecuted |
| T16 | exact snapshots/config -> owner reuse | implemented by #250; Library set canonicalized | unexecuted |
| T17 | TOC metadata/file clauses -> package/unit context | source-traced; handoff open | unexecuted |
| T18 | selected load order -> snapshots + validated context | source-traced; context open | unexecuted |
| T19 | product channel + `LoadWith` -> reachability/trigger | source-traced; open | unexecuted |
| T20 | implicit `Bindings*.xml` -> mapped callbacks | source-traced; open | unexecuted |
| T21 | Interface zero + unsupported `Secure` -> admission | source-traced; open | unexecuted |
| T22 | chunk table + retrieval/suppression metadata | source-traced; open | unexecuted |
| T23 | TOC metadata -> issue class/completeness | designed; not implemented in `main` | unexecuted |
| T24 | virtual snapshot -> diagnostics without Project publication identity | implemented by #259 | unexecuted |
| T25 | direct result without snapshot/analysis receipts | implemented by #260 | unexecuted |
| T26 | selected-input limitation -> optional exact location | implemented by #261 | unexecuted |
| T27 | virtual diagnostic -> exact XML location | #262 hides unprojected findings; projection open | unexecuted |
| T28 | Project issue -> finite direct limitation category | implemented by #263 | unexecuted |
| T29 | retained reachable package issues -> limitations | implemented by #264 | unexecuted |
| T30 | line/package predicates -> closure/completeness | implemented by #265 | unexecuted |
| T31 | requested `LuaTypes` facts -> completeness | unconditional chunk limitation/`Partial` remains | unexecuted |
| T32 | ordinary root/TOC/flavor -> profile-free target-bound plan | source-traced; current plan still profile-bound | unexecuted |
| T33 | physical plan -> standalone scope blockers + physical Main | source-traced; current loader opens XML/package mapper only | unexecuted |
| T34 | admitted Main/Library -> compiled-pin snapshots + canonical errors/paths | source-traced; direct construction open | unexecuted |
| T35 | empty physical Main + deferred units -> partial without dummy source | source-traced; direct route open | unexecuted |
| T36 | resolved roots + admitted Lua -> collision-free role/root authority | source-traced; cross-role authority validation open | unexecuted |
| T37 | ordinary config -> one Service-owned prepared-addon operation -> lower check | source-traced; trusted assembly open | unexecuted |
| T38 | file failure -> structured invalid-input origin -> root/relative path | source-traced; origin currently flattened | unexecuted |
| T39 | strict CLI config + result/error streams -> deterministic exit | source-traced; direct frontend open | unexecuted |

## Durable decisions

### T01–T05 — first public path

```text
root + explicit TOC + retail + Library roots
  -> one public Service ordinary-check operation
  -> target/scope-bound Project input + requested blockers
  -> collision-free exact Main/Library snapshots
  -> existing ConfiguredAnalyzerSession / low-level check_addon
  -> root-relative text/JSON output
```

Do not wait for channel, `LoadWith`, bindings, complete XML context or WoW rules.
Unavailable unrequested features are not global incompleteness.

### T23/T31–T35 — Project admission and completeness

- Replace `ProjectLoadPlan`'s mandatory Interface/profile digest and
  `validate_profile` boundary with `AddonLoadTarget` plus projection scope.
  Legacy callers adapt profiles into the target; the direct path never fabricates
  one.
- `physical_lua` retains TOC XML records/spans but does not open XML. Deferred XML
  is a located limitation and cannot erase physical Lua diagnostics.
- Add a requested-scope mapper over one `ProjectLoadPlan`; package-only mapping and
  global `blocks_complete`/`external_files_complete()` are not substitutes.
- Cosmetic/custom and known non-selection metadata stays Project observation;
  malformed or selection/context-changing input stays invalid/limited.
- Use compiled pin, Main universe `Project`, one `BlizzardUi` snapshot per Library
  root, retained ceilings and root-relative result paths.
- Empty physical Main with selected deferred XML uses an empty Project snapshot
  plus a located partial limitation, not a dummy file or clean result.
- Remove unconditional addon-chunk incompleteness. `LuaTypes` completeness follows
  only facts required by selected units.

### T36 — input-root and admitted-file authority

Snapshot IDs do not prove host-file separation: universe changes can give one
physical file different Main/Library IDs, while overlapping recursive Library
roots can share files without equal whole-snapshot IDs.

Project retains a nonsemantic root token plus admitted relative path and rejects:

- duplicate resolved roots;
- duplicate Library snapshot IDs;
- the same admitted physical Lua file in Main and Library, or in two Libraries;
- an explicit Library root that yields no admitted Lua files.

Do not reject root ancestry alone. Nested roots are valid when admitted file sets
are disjoint. Equal relative paths in distinct roots are valid when they are
distinct admitted files. Reuse component-wise no-follow root opening,
case/UTF-8/language checks and existing byte/file ceilings.

### T37 — one trusted Service assembly boundary

The lower `AddonCheckRequest::new` plus `with_input_limitations` can combine
arbitrary snapshots and unrelated blockers. There is no production caller proving
correct assembly. Add one narrow public Service use-case operation that owns:

1. Project target/scope preparation and exact acquisition;
2. plan/Main and root/file-authority validation;
3. requested-scope limitation projection;
4. snapshot construction and snapshot-ID -> configured-root mapping;
5. lower request construction and `check_addon` invocation.

CLI/LSP/MCP call only this operation. They do not open source, build snapshots or
attach limitations. Keep the exact-snapshot constructor/check internal once
callers move. This is a direct function, not a generic backend or registry.

### T38 — structured input-error origin

Current Library health failure keeps only a relative path, loses it at the
configured-session boundary and becomes internal `ComponentUnavailable`. Carry:

- source role;
- snapshot/root token;
- root-relative file;
- stable error code and bounded message.

Service maps the snapshot ID to the configured Library root and returns invalid
caller input. Project attaches relative paths to every file-specific read, UTF-8,
NUL, language, digest/length or admission error. Never parse message text or
synthetic analyzer paths to recover origin.

### T39 — strict config and output/error streams

Read one explicit bounded config, reject unknown fields and duplicate scalar
options, and make repeated CLI `--library` replace the configured array. There is
no implicit config discovery.

- Complete/partial `AddonCheckResult`: encode once on stdout.
- Input/cancel/internal failure after format admission: stderr; under JSON, one
  bounded typed object and empty stdout.
- CLI syntax/option failure before format admission: one text usage error on
  stderr.
- Output failure: exit `4`, no retry and no second envelope.

Exit precedence: invalid input `64`; observed cancellation `130`; internal/output
failure `4`; partial result `2`; threshold finding `1`; otherwise `0`.

### Delivered lower owners: T06/T12–T16 and T24–T30

- #246 preserves diagnostic code/message/severity/range.
- #247 provides one Lua 5.1 Main/Library/virtual owner with STD and caller config.
- #248 provides the lower direct result.
- #249 separates compiled provenance from report evidence.
- #250 gates reuse on exact Main; Library snapshots are sorted by exact ID before
  registration and compared as the same canonical set.
- #259–#265 remove generation from diagnostics ownership, remove public receipts,
  retain located limitations, hide unprojected virtual paths, define finite codes,
  map the retained reachable package closure and fix three-valued predicates.

None is an observed addon workflow.

### Later retained decisions

- T08/T09: reuse one owner only while membership/context/Library/virtual/config are
  exact; relevant selection changes rebuild cold and stale work never publishes.
- T10: keep one optional typed `source-report.json`; plain Lua remains valid.
- T11/T22: ordinary chunk context, retrieval permission and special/suppressed
  contexts are distinct.
- T17/T18: exact WoW order/context is a validated Project value beside path-sorted
  analysis snapshots.
- T19–T23: channel, triggers, bindings, Interface zero, raw unsupported `Secure`
  and metadata classes remain distinct and scoped.

## Documentation reconciliation ledger

| ID | Drift/error | Resolution |
| --- | --- | --- |
| D01–D08 | delivered #246–#250 owners, links/runs and instructions disagreed | reconciled by #251–#254 |
| D09–D11 | dependency/context/order, channel/triggers/bindings, zero Interface/`Secure` incomplete | traced as T17–T21 |
| D12–D18 | summaries, donor authority, self-staling status, generic evidence, orphan marker, chunk retrieval and metadata classes | corrected; T22/T23 added |
| D19–D26 | generation, receipts, locations, virtual paths, limitation codes, root trust/placeholders and predicates | corrected by #259–#265 |
| D27 | live status copied into many guides/Issues | only current status/trace enumerate delivery |
| D28 | first slice again required most of Blizzard loading | split into physical, selected-context and special-load slices |
| D29 | Emmy guide still required Project generation | generation restricted to explicit virtual semantic reports |
| D30 | active Issues required removed receipts/repeated delivered T30 | affected Issues corrected |
| D31 | proposed Library-order defect ignored existing canonical sorting | rejected; no work item |
| D32 | docs accepted unconditional chunk limitation/`Partial` | T31 added |
| D33 | first slice hid legacy profile/positive-Interface requirement | T32 added |
| D34 | metadata classification came after first result | T23 made Slice 1A prerequisite |
| D35 | active Issue bodies copied progress/widened immediate slice | rewritten as durable staged work |
| D36 | four baseline rows lacked current `Home:` | errata recorded below |
| D37 | only reader argument changed while retained plan stayed profile-bound | T32 replaces plan identity/validation |
| D38 | physical slice reused eager XML loader | T33 adds identity-bound `physical_lua` |
| D39 | package mapper could drop standalone dependency/load blockers | standalone scope mapper required |
| D40 | global completeness was conflated with requested-scope blockers | observations and limitations separated |
| D41 | snapshot/universe/limits/paths/error mapping underspecified | T34 fixes exact handoff |
| D42 | status became audit diary and Issues retained stale dependencies | status/Issues reduced to current truth |
| D43 | first-slice flavor was not canonical | accept exactly `retail` |
| D44 | empty physical Main invited dummy/hard failure | T35 defines empty snapshot + partial |
| D45 | duplicate roots/IDs and Library error paths underspecified | reject duplicates; retain root + file |
| D46 | exit precedence was unspecified | T39 defines deterministic order |
| D47 | duplicate roots/IDs did not catch the same physical Lua across roles | T36 validates admitted file authority |
| D48 | blanket nested-root rejection would reject valid disjoint generated Libraries | reject file-set intersection, not ancestry |
| D49 | frontend could combine unrelated plan limitations and snapshots | T37 creates Service-owned assembly |
| D50 | Library failure lost root/snapshot origin and became internal failure | T38 carries structured origin |
| D51 | Project file-specific errors dropped relative path | T38 requires path attachment |
| D52 | config unknown fields/nonempty admitted Library roots underspecified | T36/T39 require strict config/admission |
| D53 | result vs pre-result JSON streams unspecified | T39 defines stdout/stderr contract |
| D54 | #191 copied moving delivery state and #195 presented old caller census as current | affected Issues require live status/caller recheck |

### Baseline register errata

The baseline register remains historical. These rows retain their original
read/source conclusions and use these current homes:

| Baseline row | Current home |
| --- | --- |
| `crates/wow-project/DATA_MODEL.md` | [updates](IMPLEMENTATION.md#updates), [results](IMPLEMENTATION.md#results-and-interfaces) |
| `crates/wow-project/e2/REAL_ADDON_FIXTURE.md` | [completion](IMPLEMENTATION.md#completion-and-verification), [queue](REFACTOR_QUEUE.md) |
| `crates/wow-project/e3/DATA_MODEL.md` | [addon configuration](IMPLEMENTATION.md#addon-configuration-and-emmy-analysis), [typed relationships](IMPLEMENTATION.md#typed-source-relationships) |
| `crates/wow-project/e3/SKELETON_INPUT_VIEW.md` | [source index](IMPLEMENTATION.md#blizzard-ui-index-search-and-context), [typed relationships](IMPLEMENTATION.md#typed-source-relationships) |

## Observations still required

Do not mark a trace observed without exact input and visible result. Minimum first
observations include:

- valid and invalid typed calls with exact root-relative code/message/severity/range;
- partial required-dependency or deferred-XML result preserving diagnostics;
- XML-only physical slice through empty Main;
- malformed Library with structured root + relative file;
- duplicate/same-file role collision rejected, nested disjoint roots accepted, and
  equal relative paths in distinct roots accepted;
- text/JSON result/error channels and deterministic exits;
- later dependency/XML/chunk, updates, LSP/MCP, source search/context and truthful
  cancellation scenarios.

Update this register only when source evidence changes a boundary or an exact
workflow changes status. Do not turn it into a percentage dashboard or PR diary.
