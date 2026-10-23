# WoW Dev Framework

A **World of Warcraft addon linter**, written in Rust on top of
[EmmyLua](https://github.com/EmmyLuaLs/emmylua-analyzer-rust), Blizzard's API/UI
source and the useful annotation semantics from
[Ketho](https://github.com/Ketho/vscode-wow-api).

The goal is to catch Lua/type errors and WoW-specific mistakes while an addon is
being written: API calls, optional/multiple returns, callbacks, widgets, TOC/XML
loading and supported restrictions. Generated libraries remain ordinary `.lua`
files. Source lookup helps explain a finding without requiring a separate
platform before the linter can run.

## Current status

The repository is being simplified. At reviewed baseline `c36a058`:

- a public native annotation generator and a shared Main+Library Emmy session
  exist;
- ordinary addon setup, complete diagnostic transport and editor reuse still need
  work; the current adapter drops most Emmy diagnostics and their messages;
- Blizzard metadata needed by the linter is only partly modeled. In particular,
  callable-level `Environment` and `MayReturnNothing` need attention;
- persisted source search still uses the old replay/storage architecture.

These are implemented components, not a finished addon checker. The
[verified audit](docs/AUDIT.md) separates inspected code, the supplied Claude
review and behavior that has not been run. The [review register](docs/AUDIT_REGISTRY.md)
accounts for every old document, its useful ideas, code evidence and open questions.

## Current executable entry points

```sh
cargo run --locked -p wow-cli --bin wow -- --help
cargo run --locked -p wow-reference-builder -- --help
```

The existing generator's exact interface is documented in
[its README](apps/wow-reference-builder/README.md). The planned simple `wow check`
and `wow library build` interfaces are described in
[FRONTENDS.md](docs/FRONTENDS.md); their examples are target behavior, not commands
already delivered by this documentation change.

## Development documentation

- [Architecture](docs/ARCHITECTURE.md): purpose, requirements and boundaries.
- [WoW linting behavior](docs/WOW_LINTER.md): the actual language/game semantics.
- [Implementation](docs/IMPLEMENTATION.md): existing code to reuse and replacement paths.
- [Donors](docs/DONORS.md): specific modules, ideas and reuse boundaries.
- [Queue](docs/REFACTOR_QUEUE.md): the next useful implementation slices.

Contributors start with [AGENTS.md](AGENTS.md). The full guide index is
[docs/README.md](docs/README.md). Historical platform specifications are retained
in Git history, not as competing instructions in the working tree.

## License

Framework-owned code is MIT. Preserve third-party notices; Blizzard/Ketho input
material retains its own provenance and applicable terms.
