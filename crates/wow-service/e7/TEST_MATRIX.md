# E7-A acceptance and mutation matrix

**Status:** normative executable gate. IDs are unique within E7-A.

## Registry and capability negotiation

| ID | Case | Expected |
|---|---|---|
| `S7A-REG-001` | Exact reviewed registry generation | pass |
| `S7A-REG-002` | Duplicate wire or service operation name | reject |
| `S7A-REG-003` | Runtime reflection adds an operation | reject |
| `S7A-REG-004` | Generic call-service/tool proxy entry | reject |
| `S7A-REG-005` | Request/result/error schema digest mismatch | reject |
| `S7A-REG-006` | Missing owner implementation advertised | mutation fails |
| `S7A-REG-007` | Negotiation narrows unsupported client capability | pass |
| `S7A-REG-008` | Negotiation widens privacy/effect/authorization | reject |
| `S7A-REG-009` | New registry mutates established session | reject |
| `S7A-REG-010` | Existing E0–E6 operation maps 1:1 without rewrite | pass |

## Session and workspace

| ID | Case | Expected |
|---|---|---|
| `S7A-SES-001` | Open exact compatible session | Ready |
| `S7A-SES-002` | Protocol/client/consumer mismatch | reject |
| `S7A-SES-003` | Session inherits another client's workspace | reject |
| `S7A-SES-004` | Session resume with invalid/expired scope | reject |
| `S7A-SES-005` | Explicit graceful close | Closed |
| `S7A-SES-006` | Close while effect outcome unknown | close transport; retain recovery state |
| `S7A-SES-007` | Configuration change preserves old semantic bindings silently | reject; new exact binding |
| `S7A-WS-001` | Explicit root/project/profile registration | pass |
| `S7A-WS-002` | Infer cwd/Git parent/AddOns/WoW install | reject |
| `S7A-WS-003` | Path escapes root by traversal/symlink/reparse | reject |
| `S7A-WS-004` | Root/profile changes mutate registration | reject; new identity |
| `S7A-WS-005` | Watched-file event treated as exact bytes | reject |
| `S7A-WS-006` | Two clients register same root with isolated policy | separate sessions |
| `S7A-WS-007` | Workspace auto-trusted from editor | reject |
| `S7A-WS-008` | One folder notification adds/removes through multiple public service effects | mutation fails |
| `S7A-WS-009` | Atomic folder-change set conflicts midway | no partial successor registration |
| `S7A-WS-010` | Watched-file hint reports successful generation without reacquisition | mutation fails |

## Document overlays and position encoding

| ID | Case | Expected |
|---|---|---|
| `S7A-DOC-001` | Open full document against exact base | overlay snapshot |
| `S7A-DOC-002` | Same version/content duplicate open | NoChange |
| `S7A-DOC-003` | Same version/different content | conflict/resync |
| `S7A-DOC-004` | Strict incremental changes | new immutable overlay |
| `S7A-DOC-005` | Out-of-order/skipped/stale version | ResynchronizationRequired |
| `S7A-DOC-006` | One invalid range among changes | reject entire snapshot |
| `S7A-DOC-007` | UTF-16 conversion at valid surrogate boundary | pass |
| `S7A-DOC-008` | UTF-16 position splits surrogate pair | reject |
| `S7A-DOC-009` | UTF-8 position splits code point | reject |
| `S7A-DOC-010` | Encoding changes inside session | reject |
| `S7A-DOC-011` | Save without text treated as disk proof | reject mutation |
| `S7A-DOC-012` | Save reconciles exact disk bytes | explicit state |
| `S7A-DOC-013` | Unsaved bytes persisted/logged by default | fail |
| `S7A-DOC-014` | Overlay result rendered as saved generation complete | mutation fails |
| `S7A-DOC-015` | Close releases overlay after active leases | pass |

## Local daemon

