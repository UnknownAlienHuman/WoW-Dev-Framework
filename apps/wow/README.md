# `apps/wow` contract router

**Status:** documentation is implementation-ready through E5-B; no Rust code exists.

`apps/wow` is a thin CLI adapter. Its only framework dependency is `wow-service`.

## Routes

- **E0-F:** root [`CONTRACT.json`](CONTRACT.json) — `wow status`, `wow check`.
- **E3-C:** [`e3/README.md`](e3/README.md) — context commands.
- **E4-C:** [`e4/README.md`](e4/README.md) — search, lineage, migration validation, impact commands.
- **E5-B:** [`e5/README.md`](e5/README.md) — calibration source/corpus/split/run/candidate/review/holdout/promotion-submission/deactivation validation commands.

The app parses strict bounded transport input, constructs one typed request, passes exact IDs and permitted symbolic selectors mechanically, invokes `wow-service` exactly once, emits exact/faithful output, and maps statuses to frozen exit codes.

It does not resolve catalogs/current, acquire owner views, inspect source or hidden holdout data, authorize reviewers, open vaults, run domain algorithms, apply migrations, edit source, publish packs, or invoke models/tools.

```text
documentation frontier: E5-B
implementation frontier: not-started
next documentation package: E5-C
Cargo.toml: absent
Rust source: absent
CI/workflows: absent
```