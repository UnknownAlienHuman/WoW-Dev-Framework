# Native Reference data notes

`wow-reference` admits selected Blizzard API-documentation source and projects
native facts for annotations, search, and analysis. The implementation is in
`src/native.rs`, `src/native_model.rs`, `src/native_view.rs`, and focused
correction/type-resolution modules.
See the [Ketho map](../../../docs/KETHO_RUST_PORT.md) and [implementation route](../../../docs/IMPLEMENTATION.md#native-annotation-generation).

## Source admission and raw observations

`native::ingest_document` admits one caller-selected file for an exact revision
and relative path, checks its SHA-256, and uses EmmyLua's parser for the bounded
declarative APIDocumentation subset. It does not execute Lua or perform file
discovery/I/O. Digest, path, revision, registration order, and half-open UTF-8
byte spans travel with the admitted document and observations.

The raw tree (`RawValue`, `RawField`, `RawKey`, and `RawKind`) retains source
order, duplicate keys, nils, original number lexemes, symbolic references,
selected additive expressions, and nested spans. An unresolved global remains
an `UnresolvedName`; it is not looked up in a host environment or treated as a
known constant. Unsupported syntax, invalid identities, digest mismatches,
limits, and cancellation are distinct admission errors.

## Typed normalization

`native_model::normalize_document` borrows the raw document and returns one
result per registration. Supported systems distinguish global, namespace,
and ScriptObject ownership. Facts include ordered callables/arguments/returns,
events/payloads, structure fields, callbacks, enumerations, constants, and
unsupported table types. Fields keep source type names, `InnerType`, nilability,
raw defaults, and `StrideIndex`; each retains its original `RawValue`.

Normalization is partial: malformed registrations stay present as span-bearing
`ModelError`s. Object reads reject duplicate/non-name keys. Arrays require
contiguous positive numeric indices and reject holes, nil slots, duplicates,
and map-shaped tables. Unsupported table types are not empty structures.

## Corrections and unresolved meaning

`native_corrections` applies reviewed, guarded overlays without changing raw
observations. Current corrections cover selected type/nilability, receiver,
alias, and single-parent inheritance cases with source bindings and outcomes.
`native_view` can expose an exact-name callable type-resolution overlay; it
does not use fallback guesses or turn prose into facts.

Unresolved names, unsupported fields, partial inputs, and conflicting
declarations remain visible through raw observations, issues, coverage, or
conflict candidates. No first/last candidate is a semantic winner. In
particular, a failed registration or absent map entry is not evidence that a
Blizzard API is absent. Restriction output records only the selected positive
source observations; it cannot establish runtime access, combat, Secret, or
taint safety.

## Native Reference view and consumers

`native_view::project_callables`, `project_native_documents`, and related
projections build the current in-memory `ReferenceView`, source records,
conflicts, coverage, and typed signature catalogs. The callable partition is
complete only when the selected TOC/document set closes over its declared API
inputs and no in-domain callable projection was lost. Otherwise its coverage
stays partial. Restriction observations have narrower coverage and remain
positive-only.

Native annotation generation consumes the admitted documents and normalized
`SystemFacts` directly in `wow-annotations/src/native.rs`; it does not need a
persisted ReferenceView first. `wow-search/src/reference.rs` and Service's
`graph/build/native_api.rs` consume the native view and signature data. Keep
those real input paths distinct; reuse the selected facts without promoting
partial results into global completeness. Paths here are relative to `crates/`.

## Boundaries

- Keep the selected source/flavor and accurate locations; never coerce raw
  unknown/unsupported input to defaults or `any`. Existing digest fields do not
  justify another identity or validation layer during refactoring.
- Treat exact lookup, conflict, unsupported input, and incomplete coverage as
  distinct outcomes. Partial data cannot prove absence.
- Reuse EmmyLua parsing and the existing restricted evaluator; never execute
  source, hooks, or generated annotation files.
