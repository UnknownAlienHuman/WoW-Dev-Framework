# E7-A MCP adapter acceptance and mutation matrix

**Status:** normative. IDs are unique within the MCP adapter package.

## Framing and initialization

| ID | Case | Expected |
|---|---|---|
| `M7A-FRAME-001` | Valid frozen stdio MCP message | one parsed message |
| `M7A-FRAME-002` | Malformed/oversized/deep/duplicate-key JSON | protocol failure, no service call |
| `M7A-FRAME-003` | Protocol stdout mixed with logs | mutation fails |
| `M7A-INIT-001` | Supported exact initialization | one `session_initialize` call |
| `M7A-INIT-002` | Unsupported protocol revision/profile | fail initialization |
| `M7A-INIT-003` | Second initialize or normal call before initialize | reject |
| `M7A-INIT-004` | Client name/capability changes semantic behavior | mutation fails |
| `M7A-INIT-005` | Advertise sampling/elicitation/dynamic tools | mutation fails |
| `M7A-INIT-006` | Initialization success before authorization/retention closure | mutation fails |

## Static tools and one-call mapping

| ID | Case | Expected |
|---|---|---|
| `M7A-TOOL-001` | `tools/list` | static authorized deterministic descriptors |
| `M7A-TOOL-002` | Unknown/dynamic source-defined tool | reject |
| `M7A-TOOL-003` | Generic `call_tool` or service operation argument | reject |
| `M7A-TOOL-004` | Protocol status tool | exactly one `protocol_status` call |
| `M7A-TOOL-005` | Workspace bind/status tool | one corresponding service call |
| `M7A-TOOL-006` | Diagnostics/hover/definition/references/symbols/actions | one corresponding service call |
| `M7A-TOOL-007` | Search/context | one corresponding service call |
| `M7A-TOOL-008` | External Candidate query | one E6-B query call |
| `M7A-TOOL-009` | External Candidate map | one exact owner mapping call |
| `M7A-TOOL-010` | External Candidate select | one explicit selection call |
| `M7A-TOOL-011` | External Candidate context | one exact selected-root context call |
| `M7A-TOOL-012` | Adapter combines query+top-1+context | mutation fails |
| `M7A-TOOL-013` | Top/best/highest-score/sole selector | reject |
| `M7A-TOOL-014` | Provider “exact” label promoted | mutation fails |
| `M7A-TOOL-015` | Operation status/cancel | one corresponding service call |
| `M7A-TOOL-016` | Direct lower owner/provider/database call | architecture failure |

## Resources

| ID | Case | Expected |
|---|---|---|
| `M7A-RES-001` | `resources/list` | bounded authorized stable list |
| `M7A-RES-002` | Exact opaque diagnostic/context/result resource | one exact read |
| `M7A-RES-003` | Arbitrary file/HTTP/repository/provider URI | reject |
| `M7A-RES-004` | Source-embedded link followed | mutation fails |
| `M7A-RES-005` | Raw DB/WAL/store/root/credential resource | reject |
| `M7A-RES-006` | Cross-session/private resource | deny |
| `M7A-RES-007` | Resource authorization revoked/expired | deny/audit |
| `M7A-RES-008` | Empty resource list interpreted global absence | mutation fails |
| `M7A-RES-009` | Source excerpt resource | exact boundary/privacy/license state |
| `M7A-RES-010` | Hidden holdout/review/signing material | unavailable |
| `M7A-RES-011` | Continuation changes generation/privacy/budget | reject |
| `M7A-RES-012` | Same exact artifact/profile | deterministic resource bytes |

## Results and authority

| ID | Case | Expected |
|---|---|---|
| `M7A-OUT-001` | Structured service result | status/IDs/evidence/coverage preserved |
| `M7A-OUT-002` | Candidate result rendered verified | mutation fails |
| `M7A-OUT-003` | Zero provider result rendered no entity exists | mutation fails |
| `M7A-OUT-004` | ExactMapped rendered provider relationship proven | mutation fails |
| `M7A-OUT-005` | Explicit selection rendered authority/edit permission | mutation fails |
| `M7A-OUT-006` | Partial/conflict/truncated/NotEvaluated hidden | mutation fails |
| `M7A-OUT-007` | Provider/source text | structured untrusted data |
| `M7A-OUT-008` | Mandatory evidence/privacy/nonclaims lost | invalid projection |
| `M7A-OUT-009` | Domain blocker represented only as protocol success text | mutation fails |
| `M7A-OUT-010` | Exact service result repeated | byte-identical structured result |

## Cancellation/progress/lifecycle

| ID | Case | Expected |
|---|---|---|
| `M7A-CTRL-001` | Cancel exact active call | one `operation_cancel` call |
| `M7A-CTRL-002` | Cancel unknown/cross-session call | reject/stale |
| `M7A-CTRL-003` | Cancel after uncertain provider/effect dispatch | OutcomeUnknown/reconcile |
| `M7A-CTRL-004` | Progress | bounded monotonic nonsemantic projection |
| `M7A-CTRL-005` | 100% progress treated final proof | mutation fails |
| `M7A-CTRL-006` | Continuation resets budget/refreshes current | mutation fails |
| `M7A-CTRL-007` | Slow client/outbound saturation | bounded backpressure/close |
| `M7A-CTRL-008` | Broken pipe after service result | no second service call |
| `M7A-CTRL-009` | EOF/disconnect | cancel/reconcile/close, no detached work |
| `M7A-CTRL-010` | Shutdown with active calls | exact drain/cancel/reconcile policy |

## Security and freeze

| ID | Case | Expected |
|---|---|---|
| `M7A-SEC-001` | MCP host/client/model approval used as authorization | reject |
| `M7A-SEC-002` | Sampling/elicitation/model completion | unsupported/absent |
| `M7A-SEC-003` | Prompt/tool chain automatically invokes another tool | mutation fails |
| `M7A-SEC-004` | Raw SQL/script/shell/plugin/network/process/editor/client | absent |
| `M7A-SEC-005` | Credential/token/key/vault secret in argument/result/log | reject/redact |
| `M7A-SEC-006` | Implicit cwd/home/Git/repository/WoW/provider discovery | absent |
| `M7A-SEC-007` | Source text registers tool/resource/authorization | reject/inert data |
| `M7A-SEC-008` | Cross-consumer privacy widening/cache reuse | reject |
| `M7A-DET-001` | 1/2/N workers and shuffled service completion | stable semantic/result ordering |
| `M7A-DET-002` | Client/request/progress identifiers differ | semantic service result unchanged |
| `M7A-FIX-001` | Required pins null before implementation | allowed |
| `M7A-FIX-002` | First app Rust commit with required nulls | fail |
| `M7A-FIX-003` | Cargo/.rs/workflow in docs package | fail |

## Acceptance

The MCP adapter is not implemented until official-spec/framing/capability/tool/resource/error vectors, one-call mappings, exact owner mapping/selection/context boundaries, candidate authority, authorization/privacy, cancellation/backpressure/shutdown, fuzzing, platform stdio behavior, output determinism, and checksum gates pass. Missing evidence remains blocked or `NotEvaluated`.
