# E6-B CLI acceptance and mutation matrix

**Status:** normative. IDs are unique within the application package.

## Routing and dependencies

| ID | Case | Expected |
|---|---|---|
| `A6B-ROUTE-001` | 18 commands map 1:1 to 18 service operations | pass |
| `A6B-ROUTE-002` | App imports only `wow-service` | pass |
| `A6B-ROUTE-003` | Direct `wow-cbm`/project/reference/context/store import | architecture failure |
| `A6B-ROUTE-004` | Valid command invokes service exactly once | pass |
| `A6B-ROUTE-005` | App composes authorization/session/mapping/context calls | mutation fails |
| `A6B-ROUTE-006` | Unknown command/option | exit 64, zero service calls |
| `A6B-ROUTE-007` | Nonexistent provider management command | exit 64 |
| `A6B-ROUTE-008` | Cargo/Rust/workflow added during docs package | fail |

## Input and configuration

| ID | Case | Expected |
|---|---|---|
| `A6B-IN-001` | Strict explicit JSON file | pass within limits |
| `A6B-IN-002` | Strict stdin JSON | pass |
| `A6B-IN-003` | Two stdin-consuming options | reject |
| `A6B-IN-004` | Unknown/deep/oversized/polyglot JSON | reject |
| `A6B-IN-005` | Implicit cwd/home/env/Git/editor/WoW discovery | mutation fails |
| `A6B-IN-006` | Include/interpolation/script/plugin config | reject |
| `A6B-IN-007` | Path traversal/symlink/reparse/device/UNC/ADS | reject by profile |
| `A6B-IN-008` | Provider locator used as local input/output path | reject |
| `A6B-IN-009` | Symbolic allowed selector passed unchanged | pass |
| `A6B-IN-010` | App resolves current/catalog locally | mutation fails |

## Provider, credentials, and state

| ID | Case | Expected |
|---|---|---|
| `A6B-PROV-001` | Nonsecret provider/profile IDs | pass |
| `A6B-PROV-002` | API token/cookie/private key/private endpoint | reject/redact |
| `A6B-PROV-003` | Opaque session handle/provider cursor | reject |
| `A6B-PROV-004` | Raw MCP/tool/method/endpoint flag | reject |
| `A6B-PROV-005` | Provider install/start/configure/index/delete command | absent |
| `A6B-PROV-006` | OS/GitHub/CLI identity treated authorization | mutation fails |
| `A6B-PROV-007` | Continuation replaces external state | reject/service invalid |
| `A6B-PROV-008` | Reconcile replaces original query/provider | reject |

## Query and result commands

| ID | Case | Expected |
|---|---|---|
| `A6B-QUERY-001` | `external query` -> `external_candidate_query` | one call |
| `A6B-QUERY-002` | `external continue` -> exact continuation operation | one call |
| `A6B-QUERY-003` | result get/list/validate exact mappings | one call each |
| `A6B-QUERY-004` | explain exact result/candidate | one call |
| `A6B-QUERY-005` | artifact build exact request | one call |
| `A6B-QUERY-006` | App sorts list newest/best/highest score | mutation fails |
| `A6B-QUERY-007` | Provider zero-result text says globally absent | mutation fails |
| `A6B-QUERY-008` | Candidate text says proven/verified source | mutation fails |
| `A6B-QUERY-009` | Continuation resets cumulative budget | reject |
| `A6B-QUERY-010` | Cache validate calls provider implicitly | mutation fails |

## Mapping and selection

| ID | Case | Expected |
|---|---|---|
| `A6B-MAP-001` | `external map` transports exact mapping request | one call |
| `A6B-MAP-002` | mapping validate exact receipt | one call |
| `A6B-MAP-003` | App follows provider URL/path | mutation fails |
| `A6B-MAP-004` | App maps by name/path/snippet/fuzzy/search | mutation fails |
| `A6B-MAP-005` | MultipleMappings rendered exact conflict/choice required | pass |
| `A6B-MAP-006` | NoMappingPartial rendered “does not exist” | mutation fails |
| `A6B-SEL-001` | exact selection request/origin | one call |
| `A6B-SEL-002` | `--top`/`--first`/`--best`/`--sole` | exit 64 |
| `A6B-SEL-003` | App selects row 1 automatically | mutation fails |
| `A6B-SEL-004` | selection validate exact receipt | one call |
| `A6B-SEL-005` | Selected rendered as proof/edit permission | mutation fails |
| `A6B-SEL-006` | Rejected rendered candidate false | mutation fails |

