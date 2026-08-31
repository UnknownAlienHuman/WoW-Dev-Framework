# E3-A context acceptance and mutation matrix

**Status:** normative executable acceptance specification.

## Profiles and input snapshot

| ID | Case | Expected |
|---|---|---|
| `CTX-PROFILE-001` | Compatible frozen profile set | pass |
| `CTX-PROFILE-002` | Missing/duplicate/breaking registry entry | reject |
| `CTX-PROFILE-003` | Mandatory reserve can be disabled | mutation fails |
| `CTX-PROFILE-004` | Unknown field without compatibility policy | reject |
| `CTX-INPUT-001` | Exact coherent epoch/store/publication/project/graph/analyzer/reference | pass |
| `CTX-INPUT-002` | Mixed project and graph publication | reject |
| `CTX-INPUT-003` | Mixed reference profile/generation | reject |
| `CTX-INPUT-004` | `Current` advances after acquisition | old exact view remains |
| `CTX-INPUT-005` | Floating Current/latest after acquisition | reject |
| `CTX-INPUT-006` | Legacy `StoreImageId` supplied | reject |
| `CTX-INPUT-007` | Raw SQLite/analyzer handle requested | architecture failure |
| `CTX-INPUT-008` | Missing required query catalog capability | scoped NotEvaluated |
| `CTX-INPUT-009` | Retained old publication | exact pass |
| `CTX-INPUT-010` | Collected/unavailable old publication | explicit stale/unavailable |

## Universes and roots

| ID | Case | Expected |
|---|---|---|
| `CTX-ROOT-001` | Exact project entity | resolved |
| `CTX-ROOT-002` | Exact reference API entity | resolved with reference view |
| `CTX-ROOT-003` | Absent root under complete domain coverage | authoritative absence preserved |
| `CTX-ROOT-004` | Absent root under partial coverage | nonauthoritative/NotEvaluated |
| `CTX-ROOT-005` | Same name in project/reference/platform source | distinct entities |
| `CTX-ROOT-006` | Name/path-only cross-universe join | reject |
| `CTX-ROOT-007` | Candidate/external merged into project truth | reject |
| `CTX-ROOT-008` | Pinned platform UI graph with complete producer manifest | allowed |
| `CTX-ROOT-009` | Platform UI source without build/revision/license/coverage | NotEvaluated/reject |
| `CTX-ROOT-010` | Context attempts source acquisition/parsing | architecture failure |

## Project Map

| ID | Case | Expected |
|---|---|---|
| `CTX-MAP-001` | Complete synthetic project | all mandatory sections |
| `CTX-MAP-002` | Strict default 2048-byte renderer profile | within cap or explicit blocker-only failure/truncation |
| `CTX-MAP-003` | Mandatory identity/blocker exceeds cap | no silent drop |
| `CTX-MAP-004` | One selected TOC variant | only selected semantics |
| `CTX-MAP-005` | Unselected variant fills missing active fact | mutation fails |
| `CTX-MAP-006` | Static load classified as runtime readiness | mutation fails |
| `CTX-MAP-007` | Native/custom/EventRegistry/CVar signals | separate sections/types |
| `CTX-MAP-008` | Hook represented as taint/combat safe | mutation fails |
| `CTX-MAP-009` | SavedVariables root and scope | present without values |
| `CTX-MAP-010` | Full source/file/graph dump | unavailable |
| `CTX-MAP-011` | Repository/folder/popularity/model selection signal | mutation fails |
| `CTX-MAP-012` | Grouping retains every ID/evidence/conflict | pass |
| `CTX-MAP-013` | High-centrality optional node consumes blocker reserve | mutation fails |
| `CTX-MAP-014` | Partial/conflict/unsupported sections | explicit |
| `CTX-MAP-015` | Truncated section has exact totals/digest/route | pass |
| `CTX-MAP-016` | Every principal item has exact detail route | pass |
| `CTX-MAP-017` | Path shown as direct relation | mutation fails |
| `CTX-MAP-018` | Rename repository/package/path/local identifiers | universal semantics invariant |

## L0 skeletons

