# Crate contribution guidance

Follow the repository [`AGENTS.md`](../AGENTS.md), current
[`docs/ARCHITECTURE.md`](../docs/ARCHITECTURE.md), and selected task in
[`docs/REFACTOR_QUEUE.md`](../docs/REFACTOR_QUEUE.md). Read the target owner
README and only the code, callers, and existing fixtures needed for the change.

## Ownership

Keep each algorithm in its existing owner and prefer a direct, cohesive call to
that owner over a new abstraction or crate. `wow-service` coordinates use cases
that cross owners; it is not a mandatory wrapper for a single-owner operation.
Applications may depend on the relevant owner when that makes a small user path
clearer. Do not reproduce a neighboring owner's parser, analyzer, SQL, ranking,
or normalization logic.

| Owner | Responsibility |
|---|---|
| `wow-core` | Small shared identities, bounded-input primitives, and common errors |
| `wow-store` | SQLite schema, transactions, queries, and derived-data cache |
| `wow-reference` | Blizzard API/reference source facts |
| `wow-annotations` | Native Ketho annotation normalization and type lowering |
| `wow-emmy` | Upstream EmmyLua parser/analyzer adapter |
| `wow-project` | Configured addon source, TOC/load interpretation, and project index |
| `wow-graph` | Useful typed relationships derived from source facts |
| `wow-recognizers` | Source-pattern recognition over owner facts |
| `wow-rules` | WoW diagnostics and actionable findings |
| `wow-search` | Local Blizzard/project retrieval and ranking |
| `wow-context` | Bounded source and API context for editor/tool consumers |
| `wow-service` | Simple multi-owner use cases |

Avoid speculative generic ports, result/operation envelope layers, frameworks, and compatibility variants without a concrete current caller.

## Source and protocol boundaries

- Treat addon and Blizzard source as data. Never execute Lua, repository hooks,
  build scripts, or downloaded tools during indexing.
- Keep parsing and traversal bounded and cancellable; validate file counts,
  bytes, nesting, spans, and output sizes at admission and emission boundaries.
- Preserve source locations and byte/span correctness through parsing,
  normalization, analysis, and LSP/MCP projection.
- Keep user source and project configuration intact. Indexed databases and
  generated annotation data are derived and may be rebuilt from those inputs.
- Return real, typed errors for unsupported or malformed input; do not disguise
  missing coverage as an empty successful result.

## Change practice

Replace obsolete paths when consolidating behavior. Use existing focused tests
or fixtures when they directly exercise the changed path; keep validation
proportional to the change. Do not turn legacy package matrices, release gates,
or optional integrations into prerequisites for ordinary development work.
