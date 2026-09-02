# `wow-service` contract router

**Status:** documentation is implementation-ready through E6-B; no Rust code exists.

`wow-service` is the only production crate allowed to coordinate multiple framework owners into one transport-independent operation. It validates exact identities, acquires retained views, sequences narrow ports, maintains durable effects, and emits canonical envelopes. It never reimplements owner algorithms.

## Routes

- **E0-F:** root E0 contract — `status`, `check`.
- **E1-D:** [`e1/README.md`](e1/README.md) — Reference Pack build/validation.
- **E3-C:** [`e3/README.md`](e3/README.md) — context acquisition/use cases.
- **E4-C:** [`e4/README.md`](e4/README.md) — search/lineage/migration/impact orchestration.
- **E5-B:** [`e5/README.md`](e5/README.md) — calibration runs, review authorization, sealed holdout, promotion submissions.
- **E5-C:** [`e5c/README.md`](e5c/README.md) — submission revalidation, immutable core-pack publication/signing, canary, activation, rollout, rollback, revocation and partition closure.
- **E6-B:** [`e6/README.md`](e6/README.md) — provider/session orchestration, durable external Candidate results, exact project/reference mapping, explicit selection and exact-root context handoff.

## Active E6-B dependency slice

```text
wow-core
wow-store
wow-project
wow-reference
wow-graph
wow-context
wow-cbm
```

The service may validate requests/profiles, resolve permitted symbolic selectors once, acquire exact retained resources, invoke narrow owner/authorization/session ports, record idempotency and response-loss receipts, preserve blockers/nonclaims, and close resources before returning.

For E6-B it may acquire a reviewed external provider session through nonsecret configuration/authorization references, invoke E6-A, publish immutable result artifacts, ask project/reference owners to map locators, record one explicit caller selection, and invoke existing context owners with one exact mapped root.

It may not parse source/provider results, open raw storage, reproduce lower-owner algorithms, choose newest/best/first/sole/nearest/default targets, inspect credentials/provider databases, expose generic MCP/tools, infer authorization from local/GitHub identity, inject external metadata into exact context truth, or publish public release channels.

```text
documentation frontier: E6-B
implementation frontier: not-started
next documentation package: E7-A transport/session surface and developer-preview release contract
Cargo.toml: absent
Rust source: absent
CI/workflows: absent
```