# E7-A decisions

**Status:** normative.

## E7A-001 — One binary, multiple explicit modes

`apps/wow` may host one-shot CLI, foreground local daemon, LSP stdio and MCP stdio/local-HTTP modes. Separate binaries are unnecessary unless measurement proves unique packaging/isolation need.

## E7A-002 — Frontends depend on `wow-service` only

No LSP/MCP/daemon path imports or calls lower owners directly.

## E7A-003 — Operation registry is closed and reviewed

Exposure comes from immutable descriptors bound to exact service schemas; reflection and generic forwarding are forbidden.

## E7A-004 — Transport request ID is not durable operation ID

Wire correlation and effect idempotency remain separate identities.

## E7A-005 — Protocol versions are independently pinned

Initial profiles bind LSP 3.18, MCP 2025-11-25 and `wow-local-jsonrpc/1`. Upgrades create new profiles/fixtures.

## E7A-006 — Capability negotiation only narrows

A handshake cannot enable an operation absent from the reviewed registry or unavailable in implementation.

## E7A-007 — No placeholder capability advertisement

Documentation-only, disabled, unavailable or `NotEvaluated` owner capabilities are not advertised.

## E7A-008 — Workspaces are explicit

Workspace/project roots require explicit registration. No cwd, parent, Git, editor, addon-folder or WoW-installation inference.

## E7A-009 — Unsaved documents are immutable overlays

Each accepted change creates a new session-scoped overlay; persisted generations are not mutated in place.

## E7A-010 — Document versions are guarded

Out-of-order or missing versions produce conflict/resynchronization, not best-effort application.

## E7A-011 — Position encoding is negotiated and exact

LSP positions are converted against exact overlay bytes/version. Invalid scalar/surrogate/range boundaries fail.

## E7A-012 — Saved and unsaved evidence remain distinguishable

Results identify retained generation, overlay generation or explicit mixed partial/stale boundaries.

## E7A-013 — Local daemon is local-only by default

Windows named pipes or Unix-domain sockets use current-user access controls. TCP/remote listening is not default.

## E7A-014 — Daemonization is not implicit

`wow daemon run` is foreground. OS-service installation requires a later profile.

## E7A-015 — LSP profile is 3.18 and capability-gated

Only standard methods with one exact service operation are advertised.

## E7A-016 — Pull diagnostics are primary

Versioned pull diagnostics are canonical; push diagnostics project the same retained result.

## E7A-017 — No editor-owned semantic fork

Hover, definitions, references, symbols, completion, signature help, diagnostics and code actions use the same service/owner results as CLI.

## E7A-018 — No automatic source mutation

Code actions expose exact guarded edits only when an owner contract proves them; otherwise plans/disabled actions.

## E7A-019 — MCP default profile is closed and non-source-mutating

The default tool list may include pure reads and exact durable analysis-artifact creation, but excludes user-source, provider, calibration, publication, activation, release and external-system mutation. Every tool retains its actual effect class and annotations; no blanket read-only claim.

## E7A-020 — MCP negotiated capabilities and utilities are deliberately small

The server advertises tools, resources and logging when implemented. It supports bounded progress and cancellation utilities. It does not support prompts, sampling, elicitation, task-augmented execution or server-requested roots initially.

## E7A-021 — MCP tools map one-to-one to service operations

No generic `wow.call`, arbitrary tool name, shell, RPC or nested workflow escapes the registry.

## E7A-022 — MCP resources are exact immutable handles

Resource URIs include exact IDs and never resolve floating current/latest/best state.

## E7A-023 — Structured output is authoritative

MCP structured content preserves the service envelope. Text cannot omit blockers or alter status.

## E7A-024 — Streamable HTTP is explicit local-only

Optional HTTP binds loopback, validates Origin, authenticates sessions and is disabled unless configured. Remote hosting is outside E7-A.

## E7A-025 — Disconnect is not cancellation

Disconnect cannot prove the owner stopped or had no effect. Explicit cancellation/reconciliation remain required.

## E7A-026 — Progress is nonauthoritative

Progress may be coalesced/lost and cannot establish completion, success, coverage or effect state.

## E7A-027 — Final responses are recoverable

Delivery loss after a durable result retrieves the exact retained result rather than recomputing it.

## E7A-028 — Backpressure is bounded and prioritized

Final results/errors/cancellations/state changes outrank progress/log events. Unbounded queues are forbidden.

## E7A-029 — Multi-client isolation is mandatory

Workspaces, overlays, authorization, private source, continuations, operations and output are session/consumer scoped.

## E7A-030 — E7-A is not a release

Working transport does not establish reproducible packaging, install/update integrity, support or public readiness; E7-B owns those gates.