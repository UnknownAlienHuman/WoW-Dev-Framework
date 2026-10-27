# Implementation and documentation trace register

**Baseline register:** [AUDIT_REGISTRY.md](AUDIT_REGISTRY.md), PR #244  
**Current audit base:** `main` after PR #269
(`f674ccc7615d63c76c0781aef2150e49ea442bf8`)  
**Current traces:** T01–T39  
**Documentation corrections:** D01–D82

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
| T13 | diagnostic config -> selected collection | owner implemented #247; strict admission/canonical semantics open | unexecuted |
| T14 | exact snapshots -> direct findings/limitations | implemented #248 | unexecuted |
| T15 | compiled identity vs report evidence | implemented #249 | unexecuted |
| T16 | exact snapshots/config -> owner reuse | snapshot admission #250; semantic config equivalence open | unexecuted |
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
| T31 | selected Main requirements -> scope completeness | traced; unconditional chunk fallback remains | unexecuted |
| T32 | ordinary target -> profile-free target/scope plan | traced; current plan profile-bound | unexecuted |
| T33 | physical plan -> standalone blockers + Main | traced; current loader opens XML/package mapper only | unexecuted |
| T34 | admitted Main/Libraries -> exact snapshots/paths/errors | traced; construction open | unexecuted |
| T35 | empty Main + deferred units -> partial without dummy | traced; route open | unexecuted |
| T36 | opened roots + admitted paths -> collision-free authority | traced; authority validation open | unexecuted |
| T37 | ordinary config -> one Service assembly -> lower check | traced; operation open | unexecuted |
| T38 | file failure -> structured operation error | traced; origin flattened | unexecuted |
| T39 | strict config/result/error streams -> deterministic exit | traced; frontend open | unexecuted |

## Durable Slice 1A decisions

### T23/T32–T35 — Project selection and scope

- Replace retained mandatory Interface/profile digest with `AddonLoadTarget +
  projection scope`; legacy callers adapt real profiles, direct path fabricates no
  profile/source/report/schema/correction/generation identity.
- `physical_lua` parses TOC/direct Lua, retains XML records/spans and does not open
  XML. Deferred XML is located and cannot erase physical diagnostics.
- Service maps one `ProjectLoadPlan + AddonCheckScope` to only requested blockers;
  package mapping/global completeness are not substitutes.
- Cosmetic/custom/non-selection metadata stays observation; malformed or selected-
  context-changing input stays invalid/limited.
- Compiled pin, Main `Project`, one `BlizzardUi` snapshot per Library root,
  canonical role-relative paths.
- Empty Main is complete with no selected unit/blocker and partial with deferred
  selected input. No dummy source.

### T31 — same-owner chunk requirement

Add one bounded collector to the same configured Emmy owner. Walk registered Main
expression AST, match `LuaLiteralExpr`/`LuaLiteralToken::Dots`, exclude any match
inside `LuaClosureExpr`, retain the lowest-starting exact path/content/span per
file and validate against Main. Named/local/anonymous functions all contain the
closure node at the pinned parser revision. No raw scan or second parser. Limit
only affected files; injection remains Slice 1B.

### T36 — safe root/path authority

- Slice 1A wraps crate-private component-wise `open_workspace`; ambient
  `ProjectInputDirectory::open` is not its trust boundary.
- Each root has request-local `InputRootId = role + ordinal`, display label,
  normalized lexical absolute authority descriptor and held no-follow capability.
- `InputPathAuthority = root descriptor + canonical relative path`, slash-
  normalized and Unicode-lowercased with Project's portable case policy.
- This is selected-path authority, not inode/file-ID/hard-link/content/snapshot
  identity. Reject duplicate joined authorities, not ancestry.
- Nested roots/equal relative paths are valid when joined authority differs;
  repeated TOC occurrences are load records.
- Reject duplicate root keys, empty Libraries and duplicate whole snapshot IDs.
  Aggregate Library ceilings: 64 roots, 8,192 files, 256 MiB.

### T37 — one trusted Service assembly

One narrow ordinary operation owns Project preparation, plan/Main/authority
validation, requested blockers, snapshots, snapshot/root mapping, same-owner
requirements and lower invocation. CLI/LSP/MCP call only it. Lower request/check
becomes internal. No generic backend/registry.

### T38 — structured input errors

Carry stable code, role, snapshot/`InputRootId`, role-relative path and bounded
message through Project/Emmy/session/Service. `LibraryHealthFailed` becomes caller
`invalid_input`, not `ComponentUnavailable`. Never parse messages/synthetic paths.

Public operation error:

```text
invalid_input | cancelled | internal
message <= 4 KiB
role? = config | main | library
root_index? only for library, zero-based
relative_path? relative to role root
```

Caller input/resource refusal is invalid input; analyzer/serialization/I/O
invariants internal. Do not inflate global `ServiceError`.

### T13/T16/T39 — strict canonical diagnostic configuration

Prevalidate exactly `disable`, `enable`, `globals`, `globalsRegex`, `severity`,
`enables`; compile regexes; reject unknowns/invalid regex and LS-only
`diagnosticInterval`.

Replace raw JSON equivalence with canonical analysis projection: preserve
`enable`; sort/dedup `disable`, `enables`, `globals`, `globalsRegex`; sort severity
by code; omit interval. Pinned analysis treats these lists as sets/OR regexes, so
omitted diagnostics, `{}`, reordering and duplicates reuse one owner; real
semantic changes rebuild. LS debounce stays adapter state.

