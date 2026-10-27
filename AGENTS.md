# Contributor instructions

Read, in order:

1. [current status](docs/CURRENT_STATUS.md);
2. [architecture](docs/ARCHITECTURE.md);
3. the active item in [the queue](docs/REFACTOR_QUEUE.md);
4. the relevant owner route in [IMPLEMENTATION.md](docs/IMPLEMENTATION.md) and
   domain behavior in [WOW_LINTER.md](docs/WOW_LINTER.md);
5. the actual producer, handoff and consumer code.

`CURRENT_STATUS.md` is the only live delivery checkpoint. The queue is the only
ordered work list; the trace register records evidence and observations. README,
package guidance, skills, Issues and historical specifications cannot expand or
narrow the active slice.

## Work

- One manager, one worktree/branch. Delegate only bounded reads or disjoint paths.
- Finish one producer-to-user path. Change the real producer and consumer together;
  remove displaced wrappers, options and prose with their last caller.
- Reuse [pinned donors](docs/DONORS.md) only after reading the exact relevant
  source. Blizzard facts, Ketho behavior and Wowless comparison behavior have
  different authority.
- Preserve exact bytes, locations, ambiguity, incomplete input and cancellation.
  Never execute analyzed Lua or rewrite user source.
- After two failed attempts at one approach, identify the failed assumption and
  change method.

## Active slice

Implement Slice 1A exactly as specified in
[the queue](docs/REFACTOR_QUEUE.md#1-first-usable-ordinary-addon-check) and
[the owner guide](docs/IMPLEMENTATION.md#first-implementation-slice). Do not copy
that contract here or invent missing policy.

Critical boundaries:

- frontends call one narrow public Service operation;
- Service owns Project preparation, requested-scope limitations, exact snapshots,
  same-owner Emmy requirements and the lower check;
- Project owns safe root/file admission and selected source authority;
- no `ProfileIdentity`, publication/replay/proof, Reference/Rules, graph/store,
  second parser/analyzer, generic result hierarchy or hidden compatibility route;
- channel, dependencies, full XML/chunk injection, bindings, secure contexts and
  WoW checks remain later slices unless the queue explicitly moves them.

## Validation

Write the product path first. For Rust changes run formatting and the smallest
strict Clippy invocation covering affected crates and callers; do not duplicate it
with a separate `cargo check`. Tests remain deferred until functional completion,
as directed by the owner.

For documentation changes, inspect source claims, current `main`, links/anchors
and instruction consistency. Update the trace immediately when a finding changes a
boundary. Do not cite compilation or Clippy as documentation evidence.

A build, source trace or document review never makes a workflow **observed**.
Observation requires the exact ordinary command/protocol input and visible result
recorded in the trace register.

Publish one coherent branch/PR after rereading the remote head. Do not create
acceptance-tier, proof-archive, migration-inventory or status-diary machinery.
