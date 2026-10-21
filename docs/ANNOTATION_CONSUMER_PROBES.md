# Annotation consumer probes

These tests check selected type semantics with real EmmyLua (`emmylua_check`)
and LuaLS command-line consumers. They do not establish full-corpus parity or
that the supported generator-to-addon workflow works.

## Actual producer under test

At `1a87ce15`, `tests/consumer_support/source.rs` includes
`../../examples/support/mod.rs`; `catalog_source.rs` calls that same driver.
It is not `apps/wow-reference-builder::run_with_stop`. The old driver emits
`wow-native-source-build/1` with an embedded library, while the supported command
emits `/2` with a smaller report. The source-report assertions expect `/1`.
The two drivers also contain identical Git-I/O and catalog helper files.

The probes remain useful evidence about the shared renderer and selected source
semantics. They do **not** verify the public command's argument handling, exit
codes, cancellation, report format or integration with ordinary `wow check`.
For #201, move application entrypoint checks to the builder's integration tests
and reuse its actual producer. Keep lower-level renderer checks in the annotation
crate. Remove the superseded driver/helpers after moving their useful cases;
do not add an application dependency to a lower owner or a third driver.

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
cargo test --locked -p wow-annotations --test consumers -- --ignored --nocapture
```

`both_consumers_interpret_generated_library` is ignored in normal test runs.
This command still uses the old producer described above; no execution is claimed
by this documentation update. Existing process isolation, time/output limits and
hash requirements are harness behavior, not additional product configuration or
proof of trusted authorship. No whole-corpus or all-editor campaign is required
for a bounded change. See the [Ketho semantics](KETHO_RUST_PORT.md) and
[type lowering](../crates/wow-annotations/TYPE_LOWERING.md).
