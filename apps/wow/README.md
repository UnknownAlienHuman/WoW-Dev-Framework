# `wow` application

`wow-cli` builds `wow`, the target offline entry point for Library generation,
addon checks from root/TOC/flavor/Library inputs, and source search. LSP and MCP
are target stdio adapters to the same configured operations. See
[FRONTENDS](../../docs/FRONTENDS.md), [Architecture](../../docs/ARCHITECTURE.md),
and [Implementation](../../docs/IMPLEMENTATION.md).

## Current state at `c36a058`

Build and inspect the actual current command surface:

```sh
cargo build --locked -p wow-cli --bin wow
cargo run --locked -p wow-cli --bin wow -- --help
```

The ordinary `status` and `check` commands still require `--config`; `check`
also requires an internal Project ID and accepts generation/file/rule selectors.
LSP requires an explicit host and profile. These command forms are transitional;
they are not instructions for the target user setup. Consult `--help` for
available current syntax.

The target check separates JSON findings from incompleteness, uses
document-version-aware locations, and requires no compatibility/probe reports or
internal IDs. CLI, LSP, and MCP must call the same configured analysis owner.
These workflows are not all complete at this baseline.

Source is read as data and never executed or rewritten. General Lua language
features belong to upstream EmmyLua. The command dispatcher is in
[`src/main.rs`](src/main.rs); argument parsing and help are in
[`src/args.rs`](src/args.rs); LSP and MCP adapters are in
[`src/lsp.rs`](src/lsp.rs) and [`src/mcp.rs`](src/mcp.rs).
