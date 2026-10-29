# Documentation

Start with [current status](CURRENT_STATUS.md) and
[architecture](ARCHITECTURE.md), then select one [queue item](REFACTOR_QUEUE.md).
Read its linked owner/domain sections and actual code. The historical audits are
background, not a prerequisite to implementation.

## Working guides

| Need | Read |
| --- | --- |
| What exists and what is next? | [Current status](CURRENT_STATUS.md), [queue](REFACTOR_QUEUE.md) |
| Product purpose and exclusions | [Architecture](ARCHITECTURE.md) |
| Code owners and the active check contract | [Implementation](IMPLEMENTATION.md) |
| WoW semantics and known static-analysis limits | [WoW linter](WOW_LINTER.md) |
| CLI/config/output and later protocol adapters | [Frontends](FRONTENDS.md) |
| Pinned Lua engine, diagnostic configuration and chunk context | [EmmyLua](EMMYLUA_UPSTREAM.md) |
| Blizzard/Ketho annotation generation | [Native projection](KETHO_RUST_PORT.md) |
| Exact sources and reuse boundaries | [Donors](DONORS.md) |
| Contribution workflow | [AGENTS.md](../AGENTS.md) |

## Maintaining the documentation

- Status owns delivered/open behavior; the queue owns work order and completion
  cases. Neither repeats the implementation contract.
- Implementation owns source admission and Service assembly. Frontends owns
  command/config/output semantics; EmmyLua owns engine-specific rules; WoW linter
  owns game semantics. Link across these boundaries rather than copying rules.
- [Implementation traces](IMPLEMENTATION_TRACE_REGISTER.md) retain source
  pointers and actual observations. Update the affected row when it changes;
  do not add a correction counter or another progress log.
- [Documentation work](AUDIT_REGISTRY.md) tracks cleanup and ongoing maintenance.
  A source trace, compiled change and executed workflow are different evidence.

README/package instructions and Issues route to these guides. Existing fixtures
and old specifications do not add product scope.

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
