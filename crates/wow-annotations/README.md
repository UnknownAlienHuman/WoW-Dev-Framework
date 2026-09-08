# `wow-annotations` contract router

**Status:** the Rust Ketho emitters and native source-to-library connection are active in Cargo.
The full E1-C ReferenceView-to-artifact service is not complete.

See [`src/ketho.rs`](src/ketho.rs), [`src/literals.rs`](src/literals.rs),
[`tests/ketho.rs`](tests/ketho.rs), [`tests/literals.rs`](tests/literals.rs),
[`THIRD_PARTY_NOTICES.md`](THIRD_PARTY_NOTICES.md) and the
[Rust port map](../../docs/KETHO_RUST_PORT.md).

The pure emitter slice ports Ketho's GetType/GetField/GetFunction/GetTable/GetCallbackType/
GetSystem behavior and explicit function/method naming. It has no dependencies,
source ingestion, IO, external interpreter runtime, editor mutation, or reference-authority
claims. Callers provide ordered declaration data, enum membership and resolved
widget aliases. Unsafe/unrepresentable input is rejected, not widened or dropped.
The literal lane also ports event/CVar aliases and enum/constant files from
typed input, including scalar-preserving values and explicit order/format policy.
The native connection in `src/native.rs` uses the reference-owned restricted
EmmyLua evaluator and typed normalization. It retains raw metadata, unprojected
fields, source hashes and declaration/ordered-member maps. The complete service
still needs persistent ReferenceView integration, full correction/type closure,
type/documentation-fragment maps, artifact publication and full-corpus semantic probes.

The native report is `wow-native-annotation-library/3`: scalar-resolution evidence
is retained and reserved return labels have explicit collision-safe name maps.
Prose controls are escaped with source links; directives remain rejected.
Invalid literal groups and duplicate members do not erase valid sibling groups.
These are bounded consumer projections, not changes to Blizzard facts.

```text
cargo test -p wow-annotations
cargo clippy -p wow-annotations --all-targets -- -D warnings
```

`wow-annotations` projects one exact read-only `wow-reference` generation into deterministic, analysis-only LuaCATS/Emmy annotation artifacts. It owns semantic projection, type lowering, versioned layout/rendering, source maps, projection-loss reporting, Ketho semantic parity, and consumer compatibility profiles. It does not own platform truth.

## Canonical route

Read the E1-C contract package in this order:

1. [`e1/README.md`](e1/README.md) — scope, authority, dependency, input/output, and completion contract.
2. [`e1/AGENTS.md`](e1/AGENTS.md) — mandatory implementation rules.
3. [`e1/DECISIONS.md`](e1/DECISIONS.md) — accepted annotation-projection decisions.
4. [`e1/DATA_MODEL.md`](e1/DATA_MODEL.md) and [`e1/SEMANTIC_MODEL.md`](e1/SEMANTIC_MODEL.md).
5. [`e1/TYPE_LOWERING.md`](e1/TYPE_LOWERING.md).
6. [`e1/LAYOUT_AND_RENDERING.md`](e1/LAYOUT_AND_RENDERING.md).
7. [`e1/DIALECT_AND_GLOBALS.md`](e1/DIALECT_AND_GLOBALS.md).
8. [`e1/SECURITY_AND_SANITIZATION.md`](e1/SECURITY_AND_SANITIZATION.md).
9. [`e1/SOURCE_MAP_AND_LOSS.md`](e1/SOURCE_MAP_AND_LOSS.md).
10. [`e1/PARITY_AND_CONSUMER_PROBES.md`](e1/PARITY_AND_CONSUMER_PROBES.md).
11. [`e1/ERROR_MODEL.md`](e1/ERROR_MODEL.md), [`e1/TEST_MATRIX.md`](e1/TEST_MATRIX.md), and [`e1/IMPLEMENTATION_PLAN.md`](e1/IMPLEMENTATION_PLAN.md).
12. [`e1/CONTRACT.json`](e1/CONTRACT.json) and the closed [`e1/examples/`](e1/examples/README.md) fixture package.

Optional operator context is advisory, disabled by default, and configured outside the repository; current Blizzard source remains authoritative.

## Direct framework dependencies