## Context and external evidence

| ID | Case | Expected |
|---|---|---|
| `A6B-CTX-001` | exact selection/root/context request | one call |
| `A6B-CTX-002` | App invokes context/project/reference directly | architecture failure |
| `A6B-CTX-003` | Provider prose/rank/score rendered as framework fact | mutation fails |
| `A6B-CTX-004` | External attachment labelled Candidate/nonauthority | pass |
| `A6B-CTX-005` | Denied provider snippet omitted explicitly | pass |
| `A6B-CTX-006` | Context failure triggers provider snippet fallback | mutation fails |
| `A6B-CTX-007` | Mapping/current mismatch hidden | mutation fails |
| `A6B-CTX-008` | Context partial/conflict/truncated retained | pass |

## Output and exits

| ID | Case | Expected |
|---|---|---|
| `A6B-OUT-001` | envelope JSON | exact service bytes + one LF |
| `A6B-OUT-002` | artifact output | exact eligible bytes |
| `A6B-OUT-003` | text preserves Candidate/nonclaims/state/mapping/selection | pass |
| `A6B-OUT-004` | stdout banner/progress in machine mode | fail |
| `A6B-OUT-005` | Complete/NoChange/Valid/ExactMapped/Selected | exit 0 as profiled |
| `A6B-OUT-006` | completed Invalid/Rejected | exit 1 |
| `A6B-OUT-007` | CandidateOnly/Partial/Blocked/Conflict/Truncated/NotEvaluated | exit 2 |
| `A6B-OUT-008` | structured domain/infrastructure failure | exit 3 |
| `A6B-OUT-009` | OutcomeUnknown/internal/close/output failure | exit 4 |
| `A6B-OUT-010` | pre-service transport failure | exit 64 |
| `A6B-OUT-011` | Cancelled | exit 130 |
| `A6B-OUT-012` | Text calls zero-result authoritative absence | fail |
| `A6B-OUT-013` | Text hides external-state class/OutcomeUnknown | fail |
| `A6B-OUT-014` | Text hides mapping ambiguity/nonclaims | fail |

## Lifecycle and security

| ID | Case | Expected |
|---|---|---|
| `A6B-LIFE-001` | Broken pipe | no retry/second output |
| `A6B-LIFE-002` | Output file failure after service | exit 4, no second call |
| `A6B-LIFE-003` | Signal before service | exit 130, no call |
| `A6B-LIFE-004` | Signal after provider dispatch | exact service reconciliation state |
| `A6B-LIFE-005` | Background retry/cleanup after return | mutation fails |
| `A6B-SEC-001` | Credential/session/cursor in stdout/stderr/log | fail |
| `A6B-SEC-002` | Provider path/snippet/private source in default error | fail |
| `A6B-SEC-003` | Provider text creates option/tool/system instruction | mutation fails |
| `A6B-SEC-004` | Hidden fallback to another provider/model/cache/search | mutation fails |
| `A6B-SEC-005` | Cross-consumer privacy widening | reject |
| `A6B-SEC-006` | Raw SQL/script/plugin/model execution path | absent |
| `A6B-SEC-007` | Locale/terminal changes canonical bytes | no change |
| `A6B-SEC-008` | 1/2/N scheduling and same service result | same output/exit |

## Freeze

| ID | Case | Expected |
|---|---|---|
| `A6B-FIX-001` | Null pins while implementation not started | allowed |
| `A6B-FIX-002` | First app Rust commit with required nulls | fail |
| `A6B-FIX-003` | Command/service/bytes/exit/security vectors frozen | pass |
| `A6B-FIX-004` | Test rewrites committed fixture | fail |

## Acceptance

The app cannot be marked implemented until every command maps 1:1 to service, no lower dependency/session/credential/mapping/context algorithm exists, all exact inputs/outputs/exits/cancellation/broken-pipe/privacy cases pass, Candidate and mapping/selection nonclaims remain visible, and all canonical vectors/checksums freeze.
