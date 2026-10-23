# Direct WoW checks

Reuse: `engine.rs` and the API/secret local rule algorithms, within their actual
source and flow limits.

Current engine executes both providers before Service filters. Replace the
provider/capability/evidence machinery with selected direct checks.

See [Implementation](../../docs/IMPLEMENTATION.md#code-ownership),
[WoW behavior](../../docs/WOW_LINTER.md) and [Audit](../../docs/AUDIT.md).
