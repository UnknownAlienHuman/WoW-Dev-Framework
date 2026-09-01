# E3-B test matrix

**Status:** normative acceptance, property, mutation, security, and determinism matrix.

Every case must execute before implementation is called complete unless explicitly marked as a prerequisite/freeze gate. Case IDs are stable and unique.

## Milestone, schema, and identity

| ID | Case | Expected |
|---|---|---|
| `CTX-SCHEMA-001` | Current E3-B contract/profile schemas | pass |
| `CTX-SCHEMA-002` | Unknown field or enum | reject |
| `CTX-SCHEMA-003` | Breaking profile change without version | reject |
| `CTX-SCHEMA-004` | Retired E3-A and current types both implemented | reject |
| `CTX-SCHEMA-005` | Retired and current operations both implemented | reject |
| `CTX-SCHEMA-006` | Contract says E3-B but router says E3-A | reject |
| `CTX-DAG-001` | Semantic pack includes renderer artifact ID | reject |
| `CTX-DAG-002` | Semantic pack includes metrics/evaluation ID | reject |
| `CTX-DAG-003` | Map ID includes pack ID | reject |
| `CTX-DAG-004` | Request ID includes expansion result | reject |
| `CTX-DAG-005` | Selection trace creates item identity cycle | reject/split nonidentity ref |
| `CTX-DAG-006` | Exact rendered bytes included before renderer layer | reject |
| `CTX-DAG-007` | Cache key includes physical path/hit counter | reject |
| `CTX-DAG-008` | Universe set includes mutable current pointer | reject |
| `CTX-DAG-009` | Validator rewrites same ID | reject |
| `CTX-DAG-010` | Continuation changes identity inputs | reject |
| `CTX-DAG-011` | Source excerpt ID includes Markdown output range | reject |
| `CTX-DAG-012` | Evaluation score controls semantic selection | reject |

## Universe binding and immutable inputs

| ID | Case | Expected |
|---|---|---|
| `CTX-IN-001` | Exact compatible user project/graph/reference views | bind |
| `CTX-IN-002` | Mix project and graph publications | reject generation mismatch |
| `CTX-IN-003` | ReferenceProfile incompatible with project | reject |
| `CTX-IN-004` | Optional Blizzard UI absent and profile permits | explicit omission/partial policy |
| `CTX-IN-005` | Blizzard UI required but absent | reject/NotEvaluated by operation contract |
| `CTX-IN-006` | Current changes after bind | existing operation stays exact |
| `CTX-IN-007` | Retry resolves newer current silently | reject/new request required |
| `CTX-IN-008` | Merge same display name across universes | reject |
| `CTX-IN-009` | Same path in two generations | distinct identities |
| `CTX-IN-010` | Owner returns wrong universe record | reject |
| `CTX-IN-011` | Source handle digest/coordinate mismatch | reject |
| `CTX-IN-012` | Wrong Blizzard SkeletonInputView generation | reject |
| `CTX-IN-013` | Raw SQLite/analyzer/parser handle supplied | reject |
| `CTX-IN-014` | Legacy StoreImageId supplied | reject |
| `CTX-IN-015` | Floating current/latest inside canonical request | reject |
| `CTX-IN-016` | Exact read view becomes unavailable mid-request | typed failure/partial, no substitution |
| `CTX-IN-017` | User-project-only profile receives platform rows | reject leakage |
| `CTX-IN-018` | Combined view references separate maps and exact cross-links | pass |

## Request, roots, and profiles

| ID | Case | Expected |
|---|---|---|
| `CTX-REQ-001` | Exact graph/entity root allowed by intent | pass |
| `CTX-REQ-002` | Root outside bound universe | reject |
| `CTX-REQ-003` | Replace root with fuzzy name | reject |
| `CTX-REQ-004` | Natural-language root in semantic selector | reject |
| `CTX-REQ-005` | Executable selector/callback/script/SQL/regex | reject |
| `CTX-REQ-006` | Unknown axis/relation/facet | reject |
| `CTX-REQ-007` | Root kind incompatible with intent | reject |
| `CTX-REQ-008` | Request asks to upgrade Possible to Proven | reject |
| `CTX-REQ-009` | Zero/negative/unlimited/overflow budget | reject |
| `CTX-REQ-010` | Unknown tokenizer/renderer/privacy profile | reject |
| `CTX-REQ-011` | Equivalent reordered request fields | same normalized ID |
| `CTX-REQ-012` | Source text supplied as profile/intent | reject/control data separation |
| `CTX-REQ-013` | `PrepareChangeContext` requests edit generation | reject edit; context only |
| `CTX-REQ-014` | Opaque audit text changes selection | mutation fails |

