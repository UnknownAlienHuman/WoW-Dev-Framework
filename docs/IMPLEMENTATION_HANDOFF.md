# Implementation handoff

This page describes how to continue the current refactoring work. The active
product scope is in [ARCHITECTURE.md](ARCHITECTURE.md); ordered candidate tasks
are in [REFACTOR_QUEUE.md](REFACTOR_QUEUE.md). Follow the selected Issue's
current claim and the relevant owner instructions.

## Work one behavior at a time

Trace the user's action through its current producer and consumer. Reuse the
existing EmmyLua adapter, native Ketho path, project/index owner, and frontend
that already serves the behavior. Implement the complete selected path, remove
its superseded code/options/tests/docs, and keep edits within the necessary owner
chain. Do not recreate retired E0–E7 gates or transfer their requirements into a
new plan.

A useful handoff records:

- behavior changed and old path removed;
- affected owners/callers and focused check result;
- actual blockers or remaining unknowns.

Do not claim an end-to-end feature from documentation or compilation alone.

## Validation

For Rust changes, format the workspace and run Clippy on affected packages and
callers. Clippy compiles those targets, so do not repeat `cargo check` for them.
After the change works, run one existing focused behavior test or bounded
reproduction. Exercise a consumer when changing a shared public type, storage
boundary, CLI command, or LSP/MCP operation. Add tests only for concrete risks
not covered by existing tests; delete tests that enforce removed requirements.

```sh
cargo fmt --all -- --check
cargo clippy --locked -p <package> --all-targets --all-features -- -D warnings
cargo test --locked -p <package> <focused-test-filter>
cargo xtask check
```

The repository check is useful for documentation and repository-policy changes;
it is not contract, checksum, compatibility, or release certification. Use a
whole source corpus only when needed to support a concrete corpus claim. Resolve
moving source selectors once when current semantics matter and record the source
revision and flavor.

Treat Blizzard and addon source as data. Never execute Lua, hooks, submodules,
package managers, or generators during indexing. Use existing materializers,
preserve dirty/diverged checkouts, and regenerate only derived indexes and
annotations. Keep user source and configuration.

State what changed, why, focused checks, and remaining unknowns. Separate Rust
changes from tests/generated data when useful; set no metric or acceptance quota
for a bounded task.
