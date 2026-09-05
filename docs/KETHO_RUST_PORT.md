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
| `wowdoc/loader/init.lua` | `wow-reference`: selected TOC corpus, documentation systems, separate ScriptObject output, correction dispatch | Native source-to-renderer integration pending |
| `wowdoc/loader/doc_widgets.lua`, `patches.lua`, `TypeDocumentation.lua`, `luasrc/custom_doc` | reference-owned type/widget mapping and reviewed corrections | Port required; no permanent build assumptions |
| `luasrc/annotate/literals.lua` and the enum/event/CVar paths invoked by `luasrc/init.lua` | typed enum/event/CVar data and annotation projection | Port required |
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