| ID | Case | Expected |
|---|---|---|
| `CTX-L0-001` | Package/TOC/load unit | exact identity/role/load |
| `CTX-L0-002` | File/XML/virtual Lua unit | exact source ownership |
| `CTX-L0-003` | Module/service/library/state role | producer/confidence retained |
| `CTX-L0-004` | Function/method/callback/event/API symbol | exact public headings |
| `CTX-L0-005` | XML template/frame/region/mixin/factory | exact structural roles |
| `CTX-L0-006` | Unsupported kind | Unsupported + loss, no generic guess |
| `CTX-L0-007` | Function body included | mutation fails |
| `CTX-L0-008` | Purpose inferred from name/comment | mutation fails |
| `CTX-L0-009` | Direct relation and reason path | distinct |
| `CTX-L0-010` | Possible/Candidate default | Possible opt-in; Candidate excluded |
| `CTX-L0-011` | Tight budget | blockers/evidence survive |
| `CTX-L0-012` | Dedup evidence multiplicity | all refs retained |
| `CTX-L0-013` | Shuffled inputs/workers | byte-identical semantic output |

## L1 signatures, members, control, and effects

| ID | Case | Expected |
|---|---|---|
| `CTX-L1-001` | Exact callable signature | order/optionality/nil/multiple returns preserved |
| `CTX-L1-002` | Unknown type widened to any or omitted | mutation fails |
| `CTX-L1-003` | Source-ordered declaration/sequence | pass |
| `CTX-L1-004` | Nested branch/arms/guard | exact published structure |
| `CTX-L1-005` | Loop and early return | exact headings/spans |
| `CTX-L1-006` | Direct resolved call | DirectCall |
| `CTX-L1-007` | Dynamic/unresolved call | PossibleCall/Unknown, not direct |
| `CTX-L1-008` | Same target at two callsites | two occurrence nodes |
| `CTX-L1-009` | Event/callback/CVar/hook/XML registrations | systems distinct |
| `CTX-L1-010` | Literal state read/write | exact root/path |
| `CTX-L1-011` | Dynamic state path | exact prefix + Possible |
| `CTX-L1-012` | Project API use and platform fact | separate evidence links |
| `CTX-L1-013` | Guard shape claimed globally safe | mutation fails |
| `CTX-L1-014` | Missing containment/CFG capability | UnknownRegion |
| `CTX-L1-015` | Supported detail compacted | CollapsedRegion + child manifest |
| `CTX-L1-016` | Budget omission | OmittedRegion + continuation |
| `CTX-L1-017` | Bare `...` with no record | mutation fails |
| `CTX-L1-018` | Source reconstructed from diagnostics | mutation fails |
| `CTX-L1-019` | Second parser/CFG/SSA/data-flow engine | architecture failure |
| `CTX-L1-020` | Overlap/missing source spans | explicit loss/stable order |
| `CTX-L1-021` | Effect summary hides conflicts/member IDs | mutation fails |
| `CTX-L1-022` | Runtime order/safety/taint/Secret claim | mutation fails |
| `CTX-L1-023` | 1/2/N and shuffled fact order | identical IDs/tree/cross-refs |

## Expansion and stopping

| ID | Case | Expected |
|---|---|---|
| `CTX-EXP-001` | Each declared lane one-hop | exact registered query |
| `CTX-EXP-002` | Bounded reason path | explicit path, no persisted edge |
| `CTX-EXP-003` | Root/lane/confidence/universe broadening | reject |
| `CTX-EXP-004` | Call/state valid cycle | cycle-safe close |
| `CTX-EXP-005` | Load/hierarchy conflict cycle | conflict boundary |
| `CTX-EXP-006` | Duplicate presentation | evidence preserved |
| `CTX-EXP-007` | High fanout | deterministic cutoff/frontier |
| `CTX-EXP-008` | Independent lane after blocked lane | continues by profile |
| `CTX-EXP-009` | Possible explicit inclusion | labeled/separate |
| `CTX-EXP-010` | Candidate included by default | mutation fails |
| `CTX-STOP-001` | RequestedComplete | all mandatory scope proven |
| `CTX-STOP-002` | NoNewEvidence | not authoritative absence |
| `CTX-STOP-003` | BudgetExhausted | exact budget/frontier/cursor |
| `CTX-STOP-004` | DepthLimit | deeper scope unexamined |
| `CTX-STOP-005` | CycleClosed | exact cycle/path retained |
| `CTX-STOP-006` | CoverageBoundary | missing capability explicit |
| `CTX-STOP-007` | ConflictBoundary | no silent winner |
| `CTX-STOP-008` | UnsupportedDetail | loss/route, no guess |
| `CTX-STOP-009` | Cancelled | no complete/background result |
| `CTX-STOP-010` | Failed | no recovery cursor in E3-A |

