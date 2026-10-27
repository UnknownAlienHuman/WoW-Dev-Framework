# Implementation and documentation trace register

**Baseline register:** [AUDIT_REGISTRY.md](AUDIT_REGISTRY.md), PR #244  
**Current audit base:** `main` after PR #269
(`f674ccc7615d63c76c0781aef2150e49ea442bf8`)  
**Current traces:** T01–T39  
**Documentation corrections:** D01–D80

This file records producer-to-consumer decisions and exact observations. It is
evidence, not another product specification. Current delivery state is in
[CURRENT_STATUS.md](CURRENT_STATUS.md); work order is in
[REFACTOR_QUEUE.md](REFACTOR_QUEUE.md).

## Status language

- **traced** — producer, handoff and reader were inspected;
- **partially implemented** — useful code exists, visible handoff incomplete;
- **implemented** — code exists in `main` and passed its recorded code gate;
- **observed** — ordinary input/protocol and visible result were executed;
- **unexecuted** — no end-to-end observation exists.

## Active traces

| ID | Chain | Code status | Observation |
| --- | --- | --- | --- |
| T01 | public `wow check` -> direct Service result | open; legacy route remains | unexecuted |
| T02 | ordinary config -> root/TOC/flavor/Library selection | traced; strict adapter open | unexecuted |
| T03 | target/context -> TOC/package admission | predicates implemented; context open | unexecuted |
| T04 | bounded disk/Library bytes -> exact snapshots | lower readers exist; direct handoff open | unexecuted |
| T05 | selected XML -> mapped virtual Lua | extraction/maps exist; direct projection open | unexecuted |
| T06 | Main/Library/virtual -> one configured Emmy owner | implemented #247 | unexecuted |
| T07 | compiled Emmy pin -> runtime identity | implemented #249 | unexecuted |
| T08 | unsaved text -> configured project diagnostics | overlay exists; shared owner open | unexecuted |
| T09 | source/config changes -> coherent diagnostics/index | traced; rebuild route open | unexecuted |
| T10 | builder output -> ordinary Library metadata/maps | traced; manifest reader open | unexecuted |
| T11 | addon chunk arguments -> mapped context | traced; injection open | unexecuted |
| T12 | upstream diagnostic -> direct result | implemented #246 | unexecuted |
| T13 | diagnostic config -> selected collection | implemented #247; strict CLI admission open | unexecuted |
| T14 | exact snapshots -> direct findings/limitations | implemented #248 | unexecuted |
| T15 | compiled identity vs report evidence | implemented #249 | unexecuted |
| T16 | exact snapshots/config -> owner reuse | implemented #250 | unexecuted |
| T17 | TOC metadata/file clauses -> package/unit context | traced; handoff open | unexecuted |
| T18 | selected order -> snapshots + validated context | traced; context open | unexecuted |
| T19 | channel + `LoadWith` -> reachability/trigger | traced; open | unexecuted |
| T20 | implicit bindings -> mapped callbacks | traced; open | unexecuted |
| T21 | Interface zero + unsupported `Secure` -> admission | traced; open | unexecuted |
| T22 | chunk table + retrieval/suppression metadata | traced; open | unexecuted |
| T23 | TOC metadata -> issue class/completeness | designed; draft #266, not in `main` | unexecuted |
| T24 | virtual diagnostics without Project publication identity | implemented #259 | unexecuted |
| T25 | direct result without snapshot/analysis receipts | implemented #260 | unexecuted |
| T26 | input limitation -> optional exact location | implemented #261 | unexecuted |
| T27 | virtual diagnostic -> exact XML location | #262 hides unprojected findings; projection open | unexecuted |
| T28 | Project issue -> finite direct limitation code | implemented #263 | unexecuted |
| T29 | retained reachable package issues -> limitations | implemented #264 | unexecuted |
| T30 | line/package predicates -> closure/completeness | implemented #265 | unexecuted |
| T31 | selected Main requirements -> scope completeness | source-traced; unconditional chunk fallback remains | unexecuted |
| T32 | ordinary target -> profile-free target/scope plan | source-traced; current plan profile-bound | unexecuted |
| T33 | physical plan -> standalone blockers + Main | source-traced; current loader opens XML/package mapper only | unexecuted |
| T34 | admitted Main/Libraries -> exact snapshots/paths/errors | source-traced; construction open | unexecuted |
| T35 | empty Main + deferred units -> partial without dummy | source-traced; route open | unexecuted |
| T36 | opened roots + admitted paths -> collision-free authority | source-traced; authority validation open | unexecuted |
| T37 | ordinary config -> one Service-owned assembly -> lower check | source-traced; operation open | unexecuted |
| T38 | file failure -> structured operation error | source-traced; origin currently flattened | unexecuted |
| T39 | strict config/result/error streams -> deterministic exit | source-traced; frontend open | unexecuted |

