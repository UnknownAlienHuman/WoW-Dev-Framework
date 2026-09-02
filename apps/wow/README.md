# `apps/wow` contract router

**Status:** documentation is implementation-ready through E6-B; no Rust code exists.

`apps/wow` is a thin CLI adapter. Its only framework dependency is `wow-service`.

## Routes

- **E0-F:** root contract — `wow status`, `wow check`.
- **E3-C:** [`e3/README.md`](e3/README.md) — context commands.
- **E4-C:** [`e4/README.md`](e4/README.md) — search, lineage, migration validation and impact.
- **E5-B:** [`e5/README.md`](e5/README.md) — calibration source/corpus/run/review/holdout/submission commands.
- **E5-C:** [`e5c/README.md`](e5c/README.md) — core artifact/signing/publication/canary/rollout/activation/LKG/rollback/revocation/closure commands.
- **E6-B:** [`e6/README.md`](e6/README.md) — external provider/query/result/mapping/selection/context commands.

The app parses strict bounded transport input, constructs one typed request, passes exact IDs and guards mechanically, invokes service exactly once, emits exact/faithful output, and maps exact states to exit codes.

It does not resolve catalogs/current, select providers/candidates/mappings, discover tools/processes, read credentials/provider databases/source, access owner stores, run domain algorithms, build context, retry unknown effects, edit graph/project state, or distribute public releases.

```text
documentation frontier: E6-B
implementation frontier: not-started
next documentation package: E7-A after the transport/release owner contract
Cargo.toml: absent
Rust source: absent
CI/workflows: absent
```