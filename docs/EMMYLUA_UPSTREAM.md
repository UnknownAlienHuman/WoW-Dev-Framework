# EmmyLua integration guide

This guide records how the framework should reuse EmmyLua and what the current integration does. It is subordinate to [Architecture](ARCHITECTURE.md) and the [implementation route](IMPLEMENTATION.md#addon-configuration-and-emmy-analysis). For framework diagnostic behavior, see [WOW_LINTER.md](WOW_LINTER.md).

## Pinned donor

The workspace pins `emmylua_code_analysis` and `emmylua_parser` in `Cargo.toml`
and `Cargo.lock` to revision `aaaca68425d9362876228649b0b8d92f07654daa` of
[EmmyLua Analyzer Rust](https://github.com/EmmyLuaLs/emmylua-analyzer-rust).
Read donor behavior at that revision, not a moving default branch. Relevant
primary sources are:

- [`EmmyLuaAnalysis` public API](https://github.com/EmmyLuaLs/emmylua-analyzer-rust/blob/aaaca68425d9362876228649b0b8d92f07654daa/crates/emmylua_code_analysis/src/lib.rs)
- [embedded resource loader](https://github.com/EmmyLuaLs/emmylua-analyzer-rust/blob/aaaca68425d9362876228649b0b8d92f07654daa/crates/emmylua_code_analysis/src/resources/mod.rs)
- [runtime configuration and parser dialect](https://github.com/EmmyLuaLs/emmylua-analyzer-rust/blob/aaaca68425d9362876228649b0b8d92f07654daa/crates/emmylua_code_analysis/src/config/configs/runtime.rs)
- [diagnostic configuration](https://github.com/EmmyLuaLs/emmylua-analyzer-rust/blob/aaaca68425d9362876228649b0b8d92f07654daa/crates/emmylua_code_analysis/src/config/configs/diagnostics.rs)
- [diagnostic collection](https://github.com/EmmyLuaLs/emmylua-analyzer-rust/blob/aaaca68425d9362876228649b0b8d92f07654daa/crates/emmylua_code_analysis/src/diagnostic/lua_diagnostic.rs)
- [workspace configuration](https://github.com/EmmyLuaLs/emmylua-analyzer-rust/blob/aaaca68425d9362876228649b0b8d92f07654daa/crates/emmylua_code_analysis/src/config/configs/workspace.rs)
- [compilation indexing](https://github.com/EmmyLuaLs/emmylua-analyzer-rust/blob/aaaca68425d9362876228649b0b8d92f07654daa/crates/emmylua_code_analysis/src/compilation/mod.rs)
- [index removal](https://github.com/EmmyLuaLs/emmylua-analyzer-rust/blob/aaaca68425d9362876228649b0b8d92f07654daa/crates/emmylua_code_analysis/src/db_index/mod.rs)

## One analysis, three source roles

Use one `EmmyLuaAnalysis` for the selected addon Main files, selected Library
declarations, and upstream STD declarations. Configure it before loading any
source. Keep the roles explicit:

- **Main** is the addon source to diagnose and analyze.
- **Library** supplies selected addon and Blizzard API declarations used to
  resolve Main. Check Library health separately; do not report Library errors
  as if they were located in Main.
- **STD** supplies Lua language declarations. It is upstream-provided input,
  not user source, a WoW API catalog, or a source location to project into Main.

For ordinary addon analysis, set the runtime to Lua 5.1, enable diagnostics,
then call `init_std_lib(None)` once before registering Main and Library inputs.
At the pinned revision, this loads embedded resources without extracting them
to disk. `EmmyLuaAnalysis::new()` alone loads no standard declarations, and
adding a workspace root does not load files. Do not add a resource installer,
stdlib catalog, or separate analysis engine.

The pinned runtime configuration has Lua versions and generic nonstandard syntax
symbols, but no WoW runtime preset or standard-module selection switch.
`workspace.ignore_dir` and `ignore_globs` apply to collected workspace inputs;
they do not filter `init_std_lib`. Do not imply that changing those settings
removes an embedded STD declaration.

**STD typing is not WoW runtime availability.** Lua standard declarations can
improve language typing, but their presence does not prove that the game exposes
the corresponding host function or module. Conversely, do not claim that all
of `io`, `os`, `package`, or another library is unavailable without evidence
from selected WoW source. Keep any availability checking narrow: base it on
resolved original symbols and selected WoW data, and represent uncertainty
when that evidence is absent. Do not build a second general Lua analyzer or
invent a whole-runtime allowlist from the STD files.

The pinned default runtime is `LuaLatest`; always set Lua 5.1 explicitly for
the target baseline. EmmyLua also exposes configured dialect extensions, but
the framework should enable one only when selected source establishes the need.
Lua 5.1 parsing does not establish WoW API or runtime behavior.

## Configuration belongs to the shared owner

Resolve the supported upstream diagnostic settings before creating the analysis
and reuse them on refresh. The application owns target selection, WoW checks
and the exit threshold; Emmy owns language diagnostics and their configuration.
Keep Lua 5.1 explicit for this target. A configuration change that affects
analysis must invalidate the corresponding results, including editor caches.

Currently `syntax/analyzer.rs::build_registered`, the shared constructor in
`references.rs`, and `flow.rs::build_analysis` each create `Emmyrc::default`.
The first two enable diagnostics and the last disables them. This is an
implementation limitation, not configurable behavior. Pass the resolved
configuration through the shared session instead of introducing a separate
settings engine or another copy of the analyzer. Preserve upstream diagnostic
directives where the pinned analyzer supports them.

## Diagnostics and source locations

Use upstream diagnostics on Main while the same analysis has Library and STD
context. Preserve each selected upstream diagnostic's code, message, severity,
and exact source range through the Project, Service, CLI, and LSP output paths.
Keep UTF-8 half-open spans and convert coordinates only at the protocol edge.
Map diagnostics for virtual XML-derived Lua back through the existing source
mapping; never emit a synthetic STD or Library path as an addon location.

Use upstream diagnostic configuration to disable a diagnostic only when the
selected WoW environment gives a concrete reason. Do not gate all findings on
a framework allowlist of approved codes, replace upstream messages with generic
text, or turn every finding into an error. Bound and safely render messages as
display data. Missing input, unhealthy Library analysis, failed work, and
cancellation must not appear as a clean diagnostic result.

Diagnostics are static evidence. They do not prove runtime behavior, combat
safety, taint safety, or that a selected API is available in every client
flavor.

## Updates and invalidation

The public pinned bulk update API changes supplied VFS files, removes indexes
for touched file IDs, then indexes the supplied replacement IDs. It does not
automatically reanalyze every unchanged file that consumed a deleted global or
type. `require()` ordering is not a complete dependency invalidation graph.

Until a narrower update demonstrably covers semantic consumers, apply the file
delta and conservatively rebuild semantic indexes from retained VFS trees.
Reuse unchanged syntax trees where possible, but do not describe that as reuse
of semantic facts or avoided indexing. Publish a new result only after the
update succeeds; on failure or cancellation retain the previous immutable
result and discard stale candidate findings. The current update implementation is in [`session.rs`](../crates/wow-emmy/src/session.rs).

Cancellation is cooperative around framework work. The pinned diagnostic API
checks its cancellation token before entering synchronous diagnostic analysis;
it does not preempt an in-progress native analysis call. Check cancellation
before and after such calls, bound the work, and ensure a cancelled result cannot
be published as complete.

## Editor boundary

Upstream `emmylua_ls` owns general Lua language intelligence such as hover,
completion, definition, references, rename, signature help, symbols, and its
own general diagnostics. Its handler modules are private implementation, not
an importable SDK. Do not copy the complete server handler stack into this
framework.

The framework's focused stdio LSP and MCP interfaces should serve the same WoW
project workflows as the CLI, including useful project diagnostics and source
navigation. They are not substitutes for EmmyLua's general-purpose editor
server. Keep framework-specific operations limited to behavior that requires
selected WoW project inputs or mappings.

## Current state at `c36a058`

These are observed gaps, not claims of completion:

- [`syntax/analyzer.rs`](../crates/wow-emmy/src/syntax/analyzer.rs) sets Lua 5.1
  but does not call `init_std_lib`; the shared Main/Library constructor in
  [`references.rs`](../crates/wow-emmy/src/references.rs) also omits it.
- The syntax adapter accepts only four diagnostic codes, drops upstream
  messages, and fixes normalized severity to Error. The LSP projection sends
  constant generic wording. See [`syntax.rs`](../crates/wow-emmy/src/syntax.rs)
  and [`diagnostics.rs`](../crates/wow-service/src/frontend/lsp/diagnostics.rs).
- The overlay diagnostic input contains one document, not the full project
  Main and Library context. See [`overlay/state.rs`](../crates/wow-project/src/overlay/state.rs).
- The shared Main/Library session exists for its current analysis route, while
  other syntax and flow constructors still create their own analysis. See
  [`session.rs`](../crates/wow-emmy/src/session.rs),
  [`flow.rs`](../crates/wow-emmy/src/flow.rs), and
  [`IMPLEMENTATION.md`](IMPLEMENTATION.md#addon-configuration-and-emmy-analysis).
- File updates conservatively clear and rebuild semantic indexes; upstream
  touched-ID updates alone do not establish cross-file invalidation parity.
