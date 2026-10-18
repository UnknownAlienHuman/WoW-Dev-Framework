# Native type and signature lowering

Read the [Ketho map](../../docs/KETHO_RUST_PORT.md) for donor/source selection.
The current owners are `src/native.rs`, `src/native/{fields,numbers}.rs`,
`src/ketho.rs`, `src/ketho/{callbacks,mappings}.rs`, `src/aliases/` and
`src/literals.rs`. This page describes their useful behavior; it does not
require a consumer-neutral type framework or a profile per output feature.

## Values and names

| Input | Current projection |
| --- | --- |
| `bool`, `cstring`, `luaIndex` | `boolean`, `string`, `number`. |
| Supplied enum name | `Enum.<name>`; membership comes from input, not a built-in client list. |
| Other valid named type | Keep the name. Emission alone does not prove that its declaration resolves. |
| `InnerType` | Array of the lowered inner type, with required union parentheses. |
| `Nilable` or a present default | Ketho's optional marker at the relevant parameter/field/return position. `false` and zero are present defaults. |
| Unsupported expression/type shape | A located issue or rendering error; never a fabricated type or convenient `any`. |

Keep default presence, nilability and ordered return positions in the input
facts. Do not infer optionality from prose or invent an absent return value.
The emitter's `?` convention does not erase those source distinctions.
Defaults are inert display text, not Lua to execute. Unresolved scalar input
must remain visibly unresolved; a source label is not a resolved value.

The existing type lowering accepts bounded unions of valid atoms and preserves
their order; duplicate lowered atoms reject. Reuse it and the alias dependency
resolver. Do not add arbitrary generics, intersections, inferred anonymous
records or synthetic Secret wrapper types to complete an old specification.
Source restriction metadata remains available separately.

## Signatures and output

- Preserve argument, field and multiple-return order. A tuple of returns is
  distinct from a table or a union; keep union/array/nullable precedence.
- `Table::Callback` emits Ketho's argument-only `FunctionContainer|fun(...)`.
  `CallbackSignature` carries additional ordered returns, arrays and a terminal
  argument pack. Unsupported callback variadic returns/defaults reject.
- A terminal `StrideIndex` is variadic; an array is not. Receivers and namespace
  names come from supplied ownership and reviewed corrections. Do not infer
  inheritance, globals or method signatures from name conventions.
- Documentation is data. Preserve the difference between absent and empty
  documentation; escape supported prose controls and reject directive injection.
  Prose cannot change a type, default, restriction or declaration identity.
- Pure rendering returns no artifact on failure. Native composition reports
  invalid declarations and keeps independent valid neighbors with their maps.
  Output/item limits and cancellation must remain effective after assembly.

## Change and check

Use the relevant existing case in `tests/ketho.rs`, `tests/callback_signatures.rs`,
`tests/native.rs` or the selected alias/literal test. Check a useful output and
its actual failure case, including a valid neighbor when changing admission.
For changed consumer semantics use the [focused probes](../../docs/ANNOTATION_CONSUMER_PROBES.md).
No full-corpus, permanent pin/probe or checksum-freeze programme is required for
a bounded refactor. [Source maps](SOURCE_MAPS.md) describe navigation precision.
