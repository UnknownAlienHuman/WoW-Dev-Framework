# `wow` application

`wow-cli` is the target offline entry point. Delivery state lives in
[current status](../../docs/CURRENT_STATUS.md); interface contracts are in
[FRONTENDS.md](../../docs/FRONTENDS.md).

Current `status`/`check` still use the transitional `LocalCommand` route and are
not the target public setup.

Slice 1A is exactly:

```text
root + explicit TOC + flavor=retail + Library roots
  -> Project AddonLoadTarget + physical_lua plan
  -> standalone scope-aware limitations + exact snapshots
  -> existing wow_service::addon_check
  -> text/JSON and exit status
```

The application owns config/CLI precedence, rendering and exit status. It does
not fabricate profiles/Reference/Rules/publication objects, parse XML/load facts
itself or wrap the direct result in component/capability models. Result paths are
canonical relative to the configured root; text may display the joined path while
JSON preserves the relative identity.

Channel, dependencies, full XML, `LoadWith`, bindings, secure context and WoW
rules are later slices. Source is read as data and never executed or rewritten.
