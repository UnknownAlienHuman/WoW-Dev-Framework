## Why

A statement-bearing Ketho catalog was lexed once, then parsed independently by the namespace, FunctionContainer and GlobalColors validators. Each validator rebuilt the same EmmyLua Lua 5.1 tree and repeated the same syntax-error check before inspecting a different admitted shape.

## Change

- Parse the statement-bearing resource once in `native_aliases::ingest`.
- Reject parser errors at that common boundary.
- Pass one `LuaChunk` through namespace → FunctionContainer → GlobalColors validation in the existing priority order.
- Remove the three private parser invocations and duplicated syntax checks.
- Preserve lexical admission, spans, shape-specific errors, cancellation, no-execution policy and isolated parsing of comment groups.
- Mark #202 complete in the implementation queue.

Net product change: **+19/−34 lines across six files**. No AST cache, loader framework, grammar widening, pin change or compatibility layer.

## Validation

Repository policy, formatting and strict Clippy for `wow-reference` and `wow-annotations` (all targets/features) passed in [run 39260640046](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/actions/runs/39260640046). The exact PR merge candidate must pass the full Quick gate before merge.

Closes #202. Squash directly into `main`; no long-lived integration branch.
