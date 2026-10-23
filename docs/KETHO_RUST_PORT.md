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
  nullable/union/tuple precedence and the currently represented single terminal
  stride field. Multi-position repeated groups need the correction below.
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
| `NilableContents` | Preserve nullable table elements or repeated-pack values separately from `Nilable` on the field itself. |
| Numeric `StrideIndex` | Keep each position within the repeated group, alongside any fixed prefix. Do not reduce every positive value to `variadic = true`. |
| Field `Mixin` | Preserve the association and resolve it from selected declarations; do not assume arbitrary mixins form a single inheritance chain. |
| `SecretArguments`, `SecretValue`, restriction/hook flags | Preserve typed optional metadata and origins for relevant direct rules/advisories. Unknown values remain raw and uninterpreted. |

`native_view.rs::project_restrictions` already recognizes a narrow positive
`SecretReturns=true` first-return case and the exact `canaccessvalue` predicate.
Reuse that substance without its proof/partition scaffolding. It is not a
complete interpretation of conditional returns, every return slot or secret
structure fields.

Preserve the useful strict behavior of
[`native_model.rs`](../crates/wow-reference/src/native_model.rs): duplicate object
keys and sparse/nil-bearing arrays are rejected; explicit false nilability and
raw defaults remain distinct from missing values. Unsupported kinds and
unresolved symbolic values must not silently become `any`. There is a narrower
loss to inspect when changing a consumer: absent collections currently become
empty vectors in typed facts, although raw source remains available. Whether
that distinction affects a particular diagnostic or completeness decision has
not been established; do not introduce a general metadata registry to repair a
consumer that has not been identified.

Do not equate Ketho compatibility with these semantics being implemented: its
reviewed emitter does not handle all of them. `DependencyOrderedTables` was not
found in the inspected generated Blizzard corpus; do not prioritize speculative
support over fields actually present. The current-source observations and
example APIs are recorded in [AUDIT.md](AUDIT.md).

In the current path, `native/fields.rs::convert` turns positive stride indices
into booleans, while `ketho.rs::validate_fields` permits a variadic field only at
the end. Thus a valid source group with indices 1, 2, 3 is rejected with
`renderer_InvalidVariadic`. Preserve the source group in the existing model;
choose only a lowering supported by the pinned Emmy consumer. A conservative
variadic union can lose ordering/correlation and must be identified as such.
Do not invent an annotation that appears precise but the consumer cannot read.

Likewise, a nullable array element is different from a nullable array. The
existing `ketho/callbacks.rs::field_type` appends `[]` to `InnerType`, and the
field's optional marker is applied outside it. Carry `NilableContents` into
the element type before constructing the array/pack. Preserve parentheses and
mapped type atoms. `MayReturnNothing` remains a separate callable-level fact.

### What the pinned Emmy consumer can represent

