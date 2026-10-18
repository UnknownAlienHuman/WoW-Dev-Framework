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

- `AnalyzerSession` owns the updateable upstream analysis and snapshot boundary.
- `analyze_syntax` and `analyze_syntax_with_stop` expose bounded physical or
  virtual syntax analysis and cancellation-aware collection.
- `document_symbols::document_symbols` builds normalized same-document symbol
  outlines from a caller-provided snapshot.
- `SourceFunctionFact::binding_name()` exposes supported local/global and
  static dot/colon declaration names with their exact source spans.
- `references`, `function_calls`, `local_references`, and `flow` expose the
  normalized semantic slices required by current project and recognizer callers.

Keep public spans as UTF-8 byte half-open ranges. Preserve file identity,
generation, analyzer snapshot identity, coverage, cancellation and typed
failure states. An unresolved symbol means only that the analyzer could not
resolve it in the supplied workspace; it does not prove a WoW API is absent.
Analyzed Lua and addon repositories are data and are never executed.

## Ownership and reading

`wow-reference` owns selected-build API facts and restrictions; `wow-project`
owns project/source registration and generation; `wow-rules` owns WoW-specific
diagnostic decisions; Service and applications compose user workflows.

Start with [`AGENTS.md`](AGENTS.md), [`CONTRACT.json`](CONTRACT.json), and the
specific owner/caller code for the change. Use these focused references:

- [session and snapshots](SESSION_MODEL.md)
- [fact model](FACT_MODEL.md)
- [diagnostic normalization](DIAGNOSTIC_NORMALIZATION.md)
- [source coordinates](SOURCE_COORDINATES.md)
- [errors and cancellation](ERROR_MODEL.md)
- [document symbols](DOCUMENT_SYMBOLS.md)
- [upstream API boundary](../../docs/EMMYLUA_UPSTREAM.md)

Use [`PIN_AND_PROBE.md`](PIN_AND_PROBE.md) only for a concrete dependency-pin or
compatibility task. For ordinary changes, follow the repository's focused check
cadence; a full test matrix or editor/client qualification is needed only for a
task claiming that broader coverage. Runtime integration excess remains tracked
by Issues #193, #196, and #203.
