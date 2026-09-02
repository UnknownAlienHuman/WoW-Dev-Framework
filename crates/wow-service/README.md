# `wow-service` contract router

**Status:** documentation is implementation-ready through E5-C; no Rust code exists.

`wow-service` is the only production crate allowed to coordinate multiple framework owners into one transport-independent operation. It validates exact identities, acquires retained views, sequences narrow ports, maintains durable effects, and emits canonical envelopes. It never reimplements owner algorithms.

## Routes

- **E0-F:** root E0 contract — `status`, `check`.
- **E1-D:** [`e1/README.md`](e1/README.md) — Reference Pack build/validation.
- **E3-C:** [`e3/README.md`](e3/README.md) — context acquisition/use cases.
- **E4-C:** [`e4/README.md`](e4/README.md) — search/lineage/migration/impact orchestration.
- **E5-B:** [`e5/README.md`](e5/README.md) — calibration runs, review authorization, sealed holdout, promotion submissions.
- **E5-C:** [`e5c/README.md`](e5c/README.md) — independent submission revalidation, immutable core-pack publication/signing, canary, guarded activation, rollout, rollback, revocation and partition closure.

## Active E5-C dependency slice

```text
wow-core
wow-store
wow-project
wow-graph
wow-recognizers
```

The service may validate requests/profiles, resolve permitted symbolic selectors once, acquire exact retained resources, invoke narrow owner/authorization/signing/observation ports, record idempotency and response-loss receipts, preserve blockers/nonclaims, and close resources before returning.

It may not parse source, open raw storage, reproduce analyzer/recognizer/graph/project/store algorithms, choose newest/best/previous/default targets, infer authorization from local/GitHub identity, expose credentials/private cohort data, mutate E5-A/B evidence, or publish public release channels.

```text
documentation frontier: E5-C
implementation frontier: not-started
next documentation package: E6-A optional external semantic candidate bridge
Cargo.toml: absent
Rust source: absent
CI/workflows: absent
```