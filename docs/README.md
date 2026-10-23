# Documentation

Start with the full [manifesto](ARCHITECTURE.md), then read only the guide needed
for the task. The code is the implementation; these documents describe its
intended replacement and identify known gaps explicitly.

| Question | Read |
| --- | --- |
| What are we building? | [Architecture](ARCHITECTURE.md) |
| What must a correct WoW linter understand? | [WoW behavior](WOW_LINTER.md) |
| Which code should change, and in what order? | [Implementation](IMPLEMENTATION.md), [queue](REFACTOR_QUEUE.md) |
| What can be reused? | [Donors](DONORS.md) |
| How are Blizzard facts converted to annotations? | [Ketho/native projection](KETHO_RUST_PORT.md) |
| How is EmmyLua configured and updated? | [EmmyLua](EMMYLUA_UPSTREAM.md) |
| How do users, editors and agents access it? | [CLI/LSP/MCP](FRONTENDS.md) |
| What did the audit establish? | [Verified audit](AUDIT.md) |
| How do contributors work? | [AGENTS.md](../AGENTS.md) |

Package READMEs are navigation to code, not separate product specifications.
Historical audits, contracts and command recipes remain available at their Git
revisions. Compile-time fixtures and third-party notices are data, not design
instructions.
