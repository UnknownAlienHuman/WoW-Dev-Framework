# `wow-emmy` contribution rules

Follow the root [`AGENTS.md`](../../AGENTS.md), selected Issue or queue entry,
this crate's [`CONTRACT.json`](CONTRACT.json), and the caller/owner contracts
that the change touches. Consult [`EMMYLUA_UPSTREAM.md`](../../docs/EMMYLUA_UPSTREAM.md)
for the analyzer and server boundary.

## Engine and adapter

- Keep EmmyLua types and calls inside this adapter. Expose normalized
  `wow-core`/`wow-emmy` contracts downstream.
- Reuse the existing `EmmyLuaAnalysis` session and upstream syntax, resolution,
  member, or flow facts. Keep one parser/analyzer; do not add a parallel session,
  parser, cache, or Lua runtime.
- The upstream semantic API is public through `emmylua_code_analysis`.
  Hover/completion/definition and other LSP handlers are implemented in
  `emmylua_ls` but private to that crate. Use the upstream server for ordinary
  editor functionality; do not make a custom full language server part of this
  adapter.
- Ordinary work uses the compiled revision in `Cargo.lock`. Change the pin only
  for a concrete compatibility/current-source need; retain the exact revision
  and required license attribution, and check affected behavior. Keep changed
  diagnostic families unclassified until reviewed.

## Facts and identity

- Preserve explicit Lua 5.1 and diagnostic configuration; Main and Library
  workspaces remain separate. Library files aid resolution and have their own
  coverage; they are not project implementation files.
- Preserve physical/virtual unit identity, exact registration, source digest,
  and generation. `ProjectGenerationId` belongs to `wow-project`; bind all
  updates, snapshots, facts, and findings to the supplied generation. Keep
  analyzer snapshot identity separate.
- Canonical spans are UTF-8 byte half-open ranges. Convert upstream coordinates
  inside the adapter; do not expose upstream URIs or host paths.
- Emit only analyzer-observable syntax, resolution, call, local-flow,
  operation, guard, control-flow, diagnostic, and coverage facts. These do not
  prove WoW API absence, restriction/Secret truth, load ownership, or runtime,
  combat, or taint behavior.
- Keep typed uncertainty and failures. Parse/library failures narrow dependent
  capability; cancellation and invalidation bind to the exact update/snapshot.
  Do not publish uncertain facts after corruption or poisoned state.

## Inputs and checks

Analyzed Lua, addon repositories, comments, and docs are inert data. Never
execute them. Keep paths within configured logical roots, enforce existing
input/fact/output limits, and never leak absolute host paths. Do not mutate
editor settings or external editor state.

Run the smallest affected owner chain and one existing targeted reproduction
when useful. For a changed semantic consumer behavior, use a representative
positive and negative check. Consult `TEST_MATRIX.md` for its named milestones;
whole-matrix, current-source, or client qualification belongs to a task that
actually claims that coverage. Report the changed operation/facts, checks run,
and material limits or unverified capabilities.