## Durable Slice 1A decisions

### T23/T32–T35 — Project selection and scope

- Replace the retained plan's mandatory Interface/profile digest with
  `AddonLoadTarget + projection scope`; legacy callers adapt a real profile, the
  direct path never fabricates one.
- `physical_lua` parses TOC/direct Lua, retains XML records/spans and does not open
  XML. Deferred XML is a located blocker and cannot erase physical diagnostics.
- Add a standalone `ProjectLoadPlan + AddonCheckScope -> limitations` mapper in
  Service. Package-only mapping and global completeness are not substitutes.
- Cosmetic/custom/non-selection metadata remains Project observation; malformed
  or selected-context-changing input remains invalid/limited.
- Use compiled pin, Main `Project`, one `BlizzardUi` snapshot per Library root and
  canonical role-relative paths.
- Empty Main is supported: complete when nothing is selected; partial when
  deferred selected input exists. No dummy source.

### T31 — same-owner chunk requirement

Current `check_addon` adds chunk incompleteness to every run. Add one bounded
collector to the same configured Emmy owner. Walk registered Main expression AST,
match `LuaLiteralExpr`/`LuaLiteralToken::Dots`, exclude any match inside a
`LuaClosureExpr`, retain the lowest-starting exact path/content/span per affected
file, and validate it against exact Main in Service. No raw scan or second parser.
Only affected files are limited; helper injection remains Slice 1B.

### T36 — safe root/path authority

- Slice 1A wraps crate-private component-wise `open_workspace`; legacy ambient
  `ProjectInputDirectory::open` is not its trust boundary.
- Each root has request-local `InputRootId = role + ordinal`, caller display label,
  normalized lexical absolute authority descriptor and held no-follow capability.
- `InputPathAuthority = root descriptor + canonical relative path`, slash-normalized
  and Unicode-lowercased with Project's conservative portable case policy.
- This is selected-path authority, not inode/file-ID/hard-link/content/snapshot
  identity. Reject duplicate joined authorities, not root ancestry.
- Nested roots and equal relative paths are valid when joined authority differs.
  Repeated TOC occurrences are load records, not duplicate files.
- Reject duplicate root authority keys, empty Library roots and duplicate whole
  snapshot IDs. Aggregate Library ceilings: 64 roots, 8,192 files, 256 MiB.

### T37 — one trusted Service assembly

The public lower request can combine unrelated snapshots and limitations. Add one
narrow ordinary operation that owns Project preparation, plan/Main/authority
validation, requested-scope blockers, snapshot construction, snapshot/root
mapping, same-owner requirement collection and lower invocation. CLI/LSP/MCP call
only this operation. Keep lower request/check internal after callers move. Do not
create a generic backend/registry.

### T38 — structured input errors

