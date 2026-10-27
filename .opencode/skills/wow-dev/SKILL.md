---
name: wow-dev
description: Implement and review the WoW Dev Framework and its WoW-aware addon checks using EmmyLua, Blizzard source and the native Ketho annotation pipeline.
---

# WoW development

Read `docs/CURRENT_STATUS.md`, root `AGENTS.md`, `docs/ARCHITECTURE.md`, the active
queue slice, the relevant owner route and actual producer/consumer code.

Reuse EmmyLua and existing Project/Reference/Annotations owners. Preserve flavor,
channel, Interface, load/chunk context, API context and Secret flow as distinct
facts. Never execute analyzed Lua or infer runtime safety from types.

For Slice 1A implement only strict explicit root/TOC/`retail`/Library input through
one public Service ordinary-check operation. Project owns held-root/no-follow
acquisition, root/file authority, target/scope plan and limitations. Service owns
plan/Main binding, cross-role collision checks, snapshots, structured input-error
mapping and the lower configured check. Frontends own precedence/rendering only;
they do not construct snapshots or attach limitations.

Reject the same admitted physical Lua across roles/roots, not root ancestry by
itself. Preserve root token + relative path on errors. Complete/partial results go
to stdout; typed pre-result errors go to stderr. Do not add profiles, publication,
proof, graph/store or another analyzer/result framework.

Use formatting and affected strict Clippy for Rust changes; tests remain deferred
until functional completion. Documentation audits inspect source claims, links,
anchors and instruction consistency; Clippy is not prose evidence. Preserve
notices and user source.
