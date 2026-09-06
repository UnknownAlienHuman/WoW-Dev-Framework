# Executable annotation consumer probes

This is the first scoped semantic acceptance slice of
[`PARITY_AND_CONSUMER_PROBES`](../crates/wow-annotations/e1/PARITY_AND_CONSUMER_PROBES.md),
not the full E1-C contract or the production `wow-emmy` adapter.

`crates/wow-annotations/tests/consumers.rs` generates a synthetic documentation
corpus through the real reference loader and Ketho Rust emitter. Separate small
test modules own fixtures, fixed process calls, diagnostic normalization, package
identity and mutations. No production crate, dependency or WASM ABI changes.
External analyzer IO stays in the test adapter; it cannot become a host capability
of the replaceable literal module.

## Covered semantics

The same generated library is checked by EmmyLua's `emmylua_check` and LuaLS's
native command-line diagnosis mode. Positive code uses ordered multiple returns,
structure fields, ScriptObject receivers and methods, arrays, enum parameters and
optional/nil arguments. Nine negative cases must report the expected diagnostic
code and source line at error/warning severity. Library and positive-fixture
diagnostics at any severity fail. Empty output, unrelated warnings, wrong spans
or a crashing consumer cannot satisfy the negative gate.

Both consumers also run two deliberate mutations: erase a parameter type to `any`,
and remove the generated library. The same gate must reject each mutation.
These checks prove actual type feedback rather than just parser acceptance or
lack of diagnostics. Unit tests check failure modes of the probe itself.

Ketho emits namespaces as open tables. A missing namespace member is therefore
**not** a reliable generic-consumer error. `open-namespace.lua` records this
observation without making an API-existence claim or a mandatory clean assertion.
Exact API absence still needs source-owned coverage and the planned WoW rule.
An unknown method on a named ScriptObject is a separate required negative case.

## Execution and inputs

Supply absolute paths to already approved, materialized consumer distributions:

```sh
export WDF_EMMY_CHECK=/path/to/emmylua_check
export WDF_EMMY_CHECK_SHA256=sha256:<approved-executable-hash>
export WDF_LUALS=/path/to/luals/bin/lua-language-server
export WDF_LUALS_SHA256=sha256:<approved-executable-hash>
export WDF_CONSUMER_OUTPUT=/path/to/new-report-directory
cargo test --locked -p wow-annotations --test consumers -- --ignored --nocapture
```

The optional output directory must not exist; without it a temporary directory
is removed after the run. Missing binaries/hashes fail, not silently skip. Normal
offline workspace tests mark this external test ignored; the dedicated Linux and
Windows CI jobs explicitly execute it, including both mutation controls.

CI resolves each named upstream's current non-prerelease once, verifies its
reported asset SHA-256 before extraction, and records the release, asset and
executable identities. Asset layout or behavior changes fail the compatibility
job; there is no fallback to another release. No permanent consumer version is
compiled into the framework. Updating a compatible consumer input needs no host
or guest rebuild. This test approval does not promote or distribute product code.

## Isolation and limits

The adapter uses only fixed checker command shapes, cleared child environments,
test-owned homes and explicit JSON configuration. There are no Lua configurations,
added globals, diagnostic suppression, user editor settings or extension installs.
Generated files/configs are hashed before and after. LuaLS's complete shipped
resource tree is fingerprinted as well as its executable; changes fail the test.
Archive/executable hashes identify bytes, not signing or authorship.

Reports are newly created, bounded to 8 MiB, limited to 4,096 diagnostics, and
normalize local filenames to fixture-relative paths. Positions must fit the
selected file; temp host paths and diagnostic prose are excluded from the stable
summary. The direct checker process has a 90-second deadline and is reaped on
failure. This is an approved-tool harness, not an OS sandbox: descendant process,
network, total log/disk and address-space enforcement require an external runner
policy. Never run an untrusted executable under a hash-only trust assumption.

## Remaining gates

Full live-corpus type closure, all custom signatures and widget inheritance,
hover/definition source mappings, incremental editor operations, runtime Secret
behavior and production analyzer/service integration remain unimplemented or
NotEvaluated. Passing this closed synthetic slice does not certify every generated
Blizzard declaration, an actual addon or a public release. Raw CLI JSON formats
are consumer-specific; a breaking schema change must update a reviewed adapter.
