# WoW Dev Framework

A **World of Warcraft addon linter** written in Rust on top of
[EmmyLua](https://github.com/EmmyLuaLs/emmylua-analyzer-rust), Blizzard API/UI
source and the useful annotation semantics from
[Ketho](https://github.com/Ketho/vscode-wow-api).

The product targets three workflows:

1. generate an ordinary `.lua` Library from a selected Blizzard source revision;
2. check addon Lua with EmmyLua plus narrowly selected WoW-aware checks;
3. index selected Blizzard/addon source and return readable located search and
   navigation results.

Checking does not require a search index, editor extension, remote service or
Project publication/proof identities. Analyzed Lua is read as data and never
executed or rewritten.

## Status

Development is focused on connecting the first ordinary addon checker.

Read [current status](docs/CURRENT_STATUS.md) for existing code and open gaps,
then [the queue](docs/REFACTOR_QUEUE.md) for the next implementation step.
No ordinary addon, editor or source-search workflow is recorded as executed.
Historical audits are linked from the
[documentation work register](docs/AUDIT_REGISTRY.md).

## Current executables

```sh
cargo run --locked -p wow-cli --bin wow -- --help
cargo run --locked -p wow-reference-builder -- --help
```

The `wow` executable exposes transitional routes. The planned `wow check`
contract is described in [FRONTENDS.md](docs/FRONTENDS.md); direct source index,
search and context work is queued in
[the implementation order](docs/REFACTOR_QUEUE.md#4-direct-source-index-search-and-context).
None of these planned interfaces is claimed as delivered.

## Documentation

- [Architecture](docs/ARCHITECTURE.md): product and scope.
- [WoW linter behavior](docs/WOW_LINTER.md): language and game semantics.
- [Implementation](docs/IMPLEMENTATION.md): owner boundaries and direct paths.
- [Queue](docs/REFACTOR_QUEUE.md): ordered vertical slices.
- [Current status](docs/CURRENT_STATUS.md): delivered, open and observed state.
- [Trace register](docs/IMPLEMENTATION_TRACE_REGISTER.md): evidence and decisions.
- [Donors](docs/DONORS.md): pinned sources and reuse boundaries.
- [Documentation index](docs/README.md): remaining focused guides.

Contributors start with [AGENTS.md](AGENTS.md). Historical platform
specifications remain in Git history, not as competing instructions.

## License

Framework-owned code is MIT. Preserve third-party notices and the provenance and
terms of Blizzard-derived input data.