## Project Map

| ID | Case | Expected |
|---|---|---|
| `CTX-MAP-001` | Small exact user project map | valid |
| `CTX-MAP-002` | Separate Blizzard UI map | valid and distinct |
| `CTX-MAP-003` | Combined map references both | no namespace collapse |
| `CTX-MAP-004` | Every node resolves exact underlying key | pass |
| `CTX-MAP-005` | Every edge resolves relation assertion/path | pass |
| `CTX-MAP-006` | Group membership/counts exact | pass |
| `CTX-MAP-007` | Replace reason path with direct edge | reject |
| `CTX-MAP-008` | Generic parent relation synthesized | reject |
| `CTX-MAP-009` | Group by repository/addon/path popularity | reject |
| `CTX-MAP-010` | Mixed/conflicted role group | conflict retained |
| `CTX-MAP-011` | Budgeted map page | deterministic omissions/cursor |
| `CTX-MAP-012` | Total-known count under partial input | labeled partial |
| `CTX-MAP-013` | DB/hash/worker order changes | same map bytes |
| `CTX-MAP-014` | Full graph loaded for tiny map despite bounded reads | architecture test fails |
| `CTX-MAP-015` | Model-generated responsibility facet | reject |
| `CTX-MAP-016` | Static map asserts runtime readiness | reject authority upgrade |

## L0 skeleton

| ID | Case | Expected |
|---|---|---|
| `CTX-L0-001` | Project/package/file/module L0 | valid |
| `CTX-L0-002` | Body included in default L0 | reject |
| `CTX-L0-003` | Infer role from `Core`/`Manager` filename | reject |
| `CTX-L0-004` | Exact recognizer role retained | original confidence/evidence |
| `CTX-L0-005` | Large members | stable page/count/cursor |
| `CTX-L0-006` | Partial member coverage | not rendered as all |
| `CTX-L0-007` | Source documentation quote | labeled untrusted source data |
| `CTX-L0-008` | Mandatory identity/evidence pruned | reject |
| `CTX-L0-009` | Direct ownership/load facts | exact relation refs |
| `CTX-L0-010` | Same names after rename mutation | universal facets unchanged |
| `CTX-L0-011` | Unsupported scope | explicit Unsupported/NotEvaluated |
| `CTX-L0-012` | L1 cost/route points to wrong generation | reject |

## L1 and control/effect projection

| ID | Case | Expected |
|---|---|---|
| `CTX-L1-001` | Exact entity/signature/type/span | valid |
| `CTX-L1-002` | Same-name entity substitution | reject |
| `CTX-L1-003` | Direct relation and path both present | remain distinct |
| `CTX-L1-004` | Native/custom/CVar event systems | separate facets |
| `CTX-L1-005` | SetScript/HookScript/hooksecurefunc | separate facts |
| `CTX-L1-006` | Upgrade Possible relation to Proven | reject |
| `CTX-L1-007` | Literal and dynamic state paths | exact versus Possible preserved |
| `CTX-L1-008` | Same-name global treated as SavedVariables root | reject |
| `CTX-L1-009` | Exact API relation enriched by ReferenceView | pass |
| `CTX-L1-010` | Implementation source used as API contract | reject authority class |
| `CTX-L1-011` | Hook relation used as taint/combat safety | reject |
| `CTX-L1-012` | Full body included without explicit excerpt policy | reject |
| `CTX-CE-001` | Published exact call relation | projected with origins |
| `CTX-CE-002` | Published possible dynamic call | remains Possible |
| `CTX-CE-003` | Unsupported control region | Unknown/NotEvaluated |
| `CTX-CE-004` | Run second parser/CFG/SSA/data-flow | reject |
| `CTX-CE-005` | Infer runtime order from static load | reject |
| `CTX-CE-006` | Convert hook to safety claim | reject |
| `CTX-CE-007` | Call/state cycle | bounded cycle-safe projection |
| `CTX-CE-008` | Collapsed region lacks member manifest/route | reject |
| `CTX-CE-009` | Omitted region lacks reason | reject |
| `CTX-CE-010` | Unknown region rendered empty | reject |

## Expansion, selection, stopping, and continuation

