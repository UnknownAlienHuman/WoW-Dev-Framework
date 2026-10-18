# EmmyLua engine and server boundary

The workspace pins the compiled EmmyLua revision in `Cargo.toml` and
`Cargo.lock`. `wow-emmy` adapts the public `emmylua_code_analysis` and parser
APIs. The [analysis route](IMPLEMENTATION.md#addon-configuration-and-emmy-analysis)
identifies the duplicate engines and Library/diagnostic gap; #193 joins their
work. EmmyLua remains the only Lua parser/analyzer.

## Engine setup

Read the pinned donor's `crates/emmylua_code_analysis/src/lib.rs` and
`src/resources/mod.rs`. `new()` loads no standard declarations; workspace-root
registration does not load files. Default runtime is `LuaLatest`, so keep the
existing explicit `Lua51` configuration before registering/analyzing input.
Current framework constructors load only their supplied snapshots and do not
call `init_std_lib`.

For normal addon setup in #200, use the shared engine from #193: apply Lua 5.1
and the intended diagnostic settings, load upstream declarations once with
`init_std_lib(None)`, then register/load the selected Main and Library inputs.
At the compiled pin, `None` loads embedded resources without extracting them to
disk. This is required setup work, not a claim that the current CLI does it.
Do not add a standard-library catalog, resource installer or configuration flag.

Upstream STD files are not supplied addon/Library sources. Existing `bindings`,
`function_calls` and `global_access` adapters require a registered source before
emitting a target; preserve their missing-target states for STD rather than
inventing an addon span. Standard Lua declarations do not establish WoW runtime
availability. Diagnose Main while keeping Library health and virtual-source
mapping explicit; see [diagnostic normalization](../crates/wow-emmy/DIAGNOSTIC_NORMALIZATION.md).

## File updates

Public `update_files_by_path`/`update_files_by_uri` update supplied files.
`LuaCompilation::remove_index` removes their index entries; `update_index`
analyzes the passed file IDs. This path does not schedule unchanged global/type
consumers. `require()` dependency ordering is not a complete invalidation graph.
Follow the [Project update notes](../crates/wow-project/UPDATE_MODEL.md) before
narrowing reindexing; retain the existing cross-file deletion check.

## Editor features

Use upstream's `emmylua_ls` server for general Lua hover, completion, definition,
references, rename, signature help, symbols and diagnostics. Its
`crates/emmylua_ls/src/handlers/` modules are private, not an importable SDK.
`emmylua_code_analysis` exposes the semantic engine. Do not copy the full handler
stack; framework LSP operations stay limited to useful WoW project behavior.

## Pin changes

Ordinary adapter work uses the lockfile pin. For a concrete pin/API change,
read the used public APIs, update both workspace Emmy dependencies and the
lockfile, and align `wow-emmy/src/syntax.rs` plus affected
`compatibility/compiled_probe.rs` metadata. Preserve notices and check the
changed Main/Library, diagnostic, coordinate or update behavior with existing
tests. Use the existing rolling parser CI for its scoped check.

The `wow-emmy-probe` report helps investigate an actual compatibility question;
it is not a setup prerequisite or certification of arbitrary reports, editors
or runtimes. No all-fixture checksum freeze or replacement acceptance registry.
For an explicitly selected local donor checkout, the read-only freshness check is:

```sh
cargo xtask check-source /path/to/emmylua-analyzer-rust <branch-or-revision>
```

A floating branch, successful build or source-freshness check does not prove
behavioral compatibility. Network failure leaves freshness unverified. Never
execute Lua source to check the analyzer.
