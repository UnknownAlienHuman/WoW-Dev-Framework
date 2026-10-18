---
name: wow-dev
description: Implement, debug and review WoW addons and the WoW Dev Framework using EmmyLua, the native Ketho annotation path, and actual Blizzard source.
---

# WoW development

Read root `AGENTS.md` and all of `docs/ARCHITECTURE.md`; keep the manifesto in
the developer's working context. Select an Issue from `docs/REFACTOR_QUEUE.md`,
then use its route in `docs/IMPLEMENTATION.md` to inspect actual code and donors.
Do not load every package contract or historical matrix. Use
`docs/EXECUTION_MODEL.md` for coordination and proportional checks.

## Implement one useful path

Trace the existing behavior first. Reuse upstream EmmyLua and the native Ketho
pipeline. Keep one current representation and delete the replaced code, options
and exclusive tests. Derived indexes/annotations can rebuild; source, user
configuration and unpublished work must survive. Do not preserve legacy readers,
cryptographic result envelopes, migration frameworks or optional plugin layers.

Read `docs/KETHO_RUST_PORT.md` for source/type/annotation changes and
`docs/EMMYLUA_UPSTREAM.md` for analyzer/editor work. Do not add a second parser,
Lua/Node/VS Code runtime or another language-server implementation. Retain donor
attribution. Keep useful errors, input/output bounds, cancellation and correct
source spans; do not treat partial data as proof of absence or runtime safety.

## Read current source when needed

For source/API-sensitive work, resolve the intended Blizzard/Ketho revision and
flavor once, then read its actual Lua, docs, XML and TOC. Generic storage, CI and
documentation changes need no new WoW corpus. Treat source as data; never execute
repository scripts, hooks, submodules or Lua while indexing. Reuse source
materializers and respect update mode; preserve dirty/diverged checkouts.

The native annotation development driver is:

```sh
cargo run --locked -p wow-annotations --example native_library --   <checkout> <ref> <TOC> <environment> <new-output>
```

Read `source-report.json`; exit 3 is partial. For a changed semantic, use an
existing representative positive/negative probe and the relevant donor behavior.
Do not build a fixture factory or run a whole-corpus/editor/runtime campaign for
a small internal change. State what was actually checked and any unsupported
behavior. Never publish private optional context or its location.