| ID | Case | Expected |
|---|---|---|
| `S7A-DMN-001` | Windows current-user named pipe profile | pass on Windows fixture |
| `S7A-DMN-002` | Unix socket mode 0600/current user | pass on Unix fixture |
| `S7A-DMN-003` | TCP/wildcard/remote listener by default | reject |
| `S7A-DMN-004` | Insecure endpoint ACL/mode | reject |
| `S7A-DMN-005` | JSON-RPC framing over bounded length stream | pass |
| `S7A-DMN-006` | Batch request | reject in v1 |
| `S7A-DMN-007` | Operation name absent from session registry | reject |
| `S7A-DMN-008` | Cross-session operation/result access | reject |
| `S7A-DMN-009` | Foreground daemon shutdown drains/reconciles | pass |
| `S7A-DMN-010` | Process kill reported graceful | mutation fails |
| `S7A-DMN-011` | Reconnect silently accepts another daemon identity | reject |

## LSP 3.18 lifecycle and synchronization

| ID | Case | Expected |
|---|---|---|
| `S7A-LSP-001` | Initialize LSP 3.18 and negotiate capabilities | pass |
| `S7A-LSP-002` | Unsupported protocol/profile | initialization error |
| `S7A-LSP-003` | Request before initialize | reject |
| `S7A-LSP-004` | Advertise unimplemented feature | mutation fails |
| `S7A-LSP-005` | stdout includes logs/banner | fail |
| `S7A-LSP-006` | Incremental sync creates exact overlay | pass |
| `S7A-LSP-007` | Watched-file hint produces clean reindex success | mutation fails |
| `S7A-LSP-008` | Graceful shutdown then exit | close |
| `S7A-LSP-009` | Exit without shutdown | abnormal cleanup/recovery |
| `S7A-LSP-010` | Client config changes profile silently | reject/new registration |
| `S7A-LSP-011` | Folder add/remove notification is one atomic service operation | pass |
| `S7A-LSP-012` | Watched-file batch maps to one hint operation | pass |

## LSP feature projection

| ID | Case | Expected |
|---|---|---|
| `S7A-LSP-DIAG-001` | Pull document diagnostics exact version/result ID | pass |
| `S7A-LSP-DIAG-002` | Unchanged report for exact same inputs | pass |
| `S7A-LSP-DIAG-003` | Push fallback differs from pull result | fail |
| `S7A-LSP-DIAG-004` | Partial/NotEvaluated hidden from diagnostic data | fail |
| `S7A-LSP-FEAT-001` | Hover exact source/evidence | pass |
| `S7A-LSP-FEAT-002` | Definition/references Candidate rendered exact | fail |
| `S7A-LSP-FEAT-003` | Workspace symbol preserves search lanes | pass |
| `S7A-LSP-FEAT-004` | Completion inserts fuzzy/external top result automatically | reject |
| `S7A-LSP-FEAT-005` | Signature help exact ReferenceView/profile | pass |
| `S7A-LSP-FEAT-006` | Code action exact version/content guards | pass |
| `S7A-LSP-FEAT-007` | Code action applies edit without exact guards | reject/disabled |
| `S7A-LSP-FEAT-008` | Server mutates editor settings/executes command | reject |
| `S7A-LSP-FEAT-009` | Call hierarchy exposes incomplete coverage | pass with explicit state |
| `S7A-LSP-FEAT-010` | Rename/format/semantic tokens advertised without contract | reject |

## MCP 2025-11-25

| ID | Case | Expected |
|---|---|---|
| `S7A-MCP-001` | Initialize revision 2025-11-25 | pass |
| `S7A-MCP-002` | Unsupported revision | negotiation error |
| `S7A-MCP-003` | Default stdio emits only MCP JSON-RPC on stdout | pass |
| `S7A-MCP-004` | Fixed tools/resources capability projection | pass |
| `S7A-MCP-005` | Prompts/sampling/elicitation/tasks advertised | reject initial profile |
| `S7A-MCP-006` | Generic wow.call/tool proxy | reject |
| `S7A-MCP-007` | Default tool list includes effecting operation | reject |
| `S7A-MCP-008` | Tool strict input/output schemas | pass |
| `S7A-MCP-009` | Structured content omits service blocker | fail |
| `S7A-MCP-010` | Human text changes structured status | fail |
| `S7A-MCP-011` | Exact immutable resource URI read | pass |
| `S7A-MCP-012` | Resource URI contains current/latest/path traversal | reject |
| `S7A-MCP-013` | Unsaved overlay exposed as default resource | reject |
| `S7A-MCP-014` | Model invocation treated as user authorization | reject |
| `S7A-MCP-015` | Root supplied by client auto-registers project | reject |
| `S7A-MCP-016` | Tool result domain Invalid retained as structured outcome | pass |
| `S7A-MCP-017` | OutcomeUnknown loses recovery IDs | fail |

