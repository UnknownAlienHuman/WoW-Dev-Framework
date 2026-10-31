# Documentation

Start with root [AGENTS](../AGENTS.md), [current status](CURRENT_STATUS.md) and the
assigned row of the [work register](REFACTOR_QUEUE.md). Its [map route](REPOSITORY_MAP.md)
leads to existing code and the relevant contract. Read architecture on first entry
or when changing ownership; historical audits are not startup requirements.

## Working guides

| Question | One owning source |
| --- | --- |
| What exists? | [Current status](CURRENT_STATUS.md) |
| What do I implement, after what, for which caller, and what do I remove? | [Implementation work register](REFACTOR_QUEUE.md) |
| Where is its producer/consumer code? | [Repository map](REPOSITORY_MAP.md) |
| What is the product and what is excluded? | [Architecture](ARCHITECTURE.md) |
| How do I keep the design small and assemble the checker? | [Implementation](IMPLEMENTATION.md) |
| Which game semantics are required or uncertain? | [WoW linter](WOW_LINTER.md) |
| What are CLI/config/output and protocol contracts? | [Frontends](FRONTENDS.md) |
| How does the pinned engine behave? | [EmmyLua](EMMYLUA_UPSTREAM.md) |
| How do selected facts become a usable Library? | [Native projection](KETHO_RUST_PORT.md) |
| Which source/library can be reused or updated? | [Donors](DONORS.md) |
| Which checks and documentation updates apply? | [Contributing](../CONTRIBUTING.md) |
| What source/run evidence supports a claim? | [Trace register](IMPLEMENTATION_TRACE_REGISTER.md) |

## Maintaining the documentation

Change the owning rule once; update only affected routes and facts. Status is not
a task queue, the map is not a specification, and a source trace is not an executed
workflow. Issue bodies select a row and name the outcome/exclusions, not another
set of limits or implementation stages. Every open implementation Issue must have
a route in the existing register; no duplicate progress diary or correction IDs.

When a file/symbol moves, fix its map row and inbound links. For source-sensitive
changes use [the donor procedure](DONORS.md#updating-a-reviewed-revision). For a
failure start with [the failed check](../CONTRIBUTING.md#failure-and-handoff), not
a new repository-wide audit. A new guide needs a distinct responsibility and an
entry here; otherwise edit the existing guide.

## Background evidence

Read only to resolve a specific source question: [baseline source audit](AUDIT.md),
[TOC/load context](TOC_LOAD_CONTEXT_AUDIT.md), [triggers/bindings](TOC_TRIGGER_BINDINGS_AUDIT.md)
and the [historical document inventory](AUDIT_REGISTRY.md#historical-baseline).
The removed duplicate Issue-audit page remains in its
[immutable historical revision](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/blob/3fa003e3fc7c90cb6f205dc6ded5da0d627b4e1a/docs/ACTIVE_ISSUE_AUDIT.md).
These records retain evidence, not current task status or authority to restore old
platform requirements. Third-party notices remain attribution, not architecture.
