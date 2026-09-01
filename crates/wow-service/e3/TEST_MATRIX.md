# E3-C service test matrix

**Status:** normative acceptance, property, mutation, failure-injection, security, and determinism matrix.

Case IDs are stable and unique. Every active case must execute before E3-C implementation is called complete.

## Configuration and contract routing

| ID | Case | Expected |
|---|---|---|
| `SVC3-CFG-001` | Valid exact E3-C service configuration | pass |
| `SVC3-CFG-002` | Unknown configuration field/version | reject |
| `SVC3-CFG-003` | Missing owner-port catalog | block |
| `SVC3-CFG-004` | Invalid/ambiguous profile alias | reject |
| `SVC3-CFG-005` | Alias changes without configuration ID change | reject |
| `SVC3-CFG-006` | Context path activates forbidden direct dependency | architecture fail |
| `SVC3-CFG-007` | Service implements context/graph/store algorithm | architecture fail |
| `SVC3-CFG-008` | App behavior appears in service contract/type | reject |
| `SVC3-CFG-009` | E0 status/check unavailable after E3-C addition | regression fail |
| `SVC3-CFG-010` | E1 Reference Pack operations unavailable | regression fail |
| `SVC3-CFG-011` | Current KB facts hard-coded into service profile | reject |
| `SVC3-CFG-012` | Missing prerequisite implementation reported ready | reject |
| `SVC3-CFG-013` | Config derives semantic default from cwd/env/editor | reject |
| `SVC3-CFG-014` | Same exact config in 1/2/N workers | same configuration ID |
| `SVC3-CFG-015` | Invalid acquisition/release order profile | reject |

## Request and profile normalization

| ID | Case | Expected |
|---|---|---|
| `SVC3-REQ-001` | Valid operation-specific request | canonical request |
| `SVC3-REQ-002` | Unknown operation | deferred/usage-classified failure |
| `SVC3-REQ-003` | Exact configured alias | exact profile ID recorded |
| `SVC3-REQ-004` | Unknown/ambiguous alias | reject |
| `SVC3-REQ-005` | Alias spelling differs but target same | same semantic request, supplemental alias differs only as allowed |
| `SVC3-REQ-006` | Continuation re-resolves changed alias | reject |
| `SVC3-REQ-007` | Negative/unlimited/overflow budget | reject |
| `SVC3-REQ-008` | Budget override exceeds service/profile maximum | reject |
| `SVC3-REQ-009` | Privacy/source profile broadens configured ceiling | reject |
| `SVC3-REQ-010` | Renderer selected by terminal/locale/file extension | reject |
| `SVC3-REQ-011` | Exact tokenizer required but only display model name supplied | reject |
| `SVC3-REQ-012` | Search/natural-language/fuzzy root request | reject E3-C |
| `SVC3-REQ-013` | Filesystem path/glob used as semantic root | reject |
| `SVC3-REQ-014` | Source text supplied as profile/control data | reject |
| `SVC3-REQ-015` | Reordered equivalent request JSON | same request ID |

## Selector resolution

| ID | Case | Expected |
|---|---|---|
| `SVC3-SEL-001` | Exact primary StoreGeneration exists | exact selection |
| `SVC3-SEL-002` | Exact primary PublicationSet uniquely resolves | exact selection |
| `SVC3-SEL-003` | Exact generation missing | fail, no current fallback |
| `SVC3-SEL-004` | Exact publication belongs to another project/store | reject |
| `SVC3-SEL-005` | Primary CurrentPublished valid | resolve once and record exact IDs |
| `SVC3-SEL-006` | Current pointer changes after resolve | operation stays on acquired exact view |
| `SVC3-SEL-007` | Implementation rereads current after acquisition | mutation fails |
| `SVC3-SEL-008` | Expected-current guard matches | proceed |
| `SVC3-SEL-009` | Expected-current guard mismatches | fail before context execution |
| `SVC3-SEL-010` | Guard mismatch triggers automatic retry | reject |
| `SVC3-SEL-011` | Current missing/invalid | fail |
| `SVC3-SEL-012` | Last-known-good substituted for requested target | reject |
| `SVC3-SEL-013` | Optional platform selector Omitted | explicit omitted universe |
| `SVC3-SEL-014` | Platform exact generation valid | exact selection |
| `SVC3-SEL-015` | Platform CurrentPublished valid | resolve once |
| `SVC3-SEL-016` | Platform current changes during later acquisition | retained exact selection remains; compatibility decides |
| `SVC3-SEL-017` | Platform exact missing but optional profile permits omission after request asked exact | fail, no silent omission |
| `SVC3-SEL-018` | Reference selected from floating current | reject |
| `SVC3-SEL-019` | Reference derived from exact primary publication | exact selection |
| `SVC3-SEL-020` | Exact caller reference guard matches | pass |
| `SVC3-SEL-021` | Exact caller reference guard mismatches | fail |
| `SVC3-SEL-022` | Retry changes selectors but reuses request ID | reject |
| `SVC3-SEL-023` | Same selectors, scheduling changes | same resolved IDs or same typed failure |
| `SVC3-SEL-024` | Claim independent currents are globally atomic | reject claim/contract |
| `SVC3-SEL-025` | Ambiguous PublicationSet-to-StoreGeneration mapping | reject unless owner guarantees exact unique binding |

