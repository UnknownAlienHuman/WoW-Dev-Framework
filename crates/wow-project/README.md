# Addon and load context

Reuse: `disk`, `load`, `xml_lua.rs`, `xml_bindings.rs`, `analyzer.rs` and update code.

Retain exact source, product channel, dependency/trigger selection, selected
order, execution/private-table context and mapped XML/binding units. Publication,
replay reconstruction, fingerprints and proof graph are not prerequisites for
ordinary addon checks. See queue item 1 for the complete current handoff.

See [current status](../../docs/CURRENT_STATUS.md),
[Implementation](../../docs/IMPLEMENTATION.md#code-ownership) and
[WoW behavior](../../docs/WOW_LINTER.md).
