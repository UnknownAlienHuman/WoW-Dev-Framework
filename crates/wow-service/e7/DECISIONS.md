# E7-A decisions

**Status:** normative.

## E7A-001 — One binary, multiple explicit modes

`apps/wow` may host one-shot CLI, foreground local daemon, LSP stdio, and MCP stdio/local-HTTP modes. Separate binaries are not required unless measurement proves a unique packaging or isolation need.

## E7A-002 — Frontends depend on `wow-service` only

No LSP/MCP/daemon path imports or calls lower owner crates directly.

## E7A-003 — Operation registry is closed and reviewed

Transport exposure comes from immutable descriptors bound to exact service schemas; reflection and generic method/tool forwarding are forbidden.

## E7A-004 — Transport request ID is not durable operation ID

Wire correlation and effect idempotency remain separate identities.

## E7A-005 — Protocol versions are independently pinned

The initial profiles bind LSP 3.18, MCP 2025-11-25, and `wow-local-jsonrpc/1`. Upgrades create new profiles and compatibility fixtures.

## E7A-006 — Capability negotiation only narrows

A client/server handshake cannot enable an operation absent from the reviewed registry or unavailable in the implementation.

## E7A-007 — No placeholder capability advertisement

Documentation-only, disabled, unavailable, or `NotEvaluated` owner capabilities are not advertised as supported.

## E7A-008 — Workspaces are explicit

Workspace/project roots require explicit registration. No cwd, parent, Git, editor, addon-folder, or WoW-installation inference.

## E7A-009 — Unsaved documents are immutable overlays

Each accepted change creates a new session-scoped overlay snapshot; the persisted project generation is not mutated in place.

## E7A-010 — Document versions are guarded

Out-of-order or missing versions produce conflict/resynchronization, not best-effort application.

## E7A-011 — Position encoding is negotiated and exact

LSP positions are converted against the exact overlay bytes/version under the negotiated encoding. Invalid scalar/surrogate/range boundaries fail.

## E7A-012 — Saved and unsaved evidence remain distinguishable

Results identify whether evidence came from a retained project generation, an overlay generation, or a mixed operation with explicit partial/staleness boundaries.

## E7A-013 — Local daemon is local-only by default

The supported daemon uses Windows named pipes or Unix-domain sockets with current-user access controls. TCP/remote listening is not a default profile.

## E7A-014 — Daemonization is not implicit

`wow daemon run` remains a foreground owned process. Installation as an OS service requires a later explicit profile.

## E7A-015 — LSP profile is 3.18 and capability-gated

The server uses standard lifecycle/document/language/workspace methods only where one exact service operation exists.

## E7A-016 — Pull diagnostics are primary

Versioned pull diagnostics are the canonical LSP projection. Push diagnostics are a negotiated compatibility path and must preserve the same owner result.

## E7A-017 — No editor-owned semantic fork

Hover, definitions, references, symbols, completion, signature help, diagnostics, and code actions project the same service/owner results used by CLI.

## E7A-018 — No automatic source mutation

Code actions may expose exact version-guarded edits only when an existing service contract proves a safe fix. Otherwise they expose a plan or disabled action.

## E7A-019 — MCP default profile is read-only

Effecting calibration, publication, provider, selection, activation, rollback, release, or source-edit operations are absent from the default MCP tool list.

## E7A-020 — MCP capabilities are deliberately small

The initial server supports tools, exact resources, logging, progress, and cancellation. It does not support prompts, sampling, elicitation, task-augmented execution, or server-requested roots.

## E7A-021 — MCP tools map one-to-one to service operations

No generic `wow.call`, arbitrary tool name, shell, RPC, or nested workflow escapes the registry.

## E7A-022 — MCP resources are exact immutable handles

Resource URIs include exact generation/artifact IDs and never resolve floating `current`, latest, or best state at read time.

## E7A-023 — Structured output is authoritative

MCP structured content contains the exact transport projection of the service envelope. Human-readable text cannot omit blockers or alter status.

## E7A-024 — Streamable HTTP is explicit local-only

The optional profile binds loopback only, validates Origin, authenticates sessions, and is disabled unless explicitly configured. Remote hosting is not part of E7-A.

## E7A-025 — Disconnect is not cancellation

A client disconnect cannot prove the owner operation stopped or had no effect. Explicit cancellation and durable reconciliation remain required.

## E7A-026 — Progress is nonauthoritative

Progress may be coalesced or lost. It cannot establish completion, success, coverage, or effect state.

## E7A-027 — Final responses are recoverable

If delivery fails after a durable result, the exact operation/result remains retrievable through reconciliation rather than recomputation.

## E7A-028 — Backpressure is bounded and prioritized

Final results, errors, cancellations, and state changes outrank progress/log events. Unbounded queues are forbidden.

## E7A-029 — Multi-client isolation is mandatory

Workspace registrations, overlays, authorization, private source, continuations, operations, and output are session/consumer scoped.

## E7A-030 — E7-A is not a release

A working transport does not establish reproducible packaging, installation, update, support, or public-release readiness. Those belong to E7-B.