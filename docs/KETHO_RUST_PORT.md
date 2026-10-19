# Native Ketho annotation path

Ketho/vscode-wow-api is the annotation behavior and output donor; Gethe supplies
the selected Blizzard UI source. The reviewed comparison revisions are Ketho
`d0b5b51fac4c52c493371b9b18e66ce604ea4326` and EmmyLua
`aaaca68425d9362876228649b0b8d92f07654daa`. These identify evidence for this
review, not permanent WoW truth: resolve the intended source revision and
client flavor for source-sensitive work. Credit Ketho's MIT-licensed derivative
as recorded in [`THIRD_PARTY_NOTICES.md`](../crates/wow-annotations/THIRD_PARTY_NOTICES.md).

## Reuse the current owners

| Donor surface | Existing Rust owner | Useful boundary |
|---|---|---|
| `luasrc/annotate/init.lua`, `wowdoc/init.lua` | `wow-annotations/src/ketho.rs`, `ketho/callbacks.rs` | Typed emitter, receivers, callback signatures. |
| `wowdoc/loader/init.lua`, `patches.lua`, `doc_widgets.lua` | `wow-reference/src/native{,_model,_corrections}.rs` | Restricted source evaluation, normalization, guarded Type/Nilable/receiver and selected base corrections. |
| `luasrc/annotate/literals.lua` | `wow-ketho-literals`; `wow-annotations/src/literals.rs` | Native event/CVar aliases and enum/constants from explicit typed data. |
| `Annotations/Core/Type/{BlizzardType,StringEnum,EmoteToken,UnitToken}.lua` | `wow-reference/src/native_aliases{,/catalog,/open_strings}.rs`; `wow-annotations/src/aliases.rs` | Named/primitive, closed-string and open-string aliases, maps and losses. |
| `Annotations/Core/Type/{Structure,Mixin,Namespace,FunctionContainer,GlobalColors}.lua` | Matching `native_aliases/` and `aliases/` modules | Selected structures, namespace bindings, static declarations and colors. |
| `Annotations/Core/Widget/...` | `wow-annotations/src/native.rs`; `wow-reference/src/native_corrections/inheritance.rs` | Source-bound class/alias and explicit guarded single-parent corrections. |
| Generated/source navigation | `wow-annotations/src/navigation/`; [consumer probes](ANNOTATION_CONSUMER_PROBES.md) | Maps and representative semantic-consumer checks. |

Reuse these owners and their one native source-to-library composition. Do not
add a second importer/parser, Lua/Node/Wasm runtime, custom full Lua language
server, or a separate pipeline/report format for every optional feature.
`wow-emmy` uses EmmyLua's public semantic-analysis APIs; upstream LSP handlers
live in `emmylua_ls` and are private to that crate. Use its server for ordinary
editor features. The Ketho output is a normal inert `.lua` analysis library
that users can add to an existing LuaLS/EmmyLua library path; VS Code is not a
runtime or distribution requirement.

## Native source-to-library flow

The existing Git driver reads committed regular blobs from one resolved source
revision and the selected TOC/flavor, then uses EmmyLua Lua 5.1 syntax,
`wow-reference`'s restricted evaluator and typed facts, and the shared
`wow-annotations` emitter. It retains raw expressions/unknowns, revision and
source paths, UTF-8 spans, source maps, and losses. Source, hooks, and
generated stubs are never executed. Current source/API meaning must be checked
against the selected Blizzard revision; do not hard-code a moving build or
assume Mainline data covers another flavor.

Development driver:

```sh
cargo run --locked -p wow-annotations --example native_library -- \
  /path/to/wow-ui-source <resolved-ref> \
  Interface/AddOns/Blizzard_APIDocumentationGenerated/Blizzard_APIDocumentationGenerated.toc \
  Mainline /path/to/new-output \
  --corrections /path/to/reviewed-pack.json \
  --alias-catalog /path/to/ketho <resolved-donor-ref> Annotations/Core/Type/BlizzardType.lua
```

