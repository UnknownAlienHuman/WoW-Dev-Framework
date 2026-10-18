# Documentation

Start with root [AGENTS.md](../AGENTS.md) and read the complete product manifesto.
Then select one task and read only its implementation route, code and donor.

| Question | Document |
| --- | --- |
| Why does the product exist, what must it do, what is out of scope? | [Architecture manifesto](ARCHITECTURE.md) |
| What code exists, how should it change, what comes from donors? | [Implementation](IMPLEMENTATION.md) |
| What should be done next? | [Queue](REFACTOR_QUEUE.md), selected GitHub Issue |
| What works and what is still missing? | [Completion](PROJECT_COMPLETION_MATRIX.md) |
| How do contributors coordinate, check and publish? | [Execution](EXECUTION_MODEL.md) |
| Which annotation behavior matters? | [Ketho map](KETHO_RUST_PORT.md), [corrections](KETHO_NATIVE_CORRECTIONS.md) |
| How is the Lua engine/server reused? | [EmmyLua](EMMYLUA_UPSTREAM.md) |
| Which defects were verified? | [Audit](AUDIT_2026-10-17.md) |
| Which commands currently run? | [CLI](../apps/wow/README.md), [stdio](FRONTENDS.md) |

Detailed owner documents describe their current code. They cannot restore
removed scope or require historical package/certification matrices.
