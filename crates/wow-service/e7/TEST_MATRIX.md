# E7-A service acceptance and mutation matrix

**Status:** normative executable gate. IDs are unique within E7-A service tests.

## Protocol profiles and handshake

| ID | Case | Expected |
|---|---|---|
| `S7A-PROTO-001` | Exact supported LSP stdio profile | valid |
| `S7A-PROTO-002` | Exact supported MCP stdio profile | valid |
| `S7A-PROTO-003` | Floating/latest specification selector | reject |
| `S7A-PROTO-004` | Unsupported protocol revision | typed incompatibility |
| `S7A-PROTO-005` | Unimplemented client capability requested | unavailable, not advertised |
| `S7A-PROTO-006` | Client name/version changes | no semantic behavior change |
| `S7A-PROTO-007` | Source/provider attempts dynamic capability registration | reject |
| `S7A-PROTO-008` | Unknown initialization field under strict profile | reject |
| `S7A-PROTO-009` | Capability conflict/position encoding mismatch | fail initialization |
| `S7A-PROTO-010` | Server capabilities from identical implementation/profile | byte-identical canonical decision |
| `S7A-PROTO-011` | Network/daemon profile requested in stdio-only implementation | unsupported |
| `S7A-PROTO-012` | Initialization success before authorization/retention closure | mutation fails |

## Session and workspace binding

| ID | Case | Expected |
|---|---|---|
| `S7A-SESSION-001` | Exact workspace/project/reference bind | active SessionViewSet |
| `S7A-SESSION-002` | Permitted current selector resolved once | exact resolution receipt |
| `S7A-SESSION-003` | Current refreshed during request | reject/mutation fails |
| `S7A-SESSION-004` | Project/graph/reference generation mismatch | fail bind |
| `S7A-SESSION-005` | Optional Blizzard UI source unavailable | explicit partial/omission by profile |
| `S7A-SESSION-006` | Multiple root folders under single-root profile | reject |
| `S7A-SESSION-007` | Same display name across universes | distinct identities |
| `S7A-SESSION-008` | Exact rebind with expected old ID | new SessionViewSet |
| `S7A-SESSION-009` | Rebind expected-old mismatch | reject |
| `S7A-SESSION-010` | In-flight request during rebind | remains on captured old view |
| `S7A-SESSION-011` | Rebind with unsaved overlay under reject profile | blocked |
| `S7A-SESSION-012` | Last-known-good silently substituted | mutation fails |
| `S7A-SESSION-013` | Cross-session result/cursor/document handle | reject |
| `S7A-SESSION-014` | Retention unavailable | no durable handle/success |
| `S7A-SESSION-015` | Reverse close failure after work | Failed, artifact refs retained |
| `S7A-SESSION-016` | Abrupt disconnect | bounded cancel/reconcile/close; no save |

## Documents and overlays

| ID | Case | Expected |
|---|---|---|
| `S7A-DOC-001` | Full-text open with exact version | first overlay generation |
| `S7A-DOC-002` | Valid ordered incremental edit | new full-content digest |
| `S7A-DOC-003` | Full replacement under supported profile | new overlay |
| `S7A-DOC-004` | Same version/same payload exact replay | idempotent receipt when allowed |
| `S7A-DOC-005` | Same version/different payload | conflict/reject |
| `S7A-DOC-006` | Lower/stale version | reject |
| `S7A-DOC-007` | Missing/skipped version under strict profile | reject/resync required |
| `S7A-DOC-008` | UTF-16 astral-character range | exact conversion |
| `S7A-DOC-009` | Invalid surrogate/code-point boundary | reject |
| `S7A-DOC-010` | Out-of-range line/character | reject |
| `S7A-DOC-011` | Oversized edit/final document | bounded failure |
| `S7A-DOC-012` | Cross-session edit | reject |
| `S7A-DOC-013` | Concurrent mutations same document | serialized exact prior guard |
| `S7A-DOC-014` | Edit sequence order randomized | different/invalid unless semantically exact profile permits |
| `S7A-DOC-015` | Save notification | observation only |
| `S7A-DOC-016` | Save treated as disk/publication proof | mutation fails |
| `S7A-DOC-017` | Close unsaved document | overlay removed; no write |
| `S7A-DOC-018` | Reopen in new session with same version | new session identity/full content required |
| `S7A-DOC-019` | Cross-file invalidation unknown | conservative widening/NotEvaluated |
| `S7A-DOC-020` | Empty diagnostics after partial overlay analysis | not clean/complete |
| `S7A-DOC-021` | Overlay writes source file | architecture mutation fails |
| `S7A-DOC-022` | Malicious URI/path traversal | reject |
| `S7A-DOC-023` | Prompt/tool text in source | inert data |
| `S7A-DOC-024` | 1/2/N worker overlay analysis | same semantic result |

## Analysis/navigation/actions

