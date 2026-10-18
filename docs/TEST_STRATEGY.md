# Test strategy

Tests protect the selected developer workflows: EmmyLua analysis, the Rust Ketho
annotation port, practical editor-independent use, and Blizzard UI repository
search. Follow [AGENTS.md](../AGENTS.md) and the selected task; use the smallest
existing check that exercises the changed behavior.

## Test selection

- Use owner unit tests for parsing, type lowering, source spans, normalization,
  path handling, indexing, query behavior, and serialization.
- Use a small existing fixture or focused reproduction for behavior that depends
  on TOC, XML, Lua, Blizzard API data, or Ketho output.
- Exercise the caller when changing a public type, storage boundary, CLI, or
  LSP/MCP adapter. Keep protocol tests proportional to the changed behavior.
- Test the important failure at the boundary being changed: malformed input,
  unsupported syntax, ambiguous selection, missing source, or bounded-output
  failure when relevant.

Do not require every test layer for every change. Add a test only for a concrete
regression risk not already covered. Remove tests whose only purpose is to prove
retired features or frozen contract/checksum declarations. Do not impose corpus,
mutation-count, coverage, task-count, or benchmark quotas.

## Useful checks

Run formatting for Rust changes, then Clippy for affected packages and callers.
Clippy compiles those targets, so do not repeat `cargo check` for them. After the
changed behavior works, run one existing focused test or bounded reproduction.
Broaden validation only when a concrete remaining risk requires it.

```sh
cargo fmt --all -- --check
cargo clippy --locked -p <package> --all-targets --all-features -- -D warnings
cargo test --locked -p <package> <focused-test-filter>
cargo xtask check
```

`cargo xtask check` is a repository hygiene check, not product acceptance. Run it
for documentation changes or when the change affects repository policy. Report
what was run and what remains untested; do not turn unavailable tools or runtime
access into a pass.

## Source and behavior claims

Use a whole Blizzard source corpus only when making a claim that depends on that
corpus. Resolve a moving source selector once for the task and record the exact
revision and flavor. Read indexed Lua/XML/TOC as data; never execute it. A source
search result or static analysis finding does not establish runtime behavior.
For Ketho changes, compare the relevant Rust output with the donor behavior using
the existing focused fixture or consumer probe.
