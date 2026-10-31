# Current status

**Runtime source baseline:** `8ab892f040c42a9877836be8bd9818b98b1455d8`, reviewed
2026-10-29. Later documentation changes do not advance runtime implementation.
The [work register](REFACTOR_QUEUE.md) owns tasks, prerequisites and removal order;
[traces](IMPLEMENTATION_TRACE_REGISTER.md) own source/run evidence.

**The ordinary addon checker is not implemented yet.** CLI still reaches
`LocalProjectBackend -> generic Service`; `wow check --root ...` in
[Frontends](FRONTENDS.md) is a target, not a working interface.

| Area | Existing useful code | Remaining connected work |
| --- | --- | --- |
| EmmyLua | Configured Lua 5.1 Main/Library/virtual session, STD and upstream diagnostics | Strict/canonical settings, global-off, structured Library/resource failures and same-owner chunk requirements; #273 |
| Project | Bounded disk/Library readers, TOC/XML predicates and source maps | Profile-free physical plan, empty Main, safe preparation/authority and correct selected-child failures; #200 |
| Service / CLI | Lower exact-snapshot result and located limitations | Source-to-result operation and ordinary config/rendering. Lower checker still adds unconditional chunk Partial; #194/#197 |
| Generation | Native Reference/Annotations/Ketho pipeline | Failure-safe bounded output, missing projection semantics and ordinary metadata/map reader; #274 |
| Editors / lookup | Transitional LSP/MCP, overlay, Store/Search/Context routes | Shared ordinary owner, coherent candidates and direct later-process SQLite lookup; register sections 3–4 |

## Next implementation

Start with one unblocked [Slice 1A row](REFACTOR_QUEUE.md#1-first-usable-ordinary-addon-check).
[Draft #266](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/pull/266) contains
T23 metadata work and must be reconciled, not duplicated or treated as merged.
Emmy fixes can use the existing checker independently; source preparation, Service
and CLI must then join into one path. Dependencies/XML/chunk injection, special
loading, generated metadata and source lookup remain subsequent bounded work.

## Evidence boundary

No ordinary addon, editor or source-search workflow is recorded as executed.
Repository/link checks, formatting and Clippy do not establish that behavior.
Tests remain deferred under [AGENTS](../AGENTS.md#verification-and-handoff).
Update an affected row only when its delivery changes; do not copy the detailed
failure rules or task lists here. Historical records are linked from the
[documentation index](README.md#background-evidence).
