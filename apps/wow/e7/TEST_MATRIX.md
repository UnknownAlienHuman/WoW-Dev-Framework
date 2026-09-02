# E7-A application acceptance matrix

**Status:** normative executable gate.

## Modes and routing

| ID | Case | Expected |
|---|---|---|
| `A7A-MODE-001` | Existing one-shot CLI remains compatible | pass |
| `A7A-MODE-002` | `transport capabilities` invokes one service operation | pass |
| `A7A-MODE-003` | `daemon run` foreground local IPC | pass when profile enabled |
| `A7A-MODE-004` | `lsp --transport stdio` | exact LSP profile |
| `A7A-MODE-005` | `mcp --transport stdio` | exact MCP profile |
| `A7A-MODE-006` | Explicit local Streamable HTTP | disabled unless profile enabled |
| `A7A-MODE-007` | Unknown/multiple host modes | exit before service/session |
| `A7A-MODE-008` | Frontend imports lower framework crate | architecture fail |
| `A7A-MODE-009` | Valid semantic request invokes two service operations | fail |
| `A7A-MODE-010` | Generic operation/tool/RPC proxy | reject |

## Config, endpoint, and path security

| ID | Case | Expected |
|---|---|---|
| `A7A-CFG-001` | Exact strict config | pass |
| `A7A-CFG-002` | Unknown/include/interpolation/script/plugin field | reject |
| `A7A-CFG-003` | cwd/Git/editor/WoW/addon auto-discovery | fail |
| `A7A-CFG-004` | Sensitive value in argv/config | reject/redact |
| `A7A-PATH-001` | Valid explicit workspace/endpoint path | pass through typed request |
| `A7A-PATH-002` | Traversal/symlink/reparse/device/ADS escape | reject |
| `A7A-PATH-003` | TCP/wildcard/public daemon endpoint | reject |
| `A7A-PATH-004` | Insecure pipe/socket ACL/mode | reject |
| `A7A-PATH-005` | Endpoint/URI contains session secret | fail |

## LSP

| ID | Case | Expected |
|---|---|---|
| `A7A-LSP-001` | LSP 3.18 initialize and exact capabilities | pass |
| `A7A-LSP-002` | stdout contains banner/log | fail |
| `A7A-LSP-003` | Unsupported method not advertised | method-not-found/not-supported |
| `A7A-LSP-004` | Incremental change maps once to service | pass |
| `A7A-LSP-005` | Stale change asks resync | exact error/state |
| `A7A-LSP-006` | UTF-16 and UTF-8 position projections | exact mapped owner bytes |
| `A7A-LSP-007` | Pull diagnostic result projected faithfully | pass |
| `A7A-LSP-008` | Push fallback triggers second analysis | fail |
| `A7A-LSP-009` | Partial/NotEvaluated hidden | fail |
| `A7A-LSP-010` | Completion auto-inserts external/fuzzy top result | reject |
| `A7A-LSP-011` | Unguarded WorkspaceEdit | reject/disabled |
| `A7A-LSP-012` | Editor command/settings mutation | reject |
| `A7A-LSP-013` | Graceful shutdown/exit | close and exit 0 |
| `A7A-LSP-014` | Exit without shutdown | nonzero/recovery |

## MCP

| ID | Case | Expected |
|---|---|---|
| `A7A-MCP-001` | MCP revision 2025-11-25 initialize | pass |
| `A7A-MCP-002` | stdout contains non-MCP output | fail |
| `A7A-MCP-003` | Fixed implemented read-only tools/resources | pass |
| `A7A-MCP-004` | Generic `wow.call` or provider proxy | reject |
| `A7A-MCP-005` | Prompts/sampling/elicitation/tasks advertised | reject |
| `A7A-MCP-006` | Effecting tool in default profile | reject |
| `A7A-MCP-007` | Tool invokes exactly one service operation | pass |
| `A7A-MCP-008` | Unknown argument/schema mismatch | protocol/tool error |
| `A7A-MCP-009` | Structured result preserves blocker/nonclaim | pass |
| `A7A-MCP-010` | Text projection overrides status | fail |
| `A7A-MCP-011` | Exact `wow://` resource read | pass |
| `A7A-MCP-012` | Floating/path resource URI | reject |
| `A7A-MCP-013` | Unsaved overlay/default full source exposed | reject |
| `A7A-MCP-014` | Model call treated as authorization | reject |
| `A7A-MCP-015` | Client root auto-registers workspace | reject |

## Local MCP HTTP and daemon

| ID | Case | Expected |
|---|---|---|
| `A7A-HTTP-001` | Explicit loopback bind and allowed Origin | pass |
| `A7A-HTTP-002` | Non-loopback bind | reject before listener |
| `A7A-HTTP-003` | Invalid Origin | 403, no service call |
| `A7A-HTTP-004` | Cross-session ID/replay | reject |
| `A7A-HTTP-005` | SSE reconnect reexecutes operation | fail |
| `A7A-HTTP-006` | Unbounded SSE queue | bounded failure |
| `A7A-DMN-001` | Current-user named pipe/Unix socket | pass per platform |
| `A7A-DMN-002` | Daemon `operation/call` absent from registry | reject |
| `A7A-DMN-003` | Disconnect cancels operation automatically | fail |
| `A7A-DMN-004` | Response replay reexecutes service | fail |
| `A7A-DMN-005` | Graceful shutdown drains/reconciles/closes | pass |
| `A7A-DMN-006` | Forced kill reported graceful | fail |

## Lifecycle, isolation, and output

| ID | Case | Expected |
|---|---|---|
| `A7A-LIFE-001` | Progress coalesced under backpressure | explicit lost count, final intact |
| `A7A-LIFE-002` | Cancellation targets exact ticket | pass |
| `A7A-LIFE-003` | Disconnect causes blind retry | reject |
| `A7A-LIFE-004` | OutcomeUnknown loses recovery IDs | fail |
| `A7A-LIFE-005` | Reconnect retrieves exact retained result | pass |
| `A7A-LIFE-006` | Reconnect assumes overlay survived | reject/full replay |
| `A7A-LIFE-007` | Broken pipe/output failure repeats operation | fail |
| `A7A-LIFE-008` | Cross-client source/overlay/result leak | fail |
| `A7A-LIFE-009` | Session close leaves background work | fail |
| `A7A-OUT-001` | CLI JSON exact service bytes + LF | pass |
| `A7A-OUT-002` | LSP/MCP exact framing | pass |
| `A7A-OUT-003` | Protocol IDs/timing change owner semantic bytes | fail |
| `A7A-OUT-004` | Logs/crash contain source/secret | fail |
| `A7A-FIX-001` | Null pins before implementation | allowed |
| `A7A-FIX-002` | First Rust commit with required nulls | fail |

## Acceptance

E7-A app implementation requires every enabled host mode and platform/client fixture to run against real service operations. Disabled modes must not be advertised. Parser/protocol-library unit tests alone are insufficient.