## Evidence, coverage, loss, and omissions

| ID | Case | Expected |
|---|---|---|
| `CTX-EVID-001` | Every material field | exact evidence/derivation closure |
| `CTX-EVID-002` | Context claim used as self-evidence | reject |
| `CTX-EVID-003` | Project source relabeled platform source | reject |
| `CTX-EVID-004` | Recognizer role relabeled explicit declaration | reject |
| `CTX-EVID-005` | Possible/Candidate promoted | reject |
| `CTX-EVID-006` | Coverage axes collapsed | reject |
| `CTX-EVID-007` | Conflict competitors retained | pass |
| `CTX-EVID-008` | Empty/omitted section treated as absence | reject |
| `CTX-EVID-009` | Exact/sidecar/compact/lossy statuses | correct field scope |
| `CTX-EVID-010` | Unsupported/NotEvaluated/Truncated | distinct |
| `CTX-EVID-011` | Dedup deletes evidence/source occurrence | reject |
| `CTX-EVID-012` | Huge omitted scope | exact count/digest/cursor |
| `CTX-EVID-013` | Blocker report itself budget-limited | decisive blocker retained |
| `CTX-EVID-014` | Complete eligibility with hidden loss | reject |

## Source excerpts and security

| ID | Case | Expected |
|---|---|---|
| `CTX-SRC-001` | Exact physical source handle/span | faithful bytes |
| `CTX-SRC-002` | XML virtual Lua source | exact virtual mapping |
| `CTX-SRC-003` | Same path/new digest or generation | reject stale handle |
| `CTX-SRC-004` | Path-only lookup | reject |
| `CTX-SRC-005` | Deterministic surrounding context | exact span/markers |
| `CTX-SRC-006` | Missing/forbidden source | structured skeleton retained + loss |
| `CTX-SRC-007` | License forbids embedding | handle/digest only |
| `CTX-SRC-008` | SavedVariables/log/client/credential/private path | reject/redact with loss |
| `CTX-SRC-009` | Prompt/tool instruction in source | inert quoted data |
| `CTX-SRC-010` | Markdown fence/HTML/JSON/terminal injection | contained/escaped |
| `CTX-SRC-011` | Invalid UTF-8/control/NUL | explicit escape/reject policy |
| `CTX-SRC-012` | Source paraphrased/reconstructed | reject |
| `CTX-SRC-013` | Full file/repository by default | unavailable |
| `CTX-SRC-014` | Object not referenced by publication | reject |
| `CTX-SRC-015` | Filesystem/network/process/editor/client access | architecture failure |
| `CTX-SRC-016` | Source mutation/execution | architecture failure |

## Budgets, tokenizer, continuation