| ID | Case | Expected |
|---|---|---|
| `S7A-AN-001` | Diagnostics exact published view | immutable result ID |
| `S7A-AN-002` | Diagnostics exact overlay view | distinct result ID |
| `S7A-AN-003` | Previous result exact unchanged proof | NoChange/unchanged |
| `S7A-AN-004` | Previous result from another overlay/profile | reject/full result |
| `S7A-AN-005` | No findings with partial capability | explicit partial, not clean |
| `S7A-AN-006` | Hover exact entity/reference facts | evidence-bearing bounded result |
| `S7A-AN-007` | Hover source prose becomes framework fact | mutation fails |
| `S7A-AN-008` | Definition has multiple exact targets | return all, no auto-selection |
| `S7A-AN-009` | Definition unresolved under partial coverage | NotEvaluated/partial |
| `S7A-AN-010` | References bounded/truncated | explicit continuation/coverage |
| `S7A-AN-011` | Partial references rendered as all references | mutation fails |
| `S7A-AN-012` | Document symbols exact | stable ordering |
| `S7A-AN-013` | Workspace symbol fuzzy lane | Candidate retained |
| `S7A-AN-014` | Search top-1 auto-selected | reject |
| `S7A-AN-015` | Advisory code action | no edit/command |
| `S7A-AN-016` | Resolve exact action | more advisory evidence only |
| `S7A-AN-017` | Action applies workspace edit | forbidden |
| `S7A-AN-018` | Unsupported rename/format/execute command | capability unavailable |
| `S7A-AN-019` | Context request exact root | existing context owner result |
| `S7A-AN-020` | External Candidate passed as exact fact | reject |

## Cancellation, progress, and backpressure

| ID | Case | Expected |
|---|---|---|
| `S7A-CTRL-001` | Cancel before dispatch | CancelledBeforeDispatch |
| `S7A-CTRL-002` | Cancel during cancellable owner work | safe stop/Cancelled |
| `S7A-CTRL-003` | Cancel after completion | completed result retained |
| `S7A-CTRL-004` | Cancel after committed effect/response loss | OutcomeUnknown/reconcile |
| `S7A-CTRL-005` | Cross-session cancel | reject |
| `S7A-CTRL-006` | Duplicate active protocol request ID | reject |
| `S7A-CTRL-007` | Progress sequence regression | reject/drop with audit per profile |
| `S7A-CTRL-008` | 100% progress before closure | not proof/success |
| `S7A-CTRL-009` | Partial chunks reset budget | mutation fails |
| `S7A-CTRL-010` | Continuation changes view/profile | reject |
| `S7A-CTRL-011` | Inbound queue saturation | typed Busy/backpressure |
| `S7A-CTRL-012` | Outbound slow peer | bounded queue/close, no rerun |
| `S7A-CTRL-013` | Silent response drop | mutation fails |
| `S7A-CTRL-014` | Background work after terminal response | mutation fails |
| `S7A-CTRL-015` | Shutdown race | no new admission; exact close state |
| `S7A-CTRL-016` | Broken pipe after service result | output failure; no second service call |

## Authorization/privacy/security

| ID | Case | Expected |
|---|---|---|
| `S7A-SEC-001` | Valid scoped authorization | operation allowed within scope |
| `S7A-SEC-002` | Client/editor/GitHub/OS identity used as authorization | reject |
| `S7A-SEC-003` | Client capability used as tool/edit authority | reject |
| `S7A-SEC-004` | Expired/revoked/replayed grant | reject/audit |
| `S7A-SEC-005` | Cross-workspace source disclosure | deny |
| `S7A-SEC-006` | Unknown source privacy requested externally | safest denied/metadata-only |
| `S7A-SEC-007` | Raw credential/token/key in request/result/log | fail/redact |
| `S7A-SEC-008` | Raw SQL/store/owner handle | reject |
| `S7A-SEC-009` | Arbitrary filesystem/URL/network/process/editor/client access | absent |
| `S7A-SEC-010` | Script/plugin/shell/Wasm/native/source execution | reject |
| `S7A-SEC-011` | Model/sampling/embedding/reranker invocation | absent |
| `S7A-SEC-012` | Dynamic MCP tool or LSP method from source | reject |
| `S7A-SEC-013` | MCP arbitrary file/URL/provider locator read | reject |
| `S7A-SEC-014` | LSP execute-command/workspace edit/settings mutation | reject |
| `S7A-SEC-015` | Oversized/deep/duplicate-key JSON | bounded protocol failure |
| `S7A-SEC-016` | Source closes framing/boundary or injects instructions | remains escaped data |
| `S7A-SEC-017` | Session A reads session B overlay/result | reject |
| `S7A-SEC-018` | Error/telemetry leaks source/private path | mutation fails |

## Determinism and freeze

| ID | Case | Expected |
|---|---|---|
| `S7A-DET-001` | 1/2/N workers | same semantic IDs/results |
| `S7A-DET-002` | Shuffled owner/query completion | stable result ordering |
| `S7A-DET-003` | Different client name/request ID/progress cadence | same semantic result |
| `S7A-DET-004` | Different transport framing for same service result | service identity same; projection identity distinct |
| `S7A-DET-005` | Clock/host/process/queue/cache differences | no semantic identity change |
| `S7A-DET-006` | Canonical JSON repeated | byte-identical |
| `S7A-DET-007` | Protocol projection golden bytes repeated | byte-identical under frozen profile |
| `S7A-FIX-001` | Required pins null while implementation not started | allowed |
| `S7A-FIX-002` | First E7-A Rust commit with required nulls | fail |
| `S7A-FIX-003` | Official spec/profile/conformance/checksum vectors frozen | pass |
| `S7A-FIX-004` | Cargo/.rs/workflow added by documentation package | fail |

## Acceptance

E7-A cannot be marked implemented until all nondeferred service, LSP, MCP, owner-integration, protocol conformance, authorization, privacy, overlay, cancellation, backpressure, determinism, platform, and freeze tests execute with exact pinned artifacts. Missing adapters/spec vectors/owner capabilities/credentials/benchmarks are `Blocked` or `NotEvaluated`, never pass.