## Acquisition and compatibility

| ID | Case | Expected |
|---|---|---|
| `SVC3-ACQ-001` | Acquire primary -> platform -> reference | pass |
| `SVC3-ACQ-002` | Acquisition order changed | reject |
| `SVC3-ACQ-003` | Primary acquire fails | no later acquire, clean unwind |
| `SVC3-ACQ-004` | Platform acquire fails | primary released |
| `SVC3-ACQ-005` | Reference acquire fails | platform/primary released |
| `SVC3-ACQ-006` | Owner returns wrong project/generation | reject and unwind |
| `SVC3-ACQ-007` | Owner returns raw connection/transaction | reject architecture |
| `SVC3-ACQ-008` | Primary project/graph publication mismatch | reject |
| `SVC3-ACQ-009` | Platform project/graph publication mismatch | reject |
| `SVC3-ACQ-010` | Primary/reference profile mismatch | reject |
| `SVC3-ACQ-011` | Platform/reference profile incompatible | reject |
| `SVC3-ACQ-012` | Wrong E3-A SkeletonInputView generation | reject |
| `SVC3-ACQ-013` | Required capability Failed | fail/NotEvaluated exactly by operation profile |
| `SVC3-ACQ-014` | Optional capability Partial | lease can form with blockers if profile permits |
| `SVC3-ACQ-015` | Conflict blocks required universe binding | fail/not_evaluated by contract, never complete |
| `SVC3-ACQ-016` | Public partial lease exposed | reject |
| `SVC3-ACQ-017` | ContextUniverseSet binds all exact views | pass |
| `SVC3-ACQ-018` | ContextUniverseSet changes generation | reject |
| `SVC3-ACQ-019` | Current activation after lease | no effect on operation |
| `SVC3-ACQ-020` | Acquisition timing/lock ID changes | same semantic selection/lease-set ID |
| `SVC3-ACQ-021` | Optional platform absent but requested profile requires it | fail/not_evaluated, no substitution |
| `SVC3-ACQ-022` | Same name/path used to join universes | reject |
| `SVC3-ACQ-023` | Implementation source used as Reference contract | reject authority |
| `SVC3-ACQ-024` | Capability flattened to Ready boolean | mutation fails |
| `SVC3-ACQ-025` | Cancellation after each acquire stage | exact unwind and one cancelled result |

## `context_status`

| ID | Case | Expected |
|---|---|---|
| `SVC3-STATUS-001` | Exact configured context status | complete payload |
| `SVC3-STATUS-002` | Current selector resolved and exact IDs shown | pass |
| `SVC3-STATUS-003` | Partial optional capability | partial status with exact blockers |
| `SVC3-STATUS-004` | Component Ready rendered as context tested/passed | reject |
| `SVC3-STATUS-005` | E3-A/E3-B implementation absent | explicit unavailable/NotEvaluated |
| `SVC3-STATUS-006` | Last-known-good/failed target/current conflated | reject |
| `SVC3-STATUS-007` | Status builds map/source excerpts | architecture fail |
| `SVC3-STATUS-008` | Deferred E4/E7 operations marked available | reject |
| `SVC3-STATUS-009` | Close fails after status metadata | failure, no complete status |
| `SVC3-STATUS-010` | Output order differs by owner completion | mutation fails |

## `context_map`