Corrections and alias catalogs are independent optional inputs; catalogs may
repeat. Refs resolve once; only committed regular blobs are read. Missing Git
objects must be materialized through the existing source workflow. The output
directory must be new. Exit 0 is projected, 3 partial, 2 failure;
`source-report.json` is written last. A partial report remains useful evidence,
but does not prove unsupported or missing declarations absent. The example is
development tooling; #196 exposes this same generation through a supported
CLI/builder command. Reuse the native flow instead of adding another writer.
Generate the library once; regenerate derived files when source, corrections,
catalogs or projection semantics change. #204 removes the native adapter's
discarded declaration/registration trial renders by retaining emitter output
and ranges. Independent valid declarations must survive a rejected neighbor;
do not introduce a second validator or type model.

## Semantics worth preserving

The emitter consumes ordered typed systems, owners, functions, tables, and
fields; it does not parse source or decide Reference authority. Preserve bool,
cstring, and luaIndex aliases; supplied enum membership; InnerType arrays;
nilability; present defaults including `false` and zero; ordered arguments and
multiple returns; terminal StrideIndex; namespace/colon receivers; documentation
order; and analysis-only meta output. `Table::Callback` stays argument-only;
`CallbackSignature` handles ordered returns, arrays, and terminal argument
packs. Tuple/union/nullable precedence is intentional. Unsupported variadic
returns/defaults reject explicitly.

Identifiers, duplicate names, unsafe directives, and unrepresentable types must
not silently widen to `any`. Pure rendering fails without an artifact; native
composition reports a declaration-level failure while retaining independent
valid declarations. Unsupported forms remain visible in the report. See
[callback cases](../crates/wow-annotations/tests/callback_signatures.rs) and
[type lowering](../crates/wow-annotations/TYPE_LOWERING.md). The
[map guide](../crates/wow-annotations/SOURCE_MAPS.md) distinguishes member,
declaration and coarse literal mappings; unprojected raw metadata already remains
in the native result.

Literal generation consumes supplied data: FrameEvent/CVar remain open string
aliases; Enum/Constant identity, booleans, integers, finite numbers, strings,
and raw numeric lexemes remain distinct. Ordering/format are explicit policies.
Reject duplicate members and unsafe payload text; absence is not a runtime
whitelist. Do not parse numeric-looking strings or invent event payloads.
Integral values beyond ±(2^53−1), non-finite values, and unsupported
fractional Enums reject; finite fractional Constants are allowed.

Alias catalogs support selected named/primitive unions, closed literal unions,
open strings with hints, structure fields, exact empty `C_*` bindings, static
FunctionContainer declarations, and inert CreateColor declarations. Resolve
dependencies within bounds while retaining duplicates, cycles, and missing
types; never copy expanded signatures or default unknowns to `any`. Wiki
`TypeDocumentation.lua` replacements are not annotation resources. External
catalogs are overlays and cannot overwrite Blizzard facts. Preserve the existing
resource bounds and third-party notice.

Class binding combines a source class, local receiver, and methods. Guarded
renames retain an alias for the original type. Explicit single-parent
corrections exist; do not infer widget hierarchy, shorten names by convention,
copy transitive methods, or infer aliases from donor folders. Preserve exact
correction guards/outcomes and raw observations. The restricted evaluator
supports selected scalar references and bounded exact integer `+`/`-`; cycles,
absence, unresolved values, cancellation, and limits stay distinct. General Lua,
computed keys, unsupported string normalization, and arbitrary arithmetic are
outside its scope.

## Work and qualification

Trace the current owner and real consumer before adding behavior. Replace a
duplicated implementation in the same slice. For a changed semantic, use one
representative positive and negative consumer check where available. A whole
corpus/editor qualification is needed only when making that release coverage
claim, not for each bounded patch. Existing synthetic probes do not certify all
addons, runtimes, or editors. Full parity, arbitrary mixins/custom signatures,
complete widget inheritance, and whole-corpus loading remain unclaimed until
their specific acceptance work is done.
