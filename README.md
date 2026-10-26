# WoW Dev Framework

A **World of Warcraft addon linter**, written in Rust on top of
[EmmyLua](https://github.com/EmmyLuaLs/emmylua-analyzer-rust), Blizzard's API/UI
source and the useful annotation semantics from
[Ketho](https://github.com/Ketho/vscode-wow-api).

The product should catch Lua/type errors and selected WoW-specific mistakes while
an addon is being written: API calls, optional/multiple returns, callbacks,
widgets, TOC/XML loading and supported restrictions. Generated libraries remain
ordinary `.lua` files. Source lookup serves diagnostics and development; it is
not a prerequisite for checking an addon.

## Current status

The documentation/source audit was consolidated in PR #244. The authoritative
live checkpoint is [docs/CURRENT_STATUS.md](docs/CURRENT_STATUS.md); the baseline
[audit](docs/AUDIT.md) remains tied to the commit it names.

Runtime implementation currently stops at PR #250. PRs #251–#256 reconcile
current guides, Issues and additional source/load requirements; they do not add
runtime behavior.

Implemented internal components:

- PR #246 preserves configured Emmy diagnostic codes, exact messages, severities
  and source ranges instead of filtering to four families;
- PR #247 provides one configured Main + Library + optional virtual Emmy owner,
  fixes Lua 5.1, initializes embedded STD once, and separates diagnostics-only
  collection from requested semantic collection;
- PR #248 provides `wow-service::addon_check`, a direct snapshot-to-result path
  that does not construct the old Reference/Rules/publication/graph/store stack;
- PR #249 distinguishes compiled Emmy provenance from compatibility-report
  evidence, so the direct path need not fabricate report hashes;
- PR #250 requires exact Main snapshot identity for configured-owner reuse.

These are internal components, not a usable public addon checker. The ordinary
Project path still has to preserve product channel, dependencies versus
`LoadWith`, exact selected order/execution context, implicit bindings,
Interface/security uncertainty, XML mappings and addon/callback context before
calling them. CLI wiring, WoW metadata/rule repairs, project-aware LSP reuse and
direct source index/search also remain open. No end-to-end addon or editor
workflow is marked observed.

The complete current checklist is
[queue item 1](docs/REFACTOR_QUEUE.md#1-first-usable-ordinary-addon-check); this
README is only a status/navigation summary.

## Current executable entry points

```sh
cargo run --locked -p wow-cli --bin wow -- --help
cargo run --locked -p wow-reference-builder -- --help
```

The existing generator's exact interface is documented in
[its README](apps/wow-reference-builder/README.md). The simple `wow check`, index
and search interfaces in [FRONTENDS.md](docs/FRONTENDS.md) are target behavior
until their ordinary inputs reach the delivered lower owners.

## Development documentation

- [Current status](docs/CURRENT_STATUS.md): implemented slices, open handoffs and
  verification level.
- [Architecture](docs/ARCHITECTURE.md): purpose, requirements and boundaries.
- [WoW linting behavior](docs/WOW_LINTER.md): language/game semantics.
- [Implementation](docs/IMPLEMENTATION.md): existing code to reuse and target
  producer-to-consumer paths.
- [Implementation trace register](docs/IMPLEMENTATION_TRACE_REGISTER.md):
  post-merge source traces, implementation evidence and unexecuted observations.
- [Donors](docs/DONORS.md): pinned modules, ideas and reuse boundaries.
- [Queue](docs/REFACTOR_QUEUE.md): the next coherent implementation slices.

Contributors start with [AGENTS.md](AGENTS.md). The full guide index is
[docs/README.md](docs/README.md). Historical platform specifications remain in
Git history, not as competing instructions in the working tree.

## License

Framework-owned code is MIT. Preserve third-party notices; Blizzard/Ketho input
material retains its own provenance and applicable terms.
