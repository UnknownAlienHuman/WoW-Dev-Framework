# EmmyLua engine and server boundary

The workspace pins one compiled EmmyLua analyzer revision in `Cargo.toml` and
`Cargo.lock`. `wow-emmy` adapts the public `emmylua_code_analysis` and parser
APIs; it reuses an `EmmyLuaAnalysis` session for syntax, resolution, calls,
locals, diagnostics, and incremental updates. Keep that upstream engine as the
only Lua parser/analyzer. Do not add a second parser, interpreter, or analysis
cache. Ordinary adapter work uses the lockfile pin; change it only for a
concrete compatibility or current-source task, with the relevant dependency and
behavior checked.

## Editor features

Upstream's `emmylua_ls` crate contains and registers the LSP request handlers in
`crates/emmylua_ls/src/handlers/`, including hover, completion, definition,
references, rename, signature help, document symbols, and diagnostics. These
handler modules are private to that crate. `emmylua_code_analysis` exposes the
semantic engine, not a public importable handler SDK. For standard Lua editor
features, use the upstream `emmylua_ls` server. Do not port or reimplement its
full LSP handler stack in this framework. Framework-owned LSP operations remain
limited to their distinct WoW project/reference contracts; add a narrow bridge
only for a demonstrated missing product operation.

## Pin and compatibility checks

Use the existing rolling CI parser lane for its scoped upstream-consumer check.
For a deliberate pin change, follow [`PIN_AND_PROBE.md`](../crates/wow-emmy/PIN_AND_PROBE.md)
and preserve the selected revision, license attribution, and required behavior
checks. A source freshness check or successful compile alone does not prove
behavioral compatibility. The `wow-emmy-probe` backend report is useful when a
pin/API compatibility question calls for it; generating, hashing, or verifying
that report is not a prerequisite for ordinary implementation work. Reports
describe observations for their named inputs and pin; they do not authenticate
arbitrary reports or certify an entire editor/runtime matrix.

For an explicit local upstream checkout, the read-only check is:

```sh
cargo xtask check-source /path/to/emmylua-analyzer-rust <branch-or-revision>
```

Network failure leaves freshness unverified. Never infer that a floating branch
is the compiled dependency. Keep Lua input inert; never execute source while
checking the analyzer.