## MCP local Streamable HTTP

| ID | Case | Expected |
|---|---|---|
| `S7A-HTTP-001` | Explicit loopback-only listener | pass |
| `S7A-HTTP-002` | Non-loopback/wildcard bind | reject |
| `S7A-HTTP-003` | Invalid Origin | HTTP 403 |
| `S7A-HTTP-004` | Missing/invalid protocol-version header | reject |
| `S7A-HTTP-005` | Session ID cross-client replay | reject |
| `S7A-HTTP-006` | SSE reconnect replays another stream | reject |
| `S7A-HTTP-007` | Disconnect treated as cancellation | mutation fails |
| `S7A-HTTP-008` | Unbounded SSE/event replay queue | bounded failure |
| `S7A-HTTP-009` | Session secret appears in URL/log | fail |

## Progress, cancellation, reconnect, and delivery

| ID | Case | Expected |
|---|---|---|
| `S7A-LIFE-001` | Progress monotonic and nonauthoritative | pass |
| `S7A-LIFE-002` | Progress treated as completion | reject |
| `S7A-LIFE-003` | Progress/log coalesced under pressure | explicit dropped count |
| `S7A-LIFE-004` | Final response silently dropped | fail |
| `S7A-LIFE-005` | Cancel before dispatch | Cancelled/no effect |
| `S7A-LIFE-006` | Cancel after possible effect | exact owner state/reconcile |
| `S7A-LIFE-007` | Disconnect triggers blind retry | reject |
| `S7A-LIFE-008` | Reconnect retrieves retained exact result | pass |
| `S7A-LIFE-009` | Reconnect assumes unsaved overlay survived | reject/full replay required |
| `S7A-LIFE-010` | Timeout after possible effect | OutcomeUnknown |
| `S7A-LIFE-011` | Idle timeout extended beyond absolute max by progress | reject |
| `S7A-LIFE-012` | Public success before retention/close | fail |
| `S7A-LIFE-013` | Background query/cleanup after return | architecture fail |

## Isolation, security, and determinism

| ID | Case | Expected |
|---|---|---|
| `S7A-SEC-001` | Cross-client workspace/overlay/result leak | fail |
| `S7A-SEC-002` | Raw credential/private endpoint/handle in wire data | reject |
| `S7A-SEC-003` | Shell/script/plugin/model/RPC/SQL execution surface | absent/reject |
| `S7A-SEC-004` | Source/provider text alters registry/authorization | fail |
| `S7A-SEC-005` | Oversized frame/document/resource/result | bounded failure |
| `S7A-SEC-006` | Crash/log contains source or secret | fail |
| `S7A-SEC-007` | Private source resource read by wrong consumer | reject |
| `S7A-DET-001` | 1/2/N workers and request scheduling | same final semantic bytes |
| `S7A-DET-002` | Connection/queue/progress order changes semantic ID | mutation fails |
| `S7A-DET-003` | UTF-8/UTF-16 projections target same exact bytes | same owner result |
| `S7A-FIX-001` | Null pins while implementation not started | allowed |
| `S7A-FIX-002` | First Rust commit with required nulls | fail |
| `S7A-FIX-003` | All protocol/registry/adapter/vector/checksum pins frozen | pass |

## Acceptance

E7-A is not implemented until every nondeferred case runs against real service/owner implementations on the supported platform/client matrix. Documentation examples, mocked success or protocol-library unit tests alone are not implementation evidence.