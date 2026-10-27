# Documentation

Start with [current status](CURRENT_STATUS.md), then
[architecture](ARCHITECTURE.md) and the active item in
[the queue](REFACTOR_QUEUE.md). Read only the owner/domain guide needed for the
change.

| Question | Read |
| --- | --- |
| What is delivered, open and actually observed? | [Current status](CURRENT_STATUS.md) |
| What are we building and excluding? | [Architecture](ARCHITECTURE.md) |
| What must the WoW-aware linter understand? | [WoW behavior](WOW_LINTER.md) |
| Which owner changes and in what direct path? | [Implementation](IMPLEMENTATION.md) |
| What is the next vertical slice? | [Queue](REFACTOR_QUEUE.md) |
| Which source decisions and observations support it? | [Trace register](IMPLEMENTATION_TRACE_REGISTER.md) |
| Which old documents and requirements were reconciled? | [Baseline register](AUDIT_REGISTRY.md) |
| What did the baseline source audit establish? | [Baseline audit](AUDIT.md) |
| Which active Issues were checked as instructions? | [Active Issue audit](ACTIVE_ISSUE_AUDIT.md) |
| Which TOC/load details were source-traced? | [Load context](TOC_LOAD_CONTEXT_AUDIT.md), [triggers and bindings](TOC_TRIGGER_BINDINGS_AUDIT.md) |
| How does EmmyLua own parsing/types/diagnostics? | [EmmyLua](EMMYLUA_UPSTREAM.md) |
| How are Blizzard facts converted to a Library? | [Ketho/native projection](KETHO_RUST_PORT.md) |
| What code may be reused? | [Donors](DONORS.md) |
| How do CLI/LSP/MCP expose the owners? | [Frontends](FRONTENDS.md) |

## Authority and maintenance

- `CURRENT_STATUS.md` is the **only live delivery checkpoint**.
- `REFACTOR_QUEUE.md` is the ordered work list, not a PR history.
- `IMPLEMENTATION_TRACE_REGISTER.md` records source decisions and exact
  observations.
- `AUDIT.md` and `AUDIT_REGISTRY.md` remain tied to the baseline commit they name.
- Owner/domain guides define durable contracts, not moving lists of merged PRs.
- README, AGENTS, skills, package summaries and Issue bodies are navigation and
  task-specific instructions; they must link to live status instead of copying it.

Only the live status and trace register may enumerate current delivered slices or
verification runs. This prevents a merge from leaving half the documentation one
or more implementation steps behind.

Historical platform specifications remain in Git history. Compile-time fixtures
and third-party notices are data/attribution, not active architecture.
