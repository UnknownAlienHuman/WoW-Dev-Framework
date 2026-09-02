# E5-B CLI acceptance and mutation matrix

**Status:** normative.

## Routing and dependency

| ID | Case | Expected |
|---|---|---|
| `A5B-ROUTE-001` | Every documented command maps to one service operation | pass |
| `A5B-ROUTE-002` | App imports only `wow-service` | pass |
| `A5B-ROUTE-003` | Direct recognizer/store/graph/project/vault import | architecture failure |
| `A5B-ROUTE-004` | Unknown command/option/config field | exit 64, no service call |
| `A5B-ROUTE-005` | Valid command invokes service exactly once | pass |
| `A5B-ROUTE-006` | App composes multiple public service calls | mutation fails |

## Selectors and inputs

| ID | Case | Expected |
|---|---|---|
| `A5B-IN-001` | Exact artifact ID/digest guard | mechanical transport |
| `A5B-IN-002` | Latest/best/highest/sole selector | reject |
| `A5B-IN-003` | Symbolic project current on allowed field | pass unchanged to service |
| `A5B-IN-004` | App resolves current/catalog | mutation fails |
| `A5B-IN-005` | Strict JSON file input | pass within limits |
| `A5B-IN-006` | Unknown/deep/oversized/polyglot JSON | reject |
| `A5B-IN-007` | Two stdin-consuming flags | reject |
| `A5B-IN-008` | Path traversal/symlink/reparse/device case | reject by profile |
| `A5B-IN-009` | Environment/cwd/home/Git/editor discovery | mutation fails |
| `A5B-IN-010` | Script/plugin/include/interpolation | reject |

## Run commands

| ID | Case | Expected |
|---|---|---|
| `A5B-RUN-001` | Submit exact request + operation ID | one service call |
| `A5B-RUN-002` | Retry replaces pack/profile/budget | reject |
| `A5B-RUN-003` | Retry transports original run/operation/digest | pass |
| `A5B-RUN-004` | Cancel automatically retries | mutation fails |
| `A5B-RUN-005` | List sorted by app timestamp/metric | mutation fails |
| `A5B-RUN-006` | Case explain leaks hidden holdout label | fail |

## Review commands

| ID | Case | Expected |
|---|---|---|
| `A5B-REV-001` | Strict review envelope transported | pass |
| `A5B-REV-002` | Plain `--approve` shortcut | reject |
| `A5B-REV-003` | GitHub/OS/CLI identity inserted | mutation fails |
| `A5B-REV-004` | Private key/token in argv/config | reject/redact |
| `A5B-REV-005` | App verifies/overrides graph semantics | mutation fails |
| `A5B-REV-006` | Review record response-loss causes second call | mutation fails |

## Holdout commands

| ID | Case | Expected |
|---|---|---|
| `A5B-HOLD-001` | Exact request/authorization envelope | one service call |
| `A5B-HOLD-002` | Vault path/token/credential flag | reject |
| `A5B-HOLD-003` | Caller-provided evaluator code/plugin | reject |
| `A5B-HOLD-004` | App opens vault directly | architecture failure |
| `A5B-HOLD-005` | Default output exposes membership/labels | fail |
| `A5B-HOLD-006` | Audit continuation with selector/budget override | reject |
| `A5B-HOLD-007` | OutcomeUnknown text says access failed/no effect | mutation fails |
| `A5B-HOLD-008` | Consumed holdout text says untouched | mutation fails |

## Promotion and deactivation

| ID | Case | Expected |
|---|---|---|
| `A5B-PROM-001` | Prepare exact submission request | one service call |
| `A5B-PROM-002` | CLI publishes/activates/promotes candidate | no command/architecture failure |
| `A5B-PROM-003` | Prepared rendered as published | mutation fails |
| `A5B-PROM-004` | Submission validation repairs artifact | mutation fails |
| `A5B-PROM-005` | Deactivation validates exact plan | pass |
| `A5B-PROM-006` | App executes deactivation | no command |

## Output and exit codes

| ID | Case | Expected |
|---|---|---|
| `A5B-OUT-001` | Envelope JSON | exact service bytes + LF |
| `A5B-OUT-002` | Artifact output | exact eligible bytes |
| `A5B-OUT-003` | Text preserves blocker/authorization/consumption/nonclaims | pass |
| `A5B-OUT-004` | Text hides Partial/NotEvaluated/OutcomeUnknown | mutation fails |
| `A5B-OUT-005` | Complete/Valid | exit 0 |
| `A5B-OUT-006` | Completed Invalid/checked denial | exit 1 |
| `A5B-OUT-007` | Partial/CandidateOnly/Blocked/Conflict/Truncated/NotEvaluated | exit 2 |
| `A5B-OUT-008` | Structured domain/authorization failure | exit 3 |
| `A5B-OUT-009` | Internal/OutcomeUnknown/closure/output failure | exit 4 |
| `A5B-OUT-010` | Pre-service transport failure | exit 64 |
| `A5B-OUT-011` | Cancelled | exit 130 |
| `A5B-OUT-012` | Banner/progress on canonical stdout | fail |

## Lifecycle and security

| ID | Case | Expected |
|---|---|---|
| `A5B-SEC-001` | Broken pipe | no retry/double output |
| `A5B-SEC-002` | Output file failure after service | exit 4, no second call |
| `A5B-SEC-003` | Source/label/review text becomes option/tool instruction | mutation fails |
| `A5B-SEC-004` | Credential/signature/vault secret in error | fail |
| `A5B-SEC-005` | Network/process/editor/WoW invocation | absent |
| `A5B-SEC-006` | Raw SQL/script/model/CBM path | absent |
| `A5B-SEC-007` | Cancellation before input/service/output | exact mapping, no duplicate call |
| `A5B-SEC-008` | Locale/terminal changes canonical output | no change |

## Freeze

| ID | Case | Expected |
|---|---|---|
| `A5B-FIX-001` | Null pins while implementation not started | allowed |
| `A5B-FIX-002` | First app Rust commit with required nulls | fail |
| `A5B-FIX-003` | Command/service bijection and output vectors frozen | pass |
| `A5B-FIX-004` | Cargo/.rs/workflow introduced in docs package | fail |

## Acceptance

The E5-B app is not implemented until every command maps 1:1 to service, direct lower access and local authorization are absent, exact input/output/exit/cancellation behavior passes, hidden holdout and credential data cannot leak, no publication command exists, and all canonical bytes and checksums freeze.
