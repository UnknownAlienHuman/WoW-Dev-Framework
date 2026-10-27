---
name: wow-dev
description: Implement and review the WoW Dev Framework and its WoW-aware addon checks using EmmyLua, Blizzard source and the native Ketho annotation pipeline.
---

# WoW development

Read `docs/CURRENT_STATUS.md`, root `AGENTS.md` and
`docs/ARCHITECTURE.md`. Use the active slice in `docs/REFACTOR_QUEUE.md`, the
relevant owner route in `docs/IMPLEMENTATION.md`, behavior in
`docs/WOW_LINTER.md`, and concrete donor modules in `docs/DONORS.md`. Read only
the owners and callers the task needs; do not repeat a slice already recorded as
implemented in the trace register.

Reuse EmmyLua for Lua analysis and the existing Rust Reference/Annotations path
for Blizzard/Ketho data. Keep client flavor, product channel, target/source
Interface, TOC execution/private-table context, API execution context and Secret
flow distinct. Preserve useful diagnostic text, exact source spans and visible
unknowns. Never execute analyzed Lua or infer runtime safety from types.

Finish one useful input-to-output path and remove its displaced platform code and
instructions. Queue item 1 is staged: implement only the active vertical slice.
The explicit `retail` physical-Lua `LuaTypes` command does not wait for product
channel, `LoadWith`, implicit bindings, every XML handler or WoW rules. Its
Project plan is profile-free and `physical_lua`: included XML is retained as a
located limitation but is not opened, so XML failure cannot erase direct Lua
diagnostics. Use a standalone requested-scope mapper; do not use package-only or
global completeness. Snapshots use the compiled pin, Main `Project`, and one
`BlizzardUi` snapshot per Library root. Reject duplicate roots/IDs and treat
Library parse failures as invalid input. XML-only physical input uses an empty
Project snapshot plus a located limitation, never a dummy file. Unsupported
selected context remains scoped; unrelated observations do not make the request
partial.

One manager/worktree; delegate nonoverlapping reads or paths. Preserve user
source, configuration and unpublished changes. Follow root AGENTS for validation:
product first, formatting/affected strict Clippy for Rust, tests deferred until
functional completion. For documentation audits inspect claims, links, anchors
and instruction consistency; do not cite Clippy as prose evidence. Do not restore
historical gates or create status diaries.

For annotation changes read `docs/KETHO_RUST_PORT.md`; for engine changes read
`docs/EMMYLUA_UPSTREAM.md`. Preserve notices. Inspect public help for its actual
current interface; target examples in `docs/FRONTENDS.md` are not proof of
delivered commands.