Carry stable code, role, snapshot/`InputRootId`, role-relative path and bounded
message through Project/Emmy/configured session/Service. `LibraryHealthFailed`
keeps Library snapshot/path and becomes caller `invalid_input`, not
`ComponentUnavailable`. File-specific Project errors keep root/path. Never parse
messages or synthetic analyzer paths.

Public ordinary-operation errors are exactly:

```text
invalid_input | cancelled | internal
message <= 4 KiB
role? = config | main | library
root_index? only for library, zero-based
relative_path? relative to role root
```

Input/resource refusal is invalid input; analyzer/serialization/I/O invariants are
internal. Do not inflate global `ServiceError` into another provenance platform.

### T39 — strict config, deterministic result and publication

- One explicit no-follow Project config read, 256 KiB; `StrictJson` duplicate-key/
  depth bounds; unknown outer fields rejected; no implicit discovery.
- Required after merge: root, explicit relative TOC, `retail`, nonempty Libraries.
  Defaults: text, `fail_on: error`, upstream default Emmy diagnostics.
- CLI scalar overrides replace config; repeated CLI Libraries replace the array.
- Strictly prevalidate pinned nested diagnostic keys and compile `globalsRegex`.
  Reject `diagnosticInterval`: pinned code analysis never reads it; LS uses it only
  for debounce, so it is not analyzer identity.
- Slice 1A exposes no WoW-check option and runs zero WoW rules.
- Deterministic order: diagnostics by role/path/span/code/message; limitations by
  code/location/message.
- Threshold rank Error > Warning > Information > Hint; visible `Unknown` meets
  every threshold.
- Pre-encode at most 32 MiB, then one write. Results stdout; admitted-format
  pre-result errors stderr; early CLI grammar errors text stderr. Check
  cancellation after encoding/before first byte. Once writing begins, never
  retry/replace/append. Size refusal is invalid input/64; serialization, handler
  setup or I/O is internal/4.
- Exit precedence: invalid input 64; cancellation 130; internal 4; partial 2;
  threshold finding 1; clean 0. Human text visibly escapes controls; JSON strings
  remain exact.

## Delivered lower owners

- #246 preserves code/message/severity/range.
- #247 provides one Lua 5.1 Main/Library/virtual owner with STD and caller config.
- #248 provides the lower direct result.
- #249 separates compiled provenance from report evidence.
- #250 gates reuse on exact Main; Library snapshots are sorted by exact ID and
  compared as the same canonical set.
- #259–#265 remove diagnostic-generation identity, remove public receipts, retain
  located limitations, hide synthetic virtual paths, define finite codes, map the
  reachable package closure and fix three-valued predicates.

None is an observed addon workflow.

## Later retained decisions

- T08/T09: reuse only while selected membership/context/authority/Library/virtual/
  analysis config match; stale work never publishes.
- T10: keep one optional typed `source-report.json`; plain Lua remains valid.
- T11/T22: ordinary chunk injection, later retrieval permission and special
  contexts are distinct.
- T17/T18: exact WoW order/context is a validated Project value beside path-sorted
  snapshots.
- T19–T23: channel, triggers, bindings, Interface zero, raw unsupported `Secure`
  and metadata classes remain distinct/scoped.

## Documentation reconciliation ledger

