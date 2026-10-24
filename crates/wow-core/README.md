# Shared primitives

Reuse: `library_limits.rs` and useful source/path/error primitives.

Universal identity/result/capability models still have legacy callers; remove
them with those callers rather than moving them into a new shared framework.

See [current status](../../docs/CURRENT_STATUS.md),
[Implementation](../../docs/IMPLEMENTATION.md#code-ownership) and
[WoW behavior](../../docs/WOW_LINTER.md).
