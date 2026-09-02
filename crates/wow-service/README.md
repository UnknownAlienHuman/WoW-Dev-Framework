# `wow-service` contract router

**Status:** documentation is implementation-ready through E5-B; no Rust code exists.

`wow-service` is the only production crate allowed to coordinate multiple framework owners into one transport-independent operation. It validates exact identities, acquires retained owner views, sequences narrow public ports, manages durable operation state, and builds canonical envelopes. It never reimplements owner algorithms.

## Routes

- **E0-F:** [`E0_F_OVERVIEW.md`](E0_F_OVERVIEW.md), root [`CONTRACT.json`](CONTRACT.json) — `status`, `check`.
- **E1-D:** [`e1/README.md`](e1/README.md) — Reference Pack build/validate/rebuild-compare.
- **E3-C:** [`e3/README.md`](e3/README.md) — exact context acquisition and use cases.
- **E4-C:** [`e4/README.md`](e4/README.md) — search, lineage, review, migration validation, static impact, context handoff.
- **E5-B:** [`e5/README.md`](e5/README.md) — calibration artifact acquisition, durable runs, review authorization, sealed holdout audit/consumption, promotion submissions.

## E5-B dependency slice

```text
wow-core
wow-store
wow-project
wow-graph
wow-recognizers
```

Other production crates remain inactive direct dependencies for E5-B. Their relevant immutable results arrive through owner artifacts.

## Common boundaries

Service may validate requests/profiles, resolve explicitly permitted symbolic selectors once, acquire exact retained resources in fixed order, invoke narrow owner/authorization/vault ports, persist idempotency and response-loss receipts, preserve evidence/coverage/conflicts/blockers/nonclaims, and close resources before returning.

Service may not parse source, open raw SQL/storage, implement analyzer/recognizer/graph/search/context/rule algorithms, select newest/best/first/sole artifacts, infer authorization from local/GitHub identity, expose credentials or hidden holdout data, mutate source/labels/splits/candidate bytes, or publish/activate/roll out/roll back a core pack.

```text
documentation frontier: E5-B
implementation frontier: not-started
next documentation package: E5-C
Cargo.toml: absent
Rust source: absent
CI/workflows: absent
```