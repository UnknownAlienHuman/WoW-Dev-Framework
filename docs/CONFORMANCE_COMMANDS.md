# Development checks and commands

This repository supports ordinary Cargo development checks. The selected Issue
and [refactoring queue](REFACTOR_QUEUE.md) define the work; this page does not
create another acceptance matrix. Cargo.toml is authoritative for package names.
The CLI package is `wow-cli` and its binary is `wow`; repository tooling is
`xtask`.

## Useful commands

```sh
cargo fmt --all -- --check
cargo clippy --locked -p <package> --all-targets --all-features -- -D warnings
cargo test --locked -p <package> <focused-test-filter>
cargo xtask check
cargo run --locked -p wow-cli --bin wow -- <command>
```

Use `cargo test --locked -p <package>` when a package-level test run is needed.
Prefer the smallest existing test or manual reproduction that reaches the changed
behavior. Clippy compiles the selected targets; do not run `cargo check` again on
them afterward. Do not run the whole workspace or source corpus without a
specific risk or claim that needs it.

`cargo xtask check` scans tracked and untracked distributable files for forbidden
interpreter assets/invocations and invalid or duplicate-key JSON, then checks the
skill copies. It does not validate product schemas, contracts, checksum
manifests, compatibility, or release readiness. See [xtask documentation](../tools/xtask/README.md).

## Workflow-specific validation

- **EmmyLua:** exercise the changed adapter or analysis path with a focused Lua
  input and verify the relevant diagnostics or source locations.
- **Ketho annotation generation:** use the native driver and a representative
  source/annotation probe when the changed behavior depends on Ketho semantics.
  Record the selected source revision when applicable.
- **Blizzard UI repository search:** use the selected source manifest and a
  focused query whose result can be checked against the corresponding file and
  source location.
- **CLI, LSP, or MCP:** exercise the affected command or protocol operation when
  its behavior changes; retain standard protocol behavior and advertise only
  implemented capabilities.

Report the command and observed result. These checks demonstrate only the paths
actually exercised; no frozen release, signing, backup, provider, or compatibility
campaign is required for ordinary development changes.
