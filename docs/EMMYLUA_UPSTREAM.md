# EmmyLua integration guide

Read [current status](CURRENT_STATUS.md) first. This guide records the verified
upstream contract, the delivered framework seams, and the remaining WoW-specific
adapters. It is subordinate to [Architecture](ARCHITECTURE.md) and the
[implementation route](IMPLEMENTATION.md#addon-configuration-and-emmy-analysis).

## Pinned donor

The workspace pins `emmylua_code_analysis` and `emmylua_parser` to revision
`aaaca68425d9362876228649b0b8d92f07654daa` of
[EmmyLua Analyzer Rust](https://github.com/EmmyLuaLs/emmylua-analyzer-rust/tree/aaaca68425d9362876228649b0b8d92f07654daa).
Read behavior at that revision rather than a moving default branch. Primary
sources include:

- [`EmmyLuaAnalysis`](https://github.com/EmmyLuaLs/emmylua-analyzer-rust/blob/aaaca68425d9362876228649b0b8d92f07654daa/crates/emmylua_code_analysis/src/lib.rs)
- [embedded resources](https://github.com/EmmyLuaLs/emmylua-analyzer-rust/blob/aaaca68425d9362876228649b0b8d92f07654daa/crates/emmylua_code_analysis/src/resources/mod.rs)
- [runtime configuration](https://github.com/EmmyLuaLs/emmylua-analyzer-rust/blob/aaaca68425d9362876228649b0b8d92f07654daa/crates/emmylua_code_analysis/src/config/configs/runtime.rs)
- [diagnostic configuration](https://github.com/EmmyLuaLs/emmylua-analyzer-rust/blob/aaaca68425d9362876228649b0b8d92f07654daa/crates/emmylua_code_analysis/src/config/configs/diagnostics.rs)
- [diagnostic collection](https://github.com/EmmyLuaLs/emmylua-analyzer-rust/blob/aaaca68425d9362876228649b0b8d92f07654daa/crates/emmylua_code_analysis/src/diagnostic/lua_diagnostic.rs)
- [workspace configuration](https://github.com/EmmyLuaLs/emmylua-analyzer-rust/blob/aaaca68425d9362876228649b0b8d92f07654daa/crates/emmylua_code_analysis/src/config/configs/workspace.rs)
- [compilation/indexing](https://github.com/EmmyLuaLs/emmylua-analyzer-rust/blob/aaaca68425d9362876228649b0b8d92f07654daa/crates/emmylua_code_analysis/src/compilation/mod.rs)
- [index removal](https://github.com/EmmyLuaLs/emmylua-analyzer-rust/blob/aaaca68425d9362876228649b0b8d92f07654daa/crates/emmylua_code_analysis/src/db_index/mod.rs)

EmmyLua owns Lua parsing, type inference and language diagnostics. Do not build a
parallel Lua parser/type checker or depend on private `emmylua_ls` handlers as an
SDK.

## Engine setup

Use one analysis for three explicit source roles:

- **Main** — selected addon source to diagnose;
- **Library** — selected addon/Blizzard declarations used to resolve Main;
- **STD** — upstream Lua declarations, not user source or WoW API authority.

For the supported baseline, set Lua 5.1 explicitly and call `init_std_lib(None)`
once in the retained owner. At the pinned revision this loads embedded resources;
it does not extract files to disk. `EmmyLuaAnalysis::new()` does not load them,
and adding a workspace root does not discover files automatically.

STD typing is not WoW runtime availability. A declaration resolving through STD
cannot by itself prove that the game exposes that host API, and absence from STD
does not prove WoW absence. Availability checks require the original resolved
symbol and selected WoW facts. Do not invent a whole-runtime allowlist from the
embedded files.

## Delivered configured owner

PR #247 added `EmmyAnalysisConfiguration` and
`ConfiguredAnalyzerSession` in `wow-emmy`:

- exact caller-supplied Main, explicit Library and optional virtual snapshots;
- fixed Lua 5.1 runtime;
- caller-supplied pinned `EmmyrcDiagnostic` without `.emmyrc` discovery;
- one embedded STD initialization in the retained owner;
- diagnostics-only collection that does not invoke member/call/local-flow;
- explicit semantic collection through the same owner when a selected WoW check
  needs those facts;
- compatibility over exact Main identity, order-insensitive Library identities,
  exact virtual snapshot/generation and equivalent diagnostic configuration.

PR #250 corrected that compatibility boundary to require the exact Main
`snapshot_id`; backend/universe equality alone is insufficient for an immutable
owner with no update method.

The legacy registration constructor still creates its fixed default diagnostic
configuration during file registration. The configured owner replaces the
supported diagnostic policy before collection and then initializes STD. Because
the public configuration cannot alter parser/runtime features, no already-parsed
runtime setting differs. Move configuration/STD initialization into the lower
constructor only when legacy callers are adapted or removed; do not create a
second owner to make the code look cleaner.

PR #248 added `wow_service::addon_check::check_addon`, which opens this owner from
exact snapshots and returns a direct diagnostic/limitation result. It is not yet
the public root/TOC/Library workflow.

## Diagnostic configuration

The target `wowdev.json` may supply `emmy_diagnostics`, decoded directly as the
pinned `EmmyrcDiagnostic`: `enable`, `disable`, `enables`, `globals`,
`globalsRegex`, `severity`, and `diagnosticInterval`. Missing fields retain
upstream defaults. Invalid typed values are configuration errors. The interval is
editor scheduling data and has no effect on a one-shot CLI check.

Keep three controls distinct:

1. upstream diagnostic configuration determines which language diagnostics are
   collected and their configured severity;
2. selected WoW checks determine which additional semantic facts/rules run;
3. `fail_on` determines the CLI exit threshold without deleting diagnostics.

Do not use the lenient upstream config-file loader for explicit operation config;
it can log/skip invalid input or fall back to defaults. Do not gate diagnostics
through a framework code allowlist.

## Diagnostics and source locations

PR #246 changed the framework diagnostic schema so every configured upstream
finding can retain:

- string, numeric, or missing upstream code;
- exact bounded message;
- actual upstream severity;
- source path/content identity;
- exact half-open UTF-8 byte span.

Four known families retain typed framework kinds; all other configured codes are
`Other`, not discarded. The transitional LSP now publishes the retained message
and severity, but still uses isolated one-file input.

The pinned upstream LSP-shaped range is not already in the editor's negotiated
encoding. [`LineIndex::get_line_col`](https://github.com/EmmyLuaLs/emmylua-analyzer-rust/blob/aaaca68425d9362876228649b0b8d92f07654daa/crates/emmylua_parser/src/text/line_index.rs)
counts Unicode scalar values, and
[`LuaDocument::to_lsp_range`](https://github.com/EmmyLuaLs/emmylua-analyzer-rust/blob/aaaca68425d9362876228649b0b8d92f07654daa/crates/emmylua_code_analysis/src/vfs/document.rs)
copies those columns. The framework's
[`syntax/coordinates.rs`](../crates/wow-emmy/src/syntax/coordinates.rs) converts
that contract to UTF-8 bytes before Project's overlay converts to negotiated
UTF-8/UTF-16 positions. Preserve captured text identity, CRLF handling and the
final empty line. Never forward upstream `character` directly or apply a range
to changed text.

Library and STD diagnostics are input-health information; they are not Main
findings. Virtual XML diagnostics must be projected through the exact existing
map before reaching the user. `check_addon` currently reports that projection as
unavailable rather than fabricating a physical location.

Diagnostics are static evidence. They do not prove event execution, runtime
availability, combat/taint safety or complete Secret-value flow.

## Addon chunk context

Pinned Emmy infers an unbound chunk `...` as homogeneous `any...`; workspace
registration or unrelated globals do not assign the two WoW chunk arguments.
The public API has no WoW chunk-argument setting.

For supported ordinary external Lua chunks, the source-traced design is:

1. create one analysis-only helper unit per addon with one stable private table
   allocated outside a helper returning `(addonNameLiteral, privateTable)`;
2. use Emmy's AST to identify ellipsis expressions owned by the top-level chunk,
   including top-level control flow, but excluding nested function varargs,
   comments, strings and user files;
3. replace only those expressions in an in-memory view with a helper call;
4. preserve identity mappings for unchanged text and map each replacement to the
   original ellipsis span;
5. compose those maps with XML maps where applicable;
6. mark helper declarations/calls as context-only so they never become user
   API/call/flow facts or output.

The pinned [`unwrapp_return_type`](https://github.com/EmmyLuaLs/emmylua-analyzer-rust/blob/aaaca68425d9362876228649b0b8d92f07654daa/crates/emmylua_code_analysis/src/semantic/infer/infer_call/mod.rs)
retains a multi-result call in final-expression position and selects the first
slot otherwise. The second result's table identity survives in the multi-result
value, and
[`set_index_expr_owner`](https://github.com/EmmyLuaLs/emmylua-analyzer-rust/blob/aaaca68425d9362876228649b0b8d92f07654daa/crates/emmylua_code_analysis/src/compilation/analyzer/lua/stats.rs)
attaches fields to that referenced table identity. These mechanisms support the
design; cross-file behavior and mappings remain unobserved until implemented.

Do not apply a universal two-argument/private-table model to every XML body or
special Blizzard context. External chunks, free inline Script bodies, verified
handler signatures, `SuppressLocalTableRef`, secure environments and bootstrap
loads are distinct. Unsupported contexts remain explicit limitations.

## Updates and invalidation

The pinned bulk update API removes indexes for touched file IDs and indexes the
replacement IDs. It does not automatically reanalyze every unchanged consumer
of a deleted global/type. Until narrower invalidation is demonstrated, retain the
existing conservative semantic rebuild over the retained VFS while reusing
unchanged source/syntax where valid.

`ConfiguredAnalyzerSession` is currently immutable. Reuse it only when its exact
compatibility contract holds. A physical delta can use the older CAS-validated
`AnalyzerSession` route only while selected membership, load context, Library
identity and configuration remain unchanged. TOC/XML/dependency/Library/config
changes rebuild selection/context and the owner cold.

A successful replacement is published atomically. Failed, cancelled or stale
work cannot clear or replace a newer result. Removal also removes declarations,
virtual units, maps, diagnostics and later index rows.

Cancellation is cooperative around framework stages/files. The pinned diagnostic
API does not preempt an in-progress synchronous native call. Check before and
after it and do not advertise preemption that does not exist.

## Editor boundary

Upstream `emmylua_ls` owns general Lua completion, hover, definition, references,
rename, signature help and symbols. Its private handlers are not an importable
SDK. The framework's focused LSP/MCP should expose the same WoW project operations
as CLI, not copy a complete language server.

Existing `navigation.rs` and `local_references.rs` use native declaration and
reference identities for narrow same-file queries. Their one-file constructors
are not whole-project language-server coverage. Retain useful algorithms only
where a surviving WoW-specific consumer needs them and move that caller to the
shared configured owner.

The LSP overlay correctly owns exact unsaved text, client version, internal
revision and UTF-8/UTF-16 conversion. It currently feeds a separate one-file
analysis. The replacement passes exact text overrides to the same configured
project owner, accepts increasing nonconsecutive client versions and rejects
stale result publication.

## Current checkpoint

Implemented in `main`:

- T12 / PR #246: full configured diagnostic payload and standalone STD;
- T06/T13 / PR #247: configured Main/Library/virtual owner and optional semantic
  collectors;
- T14 / PR #248: direct Service language-check result;
- T07/T15 / PR #249: compiled identity separated from report evidence;
- T16 / PR #250: exact Main identity for owner reuse.

Still open:

- ordinary Project acquisition/configuration and public CLI wiring;
- pure XML preparation for the direct route and exact XML projection;
- addon chunk and verified handler context;
- generated-Library manifest handoff;
- project-aware LSP reuse and executed editor scenarios;
- any exact addon/editor observation.

See [the trace register](IMPLEMENTATION_TRACE_REGISTER.md) for evidence and
[the queue](REFACTOR_QUEUE.md) for the next implementation slice. Formatting,
compilation and Clippy are build evidence only.