# Implementation status and source route

Use [PROJECT_COMPLETION_MATRIX.md](PROJECT_COMPLETION_MATRIX.md) for current
capability and [REFACTOR_QUEUE.md](REFACTOR_QUEUE.md) for work. Scope is the
2026-10-18 [architecture](ARCHITECTURE.md); old E0–E7 statuses are superseded.
Cargo manifests list compiled packages. Do not recreate code from stale package
labels such as `not-started` or `partial`.

## Existing paths worth reusing

- EmmyLua parser and analysis are embedded through `wow-emmy`; use the selected
  Cargo lockfile pin. No parser/engine replacement is planned.
- `wow-reference` and `wow-annotations` already provide native source evaluation,
  Ketho-style projection, aliases/literals, selected corrections, source maps
  and representative consumer probes. See [KETHO_RUST_PORT.md](KETHO_RUST_PORT.md).
- Project owns TOC/XML/load/source inputs. Search, context, persistence and
  selected LSP/MCP routes are implemented; the queue removes repeated work,
  excess representations and inconvenient public inputs.
- The optional literal guest/host remains code until #203 removes it. Native
  literals must remain usable throughout that change.

## Source-sensitive work

Select one Blizzard revision/flavor, read its actual docs/implementation/XML/TOC,
and record the donor revision used for comparison. Current source selection is
an operation input, not a permanent source pin. Use the existing materializer and
source update commands; preserve private/dirty/diverged checkouts.

Generic storage, documentation and CLI plumbing need no external WoW corpus.
An annotation change uses the native driver and relevant donor cases; unavailable
consumer/runtime checks remain untested. Keep optional private context advisory
and out of public logs, fixtures and Issues.
