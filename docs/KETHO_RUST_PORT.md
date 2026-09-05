# Ketho annotation service: Rust port

Ketho/vscode-wow-api is the primary implementation donor, not just an oracle.
The task is to port its useful WoW annotation-service behavior into the existing
Rust owners, not to invent a separate Python extractor or rewrite a language
server. Gethe remains the current Blizzard data source. EmmyLua/LuaLS consume
annotation libraries; the framework's semantic analyzer remains behind wow-emmy.

## Implementation map

| Donor surface | Rust responsibility | Current scope |
|---|---|---|
| `luasrc/annotate/init.lua` | `wow-annotations`: GetType, GetField, GetFunction, GetTable, GetCallbackType, GetSystem | Pure emitter implemented |
| `wowdoc/init.lua`: GetBaseName/GetArguments/GetFullName | `wow-annotations`: globals, namespaces, ScriptObject receivers and varargs | Implemented with explicitly supplied widget alias |
| `wowdoc/loader/init.lua` | `wow-reference`: selected TOC corpus, documentation systems, separate ScriptObject output, correction dispatch | Native declarative input, typed normalization and emitter integration implemented; correction dispatch pending |
| `wowdoc/loader/doc_widgets.lua`, `patches.lua`, `TypeDocumentation.lua`, `luasrc/custom_doc` | reference-owned type/widget mapping and reviewed corrections | Port required; no permanent build assumptions |
| `luasrc/annotate/literals.lua` and the enum/event/CVar paths invoked by `luasrc/init.lua` | typed enum/event/CVar data and annotation projection | Native event/enum/constant projection connected; external CVar/resource acquisition pending |
| `luasrc/WikiParser`, `wowdoc` resource acquisition, TypeScript resource export | explicit enrichment and equivalent native query data where required | Port/select by feature parity, not a mandatory extra runtime |
| VS Code activation and LuaLS configuration | thin host adapter over an editor-independent Rust service | Port behavior, not an extension/Node dependency |

The reviewed donor revision is `d0b5b51fac4c52c493371b9b18e66ce604ea4326`.
It identifies this port's evidence only. Re-resolve the donor branch before the
next porting task, and revalidate patch-sensitive mappings against current Gethe.

## Executable emitter slice

The library consumes an ordered `System` with explicit `Owner`, `Function`,
`Table` and `Field` data. No raw Lua, JSON interpretation, reference lookup or
source acquisition occurs in the renderer. It emits inert Lua annotation stubs.
The current public model is the renderer input slice, not the complete E1
AnnotationSemanticModel or a replacement for the ReferenceView boundary.

Preserved donor behavior:

- bool/cstring/luaIndex type aliases; explicit enum membership becomes Enum.Name;
- InnerType arrays, Nilable and present default values (including false and zero);
- positional parameters and multiple returns; terminal StrideIndex varargs;
- separate namespace functions and colon-receiver ScriptObject methods;
- structure fields and argument-only FunctionContainer callback aliases;
- documentation ordering, wiki links, and the analysis-only meta header.

Two committed output vectors were compared with the actual reviewed Ketho emitter
(blob `1f0b902a809472c30b28a0e220b70235b29235b8`) in an isolated local Lua probe:
a namespace system and a widget system. Only the reviewed emitter and a donor-derived pure helper harness
ran, with synthetic inputs; the network loader and generated stubs were not run.
Rust tests compare these bytes without requiring a Lua interpreter or Python.
This is scoped donor parity, not full Ketho parity or an EmmyLua/LuaLS load test.

The renderer deliberately rejects duplicate names, nonterminal varargs, unsafe
identifiers, control-bearing/directive-like documentation, unsupported type
expressions and callback arrays rather than copying malformed or lossy output.
The initial callback surface is argument-only: adapters must report unsupported
return/restriction metadata instead of omitting it. The complete E1 loss sidecar
is still required before arbitrary reference facts can use this renderer.
Input counts/text lengths and total emitted bytes are bounded. Errors return no
artifact. Widget/enum inventories are caller-supplied, never global static truth.

## Migration order

1. Extend the Rust reference input path using the donor loader/correction model;
   keep system/namespace/ScriptObject ownership and exact source provenance.
2. Connect that typed reference input to the Rust emitter with source maps and
   explicit projection loss. Port literals and the remaining useful Ketho output.
3. Validate the resulting libraries in both current compatible EmmyLua and LuaLS.
4. Replace the corresponding legacy Python producers, consumers and CI invocations
   with native Rust tests/commands, then remove those legacy files. Do not extend
   the Python pipeline or route the native renderer through it as the product.
5. Expose the native service through the planned CLI and agent/editor adapters.

