# `wow-core`

`wow-core` is an active workspace crate of shared Rust types and pure helpers.
Its current implementation is broad: public modules cover IDs, profiles,
generations, source handles, evidence, findings, coverage, budgets,
canonicalization, and E0 result envelopes. The narrower target is small common
IDs, source/span and bounded-input helpers, and useful errors needed by actual
callers. It should not become a framework for result proofs, workflow policy, or
all possible future owner contracts.

## Where to look

- Public exports: [`src/lib.rs`](src/lib.rs)
- Identifier and path/span behavior: `src/ids.rs`, `src/source.rs`
- Common errors, byte bounds and canonical JSON: `src/error.rs`,
  `src/budget.rs`, `src/canonical.rs`
- Existing envelope and integrity code, targeted for removal:
  `src/envelope.rs`, `src/envelope_decode.rs`, `src/integrity.rs`
- Current executable examples: [`examples/`](examples/) and focused tests under
  [`tests/`](tests/)

Use the existing APIs where they support the CLI, project, and editor paths.
Keep byte offsets end-exclusive and tied to the original UTF-8 source. Reject
malformed identifiers and paths instead of silently making them plausible.
Keep limits useful at input/output boundaries, not as a new flag for every
internal counter.

The old envelope/result digest surface remains in code and has real downstream
callers; its removal is queued as [#194](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/issues/194).
That issue is pending work, not a claim that the code is already gone. When
simplifying it, move to plain typed results and serialize once at the output
boundary, removing obsolete callers and envelope-only tests with it.

Core does not parse Lua/XML/TOC, own SQLite, rank search results, configure
projects, or implement transport protocols. See the root
[`AGENTS.md`](../../AGENTS.md), [architecture](../../docs/ARCHITECTURE.md), and
[reduction queue](../../docs/REFACTOR_QUEUE.md) for current scope and task order.
