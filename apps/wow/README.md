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

The executable currently parses directly into `LocalCommand`. Its check route
constructs `LocalProjectInput`, `LocalProjectBackend` and the generic `Service`,
which require Reference, Rules, analyzer-compatibility and Project publication
state even for type-only analysis. Empty rule selection currently expands to all
registered rule descriptors. This path is not the target owner and must not be
kept alive with fabricated empty proof objects.

The replacement follows the existing dependency direction:

- `wow-project` reuses bounded disk/Library reading, exact `ProjectInputFile`
  bytes, TOC/dependency resolution and XML maps, but exposes a small load target
  with flavor, optional Interface and load context instead of requiring a full
  `ProfileIdentity`/`ProjectConfiguration`;
- `wow-emmy` owns one configurable Main/Library/STD/virtual session and an
  internal compiled backend identity. Compatibility-report SHA values are not
  user or Service inputs;
- `wow-service` owns ordinary configuration, check orchestration, selected WoW
  rule dependencies and the diagnostic/limitation result;
- `wow-cli` owns argument precedence, cancellation, rendering and exit status.

Reference/Rules are created only for explicitly selected WoW checks. The direct
result carries source diagnostics and scoped limitations, uses
document-version-aware locations, and requires no compatibility/probe reports,
Project inventory/publication or internal IDs. Human output prints the actual
message/code/severity/location rather than component/capability identities and
JSON fragments.

CLI, LSP, and MCP must call this same configured owner. The exact replacement
boundary and owner split are recorded in
[the queue](../../docs/REFACTOR_QUEUE.md#source-traced-replacement-boundary).
These workflows are not all complete at this baseline.

Source is read as data and never executed or rewritten. General Lua language
features belong to upstream EmmyLua. The command dispatcher is in
[`src/main.rs`](src/main.rs); argument parsing and help are in
[`src/args.rs`](src/args.rs); current rendering is in
[`src/output.rs`](src/output.rs); LSP and MCP adapters are in
[`src/lsp.rs`](src/lsp.rs) and [`src/mcp.rs`](src/mcp.rs).
