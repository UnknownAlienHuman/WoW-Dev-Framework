# Development tools

## `xtask`

`xtask` is the active repository/source maintenance binary, not a product or
alternate service implementation. See [`tools/xtask/README.md`](xtask/README.md)
for scope and request formats.

Common checks and source workflows:

```sh
cargo xtask check
cargo xtask sync-skill --check
cargo xtask check-source /path/to/checkout live
cargo xtask manifest /path/to/wow-ui-source HEAD live /path/to/new-manifest.json
cargo xtask verify-manifest /path/to/manifest.json /path/to/wow-ui-source HEAD
```

Other explicit commands include guarded source updates, managed checkout or API
materialization, and native annotation-library verification. They operate on
selected inputs; they do not execute indexed Lua, repository scripts, hooks, or
submodules. `check` is repository hygiene, not product or release certification.

Do not add a second parser, semantic implementation, generic shell/plugin runner,
release executor, or hidden source/network discovery. A tool needs a concrete
current developer workflow and should delegate to its owning crate or the
existing fixed maintenance path.