| ID | Case | Expected |
|---|---|---|
| `SVC3-MAP-001` | Primary project map | unchanged validated ProjectMap |
| `SVC3-MAP-002` | Platform map | exact separate universe |
| `SVC3-MAP-003` | Combined map | separate maps + explicit cross-links |
| `SVC3-MAP-004` | No narrower root | exact acquired project root derived |
| `SVC3-MAP-005` | Nonexact map root | reject |
| `SVC3-MAP-006` | Service invents edge/group/role | architecture fail |
| `SVC3-MAP-007` | Map partial/truncated | service status preserves state |
| `SVC3-MAP-008` | Service renders map semantically itself | reject |
| `SVC3-MAP-009` | Context map invalid origin closure | failure |
| `SVC3-MAP-010` | Map continuation advertised without retention when required | reject/failure |

## `context_inspect`

| ID | Case | Expected |
|---|---|---|
| `SVC3-INSP-001` | Exact L0 root | validated L0 payload |
| `SVC3-INSP-002` | Exact L1 root | validated L1 payload |
| `SVC3-INSP-003` | L0AndL1 multiple exact roots under budget | deterministic order |
| `SVC3-INSP-004` | Fuzzy/path root | reject before acquisition where possible |
| `SVC3-INSP-005` | Root from another generation | reject |
| `SVC3-INSP-006` | Service reconstructs signature/source relation | reject |
| `SVC3-INSP-007` | Possible relation promoted | reject/status invalid |
| `SVC3-INSP-008` | Source excerpt candidate becomes raw source automatically | reject |
| `SVC3-INSP-009` | Inspect profile requests undeclared expansion lane | reject |
| `SVC3-INSP-010` | Partial/conflict/omission hidden in envelope | reject |

## `context_build`

| ID | Case | Expected |
|---|---|---|
| `SVC3-BUILD-001` | Exact full build, no renderer | validated semantic pack |
| `SVC3-BUILD-002` | Exact build + canonical JSON renderer | pack + artifact |
| `SVC3-BUILD-003` | Exact build + deterministic Markdown | pack + artifact |
| `SVC3-BUILD-004` | Multiple permitted renderers | deterministic profile order |
| `SVC3-BUILD-005` | Renderer changes semantic selection | reject |
| `SVC3-BUILD-006` | Semantic pack valid, optional renderer fails under partial policy | exact partial/failure policy applied |
| `SVC3-BUILD-007` | Mandatory renderer fails | failure |
| `SVC3-BUILD-008` | Build returns truncation + continuation | retention admitted before advertisement |
| `SVC3-BUILD-009` | Retention denied and profile permits no-continuation truncated output | explicit continuation unavailable record |
| `SVC3-BUILD-010` | Retention denied but continuation still advertised | reject |
| `SVC3-BUILD-011` | Service edits pack to fit envelope | reject |
| `SVC3-BUILD-012` | Service runs search/model/rules | architecture fail |
| `SVC3-BUILD-013` | Exact token claim without exact E3-B tokenizer | reject |
| `SVC3-BUILD-014` | Source privacy denied | omission retained, no broadening |
| `SVC3-BUILD-015` | Build closes resources before success | pass |

## Continuation

| ID | Case | Expected |
|---|---|---|
| `SVC3-CONT-001` | Valid exact retained continuation | next exact page/pack |
| `SVC3-CONT-002` | Continuation contains current selector | reject |
| `SVC3-CONT-003` | Service resolves current during continuation | mutation fails |
| `SVC3-CONT-004` | Referenced generation unavailable/GCed | fail, no restart |
| `SVC3-CONT-005` | Receipt missing/invalid | fail |
| `SVC3-CONT-006` | Root/profile/privacy/renderer changed | reject |
| `SVC3-CONT-007` | Total budget reset | reject |
| `SVC3-CONT-008` | Original pack/request guard mismatch | reject |
| `SVC3-CONT-009` | New continuation replaces retention receipts | old released/new retained |
| `SVC3-CONT-010` | Continuation completes | receipts released idempotently |
| `SVC3-CONT-011` | Continuation expires | unavailable, exact failure |
| `SVC3-CONT-012` | Tampered oversized object | reject before acquisition |
| `SVC3-CONT-013` | Cancellation during continuation | unwind and preserve receipts per policy |
| `SVC3-CONT-014` | Previous omissions/truncation disappear | reject |
| `SVC3-CONT-015` | Same exact continuation 1/2/N scheduling | same result or idempotent owner classification |

## Validate and render artifact operations

