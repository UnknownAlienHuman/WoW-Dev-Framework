# `wow-emmy` implementation status

## Implemented executable boundaries

- Cross-platform rolling manager for the current upstream analyzer source.
- Local clone preference with configurable cache path and remote/branch.
- `auto`, `prompt`, and `never` update policies.
- Clean matching-branch fast-forward only; dirty, ahead, diverged, detached/wrong-branch, and wrong-origin checkouts are preserved and reported.
- Operation-scoped exact commit/tree identity and deterministic public-surface compatibility report.
- Rust revalidation of the compatibility report before backend identity is accepted.
- Explicit content-addressed in-memory Lua workspaces bound to one backend identity and one source universe.
- UTF-8/NUL, path, extension, file count, per-file bytes, total bytes, duplicate-path, and case-collision guards.
- Exact source text preservation and case-sensitive file lookup.
- A direct pinned `emmylua_code_analysis` adapter operation for Lua/documentation syntax diagnostics.
- Exact snapshot-relative source paths, content digests, canonical UTF-8 byte spans, deterministic diagnostic ordering, and content-addressed analysis identity.
- Explicit Main plus Library semantic workspaces for direct static `receiver.member(...)` calls.
- Linked member-reference and call facts with exact receiver/member/reference/call spans, argument count, colon-call shape, and `resolved` / `unresolved` / `possible` analyzer state.
- Library parse-health fail-closed behavior and per-Main-file fact capability state.
- Linux and Windows workspace tests plus rolling parser/current-source compatibility CI.

## Not yet implemented

- General symbol, definition, reference, type, hover, call-hierarchy, or rename operations.
- Non-call member/reference inventory and dynamic/computed member facts.
- Local binding, producer/use, operation, guard, and control-flow facts.
- Mapping analyzer output into complete `wow-core` evidence, coverage, generation contexts, findings, and result envelopes.
- Incremental overlays, cancellation, budgets beyond the current bounded operations, concurrent sessions, and multi-generation cache behavior.
- Behavioral compatibility fixtures for the remaining semantic surfaces across analyzer updates.

No semantic result is promoted into WoW platform truth. In particular, an unresolved member is an analyzer observation only; it does not prove API absence, Secret status, hook safety, replacement, or runtime behavior.

## Next package

Implement the smallest function-local fact slice against the same exact upstream pin:

1. producer member call and local binding identity;
2. direct local copies and selected operations;
3. exact binding/use/operation spans under shadowing;
4. `canaccessvalue(value)` guard observation without a safety verdict;
5. deterministic facts, failure isolation, and compatibility fixtures.
