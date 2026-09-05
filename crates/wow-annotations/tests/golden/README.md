# Ketho renderer vectors

These are synthetic output fixtures, not current Blizzard data, consumer-probe
results, an API completeness claim or a permanent upstream version selection.
Expected bytes are committed and must not be rewritten by tests.

The donor revision inspected for these vectors is
`Ketho/vscode-wow-api@d0b5b51fac4c52c493371b9b18e66ce604ea4326`.
`namespace.lua` and `widget.lua` cover the callable/structure/callback emitter;
its input models are in `tests/ketho.rs`.

Literal vectors use `luasrc/annotate/literals.lua` with Git blob
`1c79f0e9c92a9836218938a34244540db2a999e6` and SHA-256
`db9c4da57686030aed81a7f5c5257d847e5d15e4ddc4a17025761cde9aba55ec`.
The observed donor output received the standard `---@meta _` file prefix.
Input equivalents are `events()`, the CVar test and `sample()` in
`tests/literals.rs`.

| File | SHA-256 |
|---|---|
| `events.lua` | `e3ca9b55708399845e9db48082c6fe58658efe22778feb6fb1ed8d21001cac4f` |
| `cvars.lua` | `9f32fdbd5f28ea9e50b8007e28fe4ce29d865702e41e00ba838e87be710a6a46` |
| `enum-constants.lua` | `0e97d4910ef10d06cef322e410b4a3768a53e7dc6facd2979267ef8a460575ec` |

The manual donor probe ran the exact blob using Lua 5.4 and synthetic globals.
The `wowdoc` resource loader was replaced with in-memory values; no download,
external resource Lua, addon, generated output or source repository script ran.
Event `GetPayloadString(false, false)` was supplied by a test double, so this
probe checks rendering of payload text, not derivation of that payload. Sorting
helpers were restricted pure equivalents for the tested string-key and numeric
value cases. CI needs only Rust to verify the resulting golden bytes.

The enum fixture covers negative/zero/aliased integers, explicitly selected hex
format, boolean values, string-valued wide numbers, and constant groups ordered
by name and by value. That last choice is explicit Rust input, not a hardcoded
special-case name. The remaining Rust tests exercise deterministic tie-breaking,
escaping, unsupported values, input limits and output-budget boundaries where
exact Ketho byte parity is neither safe nor the intended contract.

This is scoped donor parity only. Real EmmyLua and LuaLS interpretation, type
inference, negative diagnostics and source-map validation remain separate gates.
Third-party terms are retained in `../../THIRD_PARTY_NOTICES.md`.
