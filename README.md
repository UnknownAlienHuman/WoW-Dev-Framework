# WoW Dev Framework

A World of Warcraft addon linter written in Rust on top of
[EmmyLua](https://github.com/EmmyLuaLs/emmylua-analyzer-rust), Blizzard API/UI source
and selected annotation semantics from [Ketho](https://github.com/Ketho/vscode-wow-api).

The target workflows are ordinary Lua Library generation, configured addon
checking, and selected-source search with readable located context. Checking must
not require a search index, editor extension or publication/proof platform.
Analyzed Lua is read as data, never executed or rewritten.

## Status

**The ordinary addon checker is not implemented yet.** Useful lower components
exist; the current CLI still exposes transitional routes. See
[current status](docs/CURRENT_STATUS.md) for delivery, not planned command examples.
No ordinary addon/editor/source-search workflow is recorded as executed.

## Start development

Read [AGENTS.md](AGENTS.md), then one row of the
[implementation work register](docs/REFACTOR_QUEUE.md). Each row identifies the
owner, required inputs, consuming operation, completion boundary and cleanup.
The [repository map](docs/REPOSITORY_MAP.md) supplies code entry points;
[CONTRIBUTING.md](CONTRIBUTING.md) supplies checks and update destinations.

Start with the physical-Lua checker. Do not rebuild the retired platform or run
another general preparation campaign before a bounded implementation change.
Use the [documentation index](docs/README.md) for the relevant contract and
historical evidence, not a mandatory reading list.

## Current executables

```sh
cargo run --locked -p wow-cli --bin wow -- --help
cargo run --locked -p wow-reference-builder -- --help
```

The [ordinary CLI contract](docs/FRONTENDS.md) remains planned. The existing
[generator command and limitations](apps/wow-reference-builder/README.md) are
separate from that unfinished checker. Do not confuse successful compilation
with a connected user workflow.

## License

Framework-owned code is MIT. Preserve third-party notices and the provenance and
terms of Blizzard-derived input data.
