# `wow-emmy`

`wow-emmy` is the framework's narrow adapter over the pinned upstream EmmyLua
Rust analyzer. It owns analyzer sessions, Main/Library workspace registration,
file updates, generic diagnostic normalization, and normalized syntax and
semantic facts consumed by other framework owners. It does not parse TOC/XML,
decide WoW API authority, generate annotations, own project generations, or
implement general Lua editor features.

Upstream `emmylua_code_analysis` provides the public semantic APIs used here.
The general LSP handlers are private inside upstream `emmylua_ls`; use that
server for hover, completion, definitions, references, rename, and signature
help. Do not implement a second parser, analyzer, cache, interpreter, or full
language server in this crate. See [upstream boundary](../../docs/EMMYLUA_UPSTREAM.md).

## Main entry points

- `src/session.rs::AnalyzerSession` owns the updateable analysis boundary. It
  currently has separate Main-only diagnostics and Main/Library semantic engines;
  #193 joins them and fixes the diagnostic filter.
- `analyze_syntax` and `analyze_syntax_with_stop` expose bounded physical or
  virtual syntax analysis and cancellation-aware collection.
- `document_symbols::document_symbols` builds normalized same-document symbol
  outlines from a caller-provided snapshot.
- `SourceFunctionFact::binding_name()` exposes supported local/global and
  static dot/colon declaration names with their exact source spans.
- `references`, `function_calls`, `global_access`, and `flow` expose selected
  calls, declarations, global accesses and local flow for Project/recognizers.
- `bindings` resolves selected global/member paths across Main and Library;
  `navigation` and `local_references` provide same-file lexical operations.
- `virtual_syntax` and `virtual_semantics` map Project's generated TOC/XML Lua
  units; overlays have a [limited current path](E7_A_OVERLAY_ANALYSIS.md).

Keep public spans as UTF-8 byte half-open ranges. Preserve file identity,
generation, analyzer snapshot identity, coverage, cancellation and typed
failure states. An unresolved symbol means only that the analyzer could not
resolve it in the supplied workspace; it does not prove a WoW API is absent.
Analyzed Lua and addon repositories are data and are never executed.

## Ownership and reading

`wow-reference` owns selected-build API facts and restrictions; `wow-project`
owns project/source registration and generation; `wow-rules` owns WoW-specific
diagnostic decisions; Service and applications compose user workflows.

Start with [`AGENTS.md`](AGENTS.md), the selected
[implementation route](../../docs/IMPLEMENTATION.md#addon-configuration-and-emmy-analysis)
and actual owner/caller code. Use only the needed reference:

- [fact model](FACT_MODEL.md)
- [diagnostic normalization](DIAGNOSTIC_NORMALIZATION.md)
- [source coordinates](SOURCE_COORDINATES.md)
- [errors and cancellation](ERROR_MODEL.md)
- [document symbols](DOCUMENT_SYMBOLS.md)
- [upstream API boundary](../../docs/EMMYLUA_UPSTREAM.md)

For pin changes, use the [upstream guide](../../docs/EMMYLUA_UPSTREAM.md).
`examples/workspace-fixture.json` retains source read by Project's
`tests/primary_callable_replay.rs`; it is test data, not a pending byte-freeze
programme. Ordinary checks use the relevant Rust tests and session helpers.
