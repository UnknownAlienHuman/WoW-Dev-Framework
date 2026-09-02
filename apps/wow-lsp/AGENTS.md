# AGENTS.md — `apps/wow-lsp`

## Dependency

```text
apps/wow-lsp -> wow-service
```

Any direct dependency on another framework crate is an architecture failure.

## Responsibilities

- implement the exact frozen LSP transport/framing/profile;
- parse and validate initialization, requests, notifications, IDs, capabilities, URIs, positions, and cancellation/progress tokens;
- map every supported request to exactly one `wow-service` operation;
- map document notifications to the exact session overlay operations;
- convert coordinates under the negotiated position-encoding profile;
- emit exact protocol responses/errors/progress and close cleanly.

## Prohibited behavior

- no parser/analyzer/rule/search/context/graph/storage implementation;
- no implicit workspace/Git/editor/addon/WoW discovery;
- no filesystem write, workspace edit, execute-command, formatting, rename, or settings mutation;
- no shell/network/process/model/provider/tool execution;
- no automatic result/candidate selection;
- no authorization inferred from client/editor/OS/GitHub identity;
- no background work after response, cancellation, shutdown, or transport loss.

## Document rules

- full content is required on open;
- versions and edits follow the frozen sync profile;
- stale/conflicting/out-of-range edits fail;
- save is observation only;
- close never saves;
- source text remains untrusted data;
- no cross-session document/result/cursor reuse.

## Output rules

- preserve service evidence, coverage, conflicts, blockers, partial/truncated/`NotEvaluated`, and nonclaims in protocol data;
- no diagnostic clean result without service authority;
- code actions contain no edit/command in E7-A;
- broken pipe/output failure never repeats service;
- stderr/logs exclude source, private roots, credentials, and raw owner handles.

## Completion report

```text
LSP/spec/profile/transport IDs
client capabilities and server capability decision
session/workspace/view/overlay IDs
method -> service operation
position/sync/framing result
cancellation/progress/backpressure/shutdown result
protocol/service error projection
security/privacy tests and explicit unsupported methods
```
