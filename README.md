# WoW Dev Framework

Rust tooling for WoW addon development, built around **EmmyLua** and a native
port of **Ketho's annotation pipeline**. The goal is to generate usable WoW
annotation libraries, check addon code, and search Blizzard UI source without
requiring VS Code.

## Product scope

- Generate `.lua` annotation libraries from selected Blizzard source/flavor.
- Analyze an addon through upstream EmmyLua, with useful WoW diagnostics.
- Search Blizzard UI and addon code; return symbols, text, source locations and
  concise context, including relevant TOC/XML relationships.
- Provide a usable CLI and focused stdio LSP/MCP integrations. General Lua editor
  features use EmmyLua's existing engine/server.

[Architecture](docs/ARCHITECTURE.md) is the current scope.
[Refactoring queue](docs/REFACTOR_QUEUE.md) contains concrete work in order.
[Current state](docs/PROJECT_COMPLETION_MATRIX.md) separates existing code from
remaining user workflows. This is an active refactor, not a finished release.

## Build and existing entry points

```sh
cargo build --locked -p wow-cli --bin wow
cargo run --locked -p wow-cli --bin wow -- --help
```

The existing executable has status/check, native project, search, context, LSP
and MCP paths, but several still require complex materialized inputs. Replacing
those inputs with normal root/TOC/flavor configuration is queued work; examples
below document current commands, not the finished UX.

- [Native annotation development driver](docs/KETHO_RUST_PORT.md#native-source-to-library-flow)
- [Current status/check input](apps/wow/LOCAL_INPUT.md)
- [Current Search commands](apps/wow/SEARCH_BUILD_QUERY.md)
- [Current LSP](apps/wow/LSP.md) and [MCP](docs/FRONTENDS.md)

Generated libraries are ordinary analysis files. Point EmmyLua/LuaLS at their
directory; no extension activation or editor-settings mutation is required.
Source is parsed, never executed during indexing.

## Keep the implementation small

One upstream parser/analyzer, one native annotation pipeline, one current local
index format. Derived indexes and annotations can be rebuilt; user source and
configuration are preserved. Remove redundant code and feature-only tests/docs
with each refactor. The old governance, cryptographic envelope, compatibility,
provider, Wasm plugin and custom release/updater plans are retired.

Contributors start with [AGENTS.md](AGENTS.md), the selected Issue and its few
named implementation/donor paths. See [docs/README.md](docs/README.md) for routes.

## License

Framework-owned code is MIT. Keep third-party notices for ported code and the
source/artifact licenses that apply to Blizzard and other input data.
