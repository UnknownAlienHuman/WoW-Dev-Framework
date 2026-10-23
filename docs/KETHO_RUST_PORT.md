# Blizzard facts to an EmmyLua Library

Reuse the native Rust pipeline already present. Ketho is the donor for selected
annotation behavior, not a complete specification of the custom WoW linter.
Reviewed revisions and direct module links are in [DONORS.md](DONORS.md).
[WOW_LINTER.md](WOW_LINTER.md) owns the required game semantics.

## Existing source-to-output path

| Step | Code |
| --- | --- |
| Command, exact Git blobs, document TOC | `apps/wow-reference-builder/src/{lib,io,catalogs}.rs` |
| Restricted Lua-shaped data ingestion | `wow-reference/src/native.rs` and `native/` |
| Ordered normalized systems/types/functions | `wow-reference/src/native_model.rs` |
| Explicit corrections and alias catalogs | `wow-reference/src/native_corrections.rs`, `native_aliases/` |
| Native composition and metadata | `wow-annotations/src/native.rs` |
| Type/receiver/callback emission | `wow-annotations/src/ketho.rs`, `ketho/` |
| Literals and source navigation | `wow-ketho-literals`, `wow-annotations/src/{literals,navigation}` |

`wow-*` paths mean `crates/wow-*`. Use the same owners for extensions. Do not add
a second parser, generator or a separate format for each optional feature.

The donor's `wowdoc/loader/init.lua` executes Lua through `loadfile`. This project's
builder instead parses and evaluates only a restricted data subset with EmmyLua.
Port the useful semantics, never that source-execution mechanism. Unsupported
computed expressions retain their original location and an explicit limitation.

The current command is documented in
[the builder README](../apps/wow-reference-builder/README.md). It writes a new
output directory, `.lua` files and `source-report.json`; exit 3 means partial.
The output is not loaded or executed by the game. An ordinary Library reader
consumes the files directly.

## Semantics already worth retaining

- System, namespace and ScriptObject ownership; exact receiver style and ordered
  arguments/returns. A colon receiver changes the call signature.
- `bool`, `cstring`, `luaIndex` and selected named aliases; arrays from `InnerType`;
  nullable/union/tuple precedence and terminal stride packs.
- Present defaults including `false` and zero. Never decide whether a default
  exists with Lua/Rust truthiness shortcuts.
- Callback argument and return order; distinction between the legacy
  argument-only callback form and explicit callback signatures.
- Exact integer/finite-number/string/boolean literals. Do not parse a numeric
  string as a number, lose a raw numeric lexeme, or silently round an integer
  outside the supported exact range.
- Closed literal enums versus open FrameEvent/CVar strings with known hints.
  A partial inventory of events is not a whitelist of all valid runtime names.
- Selected alias/structure/namespace catalogs and explicit widget mappings.
  Preserve unresolved named references, cycles and conflicts instead of expanding
  everything into duplicate signatures or `any`.
- Declaration/member source maps and third-party notices. A literal with only
  coarse source provenance must not advertise a precise member mapping.

These are implemented mechanisms, not a claim of full current-Blizzard or
LuaLS/Emmy consumer parity. Existing renderer cases can be read as examples;
test execution follows the owner policy in [AGENTS.md](../AGENTS.md).

## Missing semantics the linter needs

At `c36a058`, `FieldFact` models type/inner type/nilability/default/stride;
`CallableFact` models documentation/arguments/returns; `SystemFacts` models
system environment. Raw data is retained, but the typed/emitted contract does
not cover several meaningful fields from the reviewed Blizzard revision:

| Field | Required change |
| --- | --- |
| Callable `Environment` | Inherit/override the system context explicitly; exclude or mark secure-only members for ordinary addon analysis. |
| `MayReturnNothing` | Preserve the callable's zero-result alternative; non-nil individual return types do not cancel it. |
| Field `Mixin` | Preserve the association and resolve it from selected declarations; do not assume arbitrary mixins form a single inheritance chain. |
| `SecretArguments`, `SecretValue`, restriction/hook flags | Preserve typed optional metadata and origins for relevant direct rules/advisories. Unknown values remain raw and uninterpreted. |

`native_view.rs::project_restrictions` already recognizes a narrow positive
`SecretReturns=true` first-return case and the exact `canaccessvalue` predicate.
Reuse that substance without its proof/partition scaffolding. It is not a
complete interpretation of conditional returns, every return slot or secret
structure fields.

Do not equate Ketho compatibility with these semantics being implemented: its
reviewed emitter does not handle all of them. `DependencyOrderedTables` was not
found in the inspected generated Blizzard corpus; do not prioritize speculative
support over fields actually present. The current-source observations and
example APIs are recorded in [AUDIT.md](AUDIT.md).

## Corrections and supplemental declarations

Generated Blizzard tables are the source facts. Explicit corrections can repair
a known field, receiver name or reviewed single parent; supplemental Ketho/user
declarations can fill a known gap. They must not silently override conflicting
Blizzard facts or each other. Keep exact selected source, expected old value,
replacement and reason. If a target changes, retain the original and report the
unapplied correction. Unknown input does not become a guessed correction.

Current correction JSON and its guards live in
[`native_corrections.rs`](../crates/wow-reference/src/native_corrections.rs) and
its modules; existing input examples are in
[`corrections.rs`](../crates/wow-annotations/tests/corrections.rs) and
[`inheritance.rs`](../crates/wow-annotations/tests/inheritance.rs).
Their `/1`/`/2` schema and per-record raw hashes describe existing code, not a
requirement to preserve a certification platform. Simplify producer/reader
together as specified in [IMPLEMENTATION.md](IMPLEMENTATION.md).

Widget names come from explicit donor/source mappings. Do not infer a parent
from filenames, shorten names by convention or copy all transitive methods.
Unknown/conflicting bases remain unresolved. Only emit an inherited type when
its selected base is representable; otherwise report the missing relation.

## Output and source locations

Emit normal inert `.lua` declarations once. Keep extra API context/restriction
facts and source maps beside them when a real consumer needs them. A user-supplied
plain Lua Library remains supported for types without a generator certificate.

Mappings refer to exact source bytes and declaration/member ranges. Keep
original source, corrected value and generated span distinct. Generated
identifiers/directives must be valid and escaped; unsafe/unrepresentable data
must not alter a later declaration. One rejected declaration need not discard
independent valid declarations, but the omission must remain visible.

Shared output/acquisition ceilings already exist in `wow-core::library_limits`
(4,096 files, 8 MiB/file, 64 MiB total). This does not establish the same capacity
for the old replay/graph paths or prove maximum-size consumer behavior. Fix the
actual check reader rather than restoring old reports or raising every budget.

Keep the existing [annotation notice](../crates/wow-annotations/THIRD_PARTY_NOTICES.md),
[literal notice](../crates/wow-ketho-literals/THIRD_PARTY_NOTICES.md) and
[reference notice](../crates/wow-reference/THIRD_PARTY_NOTICES.md).
A `.lua` extension alone does not prove LuaLS/Emmy annotation compatibility;
use only syntax supported by the chosen consumer and record any loss.
