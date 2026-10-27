# Addon and load context

Delivery state: [CURRENT_STATUS.md](../../docs/CURRENT_STATUS.md). Project owns
bounded source selection, exact bytes, source projection and only the load/context
facts needed by the active [queue slice](../../docs/REFACTOR_QUEUE.md#1-first-usable-ordinary-addon-check).

Reuse `disk`, `load`, `xml_lua.rs`, `xml_bindings.rs`, `analyzer.rs` and update
code. Slice 1A needs explicit TOC and selected physical Lua plus exact user-path
projection. Later slices add dependency/order context, mapped XML/chunk context,
then channel/triggers/bindings/secure special cases.

Retain exact source-backed unknowns and independent findings, but do not require
the complete Blizzard loading model before the first type-check command.
Publication, replay reconstruction, fingerprints and proof graph are never
prerequisites for ordinary addon checks.

See [Implementation](../../docs/IMPLEMENTATION.md#code-ownership) and
[WoW behavior](../../docs/WOW_LINTER.md).
