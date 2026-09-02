# `wow-service` contract router

**Status:** documentation is implementation-ready through E7-A; no Rust code exists.

`wow-service` is the only production crate allowed to coordinate multiple framework owners into one transport-independent public operation. It validates exact identities, acquires retained views, sequences narrow ports, maintains durable effects, and emits canonical envelopes. It never reimplements owner algorithms or wire protocols inside domain owners.

## Routes

- **E0-F:** root E0 contract — `status`, `check`.
- **E1-D:** [`e1/README.md`](e1/README.md) — Reference Pack build/validation.
- **E3-C:** [`e3/README.md`](e3/README.md) — context acquisition/use cases.
- **E4-C:** [`e4/README.md`](e4/README.md) — search/lineage/migration/impact orchestration.
- **E5-B:** [`e5/README.md`](e5/README.md) — calibration runs, review authorization, sealed holdout, promotion submissions.
- **E5-C:** [`e5c/README.md`](e5c/README.md) — core-pack artifact/signing/publication/canary/activation/rollout/rollback/closure.
- **E6-B:** [`e6/README.md`](e6/README.md) — optional provider/session/result/mapping/selection/exact-root context orchestration.
- **E7-A:** [`e7/README.md`](e7/README.md) — closed frontend operation registry, sessions, explicit workspaces, project-owned overlays, local daemon, LSP 3.18, MCP 2025-11-25, cancellation/reconnect/backpressure and isolation.

## E7-A dependency rule

E7-A invokes only implemented capabilities required by the exact registry entry. The maximum reviewed service slice is:

```text
wow-core
wow-store
wow-reference
wow-emmy
wow-project
wow-graph
wow-rules
wow-search
wow-context
wow-cbm
```

E5 effecting operations remain under their own authorization profiles and are absent from default LSP/MCP exposure.

## Service responsibilities

Service may:

- validate strict public requests and exact profiles;
- resolve explicitly permitted symbolic selectors once;
- acquire exact retained owner views in fixed order;
- invoke narrow owner/authorization/session/signing/provider ports;
- maintain `OperationId + CanonicalRequestDigest`, response-loss, retention, audit and closure state;
- preserve evidence, coverage, conflicts, blockers, omissions and nonclaims;
- publish immutable operation/registry/session/result records;
- map one reviewed frontend registry entry to one service operation;
- coordinate exact workspace/overlay owner operations and transport-neutral language features;
- close resources before returning a canonical envelope.

Service may not:

- parse source or provider results outside owning crates;
- open raw storage or expose physical handles;
- choose newest/best/first/sole/nearest/default artifacts, providers, mappings or candidates;
- infer authorization from GitHub/OS/editor/client/model identity;
- expose generic MCP/RPC/tool/shell/plugin/model execution;
- inject provider or transport metadata into exact semantic truth;
- advertise unimplemented capabilities;
- treat disconnect/progress as cancellation/completion;
- publish public release channels or install updates;
- add background work or CI by convention.

```text
documentation frontier: E7-A
implementation frontier: not-started
next documentation package: E7-B reproducible packaging, distribution, update and support lifecycle
Cargo.toml: absent
Rust source: absent
CI/workflows: absent
```