| ID | Case | Expected |
|---|---|---|
| `CTX-EXP-001` | Ordered stages with prerequisites | pass |
| `CTX-EXP-002` | Stage executes before prerequisite | reject |
| `CTX-EXP-003` | Candidate origin/dependency/cost closure | pass |
| `CTX-EXP-004` | Candidate dependency cycle | reject |
| `CTX-EXP-005` | Mandatory closure selected first | pass |
| `CTX-EXP-006` | Mandatory candidate pruned | reject |
| `CTX-EXP-007` | Optional tiers and stable ties | deterministic |
| `CTX-EXP-008` | Same-tier DB/worker order change | same selection |
| `CTX-EXP-009` | Dedup identical item | all reasons/evidence retained |
| `CTX-EXP-010` | Dedup same name across universes | reject |
| `CTX-EXP-011` | Hidden root/axis/confidence broadening | reject |
| `CTX-EXP-012` | One round adds unseen evidence | continue if profile permits |
| `CTX-EXP-013` | One round adds duplicates only | NoNewEvidence |
| `CTX-EXP-014` | Budget-pruned item repeated | not new evidence |
| `CTX-EXP-015` | Stop at max depth/fanout | explicit truncation/continuation |
| `CTX-EXP-016` | Conflict blocks required facet | explicit blocked state |
| `CTX-EXP-017` | Source excerpts fetched before selection | architecture test fails |
| `CTX-EXP-018` | Natural-language/model relevance score | reject |
| `CTX-EXP-019` | Cancel between owner queries | bounded cancellation |
| `CTX-EXP-020` | Late worker results change order | mutation fails |
| `CTX-EXP-021` | Change privacy profile on continuation | reject |
| `CTX-CONT-001` | Same universe/request/profile/budget chain | stable continuation |
| `CTX-CONT-002` | Continue on another graph generation | reject |
| `CTX-CONT-003` | Tamper cursor/frontier/stable key | reject |
| `CTX-CONT-004` | Reset total budget on next page | reject |
| `CTX-CONT-005` | Hide prior omissions/truncation on next page | reject |
| `CTX-CONT-006` | Future page ID in prior pack | reject DAG |
| `CTX-CANCEL-001` | Cancel before expansion | cancelled, no artifact complete |
| `CTX-CANCEL-002` | Cancel during excerpt/tokenization | cancelled, no background work |
| `CTX-CANCEL-003` | Cache cancelled pack as complete | reject |

## Coverage, authority, conflicts, loss, and omissions

| ID | Case | Expected |
|---|---|---|
| `CTX-COV-001` | Complete required scopes | CompleteForRequest |
| `CTX-COV-002` | Required upstream capability NotEvaluated | partial/fail by request policy |
| `CTX-COV-003` | Optional facet unavailable | omission with impact |
| `CTX-COV-004` | Optional platform universe absent | explicit policy-scoped result |
| `CTX-COV-005` | Coverage axes collapsed into one boolean | reject |
| `CTX-COV-006` | Every claim origin/evidence closure | pass |
| `CTX-COV-007` | Source-class Proven rendered platform/runtime proof | reject |
| `CTX-COV-008` | Multiple Possible assertions become Proven | reject |
| `CTX-COV-009` | Conflicting exclusive assertions | retain conflict/block exact claim |
| `CTX-COV-010` | Conflict hidden by renderer | reject |
| `CTX-COV-011` | Known omitted candidate lacks record | reject |
| `CTX-COV-012` | Unenumerated query frontier reported pruned | reject classification |
| `CTX-COV-013` | Candidate budget-pruned | BudgetPruned omission |
| `CTX-COV-014` | Empty result under partial coverage | no negative authority |
| `CTX-COV-015` | Owner supplies exact negative decision | include with exact scope |
| `CTX-COV-016` | NoNewEvidence rendered as absence | reject |
| `CTX-COV-017` | Projection loss hidden | reject |
| `CTX-COV-018` | Existing finding from another generation | reject |
| `CTX-COV-019` | Count under partial coverage rendered total | reject |
| `CTX-COV-020` | DuplicateCovered omission loses selected ref | reject |

## Source, privacy, license, and boundaries

