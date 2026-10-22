# Documentation

Read root [AGENTS.md](../AGENTS.md) and the complete manifesto. Then select one
Issue and read only its implementation route, code and donor.

| Question | Document |
| --- | --- |
| Why does the product exist and what is out of scope? | [Architecture manifesto](ARCHITECTURE.md) |
| What is wrong with the current system and what must be removed/reused? | [Source and architecture audit](AUDIT.md) |
| What has actually been inspected, and what remains unverified? | [Audit coverage registry](AUDIT_REGISTRY.md) |
| What are the target owners and replacement routes? | [Implementation](IMPLEMENTATION.md) |
| What is the ordered work? | [Queue](REFACTOR_QUEUE.md), selected GitHub Issue |
| Which user workflows are actually exercised? | [Completion](PROJECT_COMPLETION_MATRIX.md) |
| How do contributors coordinate and check a change? | [Execution](EXECUTION_MODEL.md) |
| Which annotation semantics matter? | [Ketho map](KETHO_RUST_PORT.md), [corrections](KETHO_NATIVE_CORRECTIONS.md) |
| How is upstream Lua analysis reused? | [EmmyLua](EMMYLUA_UPSTREAM.md) |
| How should CLI/LSP/MCP behave? | [Frontends](FRONTENDS.md) |

Package owner notes may describe current code but cannot expand the manifesto or
restore retired compatibility, proof, lifecycle or certification requirements.
Historical plans remain in Git history. Do not load the audit, registry or every
package specification into a small implementation task unless that task changes
the architecture or continues the audit.
