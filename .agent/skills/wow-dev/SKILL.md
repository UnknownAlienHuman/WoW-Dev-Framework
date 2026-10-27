---
name: wow-dev
description: Implement and review the WoW Dev Framework and its WoW-aware addon checks using EmmyLua, Blizzard source and the native Ketho annotation pipeline.
---

# WoW development

Read `docs/CURRENT_STATUS.md`, root `AGENTS.md`, the active
`docs/REFACTOR_QUEUE.md` slice, the relevant `docs/IMPLEMENTATION.md` owner route
and actual producer/consumer code.

Reuse EmmyLua and existing Project/Reference/Annotations owners. Keep flavor,
channel, Interface, load/chunk context, API context and Secret flow distinct.
Never execute analyzed Lua or infer runtime safety from types.

For Slice 1A follow the canonical queue literally. Frontends call one public
Service operation; Service owns trusted assembly; Project owns safe acquisition
and source authority; Emmy owns one configured analysis. Do not recreate the
profile/publication/proof platform, construct lower requests in frontends, scan
Lua text instead of using the same Emmy AST, or widen the slice to later loading
features.

Use formatting and affected strict Clippy for Rust changes; tests remain deferred
until functional completion. Documentation audits validate sources, links,
anchors and instruction consistency—not Clippy. Preserve notices and user source.