| ID | Case | Expected |
|---|---|---|
| `CTX-SOURCE-001` | Exact handle/digest/range/local policy | exact/transformed typed item |
| `CTX-SOURCE-002` | Arbitrary path fallback | reject |
| `CTX-SOURCE-003` | Wrong generation/source handle | reject |
| `CTX-SOURCE-004` | Source line attempts boundary close | stays quoted; malformed renderer rejected |
| `CTX-SOURCE-005` | Markdown fence/JSON closer/tool request in source | inert data |
| `CTX-SOURCE-006` | Unsupported encoding | explicit unsupported/metadata only |
| `CTX-SOURCE-007` | External consumer, unknown license | deny source bytes |
| `CTX-SOURCE-008` | Private source allowed only locally | enforce consumer class |
| `CTX-SOURCE-009` | Unknown privacy defaults to external allowed | reject |
| `CTX-SOURCE-010` | Source denied by trust | metadata/omission only |
| `CTX-SOURCE-011` | Deterministic exact-range redaction | transformation record valid |
| `CTX-SOURCE-012` | Heuristic detector silently deletes text | reject |
| `CTX-SOURCE-013` | Absolute private path in output | reject/redact by exact policy |
| `CTX-SOURCE-014` | Credential/token in error/log | confidentiality failure |
| `CTX-SOURCE-015` | Virtual Lua excerpt without physical map | reject |
| `CTX-SOURCE-016` | Source closes framework boundary | boundary round trip fails if possible |
| `CTX-SOURCE-017` | Source text changes profile/selection/tool policy | mutation fails |
| `CTX-SOURCE-018` | Adjacent context expands beyond profile | reject |
| `CTX-SOURCE-019` | Truncated excerpt lacks original/returned ranges | reject |
| `CTX-SOURCE-020` | Source fetched from wrong universe after same path | reject |
| `CTX-SEC-001` | Filesystem/network/process/editor/client access | absent/reject |
| `CTX-SEC-002` | Lua/XML/JS/Wasm/plugin/source execution | absent/reject |
| `CTX-SEC-003` | Raw SQL/storage/analyzer access | absent/reject |
| `CTX-SEC-004` | SavedVariables/log/runtime payload input | reject |
| `CTX-SEC-005` | Oversized/deep/high-fanout records | bounded failure/truncation |
| `CTX-SEC-006` | Unicode controls/control characters | canonical safe handling |
| `CTX-SEC-007` | Context pack claims downstream tool authorization | reject |
| `CTX-SEC-008` | Cache reuse crosses privacy class | reject |

## Budgets and tokenization

| ID | Case | Expected |
|---|---|---|
| `CTX-BUDGET-001` | Mandatory closure fits | optional selection proceeds |
| `CTX-BUDGET-002` | Mandatory closure exceeds hard limit | fail |
| `CTX-BUDGET-003` | Optional item fits with dependency closure | select |
| `CTX-BUDGET-004` | Optional item overflows | whole-item omission |
| `CTX-BUDGET-005` | Cut structured item/evidence mid-record | reject |
| `CTX-BUDGET-006` | Semantic fits, Markdown overflows | renderer failure/replan by explicit profile |
| `CTX-BUDGET-007` | Raw rendered output sliced | reject |
| `CTX-BUDGET-008` | Predicted versus exact cost mismatch | deterministic rollback/fail |
| `CTX-BUDGET-009` | Source excerpt pool exhausted | explicit omission/continuation |
| `CTX-BUDGET-010` | Mandatory boundary metadata removed for budget | reject |
| `CTX-BUDGET-011` | Allocation changes with worker order | reject |
| `CTX-BUDGET-012` | Unsupported unlimited request | reject |
| `CTX-BUD-013` | Exact-token claim without exact profile | reject |
| `CTX-TOKEN-001` | Frozen tokenizer vectors | exact stable count |
| `CTX-TOKEN-002` | Tokenizer digest mismatch | unavailable/fail hard-token gate |
| `CTX-TOKEN-003` | Exact claim without frozen tokenizer | reject |
| `CTX-TOKEN-004` | Deterministic estimate labeled exact | reject |
| `CTX-TOKEN-005` | Upper-bound assumptions violated | unavailable |
| `CTX-TOKEN-006` | Model display name only | insufficient profile |
| `CTX-TOKEN-007` | Framing/special-token policy changes | new profile/artifact identity |
| `CTX-TOKEN-008` | Tokenizer fails after selection | explicit fallback/new identity or fail |

## Semantic pack and rendering

