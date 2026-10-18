# Ketho annotation service: Rust port

Ketho/vscode-wow-api is the implementation and output donor. Gethe supplies
Blizzard source; `wow-reference` admits and normalizes it, `wow-annotations`
projects inert annotations, and `wow-emmy` owns semantic analysis. Reuse these
owners. No second extractor, Lua runtime, Node dependency or language server.

This is the current route, checked against code on 2026-10-17. Earlier port
checkpoints remain in [Git history](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/blob/888fd087f9ede57afed02fae34e4fe99c716a1cb/docs/KETHO_RUST_PORT.md).
Choose work from [the queue](REFACTOR_QUEUE.md); read the matching row and owner
contract below.

## Implementation map

The reviewed donor is [`d0b5b51fac4c52c493371b9b18e66ce604ea4326`](https://github.com/Ketho/vscode-wow-api/tree/d0b5b51fac4c52c493371b9b18e66ce604ea4326).
It identifies comparison evidence, not permanent current WoW truth. Resolve the
donor again for source-sensitive porting; internal refactors use selected pins
and retained inputs. Preserve [third-party notices](../crates/wow-annotations/THIRD_PARTY_NOTICES.md).

| Donor surface | Existing Rust path | Current boundary |
|---|---|---|
| `luasrc/annotate/init.lua`, `wowdoc/init.lua` helpers | `wow-annotations/src/ketho.rs`, `ketho/callbacks.rs` | Typed emitter, receiver naming and richer native callback signatures exist. |
| `wowdoc/loader/init.lua`, `patches.lua`, `doc_widgets.lua` | `wow-reference/src/native{,_model,_corrections}.rs` | Restricted input, normalization, guarded Type/Nilable/receiver and selected single-base correction exist; full resource/correction parity remains open. |
| `luasrc/annotate/literals.lua` | `wow-annotations/src/literals.rs` | Native event/CVar aliases and enum/constants from explicit typed data; external CVar/resource acquisition remains separate. |
| `Annotations/Core/Type/{BlizzardType,StringEnum,EmoteToken,UnitToken}.lua` | `wow-reference/src/native_aliases{,/catalog,/open_strings}.rs`; `wow-annotations/src/aliases.rs` | Named/primitive, closed-string and open-string aliases; cross-file resolution and explicit losses exist. |
| `Annotations/Core/Type/{Structure,Mixin,Namespace,FunctionContainer,GlobalColors}.lua` | Matching `native_aliases/` and `aliases/` modules | Selected structures, namespace bindings, static empty methods and color declarations exist. Full arbitrary mixin/method semantics do not. |
| `Annotations/Core/Widget/...` class/local binding | `wow-annotations/src/native.rs`; `wow-reference/src/native_corrections/inheritance.rs` | Source-bound class/alias and guarded single-parent projection; no inferred donor-folder hierarchy or runtime method copying. |
| Generated maps and semantic consumers | [Annotation owner](../crates/wow-annotations/README.md#generated-to-source-navigation), [consumer probes](ANNOTATION_CONSUMER_PROBES.md) | Bidirectional/range/buffer navigation and two synthetic dual-consumer semantic slices exist; whole-corpus/editor certification remains open. |
| Wiki/resources, VS Code activation/settings | Select a concrete remaining feature through existing owners | Wiki tables are not annotation contracts. No hidden network acquisition, extension installation or diagnostic suppression. |

## Executable emitter slice

The emitter consumes ordered typed systems, owners, functions, tables and fields.
It does not parse Lua/JSON, acquire source or infer Reference authority. Preserve
bool/cstring/luaIndex aliases, caller-supplied enum membership, InnerType arrays,
nilability and present defaults (including false/zero), positional arguments and
multiple returns, terminal StrideIndex, namespace/colon receivers, documentation
ordering and analysis-only meta output.

The original `Table::Callback` remains argument-only donor compatibility;
`CallbackSignature` supports ordered returns, arrays and terminal argument packs.
Both share bounded lowering. Preserve tuple/union/nullable precedence;
unsupported variadic returns/defaults reject explicitly. Read
[callback cases](../crates/wow-annotations/tests/callback_signatures.rs) and
[type lowering](../crates/wow-annotations/e1/TYPE_LOWERING.md) for that change.

Invalid identifiers, duplicate names, unsafe directives and unrepresentable types
must not silently widen to `any`. Pure-renderer failure yields no artifact; native
composition records per-declaration failure and preserves independent valid data.
The original emitter/literal vectors have [golden provenance](../crates/wow-annotations/tests/golden/README.md).
They prove scoped donor output, not full E1 acceptance.

## Migration order

1. Trace the current native owner and real consumer before adding a port.
2. Replace duplicated parsing, lowering or representation in that path; remove
   the old implementation in the same slice.
3. For a missing source/type feature, reuse the resource parser, normalization and
   emitter; retain exact maps, losses and authority boundaries.
4. Use one existing reproduction after the change works. Current-source and
   dual-consumer campaigns belong to their selected acceptance gate.

Legacy wire readers remain until their consumer/migration decision is recorded.
Retired interpreter producers are not a fallback. Native XML/TOC and remaining
E1 work follow their owner contracts, not a new parser.

## Literal-generator port

`GetEventLiterals`, `GetCVarLiterals` and `GetEnumTable` consume supplied data.
FrameEvent/CVar stay open string aliases. Enum/Constant identity, booleans,
integers, finite numbers, strings and raw numeric lexemes remain distinct.
Format/order are explicit policies, never name/build heuristics.

Strings are escaped; equal booleans use stable name ordering. Duplicate members
and unsafe payload text reject; absence never becomes a runtime whitelist.
Integral values outside ±(2^53−1), nonfinite values and unsupported fractional
Enums reject; finite fractional Constants are allowed. Do not parse
numeric-looking strings or invent missing event payload signatures.

## Native source-to-library path

The Git driver reads one resolved commit and TOC, then uses Emmy Lua 5.1 syntax,
Reference's restricted evaluator/typed facts, and the existing emitter. It keeps
raw fields, expressions, unknowns, revisions, hashes and UTF-8 spans. No source,
loader hook or generated stub executes.

```sh
cargo run --locked -p wow-annotations --example native_library -- \
  /path/to/wow-ui-source <resolved-ref> \
  Interface/AddOns/Blizzard_APIDocumentationGenerated/Blizzard_APIDocumentationGenerated.toc \
  Mainline /path/to/new-output \
  --corrections /path/to/reviewed-pack.json \
  --alias-catalog /path/to/ketho <resolved-donor-ref> Annotations/Core/Type/BlizzardType.lua
```

The last two options are independent and optional; `--alias-catalog` is repeatable.
Refs resolve once; only committed regular blobs are read. No auto-fetch, dirty
file/symlink/submodule fallback or current-build inference. Materialize missing
Git objects explicitly. Output must be new. Exit 0 means projected with sidecars,
3 partial, 2 failure; none certifies consumer compatibility. `source-report.json`
is written last; an incomplete directory is not publication. This development
driver is separate from the Service-owned public CLI/builder.

Constants use source `Values`, enums their declared `Fields`; duplicate identities
remain conflicts. Scalar resolution follows selected-corpus references and
bounded exact integer `+`/`-`, preserving transitive evidence. Cycles, absence,
unresolved names, cancellation and limits remain distinct; no host globals,
string coercion or nil/zero substitution. General Lua, computed keys, unsupported
byte-string escapes/long-string normalization and arbitrary arithmetic remain
outside this evaluator. One failed member does not erase independent declarations.

Final declaration and ordered-member maps bind generated/source ranges and hashes;
literal maps remain file-level. Original return names, collision-safe projections,
raw restrictions and escaped-control evidence survive. Source prose directives
reject. Full E1 map/type closure and runtime authority are not implied.

## Reviewed correction connection

[Correction usage and schema](KETHO_NATIVE_CORRECTIONS.md) own Type/Nilable,
receiver and selected `widget_base` behavior. Guards include exact source/raw
value and original before-state; mismatches never auto-refresh. Raw observations
stay unchanged; applied/expired/rejected/conflict/not-applicable outcomes survive.
Review strings record supplied evidence, not authenticated review authority.

## ScriptObject class binding

Native projection combines a source class, local receiver and methods. A guarded
rename adds an alias for the original type name. Original/corrected names are
collision-checked across the corpus; invalid receivers do not erase independent
tables/events. Empty valid objects still declare their type.

`render_library_mapped` and standalone `render_mapped` share the emitter. The
standalone donor vectors retain their bytes. Explicit single-parent corrections
exist; complete widget inventory and runtime inheritance do not. No shortening
by convention, inferred aliases, copied transitive methods or donor-folder
identity. See [receiver](../crates/wow-annotations/tests/receivers.rs) and
[inheritance cases](../crates/wow-annotations/tests/inheritance.rs).

## Alias resource connection

Use `ingest_alias_catalog` → `project_with_alias_catalogs` through the shared
Git driver. `ingest_aliases` is the retained narrower single-comment reader.
External catalogs remain annotation overlays with independent scope/revision;
they cannot overwrite Blizzard facts or borrow authority from rejected types.

Current forms include named/primitive unions, closed literal unions, open strings
with hints, selected structure fields, exact empty `C_*` bindings, static
FunctionContainer declarations and `CreateColor` declarations without execution.
Unsupported declarations retain outcomes. Contiguous comments may form catalog
unions; malformed syntax cannot consume independent siblings. Bounded dependency
resolution retains duplicates/cycles/missing types and never expands alias chains
into copied signatures or defaults to `any`. The wiki `TypeDocumentation.lua`
replacement table is a different input from annotation resources.

All resources share one exact donor revision and distinct paths; order does not
change output. Bounds are 32 files, 2 MiB aggregate, 4,096 declarations and 256 KiB
per resource, plus per-shape/output limits. Derivatives keep the MIT notice.
Read [combined catalog scope](../crates/wow-annotations/KETHO_TYPE_CATALOG.md),
[inventory drift](../crates/wow-annotations/KETHO_TYPE_INVENTORY.md) and
[GlobalColors](../crates/wow-annotations/GLOBAL_COLORS.md) for those tasks.
The source workflow already composes the reviewed type directory; do not recreate
it from older checkpoints.

Native reports stay `/3` plain, `/4` corrections, `/5` catalogs, `/6` selected
literal bridge. Optional features already compose through one
`project_with_alias_catalogs_and_literal_bridge`; do not add a pipeline or report
variant for every combination. Resource/projection schemas and verifier checks
need an explicit reader decision before consolidation (#112).

## Selected Wasm literals

Native literals are the default. The optional retained `LiteralBridge` shares
the normalization/correction/catalog path and adds module/call/final-file
bindings. Failure does not fall back. No VM enters the stable owner dependency
graph. [Wasm commands and contract](WASM_BRIDGES.md#source-library-composition)
own this lane; its existence does not imply per-annotation VM cost.

## Remaining completion work

Full E1 requires remaining source/type/custom-signature closure, maps/losses,
complete ReferenceView integration, current-source parity and full consumer
qualification. Existing [dual-consumer probes](ANNOTATION_CONSUMER_PROBES.md)
exercise synthetic signatures and closed string enums with negative/mutation
controls. They do not certify the whole corpus, addons, runtime or editors.
Existing bidirectional navigation is not complete LSP integration. Keep these
boundaries in [the completion matrix](PROJECT_COMPLETION_MATRIX.md).
