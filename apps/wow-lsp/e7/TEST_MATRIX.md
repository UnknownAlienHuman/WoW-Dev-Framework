# E7-A LSP adapter acceptance and mutation matrix

**Status:** normative. IDs are unique within the LSP adapter package.

## Framing and initialization

| ID | Case | Expected |
|---|---|---|
| `L7A-FRAME-001` | Valid frozen stdio frame | one parsed message |
| `L7A-FRAME-002` | Missing/conflicting/duplicate content length | protocol error/close |
| `L7A-FRAME-003` | Oversized body before allocation | reject |
| `L7A-FRAME-004` | Truncated/polyglot frame | reject without service call |
| `L7A-FRAME-005` | Protocol stdout mixed with logs | mutation fails |
| `L7A-INIT-001` | Supported exact initialize | one `session_initialize` call |
| `L7A-INIT-002` | Second initialize | reject |
| `L7A-INIT-003` | Normal request before initialize | reject |
| `L7A-INIT-004` | Unsupported protocol/profile/position encoding | fail initialization |
| `L7A-INIT-005` | Client name changes server semantics | mutation fails |
| `L7A-INIT-006` | Advertise unimplemented/denied capability | mutation fails |
| `L7A-INIT-007` | Initialization success before service retention/closure | mutation fails |

## Document synchronization

| ID | Case | Expected |
|---|---|---|
| `L7A-DOC-001` | `didOpen` full text/version | one `document_open` call |
| `L7A-DOC-002` | Valid incremental `didChange` | one `document_change` call |
| `L7A-DOC-003` | Valid full replacement where profiled | one change call |
| `L7A-DOC-004` | Stale/conflicting version | exact sync error |
| `L7A-DOC-005` | Out-of-range/invalid Unicode position | reject |
| `L7A-DOC-006` | UTF-16 astral position conversion | exact source coordinate |
| `L7A-DOC-007` | CRLF/LF and final line cases | exact vectors |
| `L7A-DOC-008` | Huge change/content | bounded failure |
| `L7A-DOC-009` | `didSave` | observation-only service call |
| `L7A-DOC-010` | Adapter writes disk on save | mutation fails |
| `L7A-DOC-011` | `didClose` | one close call, no save |
| `L7A-DOC-012` | URI traversal/cross-workspace | reject |
| `L7A-DOC-013` | App repairs version by reading disk/current | mutation fails |
| `L7A-DOC-014` | Source contains fake frame/tool instructions | inert document data |

## Analysis mapping

| ID | Case | Expected |
|---|---|---|
| `L7A-MAP-001` | Document diagnostic | exactly one `analysis_diagnostics` call |
| `L7A-MAP-002` | Workspace diagnostic under enabled profile | one bounded diagnostics call |
| `L7A-MAP-003` | Hover | one `analysis_hover` call |
| `L7A-MAP-004` | Definition | one `analysis_definition` call |
| `L7A-MAP-005` | References | one `analysis_references` call |
| `L7A-MAP-006` | Document symbol | one `analysis_symbols` call with document scope |
| `L7A-MAP-007` | Workspace symbol | one `analysis_symbols` call with workspace scope |
| `L7A-MAP-008` | Code action | one `analysis_code_actions` call |
| `L7A-MAP-009` | Code action resolve | one `analysis_resolve_action` call |
| `L7A-MAP-010` | Adapter composes search/select/context | mutation fails |
| `L7A-MAP-011` | Direct lower-crate call | architecture failure |
| `L7A-MAP-012` | Unsupported method falls back to raw owner call | mutation fails |

## Result projection

| ID | Case | Expected |
|---|---|---|
| `L7A-OUT-001` | Diagnostic result preserves exact result ID/findings | pass |
| `L7A-OUT-002` | Empty partial diagnostic rendered clean | mutation fails |
| `L7A-OUT-003` | Multiple definitions | all targets preserved |
| `L7A-OUT-004` | References truncated | explicit partial metadata |
| `L7A-OUT-005` | Candidate workspace symbol | Candidate retained |
| `L7A-OUT-006` | Hover source text | structured untrusted boundary |
| `L7A-OUT-007` | Advisory action | no `edit`/`command` |
| `L7A-OUT-008` | Adapter raises severity/confidence | mutation fails |
| `L7A-OUT-009` | Mandatory evidence/coverage lost | invalid projection |
| `L7A-OUT-010` | Exact same service result | deterministic LSP bytes |

## Cancellation/progress/lifecycle

| ID | Case | Expected |
|---|---|---|
| `L7A-CTRL-001` | `$/cancelRequest` exact active request | one `operation_cancel` call |
| `L7A-CTRL-002` | Cancel unknown/cross-session request | typed stale/error |
| `L7A-CTRL-003` | Cancel after committed effect | actual state/OutcomeUnknown, not guessed |
| `L7A-CTRL-004` | Progress token mapping | monotonic bounded projection |
| `L7A-CTRL-005` | Progress 100% treated as final success | mutation fails |
| `L7A-CTRL-006` | Partial result resets budget | mutation fails |
| `L7A-CTRL-007` | `shutdown` | exactly one `session_shutdown` call |
| `L7A-CTRL-008` | `exit` after shutdown | transport close, no second shutdown |
| `L7A-CTRL-009` | EOF/broken pipe | cancel/reconcile/close, no rerun/save |
| `L7A-CTRL-010` | Request after shutdown | reject |
| `L7A-CTRL-011` | Background task after exit | mutation fails |
| `L7A-CTRL-012` | Slow client/outbound saturation | bounded backpressure/close |

## Security and freeze

| ID | Case | Expected |
|---|---|---|
| `L7A-SEC-001` | Client/workspace trust used as authorization | reject |
| `L7A-SEC-002` | Execute command/workspace edit/format/rename | unavailable/reject |
| `L7A-SEC-003` | Implicit cwd/home/Git/editor/WoW discovery | absent |
| `L7A-SEC-004` | Raw SQL/script/shell/plugin/model/network access | absent |
| `L7A-SEC-005` | Credential/private path/source leak in error/log | mutation fails |
| `L7A-SEC-006` | Cross-session document/result/cancel | reject |
| `L7A-SEC-007` | Dynamic method registration from source | reject |
| `L7A-DET-001` | 1/2/N workers/request completion order | same semantic result/projection order |
| `L7A-DET-002` | Client/request/progress identifiers differ | semantic service result unchanged |
| `L7A-FIX-001` | Required pins null before implementation | allowed |
| `L7A-FIX-002` | First app Rust commit with required nulls | fail |
| `L7A-FIX-003` | Cargo/.rs/workflow in docs package | fail |

## Acceptance

The LSP adapter is not implemented until official-spec/framing/position/sync/capability/error vectors, one-call mappings, platform stdio behavior, authorization/privacy, cancellation/backpressure/shutdown, fuzzing, output determinism, and checksum gates pass. Missing evidence remains blocked or `NotEvaluated`.
