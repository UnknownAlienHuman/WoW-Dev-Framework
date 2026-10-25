# EmmyLua adapter

Current status: [PRs #246–#250 and open handoffs](../../docs/CURRENT_STATUS.md).

Delivered: `ConfiguredAnalyzerSession` owns exact Main, explicit Library and
optional virtual snapshots in one Lua 5.1 analysis; embedded STD and caller
diagnostic configuration are applied once; diagnostics-only and explicit
semantic collection are separate. Diagnostic code, exact message, severity and
byte range reach the direct result. Exact Main identity is required for immutable
owner reuse.

Reuse `configured_session.rs`, `session.rs`, `references.rs`, existing semantic
collectors and coordinate mapping. Do not add another analyzer owner or repeat
the diagnostic adapter. Remaining work is Project-owned ordinary acquisition and
validated order/context plus narrow mapped addon/XML/binding adapters, coherent
updates and project-aware frontend reuse.

Read [EmmyLua](../../docs/EMMYLUA_UPSTREAM.md) for the exact-pin contract and
[Implementation](../../docs/IMPLEMENTATION.md#addon-configuration-and-emmy-analysis)
for owner boundaries.
