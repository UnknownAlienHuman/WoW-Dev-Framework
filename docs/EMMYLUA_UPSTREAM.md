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

For ordinary addon analysis, set the runtime to Lua 5.1, enable diagnostics by default,
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

The target `wowdev.json` supplies optional `emmy_diagnostics`, directly decoded
as the pinned [`EmmyrcDiagnostic`](https://github.com/EmmyLuaLs/emmylua-analyzer-rust/blob/aaaca68425d9362876228649b0b8d92f07654daa/crates/emmylua_code_analysis/src/config/configs/diagnostics.rs):
`enable`, `disable`, `enables`, `globals`, `globalsRegex`, `severity` and
`diagnosticInterval`. Missing fields retain upstream defaults; the interval is
an editor scheduling value and has no effect on a one-shot check. Invalid typed
values are configuration errors. Start from `Emmyrc::default`, replace only its
diagnostics and set runtime Lua51; apply that configuration to the shared owner.
This does not load another workspace or silently discover `.emmyrc` files.
The upstream [`load_configs`](https://github.com/EmmyLuaLs/emmylua-analyzer-rust/blob/aaaca68425d9362876228649b0b8d92f07654daa/crates/emmylua_code_analysis/src/config/config_loader.rs)
logs and skips invalid files or returns defaults after a typed decoding error;
that behavior would hide an explicit configuration failure in this operation.

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

The pinned upstream's `lsp_types::Range` is not already in the editor's negotiated
encoding. [`LineIndex::get_line_col`](https://github.com/EmmyLuaLs/emmylua-analyzer-rust/blob/aaaca68425d9362876228649b0b8d92f07654daa/crates/emmylua_parser/src/text/line_index.rs)
counts Unicode scalar values, and
[`LuaDocument::to_lsp_range`](https://github.com/EmmyLuaLs/emmylua-analyzer-rust/blob/aaaca68425d9362876228649b0b8d92f07654daa/crates/emmylua_code_analysis/src/vfs/document.rs)
copies those columns into the range. The current
[`syntax/coordinates.rs`](../crates/wow-emmy/src/syntax/coordinates.rs) correctly
interprets that source contract and converts to UTF-8 bytes. Preserve it, then
use Project's snapshot-bound overlay index for UTF-8/UTF-16 client positions.
Do not normalize CRLF or count graphemes under the original content identity.
This chain was inspected at source level; an editor session was not exercised.

Use upstream diagnostic configuration to disable a diagnostic only when the
selected WoW environment gives a concrete reason. Do not gate all findings on
a framework allowlist of approved codes, replace upstream messages with generic
text, or turn every finding into an error. Bound and safely render messages as
display data. Missing input, unhealthy Library analysis, failed work, and
cancellation must not appear as a clean diagnostic result.

Diagnostics are static evidence. They do not prove runtime behavior, combat
safety, taint safety, or that a selected API is available in every client
flavor.

## Addon chunk context

The pinned [`infer_literal_expr`](https://github.com/EmmyLuaLs/emmylua-analyzer-rust/blob/aaaca68425d9362876228649b0b8d92f07654daa/crates/emmylua_code_analysis/src/semantic/infer/mod.rs)
returns a homogeneous `any...` for unbound chunk ellipsis. The workspace API has
no WoW chunk-argument setting. A prelude that merely declares two globals does
not make `local arbitraryName, arbitraryTable = ...` use them.

For ordinary selected addon Lua chunks, implement this narrow adapter:

1. Create one analysis-only Library unit per addon containing one stable private
   table and a helper that returns the addon-name literal followed by that table.
   Allocate the table outside the helper. Expose the helper as an analysis-only
   global resolvable from its Main chunks. Choose names that cannot collide
   with identifiers in the selected source/Library; rebuild the context on changes.
2. Use Emmy's AST to identify ellipsis expressions owned by the chunk, including
   those inside top-level control flow. In an in-memory virtual copy, replace each
   with a call to its addon's helper. Do not touch nested function varargs, comments,
   strings, or user files. Keep each file as a separate lexical chunk.
3. Preserve unchanged text as identity map segments and map each replacement
   expression to the original ellipsis span. Compose these maps with existing XML
   mappings where applicable. All semantic facts and navigation must refer to the
   same analysis view; never combine original offsets with transformed syntax.
4. Mark generated helper declarations/calls as context-only origins and filter
   them from user call/reference/API/flow facts and output. A source map alone
   cannot make a synthetic call a user operation. Wrapper-only problems become
   context-analysis problems; mapped user problems retain their original location.

This is a **design inference from the pinned source**, not implemented or executed
support. [`unwrapp_return_type`](https://github.com/EmmyLuaLs/emmylua-analyzer-rust/blob/aaaca68425d9362876228649b0b8d92f07654daa/crates/emmylua_code_analysis/src/semantic/infer/infer_call/mod.rs)
retains a multi-result call in a final expression position and selects its first
slot otherwise. A table identity in the second result is retained inside that
multi-result value; the direct table-return instance conversion is a different
branch. [`set_index_expr_owner`](https://github.com/EmmyLuaLs/emmylua-analyzer-rust/blob/aaaca68425d9362876228649b0b8d92f07654daa/crates/emmylua_code_analysis/src/compilation/analyzer/lua/stats.rs)
attaches fields to the referenced table identity. These are the existing mechanisms
to reuse, with actual cross-file typing and mappings verified when the implemented
workflow can be exercised. No second Lua type engine is needed.

Do not apply this two-argument model to every XML body or Blizzard-owned load
context. Give an XML handler implicit parameters only when the exact script
element has a source-verified signature; an inline load script is a different
unit. `SuppressLocalTableRef`, secure-environment and bootstrap
directives also need explicit selected-source treatment. Unsupported contexts
remain limited rather than receiving a fabricated non-nil private table. See
[load context boundaries](WOW_LINTER.md#4-toc-and-xml-form-the-load-model) and
the comparative [Wowless loader](DONORS.md#wowless).

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

The existing [`navigation.rs`](../crates/wow-emmy/src/navigation.rs) and
[`local_references.rs`](../crates/wow-emmy/src/local_references.rs) already use
`find_decl(..., NoTrace)` and native declaration/reference identities for narrow
same-file lexical queries. Their Service consumers preserve the captured text
and negotiated positions. Retain those useful algorithms where needed; they are
not whole-project definition/reference support, and their current constructors
still create separate one-file analyses. Move callers to the shared configured
owner instead of rebuilding a language server or searching identifiers by text.

The framework's focused stdio LSP and MCP interfaces should serve the same WoW
project workflows as the CLI, including useful project diagnostics and source
navigation. They are not substitutes for EmmyLua's general-purpose editor
server. Keep framework-specific operations limited to behavior that requires
selected WoW project inputs or mappings.

## Current implementation state

The diagnostic-handoff slice on `feat/direct-addon-check-2026-10-24` changes the
standalone exact-snapshot path, not the complete configured addon check:

- [`syntax/analyzer.rs`](../crates/wow-emmy/src/syntax/analyzer.rs) now sets Lua
  5.1, calls `init_std_lib(None)`, preserves configured string/numeric/uncoded
  diagnostics, exact bounded messages and upstream severity. The four named
  families retain typed kinds; other diagnostics are no longer discarded.
- The transitional LSP projection now sends the retained upstream message and
  severity. Its input still contains one document rather than the complete
  project Main, Library and mapped XML context. See
  [`overlay/state.rs`](../crates/wow-project/src/overlay/state.rs).
- The shared Main/Library/virtual constructor in
  [`references.rs`](../crates/wow-emmy/src/references.rs) still creates fixed
  `Emmyrc::default`, omits embedded STD initialization and constructs semantic
  collectors regardless of selected checks. `flow.rs` still creates another
  analysis. T06 in the [trace register](IMPLEMENTATION_TRACE_REGISTER.md#t06--one-shared-emmy-engine-exists-but-is-fixed-and-private)
  remains open.
- The old generic CLI/Service result still lacks the direct message-bearing
  diagnostic contract and ordinary configuration. The standalone/LSP fix must
  not be presented as the completed addon workflow.
- File updates conservatively clear and rebuild semantic indexes; upstream
  touched-ID updates alone do not establish cross-file invalidation parity.

Formatting, compilation and Clippy are build evidence only. No addon or editor
workflow is marked observed until the corresponding direct owner exists and the
exact command/input/result is recorded in the trace register.
