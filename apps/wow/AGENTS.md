# CLI and protocols

Follow root [AGENTS.md](../../AGENTS.md), the full
[manifesto](../../docs/ARCHITECTURE.md) and the relevant
[implementation route](../../docs/IMPLEMENTATION.md).

Expose ordinary inputs and the same configured operations through CLI/LSP/MCP.
For the first public check, use
[queue item 1](../../docs/REFACTOR_QUEUE.md#1-first-usable-ordinary-addon-check)
as the complete handoff; do not reconstruct or subset Project load/context facts
inside the application. Keep protocol stdout clean, positions/version accurate
and errors meaningful. Read [frontends](../../docs/FRONTENDS.md). Do not reproduce
language/domain logic or preserve internal IDs/profile families as normal user
requirements.