| IDs | Finding and resolution |
| --- | --- |
| D01–D08 | #246–#250 owners, links/runs and instructions reconciled by #251–#254 |
| D09–D18 | dependency/context/order, channel/triggers/bindings, Interface/`Secure`, donor authority, status drift, evidence and metadata traced/corrected; T17–T23 added |
| D19–D26 | generation, receipts, locations, virtual paths, limitation codes, root trust/placeholders and predicates corrected by #259–#265 |
| D27–D32 | one live status established; first slice reduced; Emmy generation fixed; stale Issues corrected; false Library-order defect rejected; T31 added |
| D33–D36 | profile requirement, early metadata, Issue drift and four missing baseline `Home:` entries corrected |
| D37–D46 | retained plan identity, eager XML, standalone mapper, scope completeness, snapshot/error handoff, status compaction, canonical flavor, empty Main, duplicates and exit precedence fixed |
| D47–D54 | cross-role path duplication, nested-root rule, trusted Service assembly, structured origin, file paths, strict config, output channels and stale caller censuses fixed |
| D55 | root token + relative path could not compare one nested-root selected path -> normalized joined path authority |
| D56 | safe opener was crate-private while ambient `open` was tempting -> public Project preparation wraps safe opener |
| D57 | only per-root Library bounds -> fixed aggregate 64/8,192/256 MiB |
| D58 | config omitted duplicate-key/depth/size -> `StrictJson`, 256 KiB, unknown rejection |
| D59 | clean empty vs deferred empty ambiguous -> explicit complete vs partial empty Main |
| D60–D62 | global error inflation/streaming, opaque root tokens and invalid-vs-partial selection -> operation error, role+ordinal root, exact classification |
| D63–D65 | unconditional chunk fallback lacked requirement detection; repeated/interface/lifecycle states ambiguous; nonexistent `checks` field -> same-owner AST collector, exact classification, zero WoW rules |
| D66–D70 | open-ended errors, unsafe human text, config path bases, platform case drift and budget/internal confusion -> finite error codes, escaping, no-follow config, portable fold and exact classification |
| D71–D75 | severity order, merged CLI defaults, duplicate snapshot nuance, output order and permissive upstream diagnostic config -> explicit contracts and regex/key validation |
| D76 | chunk requirement owner unspecified -> same-session Emmy AST collector over literal dots outside closures |
| D77 | no final publication cancellation point -> after-encoding checkpoint; first byte commits sole envelope |
| D78 | ambiguous error root/path fields -> capped message, role-relative path, Library-only zero-based root index |
| D79 | audit recopied the full contract into secondary instructions -> AGENTS/skills/package README reduced to navigation/owners/prohibitions |
| D80 | upstream `diagnosticInterval` is LS debounce only -> reject it in direct checker; exclude from analyzer compatibility |

### Baseline register errata

The baseline register remains historical. Current homes for the four rows missing
`Home:` are:

| Baseline row | Current home |
| --- | --- |
| `crates/wow-project/DATA_MODEL.md` | [updates](IMPLEMENTATION.md#updates), [results](IMPLEMENTATION.md#results-and-interfaces) |
| `crates/wow-project/e2/REAL_ADDON_FIXTURE.md` | [completion](IMPLEMENTATION.md#completion-and-verification), [queue](REFACTOR_QUEUE.md) |
| `crates/wow-project/e3/DATA_MODEL.md` | [addon configuration](IMPLEMENTATION.md#addon-configuration-and-emmy-analysis), [typed relationships](IMPLEMENTATION.md#typed-source-relationships) |
| `crates/wow-project/e3/SKELETON_INPUT_VIEW.md` | [source index](IMPLEMENTATION.md#blizzard-ui-index-search-and-context), [typed relationships](IMPLEMENTATION.md#typed-source-relationships) |

## Observations still required

Do not mark observed without exact input/result. Minimum Slice 1A observations:

- valid and invalid typed calls with exact code/message/severity/root-relative span;
- top-level ellipsis partial and nested-function/string/comment near-negatives;
- required dependency or deferred XML partial preserving diagnostics;
- safely referenced missing child Lua partial;
- XML-only physical slice through empty Main;
- malformed Library with structured root/path;
- duplicate path authority rejected; disjoint nested roots and equal relative paths
  in distinct roots accepted;
- strict diagnostic config, including invalid regex/unknown key and rejected
  `diagnosticInterval`;
- text/JSON result/error channels, output bound and deterministic exits;
- later dependency/XML/chunk injection, updates, LSP/MCP, source search/context and
  truthful cancellation.

Update this register only when source evidence changes a boundary or an exact
workflow changes status. Do not turn it into a dashboard or PR diary.