```text
wow-core
wow-reference
```

No dependency on `wow-store`, `wow-emmy`, `wow-project`, `wow-service`, applications, editors, analyzers, Ketho, or external processes is permitted in the library crate. External parity and consumer probes use reviewed test/tool adapters.

## Owned responsibilities

- exact ReferenceView/profile/reference-generation input validation;
- consumer-neutral annotation semantic model;
- explicit type and restriction lowering;
- versioned deterministic artifact topology and inert source rendering;
- WoW dialect/global projection without editor mutation;
- safe identifier, literal, and documentation rendering;
- final-byte generated source maps;
- separate reference coverage, projection coverage, and projection-loss records;
- pinned Ketho semantic comparison without authority transfer;
- versioned EmmyLua and LuaLS consumer capability/probe contracts;
- artifact manifests, eligibility, budgets, cancellation, and deterministic checksums.

## Hard boundaries

`wow-annotations` must not:

- parse or acquire Blizzard source;
- execute Lua, generated files, oracle repositories, or addon code;
- correct or replace `wow-reference` facts;
- infer current profiles, aliases, replacements, runtime Secret state, or permanent spell whitelists;
- silently widen unsupported input to `any`;
- collapse optional, nullable, missing, default, tuple, multiple-return, or restriction semantics;
- interpolate source text into directives, code, identifiers, paths, modules, or file topology;
- mutate editor/user/workspace settings, globals, libraries, extensions, or diagnostics;
- include full Blizzard implementation source or runtime addon payloads;
- write SQLite, expose raw SQL, perform network/process/shell access, or publish final releases.

## Current implementation state

The pure emitter is a bounded executable slice authorized by revised ADR-004.
The full E1 examples remain design fixtures and do not certify this slice or
pretend its source maps, artifact manifests or consumer probes already exist.
The Rust tests verify committed Ketho-derived byte vectors; they never generate
or rewrite the expected files. Historical donor revisions identify test evidence,
not a permanent client, dependency, or source-version requirement.

## Completion gate

E1-C code is complete only after one exact ReferenceView produces byte-deterministic semantic, rendered-file, source-map, loss, parity, probe, and artifact manifests; every unsupported or transformed input is explicit; EmmyLua and LuaLS positive and negative probes pass without configuration mutation or diagnostic suppression; and all [`e1/TEST_MATRIX.md`](e1/TEST_MATRIX.md) cases and [`e1/examples/CHECKSUMS.json`](e1/examples/CHECKSUMS.json) vectors pass.

## Explicit reviewed corrections

The native connection can consume a validated reference-owned correction set via
`project_with_corrections`; the Git driver exposes `--corrections <pack.json>`.
Type/Nilable fields and ScriptObject receivers are changed only after exact
source/value checks, without mutating raw source. Correction-enabled reports use
v4 and retain the v3 fields; unconfigured reports stay v3. The renderer now accepts
bounded named/primitive unions and preserves grouping for array elements.
See [the correction contract](../../docs/KETHO_NATIVE_CORRECTIONS.md).
Receiver naming does not implement full widget inheritance or named-type closure.

## Source-owned ScriptObject classes

The native library path now composes Ketho-style `---@class` and file-local
receiver tables with methods in the same lexical scope. An explicit guarded
widget-owner correction also emits an alias from the original source system name
to the corrected class. There are no new runtime globals or constructor claims.
Class/binding/alias blocks and individual methods retain final-byte source maps.

The standalone `Renderer::render_mapped` donor-emitter profile is unchanged;
`render_library_mapped` is the combined class/library profile used by native
ScriptObject projection. Global/namespace output is unchanged. Duplicate owners,
alias chains that collide, reserved type names and cross-kind collisions are
reported before ambiguous receiver output is emitted. Independent table/event
lanes remain available. See `tests/receivers.rs` for executable cases.

This is receiver declaration closure only. Gethe system names are not shortened
by convention. Remaining custom alias/type forms, widget inheritance and
EmmyLua/LuaLS semantic certification remain incomplete.

## Optional external alias resource