| ID | Case | Expected |
|---|---|---|
| `CTX-PACK-001` | Valid semantic pack closure | pass |
| `CTX-PACK-002` | Dangling item/evidence/source/reference | reject |
| `CTX-PACK-003` | Drop required omission manifest | reject |
| `CTX-PACK-004` | Mixed generations | reject |
| `CTX-PACK-005` | Canonical JSON round trip | identical semantic object/ID |
| `CTX-PACK-006` | Renderer/metric ID in semantic identity | reject |
| `CTX-PACK-007` | Partial pack marked complete | reject |
| `CTX-PACK-008` | Same semantic pack, two renderers | same pack, distinct artifacts |
| `CTX-PACK-009` | Renderer omits required semantic item silently | reject |
| `CTX-PACK-010` | Renderer changes confidence/coverage | reject |
| `CTX-PACK-011` | Renderer adds free-form claim | reject |
| `CTX-PACK-012` | Reason path rendered as direct relation | reject |
| `CTX-PACK-013` | Source and framework fact sections merge | reject |
| `CTX-PACK-014` | Item/output mapping invalid | reject |
| `CTX-PACK-015` | Lossy renderer lacks RenderingLossRecord | reject |
| `CTX-PACK-016` | Validator repairs artifact in place | reject |
| `CTX-REN-001` | Canonical JSON exact bytes | deterministic |
| `CTX-REN-002` | Markdown exact bytes/line endings/templates | deterministic |
| `CTX-REN-003` | Independent legacy ContextBundleCore object | reject |
| `CTX-REN-004` | Source controls heading/template/boundary | reject |

## Cache and determinism

| ID | Case | Expected |
|---|---|---|
| `CTX-CACHE-001` | Exact key and valid artifact | hit |
| `CTX-CACHE-002` | Floating `current` key | reject |
| `CTX-CACHE-003` | Symbol/path/model name only | reject |
| `CTX-CACHE-004` | Different project generation | reject relabel |
| `CTX-CACHE-005` | Different reference/graph/profile | miss/reject |
| `CTX-CACHE-006` | Different privacy/consumer profile | reject |
| `CTX-CACHE-007` | Different tokenizer/framing/renderer | distinct key |
| `CTX-CACHE-008` | Partial pack used for complete request | reject |
| `CTX-CACHE-009` | Corrupted artifact bytes | reject |
| `CTX-CACHE-010` | Unresolvable source/evidence under required validation | miss/fail |
| `CTX-CACHE-011` | Physical cache callback/path enters semantic API | reject |
| `CTX-CACHE-012` | Equivalent immutable subrecord reuse | retain identity + new exact binding |
| `CTX-DET-001` | 1/2/N workers | identical semantic/rendered bytes |
| `CTX-DET-002` | Shuffled owner results | identical |
| `CTX-DET-003` | Different DB layout/checkpoint | identical logical output |
| `CTX-DET-004` | Different host/temp root/clock | identical |
| `CTX-DET-005` | Cold versus warm external cache | identical |
| `CTX-DET-006` | Hash-map iteration changes | identical |
| `CTX-DET-007` | Reordered equivalent request JSON | identical |
| `CTX-DET-008` | Rename repo/package/path only | universal semantics stable |

## Metrics and evaluation

| ID | Case | Expected |
|---|---|---|
| `CTX-METRIC-001` | Same pack, different timing | same semantic ID |
| `CTX-METRIC-002` | Timing/score enters semantic identity | reject |
| `CTX-METRIC-003` | Operational metric missing | does not invalidate semantics unless gate requires |
| `CTX-EVAL-001` | Synthetic exact corpus | mandatory recall/origin measured |
| `CTX-EVAL-002` | Pinned roth-ui corpus | compression/utility classified |
| `CTX-EVAL-003` | Pinned Blizzard UI corpus | platform map/skeleton utility classified |
| `CTX-EVAL-004` | Repository/package/path rename mutation | universal selection stable |
| `CTX-EVAL-005` | Required evaluation not executed | NotEvaluated/block completion |
| `CTX-EVAL-006` | Consumer score changes canonical selection | reject |
| `CTX-EVAL-007` | Moving unpinned model/provider baseline | reject as reproducible gate |
| `CTX-EVAL-008` | Mandatory security/authority gate failure | fail regardless optional utility |

## Prerequisites and freeze

| ID | Case | Expected |
|---|---|---|
| `CTX-FREEZE-001` | Documentation state with null pins | allowed |
| `CTX-FREEZE-002` | First Rust commit with required null pins | reject |
| `CTX-FREEZE-003` | Missing E0-E2 implementation/fixture | block |
| `CTX-FREEZE-004` | Platform profile requires missing E3-A implementation | block |
| `CTX-FREEZE-005` | Missing profile/corpus/vector/checksum | block |
| `CTX-FREEZE-006` | Tests rewrite fixtures automatically | reject |
| `CTX-FREEZE-007` | Rust/Cargo placeholder before gate | reject |
| `CTX-FREEZE-008` | Missing tool/probe reported pass | reject |

## Completion gate

E3-B is complete only when all active cases pass, required evaluation gates have executable evidence, fixture/checksum nulls are frozen, and no result relies on a retired E3-A context implementation, source parsing, raw storage/analyzer access, search/model ranking, hidden omission, authority upgrade, or external side effect.
