# Documentation

Follow [AGENTS.md](../AGENTS.md) for startup and work policy. For an assigned task,
use its [repository-map route](REPOSITORY_MAP.md), owner section and actual
producer/consumer code. Use [the queue](REFACTOR_QUEUE.md) to choose unassigned
work; [current status](CURRENT_STATUS.md) separates delivered from planned.
Historical audits are background, not a prerequisite to implementation.

## Working guides

| Need | Read |
| --- | --- |
| What exists and what is next? | [Current status](CURRENT_STATUS.md), [queue](REFACTOR_QUEUE.md) |
| Where is the code and who consumes it? | [Repository map](REPOSITORY_MAP.md) |
| Which checks should run, and what must be updated? | [Check matrix and maintenance](../CONTRIBUTING.md) |
| Product purpose and exclusions | [Architecture](ARCHITECTURE.md) |
| Code owners, smallest design and active check contract | [Implementation](IMPLEMENTATION.md), [implementation decisions](IMPLEMENTATION.md#small-implementation-decisions) |
| WoW semantics and known static-analysis limits | [WoW linter](WOW_LINTER.md) |
| CLI/config/output and later protocol adapters | [Frontends](FRONTENDS.md) |
| Pinned Lua engine, diagnostic configuration and chunk context | [EmmyLua](EMMYLUA_UPSTREAM.md) |
| Blizzard/Ketho annotation generation | [Native projection](KETHO_RUST_PORT.md) |
| Exact sources, existing primitives and revision updates | [Donors](DONORS.md) |
| Contribution workflow | [AGENTS.md](../AGENTS.md) |

For a documentation-only task, read the affected guide, its source evidence and
inbound links; do not reopen every implementation trace. For a donor update, read
[the revision procedure](DONORS.md#updating-a-reviewed-revision) and the affected
projection/engine consumer, not every donor. For a failing check, start with
[the actual command and failure](../CONTRIBUTING.md#failure-and-handoff).

## Maintaining the documentation

- Status owns delivered/open behavior; the queue owns work order and completion
  cases. Neither repeats the implementation contract.
- Implementation owns source admission and Service assembly. Frontends owns
  command/config/output semantics; EmmyLua owns engine-specific rules; WoW linter
  owns game semantics. Link across these boundaries rather than copying rules.
- The repository map owns navigation, not contracts. Keep moved paths/symbols
  current; [the maintenance table](../CONTRIBUTING.md#update-the-right-place)
  identifies which other documents change for a given edit.
- [Implementation traces](IMPLEMENTATION_TRACE_REGISTER.md) retain source
  pointers and actual observations. Update the affected row when it changes;
  do not add a correction counter or another progress log.
- [Documentation work](AUDIT_REGISTRY.md) tracks cleanup and ongoing maintenance.
  A source trace, compiled change and executed workflow are different evidence.

README/package instructions and Issues route to these guides. Existing fixtures
and old specifications do not add product scope. A new guide needs a distinct
owner and an inbound route here; otherwise improve the existing guide.

## Background evidence

Read only for the question being investigated:

- [Baseline source audit](AUDIT.md) and the
  [historical document inventory](AUDIT_REGISTRY.md#historical-baseline).
- [Issue audit at its named revision](ACTIVE_ISSUE_AUDIT.md).
- [TOC/load context](TOC_LOAD_CONTEXT_AUDIT.md) and
  [triggers/bindings](TOC_TRIGGER_BINDINGS_AUDIT.md).

Historical findings may have been fixed; compare them with current status and
code. Git retains old contracts and audit detail. Third-party notices remain
attribution, not architecture.
