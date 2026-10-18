# `wow-reference` contribution rules

Follow the root [`AGENTS.md`](../../AGENTS.md), selected Issue or queue entry,
and the specific Reference contract/data model and caller paths touched. Read
[`KETHO_RUST_PORT.md`](../../docs/KETHO_RUST_PORT.md) for annotation/source
normalization work. Check pinned dependency APIs that the change actually uses.

## Source and model boundaries

- Treat Lua-shaped source, documentation, and generated files as untrusted
  data. Parse/evaluate only through the existing restricted owner; never execute
  Lua, source hooks, addon scripts, or client code. Do not add a second parser,
  interpreter, or source-acquisition route.
- Source-sensitive operations select one exact Blizzard source revision and
  client flavor, including the relevant TOC/API documentation inputs. Preserve
  those identities in output. Do not infer `current`, `latest`, or another
  flavor/build, or use dirty/uncommitted files as source evidence.
- Preserve raw fields and expressions before typed lowering. Unknown or
  unsupported values remain visible with their source identity and capability
  impact; do not silently discard them or coerce them to defaults. A failed or
  partial capability cannot prove absence.
- Keep facts bound to one profile/reference generation. Preserve distinct
  conflicts and coverage states; input ordering must not choose a semantic
  winner. Exact lookup stays exact and reports its coverage/authority state.
- Apply existing input, nesting, record, expression, and output bounds. Reject
  path traversal and absolute host paths before constructing source handles.
- Preserve restriction metadata separately from annotations. Static source
  structure does not prove runtime, combat, Secret, protected-action, or taint
  behavior.

## Checks and reporting

Run the smallest affected owner chain and an existing targeted reproduction
when useful. For changed semantics, use one representative positive and
negative case. Full fixture, current-source, or release-profile campaigns are
for tasks that make those broader claims, not prerequisites for every bounded
change. Report selected source/profile identity, behavior changed, checks run,
and material partial or unverified coverage.
