# EmmyLua engine and server boundary

The workspace pins one compiled EmmyLua analyzer revision in `Cargo.toml` and
`Cargo.lock`. `wow-emmy` adapts the public `emmylua_code_analysis` and parser
APIs. The [analysis route](IMPLEMENTATION.md#addon-configuration-and-emmy-analysis)
identifies the current duplicate engines and Library/diagnostic gap; #193 joins
their work in one `EmmyLuaAnalysis`. Keep that upstream engine as the only Lua
parser/analyzer. Do not add a second parser, interpreter, or analysis
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
For a deliberate pin change, read the used public APIs at the selected revision;
update both workspace Emmy dependencies and the lockfile together. Align the
compiled identity in `wow-emmy/src/syntax.rs` and affected probe metadata in
`wow-emmy/src/compatibility/compiled_probe.rs`; preserve license attribution.
Check the affected Main/Library, diagnostic, coordinate or update behavior with
existing tests. No all-fixture checksum freeze or new acceptance registry.
A source freshness check or compile alone does not prove behavioral compatibility.
The `wow-emmy-probe` backend report is useful when a
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

The pinned public `EmmyLuaAnalysis` exposes `add_main_workspace`,
`add_library_workspace`, `update_files_by_path` and `diagnose_file`. Use those
operations for setup and checking. `new()` does not load standard declarations;
upstream provides separate `init_std_lib` if the selected setup needs them.
Do not recreate a standard-library catalog or silently change diagnostic settings
while consolidating sessions.
