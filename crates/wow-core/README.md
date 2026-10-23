# Shared primitives

Reuse: `library_limits.rs` and useful source/path/error primitives.

Universal identity/result/capability models still have legacy callers; remove
them with those callers rather than moving them into a new shared framework.

See [Implementation](../../docs/IMPLEMENTATION.md#code-ownership),
[WoW behavior](../../docs/WOW_LINTER.md) and [Audit](../../docs/AUDIT.md).
