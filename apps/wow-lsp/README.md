# `apps/wow-lsp` contract router

**Status:** E7-A LSP adapter contract is implementation-ready documentation; no Rust code exists.

`wow-lsp` is a thin Language Server Protocol adapter over `wow-service`. Its only framework dependency is `wow-service`.

Read:

1. [`AGENTS.md`](AGENTS.md)
2. [`e7/README.md`](e7/README.md)
3. [`e7/CONTRACT.json`](e7/CONTRACT.json)
4. [`../../crates/wow-service/e7/`](../../crates/wow-service/e7/README.md)
5. official [Language Server Protocol specification](https://microsoft.github.io/language-server-protocol/)

Initial profile candidate:

```text
lsp-stdio-single-client-read-analysis-v1
```

It supports initialization/shutdown, exact workspace/session binding, document open/change/save-observation/close, diagnostics, hover, definition, references, document/workspace symbols, advisory code actions, cancellation, progress, and exact protocol projection.

It does not support source writes, workspace edits, execute-command, formatting, rename, arbitrary configuration discovery, network daemon operation, model calls, or direct lower-crate access.
