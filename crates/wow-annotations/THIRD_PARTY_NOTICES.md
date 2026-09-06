# Third-party notice

`src/ketho.rs` ports the annotation behavior of Ketho/vscode-wow-api:
`luasrc/annotate/init.lua` and the naming helpers in `wowdoc/init.lua`.
`src/literals.rs` ports `luasrc/annotate/literals.lua` rendering, with
explicit formatting policies, deterministic ties and escaped string values.
The committed synthetic golden outputs are derived from that renderer.
Reviewed revision: d0b5b51fac4c52c493371b9b18e66ce604ea4326.
Source: https://github.com/Ketho/vscode-wow-api

The native loader/projection connection follows the same donor's
`wowdoc/loader/init.lua`, system/member naming and literal-generation behavior.
It uses the EmmyLua Rust syntax frontend; no source Lua is executed.

## Ketho — MIT License

Copyright (c) 2020 Ketho

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.

The optional native correction connection consumes the reference-owned port of
`wowdoc/loader/patches.lua` and `doc_widgets.lua`; see the reference crate's notice.
The renderer's bounded union support also covers the donor's `string|number`
correction without copying source execution or arbitrary type syntax.

The combined ScriptObject library profile follows the class/local-table/method
representation in `Annotations/Core/Widget/Frame/Frame.lua` at the reviewed
revision above (blob `330d65b107817f0bd0914692e8d9f461285240fc`). No widget inventory,
base classes, case aliases or handwritten methods from that file are imported.
Receiver names come from selected source facts and explicit guarded corrections.
Synthetic receiver tests check this representation and source mapping, not a
byte-for-byte reproduction of the donor's complete handwritten class catalogue.
