# A WoW linter built on EmmyLua

WoW Dev Framework helps authors and coding agents build correct World of Warcraft
addons. Its main product is a **WoW-aware Lua linter**: EmmyLua checks the language
and types; Blizzard source and Ketho's annotation work supply the game contracts;
a small WoW layer checks addon loading and constraints that ordinary Lua analysis
cannot express.

## Required result

An author selects an addon, an exact client target (flavor, product channel and
optional Interface compatibility target) and a compatible annotation Library.
The tool reads its TOC, Lua, XML and selected binding callbacks, then reports
useful diagnostics at the actual source locations. Messages explain the problem
and retain the upstream code and severity. The same project analysis serves CLI,
editor and agent requests.

The product must cover:

- Lua syntax and type checking against the selected Blizzard API, including
  parameters, multiple returns, nilability, callbacks, widgets and mixins;
- TOC/XML/binding selection, dependency declarations, reverse load triggers,
  ordered loading and embedded/callback Lua, with precise locations and explicit
  unknowns;
- API availability in the selected client and execution context, plus supported
  event, callback and restriction checks;
- changes to source, Library or load selection without stale diagnostics;
- generation of ordinary `.lua` annotation files from selected Blizzard source;
- source lookup for explaining a diagnostic: declarations, readable excerpts and
  useful TOC/XML relations. Persistent search is auxiliary and must not delay or
  become a prerequisite for addon checking.

[WOW_LINTER.md](WOW_LINTER.md) defines the domain behavior and its limits.
[IMPLEMENTATION.md](IMPLEMENTATION.md) defines how to build it.

## Design principles

**EmmyLua is the language engine.** Reuse its public parser, semantic model,
configuration and diagnostics. Do not implement another Lua type checker. Reuse
Ketho's useful annotation semantics through the existing Rust port; extend them
where Blizzard metadata needed by the linter is not represented.

**Keep a direct path from input to useful output.** One configured project,
one analysis session, plain typed API/load facts and plain findings. A helper or
crate exists because actual callers need it. The current crate count is not a
target architecture; wrappers, duplicated models and generic platforms are not
preserved merely because they already exist.

**Make correctness local and explainable.** A rule names the source fact and
program operation behind its finding. Unknown data is not an absent API. A
partial library cannot make a check look clean. Static analysis does not certify
combat, taint, secret-value safety or arbitrary dynamic Lua execution.

**Reuse work without inventing infrastructure.** Load declarations once per
session, share parsed data, update changed files and query a normal SQLite index
when needed. Prefer a correct conservative rebuild to stale incremental results.
Bounds and cancellation protect responsiveness; no speedup is claimed without
measurement of the actual user operation.

**Replace, then remove.** Keep useful code and domain knowledge; replace broken
handoffs and delete their displaced implementations and instructions. Derived
libraries and indexes can be rebuilt. User source, configuration and unpublished
work remain intact. Git keeps history; old specifications are not current scope.

## Boundaries

The product runs locally in Rust, without a Lua, Node, Python or VS Code runtime.
Read source as data; do not execute it. CLI is the complete entry point; focused
stdio LSP/MCP adapters reuse its operations. General Lua editor features remain
with upstream EmmyLua.

No certification or cryptographic result platform, schema-version ladder,
Reference Pack governance, generic graph/recognizer/plugin framework, historical
lineage/fingerprints, backup/retention administration, daemon/HTTP service,
provider framework or installer/updater is required. Ordinary source revisions,
cache invalidation, file bounds and database transactions remain useful.
