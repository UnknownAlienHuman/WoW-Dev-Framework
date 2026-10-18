# `wow-reference-builder` contribution guidance

Follow repository [`AGENTS.md`](../../AGENTS.md),
[`docs/ARCHITECTURE.md`](../../docs/ARCHITECTURE.md),
[`docs/REFACTOR_QUEUE.md`](../../docs/REFACTOR_QUEUE.md), and shared
[`crates/AGENTS.md`](../../crates/AGENTS.md).

The target is supported native Ketho generation to an ordinary Lua library.
Current `src/lib.rs` still runs pack build/validation/materialization; #196
replaces that path using the [native implementation route](../../docs/IMPLEMENTATION.md#native-annotation-generation).
Reuse the existing driver and Reference/Annotation owners. Keep source revision,
TOC/flavor and output directory explicit; share one producer with local native
input. Do not duplicate extraction, parsing or Ketho normalization here.

Treat checked-out source and metadata as data. Do not execute repository/addon
scripts, Lua, hooks, or arbitrary commands. Keep reads and writes within
explicitly selected roots, preserve existing user files, and report real errors
for missing, malformed, or unsupported inputs. Bound files, bytes, and output.
Generated reference/index data is rebuildable from pinned source and user
configuration.

Do not add release signing, distribution, pack promotion, update/rollback,
installation, backup, or migration workflows as obligations for this local
builder. Keep output useful to the native Emmy/Ketho and local project paths.
Use focused checks for changed extraction or output behavior.
