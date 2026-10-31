# Contributor instructions

Build a useful WoW addon linter on EmmyLua. Read [current status](docs/CURRENT_STATUS.md)
and the assigned row of the [work register](docs/REFACTOR_QUEUE.md). With no assigned
row, select its first unfinished unblocked step in the active slice. Use the
[map](docs/REPOSITORY_MAP.md) and linked owner section to read the real producer and
consumer. Read [architecture](docs/ARCHITECTURE.md) on first entry or scope changes,
not before every edit. Do not load every guide, Issue comment or historical audit.

## Work

- One manager, one worktree/branch. Inspect the remote head and preserve unpublished
  work. Delegate bounded reads or disjoint edits with exact revision, input/output,
  owner/caller and exclusions; agree shared signatures before parallel edits.
- An implementation task changes a connected producer/consumer path. Do not finish
  at unused helpers, another plan or a new audit. Remove displaced exclusive
  wrappers/options/fixtures with the last caller, not useful behavior or notices.
- Start with existing concrete owners and private helpers. Follow
  [small implementation decisions](docs/IMPLEMENTATION.md#small-implementation-decisions)
  before adding types, traits, dependencies or retained state. Reuse algorithms,
  not obsolete orchestration; shorter code must not hide failures.
- Read the relevant [pinned donor](docs/DONORS.md). Emmy owns language analysis,
  Project source selection, Service assembly and frontends transport/rendering.
  Never execute analyzed Lua, rewrite user source or fabricate missing metadata.
- Preserve exact bytes/locations, partial input and truthful cancellation. Unknown
  input is not an absent API or clean result. After two failed attempts inspect
  the failed assumption and change method, not limits or expected outcomes.

## Active slice

[Slice 1A](docs/REFACTOR_QUEUE.md#1-first-usable-ordinary-addon-check) connects physical
Lua to the existing configured Emmy owner through one Service operation. Follow
[its contract](docs/IMPLEMENTATION.md#first-implementation-slice). Later loading,
search and WoW rules do not expand this task. User instructions override this
workflow; old Issues, fixtures and removed specifications do not override scope.

## Verification and handoff

Write product code first for implementation tasks. Run formatting and the smallest
strict Clippy invocation covering changed crates/callers; use
[the check matrix](CONTRIBUTING.md#choose-the-checks), not another cargo check pass.
Do not add or run tests until functional completion, as directed by the owner.
Do not delete a test merely because it fails.

For prose, inspect source claims, links and consistency; use existing xtask checks
for local paths/anchors and skill synchronization, not a workspace build. A source
review, compiler pass and executed user workflow are different evidence.

Update only affected [status/row/contract/trace](CONTRIBUTING.md#update-the-right-place).
Re-read the remote head before publishing one coherent branch/PR. Handoff needs
the changed behavior, actual checked revision/command/result, remaining blocker
and next register row. No parallel diary, correction counter or new acceptance
platform. Do not claim an unresolved row complete because its documentation exists.