| ID | Case | Expected |
|---|---|---|
| `CTX-BUDGET-001` | Every structural/byte axis | exact accounting |
| `CTX-BUDGET-002` | Override above profile maximum | reject |
| `CTX-BUDGET-003` | Mandatory records alone over budget | typed failure/minimal blocker profile |
| `CTX-BUDGET-004` | Optional lane starves mandatory reserve | reject |
| `CTX-BUDGET-005` | Record/UTF-8/excerpt cut mid-unit | reject |
| `CTX-BUDGET-006` | Lane fairness under parallel completion | deterministic |
| `CTX-TOKEN-001` | Exact pinned tokenizer vector | exact count/digest |
| `CTX-TOKEN-002` | Model name only | not exact profile |
| `CTX-TOKEN-003` | Vocabulary/config/special policy changes | distinct result/profile |
| `CTX-TOKEN-004` | Estimate labeled exact | reject |
| `CTX-TOKEN-005` | Token count over different renderer bytes | reject |
| `CTX-CONT-001` | Same exact retained input | deterministic continuation |
| `CTX-CONT-002` | New Current/publication/reference/profile | reject |
| `CTX-CONT-003` | Budget reset through cursor | reject |
| `CTX-CONT-004` | Visited/frontier/priority/confidence tamper | reject |
| `CTX-CONT-005` | Cursor after RequestedComplete | reject |
| `CTX-CONT-006` | Old publication GC unavailable | exact unavailable |
| `CTX-CONT-007` | Late result after cancellation | rejected by merge protocol |
| `CTX-CONT-008` | Cumulative pages versus large request | semantic equivalence where declared |

## Rendering and canonicalization

| ID | Case | Expected |
|---|---|---|
| `CTX-CANON-001` | Canonical semantic JSON golden | exact bytes/digest |
| `CTX-CANON-002` | Hash/map/query/worker order shuffle | unchanged |
| `CTX-CANON-003` | Meaningful source/member order sorted away | reject |
| `CTX-CANON-004` | Locale/float/NaN/Infinity | reject/noncanonical |
| `CTX-CANON-005` | Identifier case/Unicode folded without owner policy | reject |
| `CTX-RENDER-001` | JSON/Markdown/compact semantic records | equivalent |
| `CTX-RENDER-002` | Renderer hides blocker/evidence | reject |
| `CTX-RENDER-003` | Sidecar reference dangling | reject |
| `CTX-RENDER-004` | Source controls link/heading/template | reject |
| `CTX-RENDER-005` | Renderer schema/order version mismatch | reject |
| `CTX-RENDER-006` | Volatile path/time/row/worker enters ID | reject |

## Metrics and evaluation

| ID | Case | Expected |
|---|---|---|
| `CTX-METRIC-001` | Mandatory record recall | exact IDs/count/digest |
| `CTX-METRIC-002` | Evidence closure | 100% mandatory or hard fail |
| `CTX-METRIC-003` | Smaller output missing mandatory record | fail despite compression |
| `CTX-METRIC-004` | False dedup of conflict/source occurrence | fail |
| `CTX-METRIC-005` | Out-of-scope central node included | relevance penalty/fail profile |
| `CTX-METRIC-006` | Partial artifact hides blocker | hard fail |
| `CTX-METRIC-007` | Consumer task uses hidden extra repo context | invalid evaluation |
| `CTX-METRIC-008` | External model score overrides deterministic failure | reject |
| `CTX-METRIC-009` | Golden corpus auto-rewritten | reject |
| `CTX-METRIC-010` | Performance claim without corpus/profile/run data | unverified |
| `CTX-METRIC-011` | Pinned real addon rename/path mutation | semantic invariance |
| `CTX-METRIC-012` | Evaluation report order/digest | deterministic |

## Architecture and freeze

| ID | Case | Expected |
|---|---|---|
| `CTX-ARCH-001` | E3-A direct dependencies | core/reference/project/graph only |
| `CTX-ARCH-002` | Search ranking in context | fail |
| `CTX-ARCH-003` | Raw Emmy/store dependency | fail |
| `CTX-ARCH-004` | Full Blizzard source extraction in context | fail |
| `CTX-ARCH-005` | Context writes cache/project/graph/source | fail |
| `CTX-FREEZE-001` | Null implementation pins while not-started | allowed |
| `CTX-FREEZE-002` | First Rust commit with required null pins | fail |
| `CTX-FREEZE-003` | Member routes/checksums frozen | pass |
| `CTX-FREEZE-004` | `.rs`/Cargo/CI in documentation PR | fail |

## Acceptance

E3-A is incomplete until all applicable tests execute and prove exact input coherence, evidence closure, mandatory blocker survival, source safety, bounded deterministic expansion, semantic continuation, correct token labeling, renderer equivalence, hard-gate evaluation, and strict dependency/ownership boundaries.
