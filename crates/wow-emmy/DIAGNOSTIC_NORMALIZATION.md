# Diagnostic adapter

Read `src/session.rs`, `src/syntax/analyzer.rs::{collect_registered,accepted_code}`,
`src/syntax.rs::EmmySyntaxDiagnostic`, Project `src/{analyzer,snapshot}.rs`,
Service `src/local/projection.rs` and `src/frontend/lsp/diagnostics.rs`, and
`apps/wow/src/output.rs`. The [analysis route](../../docs/IMPLEMENTATION.md#addon-configuration-and-emmy-analysis)
uses upstream EmmyLua; #201 owns the useful addon result, #198 its editor projection.

## Current behavior, not the target contract

One shared `EmmyLuaAnalysis` supplies Main diagnostics with Library types, facts
and flow (#193 delivered). The adapter currently retains only `syntax-error`,
`doc-syntax-error`, `assign-type-mismatch` and `param-type-mismatch`.
It discards other codes and upstream messages, and forces normalized severity
to Error. Service findings have no message; LSP supplies constant generic prose.
An empty report therefore covers only this restricted slice.

The former instruction to keep all other codes outside the product was too
restrictive. It preserved an adapter limitation instead of reusing the engine.

## Replacement through the existing result path

Use upstream diagnostic selection/configuration for the selected Lua/WoW inputs.
Carry code, bounded message, actual severity and exact source location through
Project, Service and frontend output. Suppress a known inappropriate diagnostic
explicitly when justified by the selected environment, not by requiring every
new upstream code to enter a framework approval enum. Do not force warnings to
errors or replace explanatory text with hashes/classification labels.

Preserve UTF-8 half-open spans via `syntax/coordinates.rs`, XML virtual-source
mapping and document versions. Treat messages as display data: bound/escape them
at rendering and never execute/interpret them as instructions. Main diagnostics
use Library declarations; do not project STD/Library locations into addon files.
Missing input, failed analysis and cancellation must not become a clean result.

#200 owns ordinary configuration/stdlib setup. #201 must verify one real consumer
receives a useful message/code/severity/location and a selected upstream code
outside the old four-code filter. Reuse existing diagnostic/consumer fixtures;
no new approval registry, custom type checker, translation catalogue or full
historical diagnostic matrix. These source findings are not fixed by this note.