Do not replace the pipeline with another custom Lua parser. Use the selected
upstream Rust Lua frontend behind its owner adapter. Loading arbitrary source via
Lua `loadfile` is not carried over from the donor. The end-user product must not
require Python, Node, a VS Code installation, or a Lua runtime to generate its
annotations. No full service, complete E1-C, or public release is claimed yet.

## Literal-generator port

`wow-annotations::literals` ports Ketho's `GetEventLiterals`, `GetCVarLiterals`
and `GetEnumTable` output from caller-supplied typed data. Event payload display
text is supplied by the reference owner; this emitter does not infer payload
signatures. The donor's open `FrameEvent string` and `CVar string` aliases are
preserved, not turned into exhaustive runtime whitelists. Enums and Constants
stay separate and fields preserve boolean, integer, finite-number or string
identity. Wide numeric-looking enum strings are not parsed or reformatted.

`IntegerFormat` and `MemberOrder` make presentation choices explicit. The Rust
renderer contains no special case for any Blizzard enum/constant name, no static
inventory, source repository, flavor, build or Interface value. Data acquisition
(including donor resource sources beyond Gethe) remains outside this library and
is not silently enabled by porting the renderer.

Three additional golden files were produced by the exact reviewed donor literal
module, blob `1c79f0e9c92a9836218938a34244540db2a999e6`, with isolated synthetic
inputs corresponding to `tests/literals.rs`. Network/resource loading was
replaced by an in-memory test harness; only reviewed pure rendering ran. Rust
tests now compare those committed event, CVar and enum/constant bytes offline.
These are scoped donor-output probes, not live-source or language-server probes.
See `crates/wow-annotations/tests/golden/README.md` for identities and differences.

Explicit differences from donor rendering:

- equal boolean values use a stable member-name tie-breaker;
- every string literal is escaped; non-ASCII/control UTF-8 bytes use Lua 5.1
  decimal escapes, preventing strings from creating physical annotation lines;
- unsafe event payload comments and duplicate names reject the whole result;
- decimal/hex and constant-group ordering are caller policies, not known-name
  heuristics; a negative or boolean member never changes later numeric formatting;
- integral scalar inputs outside the conservative Lua 5.1 exact-integer interval
  reject instead of rounding or being silently converted to string;
- finite fractional constants are supported; nonfinite values and unprobed
  fractional enum declarations reject explicitly;
- input and output size bounds apply before successful artifact return.

The pure emitter alone does not provide the full E1 artifact contract. The native
connection below now adds source links and raw/metadata sidecars; full E1 fine-grained
maps and semantic consumer compatibility remain incomplete.
Unsupported input fails with no output; the future reference adapter must turn
that failure into its explicit projection status, not drop a field or use `any`.


## Native source-to-library path

The restricted evaluator and typed loader model are implemented in
`wow-reference/src/native.rs` and `native_model.rs`; the in-memory projection is
`wow-annotations/src/native.rs`. This follows the documented E1-B/E1-C owner
split: the external EmmyLua Rust parser is used by the reference owner, not a
second lexer/parser or a Lua runtime. The separate semantic analyzer adapter is
not claimed implemented by using its upstream syntax frontend.

```text
exact selected Git revision + documentation TOC
-> EmmyLua Lua 5.1 syntax + restricted declarative evaluator
-> raw ordered fields, literals, symbolic references and UTF-8 byte spans
-> typed systems/owners, callables, tables classified by Type, events
-> Ketho Rust callable/structure/callback/literal emitters
-> annotation files, source mappings and raw/metadata/error report
```

Run the native development driver (Git and the compiled Rust executable only):

```sh
cargo run -p wow-annotations --example native_library -- \
  /path/to/wow-ui-source HEAD \
  Interface/AddOns/Blizzard_APIDocumentationGenerated/Blizzard_APIDocumentationGenerated.toc \
  Mainline /path/to/new-output
```

The output directory must not exist. Exit 0 means selected declarations projected
with explicit sidecars, 3 means partial input/projection with a report, and 2 is
an operation failure. Neither 0 nor 3 certifies reference completeness or consumer
compatibility. `source-report.json` is written last; failed filesystem writes can
leave an incomplete directory and never constitute successful publication.
This development driver is not the service-owned public `wow` CLI or the full
persistent ReferenceView implementation.

The caller selects the flavor, ref, TOC and environment. The driver resolves the
ref once, reads Git blobs from that revision rather than dirty files, and records
`not_network_verified` freshness. Materialize a partial clone's missing objects
explicitly before using it. It never auto-fetches, executes Lua, follows XML load
entries as Lua, mutates a checkout, or enables additional resource providers.

