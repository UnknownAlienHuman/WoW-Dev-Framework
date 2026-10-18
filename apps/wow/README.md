# `wow` application

`wow-cli` builds the `wow` binary. The app parses explicit CLI or stdio protocol
input, calls `wow-service`, and renders the result. Keep domain algorithms and
storage in their current owners. The CLI is the primary complete local entry
point; LSP/MCP expose selected operations through existing adapters.

## Help and recipes

Build and print the current command list:

```sh
cargo build -p wow-cli --bin wow
cargo run -p wow-cli --bin wow -- --help
```

Help includes `status`, `check`, `graph`, `search`, `context`, `project`,
`native`, `source`, `overlay`, `lineage`, `lsp`, and `mcp` routes. Use the
matching recipe for its exact inputs: [local check](LOCAL_INPUT.md),
[explicit files](FILES_INPUT.md), [native API input](NATIVE_INPUT.md),
[graph build](GRAPH_BUILD.md), [search build/query](SEARCH_BUILD_QUERY.md), and
[context build](CONTEXT_BUILD.md). Subcommands provide their own `--help` where
implemented.

The current local `status`/`check` route requires `--config` with a materialized
`wow-service/local-project-input/1` package. A normal reusable project
configuration is missing and remains tracked by issue #200; do not present the
materialized input schema as that finished user configuration.

## LSP and MCP

`wow lsp --transport stdio --profile <profile> --host <explicit.json>` exposes
implemented private Lua diagnostics and selected same-document navigation over
one configured workspace. `wow mcp --transport stdio --profile
mcp-2025-11-25-stdio-v1` exposes the current MCP profile; a separate context
profile is shown by `wow --help`. See [LSP behavior](LSP.md). General Lua parsing,
analysis, and language-server features come from upstream EmmyLua; this app does
not reimplement those features. Advertise only operations implemented by the
selected profile.

Inputs are explicit and bounded. Source is read as data and never executed or
changed by analysis. Keep configuration, path validation, and protocol handling
in the app; do not move search, context construction, or other owner algorithms
here.