| ID | Case | Expected |
|---|---|---|
| `SVC3-VAL-001` | Valid semantic pack StructuralOnly | complete + Valid payload |
| `SVC3-VAL-002` | Invalid semantic pack | complete + Invalid payload |
| `SVC3-VAL-003` | Invalid artifact classified as internal service failure | mutation fails |
| `SVC3-VAL-004` | Validator rewrites artifact | reject |
| `SVC3-VAL-005` | ExactOwnerClosure with retained generations | validate origins |
| `SVC3-VAL-006` | ExactOwnerClosure generation unavailable | failure/NotEvaluated by exact profile; no current |
| `SVC3-VAL-007` | Artifact host input path enters request/result | reject |
| `SVC3-VAL-008` | Huge/deep/unknown-schema artifact | bounded reject |
| `SVC3-VAL-009` | Rendered artifact source boundary invalid | Invalid payload |
| `SVC3-VAL-010` | Partial validation reported Valid complete closure | reject |
| `SVC3-RENDER-001` | Valid semantic pack + exact renderer | validated artifact |
| `SVC3-RENDER-002` | Invalid input pack | no rendering |
| `SVC3-RENDER-003` | Renderer/tokenizer profile mismatch | fail |
| `SVC3-RENDER-004` | Renderer changes pack/facts | reject |
| `SVC3-RENDER-005` | Output byte/token budget overflow | exact E3-B failure/truncation policy |
| `SVC3-RENDER-006` | Silent renderer fallback | reject |
| `SVC3-RENDER-007` | Exact-owner render uses current | reject |
| `SVC3-RENDER-008` | Source/privacy boundary fails | reject artifact |
| `SVC3-RENDER-009` | Service formats Markdown itself | architecture fail |
| `SVC3-RENDER-010` | Same pack/profile under terminal changes | same artifact bytes |

## Lifecycle and closure

| ID | Case | Expected |
|---|---|---|
| `SVC3-LIFE-001` | Success release reverse order | Complete closure report |
| `SVC3-LIFE-002` | Primary-only partial stack failure | release primary |
| `SVC3-LIFE-003` | Primary+platform stack failure | platform then primary release |
| `SVC3-LIFE-004` | Reference acquired then context bind fails | reference/platform/primary release |
| `SVC3-LIFE-005` | Context operation fails | all resources released |
| `SVC3-LIFE-006` | Context validation fails | all resources released |
| `SVC3-LIFE-007` | Renderer fails | all resources released |
| `SVC3-LIFE-008` | Envelope canonicalization fails | all resources released |
| `SVC3-LIFE-009` | Primary close fails | failure result, no complete success |
| `SVC3-LIFE-010` | Platform close fails | failure + remaining releases attempted |
| `SVC3-LIFE-011` | Reference close fails | failure + remaining releases attempted |
| `SVC3-LIFE-012` | Close failure hidden as warning | reject |
| `SVC3-LIFE-013` | Cancellation during close | cleanup continues synchronously |
| `SVC3-LIFE-014` | Late owner result after cancel | ignored/rejected, no second result |
| `SVC3-LIFE-015` | Background cleanup/continuation task | reject |
| `SVC3-LIFE-016` | Public success finalized before close | reject |
| `SVC3-LIFE-017` | Panic unwind scope guard | all safely unwindable resources released |
| `SVC3-LIFE-018` | Multiple public envelopes | reject |
| `SVC3-LIFE-019` | Broken pipe triggers service reinvocation | app integration fail |
| `SVC3-LIFE-020` | Handle/path/source leak in closure report | reject |

## Envelope and status

| ID | Case | Expected |
|---|---|---|
| `SVC3-ENV-001` | Complete map/build/inspect envelope | valid |
| `SVC3-ENV-002` | Partial context state | status partial |
| `SVC3-ENV-003` | Hard budget termination | status truncated |
| `SVC3-ENV-004` | No useful artifact, legitimate unsupported capability | status not_evaluated if profile permits |
| `SVC3-ENV-005` | Missing exact generation labeled not_evaluated | reject; failure |
| `SVC3-ENV-006` | Cancelled operation has success payload | reject |
| `SVC3-ENV-007` | Failed operation embeds malformed success payload | reject |
| `SVC3-ENV-008` | Invalid validation payload with service complete | valid distinction |
| `SVC3-ENV-009` | Status precedence conflict | reject |
| `SVC3-ENV-010` | Empty artifact arrays interpreted complete | reject |
| `SVC3-ENV-011` | Coverage/conflict/omission/truncation hidden | reject |
| `SVC3-ENV-012` | Symbolic selector replaces exact IDs | reject |
| `SVC3-ENV-013` | Current/LKG/failed target conflated | reject |
| `SVC3-ENV-014` | Resource closure state not Complete on success | reject |
| `SVC3-ENV-015` | CLI exit code enters result ID | reject |
| `SVC3-ENV-016` | Timing/host/terminal/log enters digest | reject |
| `SVC3-ENV-017` | Stable ordering under shuffled owner results | identical bytes |
| `SVC3-ENV-018` | Source bytes outside validated artifact | reject |
| `SVC3-ENV-019` | Raw owner handle included | reject |
| `SVC3-ENV-020` | Failure includes bounded safe resolution/closure summary | pass |

