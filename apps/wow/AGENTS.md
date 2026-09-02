# AGENTS.md — `apps/wow`

Read repository/crate/service instructions, this router, and exactly one package.

```text
E0-F -> root files
E3-C -> e3/
E4-C -> e4/
E5-B -> e5/
E5-C -> e5c/
```

The only framework dependency is `wow-service`.

- Parse explicit typed commands/options only.
- Pass exact IDs, guards, profiles, continuations, and permitted symbolic selectors without resolving them.
- Never choose latest/best/previous/default/first/sole targets.
- Never read project source, editor/client state, hidden holdout/cohort data, credential stores, signing keys, or owner internals.
- Explicit file/stdin data is bounded transport input and never executed.
- Exactly one service invocation per valid command.
- Machine output preserves all exact scoped states, blockers, authorization, consumption, signature, canary, rollout, LKG, rollback, revocation, closure and nonclaim state.
- No retry/double output after cancellation, response loss, broken pipe, or output failure.
- Never expose credentials, private source, private cohort membership, confidential review/signing material, private paths, or raw owner handles.
- No Cargo/Rust/workflow/placeholder implementation during documentation phase.