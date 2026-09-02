# E6-B CLI acceptance matrix

**Status:** normative.

| ID | Case | Expected |
|---|---|---|
| `A6B-ROUTE-001` | Every command maps 1:1 to one service operation | pass |
| `A6B-ROUTE-002` | App imports only wow-service | pass |
| `A6B-ROUTE-003` | Valid command invokes service once | pass |
| `A6B-ROUTE-004` | App composes query + mapping + selection + context locally | fail |
| `A6B-IN-001` | Exact provider/state/query/result/mapping/selection IDs | pass through |
| `A6B-IN-002` | latest/best/first/sole/nearest/default provider selector | exit 64 |
| `A6B-IN-003` | unknown/deep/oversized/polyglot JSON | reject |
| `A6B-IN-004` | two stdin consumers | reject |
| `A6B-IN-005` | cwd/home/env/Git/editor/WoW discovery | fail |
| `A6B-IN-006` | include/interpolation/script/plugin/archive/RPC object | reject |
| `A6B-SEC-001` | secret material/private endpoint/provider DB input | exit 64 |
| `A6B-SEC-002` | generic MCP/tool/SQL/shell/model option | reject |
| `A6B-SEC-003` | app starts/configures/indexes provider | fail |
| `A6B-SEC-004` | app opens provider locator path/URL | fail |
| `A6B-SEC-005` | sensitive/private source in log/error | fail |
| `A6B-QRY-001` | query submit/get/list/cancel/continue | mapped |
| `A6B-QRY-002` | app retries OutcomeUnknown | reject |
| `A6B-QRY-003` | app switches provider/state on continuation | reject |
| `A6B-QRY-004` | app invokes hidden fallback | fail |
| `A6B-RES-001` | Candidate result JSON | exact bytes + LF |
| `A6B-RES-002` | zero result text says source absent | fail |
| `A6B-RES-003` | score rendered as framework confidence | fail |
| `A6B-RES-004` | artifact builder selects top/sole IDs | fail |
| `A6B-MAP-001` | exact mapping resolve/get | mapped |
| `A6B-MAP-002` | app chooses among MultipleMappings | fail |
| `A6B-MAP-003` | mapping text says provider semantics verified | fail |
| `A6B-MAP-004` | force-map/same-name/nearest option | exit 64 |
| `A6B-SEL-001` | exact selection record/get | mapped |
| `A6B-SEL-002` | app auto-selects top/highest-score/only result | fail |
| `A6B-SEL-003` | Selected/Rejected/Deferred preserved | pass |
| `A6B-SEL-004` | selection text says accepted/correct/edit-authorized | fail |
| `A6B-CTX-001` | context build/continue | mapped |
| `A6B-CTX-002` | app builds context or reads lower owners | fail |
| `A6B-CTX-003` | provider sidecar kept separate in output | pass |
| `A6B-CTX-004` | text says context verified provider interpretation | fail |
| `A6B-OUT-001` | envelope-json | exact service bytes + LF |
| `A6B-OUT-002` | artifact mode | exact eligible bytes |
| `A6B-OUT-003` | text preserves lanes/coverage/loss/nonclaims | pass |
| `A6B-OUT-004` | partial/conflict/NotEvaluated/OutcomeUnknown hidden | fail |
| `A6B-OUT-005` | zero complete -> exit 0 with nonclaim | pass |
| `A6B-OUT-006` | partial/truncated zero -> exit 2 | pass |
| `A6B-OUT-007` | SelectionRecorded Rejected/Deferred -> exit 0 | pass |
| `A6B-LIFE-001` | cancellation | one service cancellation, exit 130 |
| `A6B-LIFE-002` | broken pipe/output failure | no second service call, exit 4 |
| `A6B-LIFE-003` | service OutcomeUnknown | unsafe-to-retry text, exit 4 |
| `A6B-DET-001` | locale/terminal/timing changes | same machine bytes/exit |
| `A6B-FIX-001` | null pins before implementation | allowed |
| `A6B-FIX-002` | first Rust commit with required nulls | fail |

Implementation requires every nondeferred case plus platform path/file/signal/broken-pipe tests and exact frozen service bytes.