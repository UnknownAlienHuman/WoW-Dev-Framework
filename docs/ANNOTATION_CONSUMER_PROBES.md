# Annotation consumer probes

These tests check selected type semantics with real EmmyLua (`emmylua_check`)
and LuaLS command-line consumers. They do not establish full-corpus parity or
that the supported generator-to-addon workflow works.

## Actual producer under test

Application probes live in `apps/wow-reference-builder/tests/consumers.rs`.
Their small fixture helper launches Cargo's built `wow-reference-builder`
executable with `generate`, committed Git/TOC inputs and optional catalogs or
corrections. It is not another source loader or renderer. The previous example
producer, duplicate Git/catalog helpers and obsolete `/1` verifier are removed.

Assertions read the current `/2` report and generated files. Source spans and
file bytes are checked against the committed synthetic inputs; generated Lua
text is not embedded back into a report. Renderer/type/map unit cases remain in
`wow-annotations`.

These are application producer/output probes, not ordinary `wow check` or
proof of full Ketho parity. The actual Project Library reader, normal addon
configuration, maximum-size mismatch and blocking/per-file Git I/O remain #201/#200.
TOC/API/catalog inputs now share the regular-file Git check.

## Useful cases to keep

The signature fixture covers ordered returns, structures, ScriptObject methods,
arrays, enums and optional arguments. Nine negative cases require the relevant
diagnostic code and source line. Controls erase a parameter type to `any` and
remove the Library: the expected checks must then stop matching. Ketho's open
namespaces do not justify an undefined-member assertion; named ScriptObject
members are checked separately.

The string-enum fixture checks allowed and disallowed literals, return types and
an optional alias. Replacing the closed alias with `string` should remove only
the corresponding closed-literal diagnostic. Unexpected positive-program or
library errors, unrelated warnings, wrong locations and consumer crashes are
not passing evidence. Keep these semantics when changing the producer.

## Run the existing external probe

Supply the explicit consumer executables and their current expected hashes:

```sh
export WDF_EMMY_CHECK=/path/to/emmylua_check
export WDF_EMMY_CHECK_SHA256=sha256:<expected-executable-hash>
export WDF_LUALS=/path/to/luals/bin/lua-language-server
export WDF_LUALS_SHA256=sha256:<expected-executable-hash>
export WDF_CONSUMER_OUTPUT=/path/to/new-report-directory
cargo test --locked -p wow-reference-builder --test consumers -- --ignored --nocapture
```

`both_consumers_interpret_generated_library` is ignored in normal test runs.
The external checker run still requires the explicit executables above; moving
the test does not claim it was executed. Process isolation, time/output limits
and hashes are harness behavior, not product configuration or proof of authorship.
No whole-corpus or all-editor campaign is required for a bounded change. See the [Ketho semantics](KETHO_RUST_PORT.md) and
[type lowering](../crates/wow-annotations/TYPE_LOWERING.md).
