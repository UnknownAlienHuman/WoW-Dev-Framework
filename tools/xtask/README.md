# Native repository maintenance

`xtask` is internal Rust development tooling. Product workflows use `wow` or the
native builder; [architecture](../../docs/ARCHITECTURE.md) sets their scope.

## Routine checks

```sh
cargo xtask check
cargo xtask sync-skill --check
cargo xtask sync-skill --write
```

`check` reads tracked/untracked distributable files, checks paths/regular-file
and size bounds, native-only policy, JSON syntax and duplicate decoded keys,
and identical skill copies. `check`/`sync-skill` accept `--root DIR`.
The old E3/seven-owner partial/pending snapshot validators are removed. There is
no replacement contract, fixture-checksum or package-certification gate.

Skill writes use the existing bounded preflight/lock/replacement path. A failed
sync is an error; repeat explicit sync after reconciling state. Checks do not
rewrite expected product fixtures.

## Source and native output tools

```sh
cargo xtask check-source /path/to/checkout live
cargo xtask update-source /path/to/checkout live --expected-head <observed-local-SHA>
cargo xtask materialize-source /path/to/request.json
cargo xtask materialize-source-api /path/to/request.json
cargo xtask manifest /path/to/wow-ui-source HEAD live /path/to/new-manifest.json
cargo xtask verify-manifest /path/to/manifest.json /path/to/wow-ui-source HEAD
cargo xtask verify-library /path/to/native-output --require-input-complete
```

These are existing developer commands. Use them only for the selected source or
annotation task; they are not prerequisites for generic code or documentation.
Read the relevant [checkout update](../../docs/SOURCE_CHECKOUT_UPDATES.md),
[materialization](../../docs/SOURCE_MATERIALIZATION.md) or
[API snapshot](../../docs/SOURCE_API_MATERIALIZATION.md) recipe when using that
operation. Preserve dirty/diverged checkouts and respect auto/prompt/never mode.
Do not execute indexed source, hooks or repository scripts, or blindly delete an
interrupted/foreign operation's files.

`verify-library` checks the current native artifact/report's recorded files and
source maps. It does not prove every annotation's meaning in an editor. Use the
[consumer guide](../../docs/ANNOTATION_CONSUMER_PROBES.md) only for relevant
semantics. Missing/partial input is not a clean complete library.

Some commands still implement old import/report versions, hashes and VM-specific
verification. That is existing code awaiting #112/#196/#203, not a compatibility
or VM support commitment. Remove obsolete validators with their producer/callers;
do not add another report version or test matrix to preserve them. Native
literal generation and useful source/output checks remain.
