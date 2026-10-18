# Diagnostic adapter

Current code: `src/syntax/analyzer.rs::{collect_registered,accepted_code}`;
`src/syntax.rs::EmmySyntaxDiagnostic`; Project consumes the report in
`../wow-project/src/{analyzer,snapshot}.rs`. The replacement is
[#193](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/issues/193) and the
[analysis route](../../docs/IMPLEMENTATION.md#addon-configuration-and-emmy-analysis).

## Current gap

The physical diagnostic engine registers Main only. Its separate semantic
engine sees Library files. The accepted diagnostic filter currently keeps only:

| Upstream code | Local kind |
| --- | --- |
| `syntax-error` | `LuaSyntax` |
| `doc-syntax-error` | `DocumentationSyntax` |
| `assign-type-mismatch` | `AssignmentTypeMismatch` |

Other codes, including `param-type-mismatch`, are discarded. An empty report
therefore establishes only this selected slice, not complete Emmy checking.
The native generator's external consumer probes bypass this adapter.

## Required behavior

Register Main and Library in the same Emmy analysis and collect Main diagnostics
from it. Carry the useful upstream code/severity and correct source location
through to the ordinary addon check. Reuse upstream diagnostic generation; no
custom type checker, code-approval registry or result fingerprint framework.

Keep UTF-8 half-open spans via `syntax/coordinates.rs`. Distinguish invalid
library/input, cancellation and unavailable analysis from clean results; a
Library declaration is not an addon implementation finding. An unresolved name
does not prove WoW API absence or runtime safety.

For #193, use a typed Library signature with one valid and one invalid addon
call, checking the upstream code and Main location. Reuse existing Project/session
fixture helpers. A local assignment error unrelated to Library types cannot
prove the handoff, and a full external Emmy/LuaLS campaign is unnecessary.
