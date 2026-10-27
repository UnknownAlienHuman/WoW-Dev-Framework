# Contributor instructions

Read, in order:

1. [current status](docs/CURRENT_STATUS.md);
2. [architecture](docs/ARCHITECTURE.md);
3. the active item in [the queue](docs/REFACTOR_QUEUE.md);
4. the relevant owner route in [IMPLEMENTATION.md](docs/IMPLEMENTATION.md) and
   domain behavior in [WOW_LINTER.md](docs/WOW_LINTER.md);
5. the actual producer, handoff and consumer code.

`CURRENT_STATUS.md` is the only live delivery checkpoint. The
[trace register](docs/IMPLEMENTATION_TRACE_REGISTER.md) records evidence and
observations. README files, package summaries, Issue history and removed
specifications are secondary and cannot expand or narrow the current task.

## Work

- One manager, one worktree/branch. Delegate only bounded reads or disjoint paths.
- Finish one useful producer-to-user path. Change the real producer and consumer
  together; remove displaced wrappers, options and prose with their last caller.
- Reuse [pinned donors](docs/DONORS.md) only after reading the exact relevant
  source. Blizzard facts, Ketho annotation behavior and Wowless comparison
  behavior are different authority classes.
- Keep source bytes, locations, ambiguity, incomplete input and cancellation
  honest. Never execute analyzed Lua or rewrite user source.
- After two failed attempts at one approach, identify the failed assumption and
  change method.

## First public check

Do not recreate analyzer or result owners already marked **implemented** in the
trace register. The first public command is a small `LuaTypes` vertical slice:

```text
ordinary explicit configuration
  -> bounded selected addon and Library bytes
  -> exact Main/Library snapshots
  -> existing ConfiguredAnalyzerSession
  -> existing wow-service::addon_check
  -> text/JSON diagnostics, relevant limitations and exit status
```

It must not wait for every Blizzard load feature. Product channel, `LoadWith`,
implicit bindings, secure/special contexts, full XML handler context and WoW
rules are subsequent slices unless the selected input actually requires them.
Unsupported selected input remains a located scoped limitation; unrelated or
unrequested features do not make `LuaTypes` globally partial.

Do not adapt the old `LocalCommand -> LocalProjectBackend -> generic Service`
route, fabricate Reference/Rules/publication objects, or wrap the direct result
inside component/capability/presentation models.

## Validation

Write the product path first. For Rust changes run formatting and the smallest
strict Clippy invocation covering affected crates and callers. Do not duplicate
that with a separate `cargo check`. Tests remain deferred until the product is
functionally complete, as directed by the owner.

For documentation changes:

- inspect source claims, current `main`, links/anchors and instruction consistency;
- compare old requirements with their current owner or explicit retirement;
- update the active register immediately when a finding changes a boundary;
- do **not** build the Rust workspace or cite Clippy as documentation evidence.

A build, source trace or document review never promotes a user workflow to
**observed**. Observation requires the exact ordinary command or protocol
sequence, input and visible result recorded in the trace register.

## Handoff

Publish one coherent branch/PR after rereading the remote head. State the change,
document checks and remaining limitations briefly. Do not create acceptance-tier,
proof-archive or status-diary machinery.
