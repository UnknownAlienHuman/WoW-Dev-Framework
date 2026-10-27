# `wow` application

`wow-cli` builds the target offline entry point for addon checking and later
source index/search/context workflows. Delivery state lives only in
[current status](../../docs/CURRENT_STATUS.md); target interfaces are in
[FRONTENDS.md](../../docs/FRONTENDS.md).

## Current executable

```sh
cargo build --locked -p wow-cli --bin wow
cargo run --locked -p wow-cli --bin wow -- --help
```

Current `status`/`check` still use the transitional `LocalCommand` route and
internal Project identities. They are not the target public setup.

## Replacement boundary

Do not create another analyzer or Service wrapper. Reuse:

- `wow_emmy::ConfiguredAnalyzerSession` for exact Main, Library and optional
  virtual input;
- `wow_service::addon_check::check_addon` for the direct result;
- Project-owned bounded source selection and exact path projection.

The first application slice is deliberately small:

```text
root + explicit TOC + flavor + Library
  -> selected physical Lua
  -> existing direct check
  -> text/JSON diagnostics, relevant limitations and exit status
```

It does not wait for product channel, `LoadWith`, implicit bindings, every XML
handler or WoW rules. Later slices add those contexts through Project.

Application responsibilities are config/CLI precedence, calling the direct
owners, rendering the same typed result and applying exit status. It must not
reconstruct load facts, fabricate Reference/Rules/publication objects or wrap the
direct result in `LocalOperationResult`/component/capability models.

Source is read as data and never executed or rewritten. General Lua editor
features remain upstream EmmyLua.
