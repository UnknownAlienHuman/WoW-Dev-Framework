# `apps/wow` contract router

**Status:** documentation is implementation-ready through E7-A; no Rust code exists.

`apps/wow` is the thin executable/transport host over `wow-service`. Its only framework dependency is `wow-service`.

## Routes

- **E0-F:** root contract — `wow status`, `wow check`.
- **E3-C:** [`e3/README.md`](e3/README.md) — context commands.
- **E4-C:** [`e4/README.md`](e4/README.md) — search, lineage, migration validation and impact.
- **E5-B:** [`e5/README.md`](e5/README.md) — calibration source/corpus/run/review/holdout/submission commands.
- **E5-C:** [`e5c/README.md`](e5c/README.md) — core-pack artifact/signing/publication/canary/rollout/activation/LKG/rollback/closure.
- **E6-B:** [`e6/README.md`](e6/README.md) — external provider/query/result/mapping/selection/context commands.
- **E7-A:** [`e7/README.md`](e7/README.md) — one-shot CLI, foreground local daemon, LSP 3.18 stdio, MCP 2025-11-25 stdio and optional disabled-by-default local Streamable HTTP.

The app parses strict bounded transport input, initializes one exact protocol/registry profile, constructs one typed service request per semantic command/method/tool call, emits exact or lossless protocol output, and maps lifecycle/cancellation/reconnect/backpressure without owning domain semantics.

It does not resolve catalogs/current, select providers/candidates/mappings, discover tools/processes/workspaces, read credentials/provider databases/source, access lower owners, build context locally, retry unknown effects, apply edits automatically, mutate editor settings, expose remote listeners, or publish/install releases.

```text
documentation frontier: E7-A
implementation frontier: not-started
next documentation package: E7-B release/distribution lifecycle
Cargo.toml: absent
Rust source: absent
CI/workflows: absent
```