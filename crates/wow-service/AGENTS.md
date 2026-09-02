# AGENTS.md — `wow-service`

Read repository/crate instructions, `README.md`, `../DEPENDENCY_GRAPH.md`, `../WORKSTREAMS.md`, and exactly one package:

```text
E0-F -> root E0 files
E1-D -> e1/
E3-C -> e3/
E4-C -> e4/
E5-B -> e5/
E5-C -> e5c/
E6-B -> e6/
E7-A -> e7/
```

Read current external WoW engineering KB routes for patch-sensitive work and actual addon instructions for addon-facing operations.

## Common rules

- Coordinate owners; never reproduce owner algorithms.
- Resolve permitted symbolic selectors once and replace them with exact IDs.
- Never select latest, best, highest score/metric, previous, first, last, sole, same-name, nearest, or default artifact/provider/mapping/candidate.
- Use narrow typed ports; no raw SQL, parser/session/process objects, filesystem roots, provider databases, or mutable graph/project handles.
- Register `OperationId + CanonicalRequestDigest` before effects.
- Response loss is not effect absence; `OutcomeUnknown` blocks blind repetition.
- Same operation ID with a different digest is rejected.
- No public success before retention, audit, and reverse-order closure.
- Authorization is independent of semantic proof and never inferred from GitHub/OS/CLI/editor/client/model/file/commit identity.
- Review, holdout, signing, publication, canary, activation, rollout, rollback, external-provider use, mapping, selection, context, transport, distribution, and runtime proof remain separate.
- Preserve partial, candidate, blocked, conflict, truncated, `OutcomeUnknown`, `NotEvaluated`, resynchronization, cancelled, failed, revoked, rolled-back, and deactivated states exactly.

## E7-A rules

- Operation exposure comes only from an exact immutable reviewed registry.
- No reflection, generic `call_service`, arbitrary MCP/RPC/tool, shell, script, plugin, or model escape hatch.
- Advertise only implemented owner capabilities under the exact workspace/profile/session.
- Workspaces are explicit; never infer cwd/Git/editor/WoW roots.
- Unsaved documents are exact project-owned immutable overlays; stale versions require resynchronization.
- LSP/MCP/daemon applications depend on `wow-service` only and invoke one service operation per semantic request.
- LSP 3.18 and MCP 2025-11-25 are pinned compatibility profiles, not floating latest versions.
- Default MCP tools are fixed and read-only; prompts, sampling, elicitation, tasks, arbitrary roots and effecting operations are absent.
- Disconnect is not cancellation; progress is not completion; response replay never reexecutes an operation.
- Multi-client workspaces, overlays, authorization, source, operations, and results are isolated.
- Local listeners are current-user IPC; optional MCP HTTP is loopback-only, Origin-validated and disabled by default.

No Cargo/Rust/workflow/placeholder/fake owner/reviewer/provider/protocol/client passing evidence during documentation phase.