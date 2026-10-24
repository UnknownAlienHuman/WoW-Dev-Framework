# Documentation

Start with [current status](CURRENT_STATUS.md), then the full
[manifesto](ARCHITECTURE.md), and read only the guide needed for the task. The
code is the implementation; target documents describe the intended product, and
baseline audit documents describe the commit they name rather than silently
tracking later merges.

| Question | Read |
| --- | --- |
| What is implemented now, and what remains unobserved? | [Current status](CURRENT_STATUS.md) |
| What are we building? | [Architecture](ARCHITECTURE.md) |
| What must a correct WoW linter understand? | [WoW behavior](WOW_LINTER.md) |
| Which code should change, and in what order? | [Implementation](IMPLEMENTATION.md), [queue](REFACTOR_QUEUE.md) |
| Which post-merge implementation boundaries were source-traced, implemented, or observed? | [Implementation trace register](IMPLEMENTATION_TRACE_REGISTER.md) |
| What can be reused? | [Donors](DONORS.md) |
| How are Blizzard facts converted to annotations? | [Ketho/native projection](KETHO_RUST_PORT.md) |
| How is EmmyLua configured and updated? | [EmmyLua](EMMYLUA_UPSTREAM.md) |
| How do users, editors and agents access it? | [CLI/LSP/MCP](FRONTENDS.md) |
| What did the baseline source audit establish? | [Verified baseline audit](AUDIT.md) |
| Which baseline documents and code paths were checked? | [Baseline review register](AUDIT_REGISTRY.md) |
| How do contributors work? | [AGENTS.md](../AGENTS.md) |

`AUDIT.md` and `AUDIT_REGISTRY.md` are tied to the reviewed `c36a058` baseline.
They remain evidence and design input, not a live progress page.
`IMPLEMENTATION_TRACE_REGISTER.md` records new producer-to-consumer traces and
executed observations after PR #244. `CURRENT_STATUS.md` reconciles those records
with current `main`. None of these documents is a readiness percentage.

Package READMEs are navigation to code, not separate product specifications.
Historical audits, contracts and command recipes remain available at their Git
revisions. Compile-time fixtures and third-party notices are data, not design
instructions.