Functions preserve namespace/ScriptObject ownership; enum/constant kinds are
selected from the source Type, including entries inside Tables. Duplicate exact
callable/type/event identities are excluded as conflicts, never first/last-wins.
One corpus-wide literal file per lane avoids redefining FrameEvent or enum roots
for every source document. Enum membership is read from this input, not compiled
into a list. Constants are read from Blizzard's `Values` collection, not enum
`Fields`. Their descriptor `Type` distinguishes an enum member label from an
ordinary string. The reference-owned scalar catalog resolves unique transitive
Enum/Constants references and additive integer expressions from this same selected
corpus, independently of document order. Unknown metadata, raw numeric lexemes,
explicit nil, unresolved names and original expressions remain in raw sidecars.
Unrendered source fields have
individual metadata links; unsupported constructs have explicit error records.
The evaluator profile is `ketho-apidoc-declarative/2`; the native library report
is `wow-native-annotation-library/3`. `scalar_resolutions` links every requested
constant/default resolution to its source value and, on success, every transitive
value used. Conflicts, cycles, absent references, cancellation and limits have
separate error classes. Bare unknown names in data are captured, never looked up
in a host environment or substituted with nil/zero. Invalid registration roots
and executable expressions still fail admission.

Only `+` and `-` are admitted beyond the previous declarative subset, using the
existing Emmy AST. Arithmetic requires exact integer operands and intermediate
results within ±(2^53−1); string coercion, fractional arithmetic and signed-zero
arithmetic are rejected rather than rounded. Direct scalar number lexemes remain
unchanged. Raw data, including the expression, is not overwritten by resolution.

A failed constant is reported at its own value span and does not erase other
members. A failed callable/structure is excluded individually before final mapped
rendering, preserving valid neighbors. The report stays partial; a missing value
is not a claim that the API or constant does not exist. Numeric results are never
borrowed from another flavor, an installed extension or a runtime global cache.
Resolution depth, steps, declaration count and accumulated evidence are bounded.
No negative/absence or runtime-safety authority is issued by this path.

Declaration maps bind final generated byte ranges and file hashes to exact source
ranges/hashes. Literal maps are explicitly whole-file mappings, not fine-grained
member maps. The Ketho nilability/default convention is preserved while raw fields
remain distinguishable. Numeric defaults retain the source lexeme in comments;
unsupported literal numeric forms are reported rather than rounded.

Deliberately unsupported in this bounded evaluator/projection: general Lua code,
mutation/control flow/helper execution, computed keys, numeric/hex/Unicode string
escapes requiring byte-string semantics, CR-bearing long-string normalization,
callback returns/arrays not supported by the current emitter, and unrepresentable
numbers/types. Restriction metadata remains advisory sidecar data, not invented
runtime wrapper types. Corrected widget aliases and named-type closure still need
the Ketho correction/type-resource port. No generated body is executable addon logic.

Rust tests exercise the actual source-to-renderer connection, source identity,
raw metadata, enum ownership, duplicate conflicts, UTF-8 ranges, bounded input,
malicious source rejection, cancellation, deterministic ordering, local Git/TOC
reading and no-clobber output. Parsing a generated fixture with EmmyLua is a
syntax check only. Real EmmyLua/LuaLS semantic consumer probes remain required.

The reconciled native adapter records collision-checked reserved return labels in
`name_projections` (source path/hash/span, original, rendered, rule). Original
labels and order remain in raw source; executable parameter identifiers remain
strict. Duplicate labels are rejected before renaming. These records are emitted
only for admitted callables, never for declarations rejected by the renderer.
The report is version 3; it preserves the version-2 `scalar_resolutions` field.

Control characters in source prose become visible escapes on one inert comment
line, with a `Documentation:escaped_control_characters` metadata link. Both input
and expanded text are bounded. Annotation directives on any source line remain
rejected; this is an explicit safety/consumer projection, not a Blizzard fact.

Enum/constant groups are validated independently before aggregation. Duplicate
members produce per-member conflict issues and do not erase valid siblings.
Malformed groups cannot abort unrelated valid groups. Enumeration `Values` and
mixed `Fields`/`Values` forms fail normalization rather than producing empty enums;
Constants still require the existing `Values` shape. These are regressions in
`wow-annotations/tests/native.rs` and `wow-reference/tests/native.rs`.

Next: port reviewed correction/type/widget mappings and remaining consumer-specific
type losses, then run real dual-consumer probes. Runtime-only global defaults and
unsupported numeric/type expressions remain explicit unresolved cases.
Retire legacy Python paths only after their native replacements cover the same
verified use cases; never extend that legacy product pipeline.