## Security, privacy, and dependency boundaries

| ID | Case | Expected |
|---|---|---|
| `SVC3-SEC-001` | Arbitrary store/project selector escape | reject |
| `SVC3-SEC-002` | Raw SQL/ATTACH/PRAGMA/connection | absent/reject |
| `SVC3-SEC-003` | Filesystem/network/process/editor/client access | absent/reject |
| `SVC3-SEC-004` | Source/repository/plugin/script execution | absent/reject |
| `SVC3-SEC-005` | Search/model/embedding/CBM invocation | absent/reject |
| `SVC3-SEC-006` | SavedVariables/log/runtime payload input | reject |
| `SVC3-SEC-007` | Source prompt changes operation/profile/tool behavior | mutation fails |
| `SVC3-SEC-008` | Private path/token/credential in error/log/envelope | reject |
| `SVC3-SEC-009` | Artifact mode broadens source privacy | reject |
| `SVC3-SEC-010` | Continuation broadens privacy/roots/budget | reject |
| `SVC3-SEC-011` | Huge selectors/roots/renderers/errors | bounded failure |
| `SVC3-SEC-012` | Integer/size overflow | reject before allocation |
| `SVC3-SEC-013` | Duplicate/unknown artifact JSON fields | reject per schema |
| `SVC3-SEC-014` | Context result authorizes tools/edits | reject claim |
| `SVC3-SEC-015` | App imports lower crate | architecture fail |
| `SVC3-SEC-016` | Service parses CLI/artifact path | architecture fail |
| `SVC3-SEC-017` | Repository-local config auto-loaded | reject |
| `SVC3-SEC-018` | External transport cryptographic claim in E3-C | deferred/unsupported |
| `SVC3-SEC-019` | Source denied but leaked in failure | reject |
| `SVC3-SEC-020` | Owner port returns unexpected extra records | reject |

## Determinism and freeze

| ID | Case | Expected |
|---|---|---|
| `SVC3-DET-001` | 1/2/N owner acquisition workers where allowed | same exact result/order |
| `SVC3-DET-002` | Owner response scheduling changes | same result or same typed failure |
| `SVC3-DET-003` | Current changes after resolution | no output switch |
| `SVC3-DET-004` | Terminal/locale/timezone/cwd/temp root changes | same canonical bytes |
| `SVC3-DET-005` | Store physical layout/WAL/checkpoint changes | same semantic result |
| `SVC3-DET-006` | Cache state changes | same semantic result |
| `SVC3-DET-007` | Equivalent request/config field order | same IDs |
| `SVC3-DET-008` | Repeated exact operation | same result unless explicit owner operational failure |
| `SVC3-FREEZE-001` | Documentation state with null pins | allowed |
| `SVC3-FREEZE-002` | First Rust commit with required null pins | reject |
| `SVC3-FREEZE-003` | Missing E3-B/E3-A implementation for requested profile | block |
| `SVC3-FREEZE-004` | Missing owner-port/profile/vector checksum | block |
| `SVC3-FREEZE-005` | Tests rewrite canonical fixtures | reject |
| `SVC3-FREEZE-006` | Cargo/Rust placeholder before gate | reject |
| `SVC3-FREEZE-007` | Missing integration/evaluation reported pass | reject |
| `SVC3-FREEZE-008` | Workflow/CI added without owner request | reject repository policy |

## Completion gate

E3-C is incomplete until every active case passes, all prerequisite implementations and exact owner-port/profile/fixture/checksum values are frozen, CLI integration cases pass, and no result relies on hidden retry/fallback, mixed generations, unclosed resources, lower-layer algorithm duplication, natural-language search, model/tool execution, or missing evidence reported as success.
