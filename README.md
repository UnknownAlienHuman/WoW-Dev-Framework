# WoW Dev Framework

Rust tooling for WoW addon development, built around **upstream EmmyLua** and a
native Rust port of **Ketho's annotation pipeline**.

The intended product has three workflows:

1. generate an ordinary `.lua` annotation Library for selected Blizzard
   source/ref/flavor;
2. check an addon with that Library and report useful diagnostics at its source;
3. index Blizzard/addon source and search text, symbols and useful TOC/XML
   relationships with readable excerpts.

The repository is undergoing a major simplification. Useful analysis and
annotation components exist, but the public producer/reader/CLI handoffs are not
yet complete. Do not treat compilation or the presence of internal graph/search
commands as a finished user workflow.

## Current source of truth

- [Architecture](docs/ARCHITECTURE.md) — product manifesto and excluded scope.
- [Audit](docs/AUDIT.md) — complete source/architecture basis for the rewrite.
- [Implementation](docs/IMPLEMENTATION.md) — target owners and replacement routes.
- [Queue](docs/REFACTOR_QUEUE.md) — ordered work and current Issues.
- [Completion](docs/PROJECT_COMPLETION_MATRIX.md) — exercised user behavior only.

## Build the current workspace

```sh
cargo build --locked -p wow-cli --bin wow
cargo run --locked -p wow-cli --bin wow -- --help
```

The current executable still exposes many internal materialized inputs, IDs,
profiles and lifecycle commands from the superseded platform. The rewrite is
replacing them with ordinary root/TOC/flavor/Library/database inputs. Current
internal recipes are documented only for development; they are not the target UX.

Generated annotations are ordinary Lua analysis files. Source is parsed, never
executed. User source and configuration must not be rewritten or deleted by
index/cache cleanup.

## Scope discipline

Keep EmmyLua's engine, Ketho's useful behavior and direct SQLite/FTS queries.
Remove redundant representations and platforms that do not serve the three
workflows: internal legacy/migrations, cryptographic proof/certification,
governed packs, lineage/promotion, Wasm plugins, generic backup/retention,
daemon/HTTP, provider frameworks and custom installer/updater/release systems.

Contributors start with [AGENTS.md](AGENTS.md).

## License

Framework-owned code is MIT. Preserve required third-party notices and the
licenses that apply to Blizzard and other selected input data.