### T39 — strict config, path bases, deterministic publication

- One explicit no-follow config, 256 KiB; `StrictJson` duplicate/depth bounds;
  unknown fields rejected; no discovery.
- Path origins: relative `--config`, CLI root and CLI Libraries resolve from cwd;
  config root/Libraries resolve from config parent; TOC is always canonical
  relative and resolves after merge against final addon root. CLI scalar overrides
  retain CLI/cwd origin; CLI Library list replaces config list. Lexical normalize
  only—never filesystem canonicalize/follow roots.
- Required after merge: root, TOC, `retail`, Libraries. Defaults: text,
  `fail_on: error`, upstream default diagnostics. Zero WoW rules.
- Deterministic diagnostic/limitation order. Threshold Error > Warning >
  Information > Hint; visible `Unknown` meets all thresholds.
- Pre-encode <=32 MiB; result stdout, admitted-format pre-result error stderr,
  early grammar error text stderr. Check cancellation after encoding/before first
  byte. No retry/second envelope. Size refusal invalid input/64; serialization,
  handler setup or I/O internal/4.
- Exit precedence: 64, 130, 4, 2, 1, 0. Human text escapes controls; JSON exact.

## Delivered lower owners

#246 preserves diagnostics; #247 one Lua 5.1 Main/Library/virtual owner with STD;
#248 lower direct result; #249 compiled vs report identity; #250 exact Main/canonical
Library-set reuse; #259–#265 remove generation/receipts, retain located limits,
hide synthetic virtual paths, define codes, map reachable package closure and fix
three-valued predicates. None is an observed workflow.

## Later retained decisions

- T08/T09: reuse only while membership/context/authority/Library/virtual/canonical
  analysis config match; stale work never publishes.
- T10: one optional typed `source-report.json`; plain Lua valid.
- T11/T22: injection, later retrieval and special contexts distinct.
- T17/T18: WoW order/context is validated Project data beside path-sorted snapshots.
- T19–T23: channel, triggers, bindings, Interface zero, raw `Secure` and metadata
  classes remain distinct/scoped.

## Documentation reconciliation ledger

| IDs | Finding and resolution |
| --- | --- |
| D01–D08 | #246–#250 owners/links/instructions reconciled #251–#254 |
| D09–D18 | dependency/context/order, channel/triggers/bindings, Interface/`Secure`, donor authority, status/evidence/metadata corrected; T17–T23 |
| D19–D26 | generation, receipts, locations, virtual paths, limitation codes, root trust/placeholders and predicates corrected #259–#265 |
| D27–D32 | one live status, smaller first slice, Emmy generation, stale Issues, false Library-order defect, T31 |
| D33–D36 | hidden profile requirement, early metadata, Issue drift, four missing baseline homes |
| D37–D46 | retained plan identity, eager XML, standalone mapper, scope completeness, snapshots/errors, compact status, retail, empty Main, duplicates, exits |
| D47–D54 | cross-role paths, nested roots, Service assembly, structured origin, file paths, strict config, output channels, stale censuses |
| D55–D62 | normalized path authority, safe opener, aggregate bounds, strict config, empty state, operation error/root/classification |
| D63–D70 | chunk requirement, exact observations, zero WoW rules, finite errors, escaping, bases, case policy, budget classes |
| D71–D78 | severity/defaults/snapshot nuance/order/permissive config, AST owner, final cancellation, error fields |
| D79 | copied contract in secondary docs -> navigation/owners/prohibitions only |
| D80 | `diagnosticInterval` is LS-only -> reject/exclude from analyzer identity |
| D81 | raw JSON equality split default/`{}` and reordered sets -> canonical analysis projection |
| D82 | path-base rules vanished during compaction -> cwd/config-parent/final-root rules restored in frontend contract and trace |

### Baseline register errata

| Baseline row | Current home |
| --- | --- |
| `crates/wow-project/DATA_MODEL.md` | [updates](IMPLEMENTATION.md#updates), [results](IMPLEMENTATION.md#results-and-interfaces) |
| `crates/wow-project/e2/REAL_ADDON_FIXTURE.md` | [completion](IMPLEMENTATION.md#completion-and-verification), [queue](REFACTOR_QUEUE.md) |
| `crates/wow-project/e3/DATA_MODEL.md` | [addon configuration](IMPLEMENTATION.md#addon-configuration-and-emmy-analysis), [typed relationships](IMPLEMENTATION.md#typed-source-relationships) |
| `crates/wow-project/e3/SKELETON_INPUT_VIEW.md` | [source index](IMPLEMENTATION.md#blizzard-ui-index-search-and-context), [typed relationships](IMPLEMENTATION.md#typed-source-relationships) |

## Observations still required

- valid/invalid typed calls with exact root-relative diagnostics;
- top-level ellipsis partial and function/string/comment near-negatives;
- dependency/deferred XML/missing child partial preserving findings;
- clean and deferred empty Main;
- malformed Library structured root/path;
- duplicate authority rejected; disjoint nested/equal-relative distinct roots accepted;
- strict diagnostic errors and rejected interval;
- default/`{}`/reordered config reuse vs real semantic cold rebuild;
- config outside cwd with config-relative root/Libraries, CLI overrides from cwd,
  and TOC resolved against final root;
- text/JSON channels, bound and exits;
- later XML/chunk injection, updates, LSP/MCP, search/context and cancellation.

Update only when source evidence changes a boundary or an exact workflow changes
status. Do not turn this into a dashboard or PR diary.
