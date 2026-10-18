# Workspace and build guidance

## Product workflows

The current product scope is upstream EmmyLua, Ketho's annotation pipeline
ported to Rust, practical editor-independent use, and Blizzard UI repository
search. See [ARCHITECTURE.md](ARCHITECTURE.md) for ownership and boundaries.
The CLI is the primary complete entry point; stdio LSP/MCP can expose useful
operations through the existing adapters.

## Workspace and package names

The root `Cargo.toml` is the source of truth for active members, dependencies,
and profiles. Preserve its normal dependency direction: owner crates hold
algorithms and data operations; `wow-service` composes multi-owner use cases;
`wow-cli` provides the `wow` binary; `xtask` provides repository/source
maintenance commands. Do not create a crate or feature layer solely to preserve
the earlier E0–E7 package map.

Use the current workspace toolchain and lockfile. Do not pin a toolchain patch or
add release profiles, protocol profiles, or Cargo feature matrices without a
specific current requirement. Build only the affected package and consumers
while developing; Clippy is sufficient compilation evidence for those targets.

## Common commands

```sh
cargo fmt --all -- --check
cargo clippy --locked -p <package> --all-targets --all-features -- -D warnings
cargo test --locked -p <package> <focused-test-filter>
cargo run --locked -p wow-cli --bin wow -- <command>
cargo xtask check
```

Run one existing focused test or bounded reproduction after the changed behavior
works. Do not repeat `cargo check` after Clippy compiled the same targets. Expand
to a broader build or test run only to resolve a concrete remaining risk. The
repository check covers file hygiene, native-only restrictions, JSON syntax and
duplicate keys, plus synchronized skill copies; it is not product acceptance.

## Inputs and generated data

The user selects the addon or Blizzard source root and flavor. Respect configured
paths, report ambiguous TOCs, and never execute indexed source. Treat indexes,
annotations, and intermediate outputs as rebuildable derived data; preserve user
source and configuration. Rebuild a changed internal output from available
inputs instead of keeping obsolete cache readers or migration machinery.

When semantics depend on a moving Blizzard/Ketho source, resolve its revision
once for the task and record the revision and flavor. Generic code, storage, and
documentation changes do not require a fresh source corpus. Use bounded inputs,
ordinary transactional SQLite operations, and existing materializers. Report the
behavior exercised, checks run, and remaining unknowns. Retired
signing, release, installer, backup, provider, and full-corpus campaigns do not
gate a selected developer task.
