# WoW Dev Framework

A **World of Warcraft addon linter** written in Rust on top of
[EmmyLua](https://github.com/EmmyLuaLs/emmylua-analyzer-rust), Blizzard API/UI
source and the useful annotation semantics from
[Ketho](https://github.com/Ketho/vscode-wow-api).

The product has three workflows:

1. generate an ordinary `.lua` Library from a selected Blizzard source revision;
2. check addon Lua with EmmyLua plus narrowly selected WoW-aware checks;
3. index selected Blizzard/addon source and return readable located search and
   navigation results.

Checking does not require a search index, editor extension, remote service or
Project publication/proof identities. Analyzed Lua is read as data and never
executed or rewritten.

## Status

The 329-document baseline reconciliation is complete. Its evidence remains in
[the baseline audit](docs/AUDIT.md) and
[baseline register](docs/AUDIT_REGISTRY.md).

[Current status](docs/CURRENT_STATUS.md) is the **only live implementation
checkpoint**. The [trace register](docs/IMPLEMENTATION_TRACE_REGISTER.md) records
source decisions and exact observations. README files and contributor summaries
must not copy a moving list of delivered PRs or narrow the active queue.

No ordinary end-to-end addon, editor or source-search workflow is currently
marked observed.

## Current executables

```sh
cargo run --locked -p wow-cli --bin wow -- --help
cargo run --locked -p wow-reference-builder -- --help
```

The current `wow` command still exposes transitional internal routes. Target
`wow check`, index, search and context shapes are documented in
[FRONTENDS.md](docs/FRONTENDS.md) and are not claims of delivered commands.

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
