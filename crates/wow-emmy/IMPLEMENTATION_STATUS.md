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
- Linux, Windows, and macOS manager tests plus current-upstream compatibility CI.

## Not yet implemented

- Direct calls into `emmylua_code_analysis`.
- Syntax tree and semantic-model ownership.
- Diagnostics, symbols, definitions, references, types, hover, call hierarchy, or rename operations.
- Mapping analyzer output into `wow-core` evidence, coverage, source handles, findings, and result envelopes.
- Incremental overlays, cancellation, budgets, concurrent sessions, and multi-generation cache behavior.
- Behavioral compatibility fixtures across analyzer updates.

No semantic capability is advertised until the corresponding adapter operation compiles against the current dependency and passes deterministic fixture tests. A successful source-manager report or upstream package compilation is compatibility evidence only; it is not a semantic-result claim.

## Next package

Implement the smallest direct adapter operation against the current upstream API:

1. construct one analyzer database from an explicit `LuaWorkspaceSnapshot`;
2. ingest exact files without filesystem discovery;
3. return syntax diagnostics with exact file/span identity;
4. prove deterministic ordering, malformed-input behavior, and backend/source identity binding;
5. add the operation's required symbols and behavior fixtures to the rolling compatibility gate.
