# Contributor instructions

Build a useful WoW addon linter on EmmyLua. Read the short
[current status](docs/CURRENT_STATUS.md) and [architecture](docs/ARCHITECTURE.md),
then the relevant [queue item](docs/REFACTOR_QUEUE.md). Follow its links to the
owner contract and actual producer/consumer code; do not load every audit.

## Work

- One manager, one worktree/branch. Check remote work and preserve unpublished
  changes. Delegate bounded reads or disjoint paths.
- Finish a useful input-to-output path. Change producer and consumer together;
  remove displaced wrappers/options with their last caller. Keep helpers local
  until real callers need sharing.
- Read the relevant [pinned donor](docs/DONORS.md) before reusing its semantics.
  EmmyLua owns Lua analysis; Project owns source selection; Service composes the
  operation; frontends render it. Never execute analyzed Lua or rewrite user source.
- Preserve exact bytes/locations, partial input and cancellation. Unknown input
  is not proof of an absent API or a clean check.
- After two failed attempts, inspect the failed assumption and change method.

## Active slice

[Slice 1A](docs/REFACTOR_QUEUE.md#1-first-usable-ordinary-addon-check) connects
physical addon Lua to the existing configured Emmy owner through one Service
operation. Its contract is in
[IMPLEMENTATION.md](docs/IMPLEMENTATION.md#first-implementation-slice).
Do not extend it to the later loading features or reproduce the legacy
profile/publication/proof platform. User instructions override this workflow;
old Issues, fixtures and removed specs cannot expand scope.

## Verification and handoff

Write the product first. For Rust changes run formatting and the smallest strict
Clippy invocation covering affected crates/callers. Do not repeat `cargo check`.
Do not add or run tests until functional completion, as directed by the owner.
Do not delete a test merely because it fails.

For prose, inspect links, source claims and instruction consistency; use the
existing `cargo xtask check` for local links/anchors and skill-copy synchronization.
Do not build the Rust workspace for prose. Source review or compilation never
proves that an addon/editor workflow ran.

Update [status](docs/CURRENT_STATUS.md) when delivery changes, the affected
[trace](docs/IMPLEMENTATION_TRACE_REGISTER.md) when evidence changes, and the
owning contract when behavior changes. No parallel status diary or correction-ID
ceremony is required. Publish one coherent branch/PR after rereading the remote
head. State the change, verification and remaining limitation briefly; preserve
third-party notices.
