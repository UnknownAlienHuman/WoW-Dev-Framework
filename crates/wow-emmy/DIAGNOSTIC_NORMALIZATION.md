# Diagnostic adapter

Current code: `src/session.rs`; `src/syntax/analyzer.rs::{collect_registered,accepted_code}`;
`src/syntax.rs::EmmySyntaxDiagnostic`; Project consumes the report in
`../wow-project/src/{analyzer,snapshot}.rs`. The implementation route is
[addon analysis](../../docs/IMPLEMENTATION.md#addon-configuration-and-emmy-analysis).

## Current behavior

One `EmmyLuaAnalysis` registers Main and Library, then supplies Main diagnostics,
member/call facts and local flow. The adapter keeps this useful diagnostic slice:

| Upstream code | Local kind |
| --- | --- |
| `syntax-error` | `LuaSyntax` |
| `doc-syntax-error` | `DocumentationSyntax` |
| `assign-type-mismatch` | `AssignmentTypeMismatch` |
| `param-type-mismatch` | `ParameterTypeMismatch` |

Other upstream codes remain outside the current product until a concrete addon
workflow needs them. An empty report therefore means no finding in this selected
slice, not complete proof of correctness or runtime safety.

Keep UTF-8 half-open spans via `syntax/coordinates.rs`. Diagnostics are collected
only for Main files even though Library declarations participate in type checking.
Distinguish invalid input, cancellation and unavailable analysis from a clean
result. Reuse upstream diagnostic generation; do not add a custom type checker,
code-approval registry or result-fingerprint framework.

Normal configuration belongs to #200 and the complete generation-to-check CLI
workflow to #201. Standard library setup uses the pinned donor's initializer as
described in `docs/EMMYLUA_UPSTREAM.md`; it does not require another catalog or
installation subsystem.
