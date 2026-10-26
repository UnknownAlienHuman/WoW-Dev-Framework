# `wow` application

`wow-cli` builds `wow`, the target offline entry point for Library generation,
addon checks from root/TOC/flavor/Library inputs, and source index/search. LSP and
MCP are target stdio adapters to the same Service operations. See
[current status](../../docs/CURRENT_STATUS.md),
[FRONTENDS](../../docs/FRONTENDS.md),
[Architecture](../../docs/ARCHITECTURE.md), and
[Implementation](../../docs/IMPLEMENTATION.md).

## Current command surface

Build and inspect the actual executable:

```sh
cargo build --locked -p wow-cli --bin wow
cargo run --locked -p wow-cli --bin wow -- --help
```

The ordinary `status` and `check` commands still use the transitional
`LocalCommand` route. They require internal configuration/Project identities;
`check` accepts generation/file/rule selectors, and LSP still selects an explicit
host/profile. These are not the target user setup. Consult `--help` for current
syntax until the replacement adapter lands.

## Lower replacement already delivered

Do not recreate these components in the application:

- `wow_emmy::ConfiguredAnalyzerSession` owns exact Main + Library + optional
  virtual input in one Lua 5.1 analysis, initializes embedded STD, accepts caller
  diagnostic configuration and separates diagnostics-only from semantic work;
- `wow_service::addon_check::check_addon` accepts exact already-acquired
  snapshots and returns source diagnostics plus scoped limitations without
  Reference, Rules, Project publication, graph, store or presentation objects;
- the diagnostic DTO retains upstream code, exact message, severity, path,
  content identity and byte span;
- compiled Emmy identity no longer requires fabricated compatibility-report
  hashes; owner reuse requires exact Main identity.

These slices were merged in PRs #246–#250. They passed their exact-head
repository/format/compile/Clippy gates but have not been observed through a real
addon or editor workflow.

## Remaining application handoff

The next application work is not another analyzer or Service wrapper. The complete
Project/Service checklist is
[queue item 1](../../docs/REFACTOR_QUEUE.md#1-first-usable-ordinary-addon-check);
this application summary must not narrow it. Application-owned work is:

1. parse ordinary CLI/config inputs and precedence;
2. request the validated Project input defined by queue item 1, including selected
   order/context and mapped XML/binding units rather than snapshot paths alone;
3. pass that input to `wow_service::addon_check::check_addon` without reconstructing
   load facts in the CLI;
4. render message/code/severity/location and scoped limitations in text/JSON;
5. apply the documented `0/1/2/64/130/4` exit policy.

The old executable path constructs `LocalProjectInput`, `LocalProjectBackend` and
the generic `Service`, which require Reference, Rules, analyzer-compatibility and
Project publication state even for type-only analysis. Empty rule selection also
expands to all registered descriptors. Do not keep this route alive with hidden
defaults or fabricated empty proof objects. Remove it with its last transitional
callers.

CLI, LSP and MCP must eventually call the same configured check/search/context
owners. The exact replacement boundary is in
[the queue](../../docs/REFACTOR_QUEUE.md#source-traced-replacement-boundary).

Source is read as data and never executed or rewritten. General Lua language
features belong to upstream EmmyLua. The command dispatcher is in
[`src/main.rs`](src/main.rs); arguments/help are in [`src/args.rs`](src/args.rs);
current rendering is in [`src/output.rs`](src/output.rs); LSP and MCP adapters are
in [`src/lsp.rs`](src/lsp.rs) and [`src/mcp.rs`](src/mcp.rs).
