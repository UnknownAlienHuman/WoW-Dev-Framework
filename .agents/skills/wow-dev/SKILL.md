---
name: wow-dev
description: Implement and review the WoW Dev Framework and its WoW-aware addon checks using EmmyLua, Blizzard source and the native Ketho annotation pipeline.
---

# WoW development

Read `docs/CURRENT_STATUS.md`, root `AGENTS.md` and all of
`docs/ARCHITECTURE.md`. Use the relevant route in `docs/IMPLEMENTATION.md`,
behavior in `docs/WOW_LINTER.md`, and concrete donor modules in `docs/DONORS.md`.
Read only the owners and callers the task needs; do not repeat a slice already
recorded as implemented in the trace register.

Reuse EmmyLua for Lua analysis and the existing Rust Reference/Annotations path
for Blizzard/Ketho data. Keep client flavor, product channel, target/source
Interface, TOC execution/private-table context, API execution context and Secret
flow distinct. Preserve useful diagnostic text, exact source spans and visible
unknowns. Never execute analyzed Lua or infer runtime safety from types.

Finish a useful input-to-output path and remove its displaced platform code and
instructions. For the first public check, follow queue item 1 as the complete
checklist; README/AGENTS/package summaries cannot narrow it. One manager/worktree;
delegate nonoverlapping work. Preserve user source, configuration and unpublished
changes. Follow root AGENTS for current validation: product first,
formatting/affected strict Clippy; tests deferred until functional completion. Do
not restore historical gates or create status diaries.

For annotation changes read `docs/KETHO_RUST_PORT.md`; for engine changes read
`docs/EMMYLUA_UPSTREAM.md`. Preserve notices. Inspect the public generator help
for its actual current interface; target `wow library build`/`wow check` examples
in `docs/FRONTENDS.md` are not proof of delivered commands.
