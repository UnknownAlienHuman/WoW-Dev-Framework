# AGENTS.md — `apps/wow`

Read repository/crate/service instructions, this router, and exactly one package:

```text
E0-F -> root files
E3-C -> e3/
E4-C -> e4/
E5-B -> e5/
E5-C -> e5c/
E6-B -> e6/
E7-A -> e7/
```

The only framework dependency is `wow-service`.

- Parse explicit typed commands/options/protocol messages only.
- Pass exact IDs, guards, profiles, continuations, workspace/document versions and permitted symbolic selectors without resolving them locally.
- Never choose latest/best/previous/default/first/sole/nearest providers, candidates, mappings, roots or artifacts.
- Never read owner stores, project source, provider databases, credential stores, signing keys, private endpoints, editor/client state, hidden holdout/cohort data or raw handles.
- Explicit file/stdin/protocol data is bounded transport input and never executed.
- Exactly one service invocation per valid semantic command/method/tool call.
- Expose only methods/tools/resources from the negotiated immutable registry.
- No generic RPC/tool/shell/script/plugin/model or editor-command escape hatch.
- LSP stdout contains LSP frames only; MCP stdout contains MCP messages only; logs are bounded/redacted on stderr.
- Workspaces are explicit and unsaved documents are exact versioned service/project overlays.
- Disconnect is not cancellation, progress is not completion, and response replay never reexecutes service.
- Machine output preserves all exact scoped states, evidence, coverage, blockers, Candidate authority, `NotEvaluated`, `OutcomeUnknown`, resynchronization and nonclaims.
- No retry/double output after cancellation, response loss, broken pipe or output failure.
- No automatic edit application, editor-setting mutation, remote listener, release/update action, Cargo/Rust/workflow placeholder during documentation phase.