The pinned
[type parser](https://github.com/EmmyLuaLs/emmylua-analyzer-rust/blob/aaaca68425d9362876228649b0b8d92f07654daa/crates/emmylua_parser/src/grammar/doc/types.rs)
and [type lowering](https://github.com/EmmyLuaLs/emmylua-analyzer-rust/blob/aaaca68425d9362876228649b0b8d92f07654daa/crates/emmylua_code_analysis/src/compilation/analyzer/doc/infer_type.rs)
were inspected, along with its
[return annotation guide](https://github.com/EmmyLuaLs/emmylua-analyzer-rust/blob/aaaca68425d9362876228649b0b8d92f07654daa/docs/emmylua_doc/annotations_EN/return.md).
This establishes source support, not an executed generated-Library check:

| Form | Meaning / limit |
| --- | --- |
| `table<number, number \| nil>` | Nil is in the value type; distinct from `table<number, number> \| nil`, which permits the table itself to be nil. The parser and `infer_special_generic_type` preserve the type arguments. |
| `(number \| nil)[]` | The parser builds an array of the parenthesized union; `infer_type` keeps the union inside `LuaArrayType`. Preserve that precedence when extending the existing array renderer. |
| `---@return number ...` | One homogeneous variadic return type, represented as `VariadicType::Base`. It does not retain the cardinality or field correlation of repeated triples. |
| `---@return_overload` | Supports finite correlated return rows and documented generic tails. It is not evidence of a repeating heterogeneous group or an empty zero-result row. |

No direct representation of repeated heterogeneous stride groups was established
in the inspected APIs. For `GetMetaPowerInfo`, all three values are numbers, so
a homogeneous number tail is a possible conservative type projection while
the group-of-three metadata remains separate. Record that loss; do not reject
all such APIs merely because an exact annotation is unavailable. For a mixed
group, only use a supported conservative form with an explicit loss, or report
the signature as unsupported. Do not present either outcome as exact typing.

## Corrections and supplemental declarations

Generated Blizzard tables are the source facts. Explicit corrections can repair
a known field, receiver name or reviewed single parent; supplemental Ketho/user
declarations can fill a known gap. They must not silently override conflicting
Blizzard facts or each other. Keep exact selected source, expected old value,
replacement and reason. If a target changes, retain the original and report the
unapplied correction. Unknown input does not become a guessed correction.

Keep correction acceptance separate from final declaration emission. A guarded
receiver rename can be accepted while the resulting class later conflicts with
another selected name and cannot be emitted. Report both outcomes and the
omission; `Applied` alone must not be presented as a successfully generated API.
An accepted replacement equal to the old value is not itself a defect.

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

Keep three existing mechanisms distinct. The selected alias catalogs are parsed
with Emmy and combined before dependency validation; `aliases/callable.rs` keeps
the reachable alias definitions used by a callable. Reference's optional
`native_view/type_resolution.rs` records exact source-type candidates and
ambiguity. The annotation `native/type_declarations.rs` attaches uniquely emitted
declarations named by retained type atoms. Neither that attachment nor a resolved
Reference candidate proves the whole emitted Library has no missing types.
Reuse these mechanisms; report unresolved dependencies where they affect an
emitted declaration instead of adding another general type-resolution engine.

For a concrete mixin case, Blizzard's
[`C_AuctionHouse.CalculateItemDeposit`](https://github.com/Gethe/wow-ui-source/blob/e7aa82019dc298e0351c7159d60c2a9b916b9b67/Interface/AddOns/Blizzard_APIDocumentationGenerated/AuctionHouseDocumentation.lua)
accepts `Type = "ItemLocation"` with `Mixin = "ItemLocationMixin"`.
The separate [`ItemLocation.lua`](https://github.com/Gethe/wow-ui-source/blob/e7aa82019dc298e0351c7159d60c2a9b916b9b67/Interface/AddOns/Blizzard_ObjectAPI/Mainline/ItemLocation.lua)
defines the constructor and mixin methods. `FieldFact` and `native/fields.rs`
currently retain the type but omit the typed mixin association. A selected
supplemental declaration may supply useful methods independently; this does not
mean the generated descriptor's relationship has been preserved. Keep that
relationship and its origin without treating every mixin as inheritance.

## Output and source locations

Emit normal inert `.lua` declarations once. Keep extra API context/restriction
facts and source maps beside them when a real consumer needs them. A user-supplied
plain Lua Library remains supported for types without a generator certificate.

Metadata is useful only if the check reader receives it. At the baseline,
sidecars contain field names/source links; values remain in raw documents.
`local/native_artifact/wire.rs::LibraryBinding` deserializes `metadata_sidecars`
as `IgnoredAny`; raw-field retention in `source-report.json` does not establish
typed WoW semantics. Emit and read only the typed context, restriction and
origin fields selected checks use. Read these beside the
generated Lua without reopening the original checkout or regenerating facts.
Absent metadata disables the affected WoW check, not ordinary Lua typing.

Source-map support also has a specific consumer boundary. `NavigationIndex`
supports exact bidirectional lookups, including external source scopes. The
old `native_library_input` requalification path rebuilds maps from retained
generation input and checks their identities; it does not read a ready-to-use
map directly from an ordinary Library directory. `NativeApiTargetResolver`
accepts a unique Blizzard declaration mapping and refuses external-alias or
other-precision mappings for that API-target operation. Preserve those checks
and honest unsupported outcomes. Wire useful maps into direct Library reads
without regenerating the Library or relabeling an alias as a Blizzard API.

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
