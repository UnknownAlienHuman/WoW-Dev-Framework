# Annotation consumer probes

`crates/wow-annotations/tests/consumers.rs` runs two existing semantic slices
against the generated native library using real EmmyLua (`emmylua_check`) and
LuaLS command-line consumers. The probes check analyzer feedback, not just Lua
parsing. They are focused examples and do not certify the complete Blizzard
library, arbitrary addons, editors, or runtime behavior.

## Signature probe

The positive program exercises ordered multiple returns, structure fields,
ScriptObject receivers/methods, arrays, enum parameters, and optional/nil
arguments. Nine negative cases expect the specified diagnostic code and source
line at error/warning severity. Unexpected diagnostics in the generated library
or positive fixture fail. Empty output, unrelated warnings, wrong positions, or
a crashing consumer do not count as a match.

Two controls mutate the generated input: erase a parameter type to `any`, then
remove the library. Each mutation must make the expected negative check fail.
This confirms that the consumer used the library's types. `open-namespace.lua`
records that Ketho's namespaces are open tables: a missing member there is not a
reliable generic error. An unknown method on a named ScriptObject is tested
separately.

## Closed string-enum probe

The catalog case generates one synthetic Ketho-style alias resource in a
separate test-owned Git input and passes it through `--alias-catalog`. Positive
code checks both allowed literals, a literal-union return, and a named optional
alias. Four negative cases check an unlisted string, number-to-enum assignment,
enum-return-to-number assignment, and boolean-to-optional-alias assignment.
The control replaces the closed alias with `string`: only the unlisted-string
diagnostic should disappear; the other three errors remain. Synthetic values
are type-semantics fixtures, not a WoW client inventory.

## Run the external probe

Provide paths to approved, materialized analyzer distributions and new output
directory (or omit output to use a removed-after-run temporary directory):

```sh
export WDF_EMMY_CHECK=/path/to/emmylua_check
export WDF_EMMY_CHECK_SHA256=sha256:<approved-executable-hash>
export WDF_LUALS=/path/to/luals/bin/lua-language-server
export WDF_LUALS_SHA256=sha256:<approved-executable-hash>
export WDF_CONSUMER_OUTPUT=/path/to/new-report-directory
cargo test --locked -p wow-annotations --test consumers -- --ignored --nocapture
```

The ignored `both_consumers_interpret_generated_library` test runs both
signature and catalog slices for EmmyLua and LuaLS. Its fixed checker calls use
the Emmy `--config`, `--output-format json`, `--output`, and
`--warnings-as-errors` options; LuaLS uses `--check`, `--configpath`,
`--checklevel=Hint`, `--check_format=json`, and `--check_out_path`. Normal offline
workspace tests leave this external-process test ignored;
the dedicated Linux and Windows consumer jobs run both slices. Missing binaries
or required hash inputs fail rather than silently skip.

The current harness also isolates child environments, checks package/executable
inputs, bounds reports and process time, and has Windows path handling. Those
runtime controls still exist; simplification is tracked separately in Issues
#193, #196, and #203. A hash identifies bytes, not trustworthiness or authorship.

## What these probes do not establish

They do not certify full-corpus type closure, every custom signature, all widget
inheritance, hover/definition mappings, incremental editor behavior, production
analyzer/service integration, or runtime Secret/combat behavior. For a bounded
semantic change, run one relevant positive and negative consumer case. Broader
consumer qualification is needed when claiming broader coverage, not as a
prerequisite for every patch. Keep the detailed annotation semantics in the
[Ketho port map](KETHO_RUST_PORT.md) and [type-lowering guide](../crates/wow-annotations/TYPE_LOWERING.md).
