# `apps/wow` contribution guidance

Follow repository [`AGENTS.md`](../../AGENTS.md),
[`docs/ARCHITECTURE.md`](../../docs/ARCHITECTURE.md),
[`docs/REFACTOR_QUEUE.md`](../../docs/REFACTOR_QUEUE.md), and the shared
[`crates/AGENTS.md`](../../crates/AGENTS.md).

This is the user-facing tool for local addon development: a clear CLI, useful
stdio LSP, and MCP integration where it serves the same local project index.
Keep commands and configuration small and explicit. A project config should
make root, WoW flavor, and TOC selection understandable; show bounded candidate
choices when discovery is ambiguous. Never guess from cwd or silently pick a
flavor/TOC.

Use EmmyLua through `wow-emmy` and native Ketho annotation behavior through the
existing annotation owner. Applications may call an existing owner directly
for a single-owner feature; use `wow-service` for genuine multi-owner flows.
Keep protocol framing and projection in the app, but do not reproduce parsing,
analysis, search ranking, or storage logic here.

Preserve standard LSP/MCP framing, notifications, empty results, and actual
protocol errors. Keep stdio streams clean: only protocol messages on protocol
stdout; send diagnostics to stderr. Preserve source byte/span locations in
editor responses. Bound arguments, config, protocol frames, document bytes, and
output; propagate cancellation where supported. Never execute addon source,
repository scripts, shell commands, or project-provided tools.

Do not retain unneeded daemon/session lifecycle, HTTP hosting, release/update,
installer, backup, migration, or generic compatibility flows as product
obligations. Preserve the simple local user path and the source/config files;
derived indexes can be rebuilt. Keep any MCP surface aligned with actual useful
operations rather than a generic service invocation mechanism.

Read only the route and owner callers affected by the change. Use a focused
existing check or fixture when it clarifies behavior; no package-wide acceptance
campaign is required for a bounded CLI/LSP/MCP change.
