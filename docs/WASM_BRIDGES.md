# Native literal path; Wasm route being removed

ADR-058 removes independently updateable Wasm algorithms from product scope.
[#203](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/issues/203) owns the
code removal. The current guest/host still exists until that task lands.

Keep the live literal algorithm in `crates/wow-ketho-literals` and its native
annotation callers. Remove `modules/ketho-literals`, `bridges/literal-host`,
VM-only request/digest/adapter layers, selection flags and exclusive CI/tests.
Do not delete the native emitter with its Wasm shell or leave a disabled plugin
fallback. The task names callers and the representative literal check.

Use [KETHO_RUST_PORT.md](KETHO_RUST_PORT.md) for the native path. Previous bridge
protocol/acceptance details remain in Git history, not active requirements.
