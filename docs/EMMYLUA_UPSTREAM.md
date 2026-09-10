# EmmyLua upstream compatibility

The upstream Rust parser dependency follows an updateable branch. Cargo.lock
identifies one tested build, not a permanently current revision. CI runs locked
builds and an independently updated dependency graph using `cargo update`, then
runs reference and annotation tests against that graph. No manifest editing or
static upstream-symbol scraping is needed to pretend a compile probe ran.

For an explicit local checkout:

```sh
cargo xtask check-source /path/to/emmylua-analyzer-rust main
```

This is a read-only public HTTPS remote-head check. A difference offers a reviewed
update; network failure is unverified-current, never success. Automatic checkout
management, the old source-surface report commands and their former interpreter
implementation have been retired, not silently replaced by equivalent claims.

The maintained workflows are listed in [.github](../.github/README.md). The
rolling parser lane deliberately isolates the branch-tracking parser consumer
from `wow-emmy`'s separately exact-pinned analyzer graph, then tests and vendors
that effective workspace. `wow-emmy` now executes the pinned analyzer for bounded
Lua/doc syntax diagnostics over explicit snapshots. This does not yet prove
combined Main/Library resolution, reference/local-flow facts, or full E0-C
acceptance. Current source checks are not a supported release/install or
client-runtime gate.
