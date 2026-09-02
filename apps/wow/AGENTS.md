# AGENTS.md — `apps/wow`

Read repository/crate/service instructions, this router, and exactly one package contract.

```text
E0-F -> root files
E3-C -> e3/
E4-C -> e4/
E5-B -> e5/
```

The only framework dependency is `wow-service`.

- Parse explicit typed commands/options only.
- Pass exact IDs, guards, profiles, continuations, and permitted symbolic selectors without resolving them.
- Never guess or select latest/best/first/sole candidates.
- Never read project source, repositories, editor state, WoW installation, SavedVariables, logs, hidden holdout data, or credential stores.
- Explicit file/stdin data is bounded transport input and never executed.
- Exactly one service invocation per valid command.
- JSON output is exact service bytes plus defined LF; artifact output is exact validated bytes; text preserves all blocker/authorization/consumption/nonclaim state.
- No retry or duplicate output after cancellation, response loss, broken pipe, or output failure.
- Never expose credentials, private source, hidden labels/membership, confidential review material, private paths, or raw owner handles.
- No Cargo/Rust/workflow/placeholder implementation during documentation phase.