`project_with_alias_catalog` consumes a reference-owned `AliasDocument` as an
explicit annotation-only overlay. The Git development driver accepts
`--alias-catalog <checkout> <ref> <resource.lua>`, independently of `--corrections`.
Supported primitive/named unions require dependencies on builtins or actually
projected source/catalog types. Source conflicts, duplicates, cycles and
unsupported terms remain explicit. No raw Blizzard facts, signatures, globals
or correction guards are replaced. This is not a source-authority path into
ReferenceView.

Catalog-enabled reports use v5 and scoped external maps; no catalog preserves
v3/v4 output. Resource revision/hash/raw text and all outcomes remain available
for review. Parser acceptance and artifact verification are not semantic
certification. See [usage and licensing](../../docs/KETHO_RUST_PORT.md#alias-resource-connection)
and [regressions](tests/aliases.rs).

## Selected literal implementation

`native::project_with_literal_bridge` accepts one immutable, identified
`wow-render-contract::LiteralBridge` for a complete projection. Normalization,
corrections and alias input remain under existing owners. The optional bridge
emits a v6 execution sidecar; transport/identity/cancellation failure returns no
library. The small `selected_literals` module owns dispatch and receipts only.
No Wasmi, IO, interpreter or host dependency enters this crate. Native callers
retain their previous output schemas and donor golden bytes.

## Generated-to-source navigation

`navigation::source_at` resolves a zero-based UTF-8 byte position in an existing
`NativeLibrary` to its source descriptors:

```rust
let result = wow_annotations::navigation::source_at(
    &library,
    generated_path,
    viewed_file_sha256,
    byte_offset,
    &cancelled,
)?;
```

The viewed-file digest and `wow-native-field-maps/1` profile are required.
Parameter, return and field maps take precedence over declaration and whole-file
maps; the smallest same-kind span wins and equal candidates remain explicit.
Each candidate includes its original revision, path, digest and source range.
External Ketho catalogs keep their own revision even when a source path matches
one in the Blizzard input. Stale bytes, invalid links/ranges, unsupported profiles
and cancellation return `LookupError`; headers/EOF without a map return `Unmapped`,
not API-absence evidence. Partial generations retain their original limitations.

This is a read-only Rust query over retained, already admitted inputs, not an
arbitrary-JSON authenticator, source fetcher, LSP connection or editor installer.
`source_at` expects UTF-8 bytes. For zero-based line/column input, use
`navigation::source_at_position` with `TextPosition { line, character }` and an
explicit `PositionEncoding::{Utf8, Utf16, Utf32}`. The adapter validates the
viewed-file digest before converting coordinates and reuses the same source-map
selection. LF, CRLF and bare CR delimit lines without rewriting file bytes.
Columns beyond the line and positions inside UTF-8 characters or UTF-16 surrogate
pairs reject; no encoding default or clamping is inferred. Returned ranges remain
UTF-8 bytes, not grapheme counts or display-cell positions. The host still owns
protocol negotiation and any protocol-specific position clamping.

## Prepared bidirectional navigation

Use `navigation::NavigationIndex::new(&library, &cancelled)` for repeated queries
over one immutable native generation. Preparation validates every generated file
and source link once, then builds per-file interval indexes and an exact reverse
index. The index borrows the generation; rebuild it when the generation changes.
No global cache, filesystem access or background refresh is involved.

`NavigationIndex::source_at` and `source_at_position` retain the one-shot APIs'
arguments, mapping precedence, tie order and coordinate rules. Each query still
requires the viewed-file digest, but does not rehash its immutable bytes or
revalidate every map. Coordinate conversion still scans the selected line prefix.
The original one-shot functions remain available for occasional queries.

`NavigationIndex::generated_for(source_revision, &source_link, &cancelled)`
returns every generated occurrence of an exact source descriptor, ordered by
generated path and range. Scope, revision, path, digest and both span boundaries
must match. A local field descriptor shared by several callables can return
several occurrences. Ketho resources retain their independent revision and scope.
This is descriptor-to-output navigation, not a general Lua reference search:
overlapping or valid unrecorded ranges return `GeneratedLookup::Unmapped`,
not API-absence evidence. Stale or invalid source identities reject.

Preparation is stricter than a one-shot selected-file query: corruption in any
generated file rejects the index. Total limits are 64 MiB of generated text and
262,144 mappings, in addition to existing per-file/source limits; each query
returns at most 65,536 candidates. Cancellation and unsupported profiles remain
explicit errors. These bounds describe admitted data, not measured memory usage
or an end-to-end performance guarantee.

## Diagnostic ranges

`navigation::source_for_range` accepts a half-open UTF-8 `Span`;
`source_for_text_range` accepts `TextRange { start, end }` with an explicit
`PositionEncoding`. Both endpoints use the same digest-bound generated file.
`NavigationIndex` provides matching methods for repeated queries, reusing its
interval index and previously validated immutable source identities.

A nonempty selection requires one map containing its entire range. Crossing
parameters can select their containing declaration, but independent declarations
are never stitched into a fabricated source range. Precision, smallest-span
ranking and equal-candidate order remain the same as for point queries. Empty
ranges retain cursor behavior, including exclusive map ends and unmapped EOF.
`Unmapped` means no single map contains the whole selection, not that every
selected byte is unmapped or an API is absent. Invalid/reversed endpoints fail
without clamping; returned source ranges remain actual stored descriptor ranges.
This lookup does not publish diagnostics or certify analyzer results.

## Source-buffer navigation

`NavigationIndex::bind_source(SourceFile { scope, revision, path, sha256 }, text,
&cancelled)` binds an immutable source buffer to the retained generation. Unlike
`generated_for`, this entry point does not require knowing a descriptor's exact
source range. `scope: None` selects Blizzard documentation; an external catalog
uses `Some("annotation_alias_catalog")` and its independent revision and digest.

The returned `SourceNavigation` provides `generated_at` and `generated_for_range`
for UTF-8 byte coordinates, plus `generated_at_position` and
`generated_for_text_range` for explicit UTF-8/UTF-16/UTF-32 editor coordinates.
A cursor or selection chooses the most precise containing source descriptor;
all equally precise generated occurrences are retained in generated path/range
order. Shared local fields can therefore lead to several annotation locations.
A selection spanning unrelated source declarations is not artificially joined.
Zero-width ranges retain cursor semantics and exclusive ends, including EOF.

Binding checks the viewed revision, scope, path, recorded length and actual text
hash, then validates source-map UTF-8 boundaries against these exact bytes. Stale
text or identity rejects. A known source with no emitted maps can still be bound:
its `Unmapped` result is not evidence that the API or all selected source is absent.

Only the selected source gets an additional interval index and sparse line
checkpoints. The view borrows both its text and the prepared generation; rebuild
it after a buffer edit. Repeated queries reuse these validated inputs without
rehashing, fetching, parsing or executing source. Existing generation and result
budgets still apply; this is not a measured performance guarantee. The operation
is source-descriptor navigation, not a general Lua reference finder or an LSP
server. Returned generated and source ranges remain the original stored UTF-8
ranges, with no offset interpolation.

## Returning editor coordinates

Navigation results retain canonical UTF-8 byte ranges. Convert those ranges
without reimplementing coordinate rules in each host:

- `NavigationIndex::generated_text_range(path, viewed_sha256, span, encoding,
  &cancelled)` converts a generated range using the same immutable file and
  required viewed-file digest.
- `SourceNavigation::source_text_range(span, encoding, &cancelled)` converts a
  source range against the exact buffer previously verified by `bind_source`.

Both return `TextRange` with zero-based endpoints in the explicitly selected
`PositionEncoding`. Preserve the original result's path, scope, revision and
hash alongside the converted coordinates. Conversion does not select a new map,
merge candidates or interpolate between source and generated spans.

The existing sparse line checkpoints serve both directions. LF, CRLF and bare
CR retain their original bytes; Unicode columns count the chosen code units,
not display cells or graphemes. Valid endpoints round-trip to the same bytes.
The boundary between CR and LF in a CRLF delimiter has no distinct editor
coordinate and rejects as `InvalidPosition`, as do reversed ranges and endpoints
inside UTF-8 characters. End-of-line, trailing empty lines, EOF and zero-width
ranges remain representable. Cancellation and existing size limits apply.
These operations do not implement LSP transport, protocol negotiation or editor
mutation; the host still attaches document identities and its negotiated encoding.
