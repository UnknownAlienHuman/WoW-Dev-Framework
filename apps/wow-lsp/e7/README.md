# `wow-lsp` E7-A read-analysis adapter

**Status:** implementation-ready documentation; no Rust code yet.

**Contract ID:** `apps/wow-lsp/e7-a/read-analysis-lsp-adapter`

## Mission

Project one exact subset of the Language Server Protocol onto the transport-independent E7-A `wow-service` session API.

```text
LSP stdio framing and initialize
-> exact service session/workspace bind
-> document overlay notifications
-> one supported request -> one service operation
-> exact service result -> bounded LSP projection
-> cancellation/progress/shutdown/closure
```

## Reading order

1. [`METHODS_AND_CAPABILITIES.md`](METHODS_AND_CAPABILITIES.md)
2. [`FRAMING_SYNC_AND_POSITIONS.md`](FRAMING_SYNC_AND_POSITIONS.md)
3. [`OUTPUT_ERRORS_AND_LIFECYCLE.md`](OUTPUT_ERRORS_AND_LIFECYCLE.md)
4. [`SECURITY_AND_INPUTS.md`](SECURITY_AND_INPUTS.md)
5. [`TEST_MATRIX.md`](TEST_MATRIX.md)
6. [`CONTRACT.json`](CONTRACT.json) and [`examples/`](examples/README.md)
7. [`../../../crates/wow-service/e7/`](../../../crates/wow-service/e7/README.md)

## Initial supported method families

```text
initialize / initialized / shutdown / exit
workspace status/binding through initialization options under the frozen profile
textDocument/didOpen
tекстDocument/didChange
textDocument/didSave
textDocument/didClose
textDocument/diagnostic
workspace/diagnostic (only when exact profile and owner capability exist)
textDocument/hover
textDocument/definition
textDocument/references
textDocument/documentSymbol
workspace/symbol
textDocument/codeAction
codeAction/resolve
$/cancelRequest
work-done/partial-result progress methods and notifications required by profile
```

The accidental Cyrillic spelling above is not a protocol method; canonical method IDs are listed in `CONTRACT.json` and `METHODS_AND_CAPABILITIES.md`. Implementations use only those machine values.

## Deliberately unsupported in E7-A

```text
completion
signatureHelp
rename
formatting
rangeFormatting
onTypeFormatting
executeCommand
workspace/applyEdit
file operations
semantic tokens
inlay hints
call/type hierarchy
code lens
remote network daemon transport
```

Unknown/unsupported methods return exact LSP-compatible errors or null/empty results only when the frozen official protocol profile requires that behavior. They are never emulated by raw owner calls.

## Hard boundaries

- only `wow-service` is imported;
- exactly one service operation per request;
- notifications are routed only to their declared session/document operation;
- no editor/workspace/source mutation;
- no hidden current refresh or candidate selection;
- no source/model/tool execution;
- no client capability or workspace trust used as operation authorization;
- exact position encoding and document versions;
- no success/clean result that hides partial coverage;
- no detached work after shutdown/exit/transport loss.
