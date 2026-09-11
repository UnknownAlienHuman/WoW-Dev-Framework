# `wow-service` implementation status

## Implemented

- Versioned operation registry with `rules.evaluate@1`.
- Stable request, success, and rejection envelopes.
- Canonical service-snapshot and response identities.
- One immutable snapshot containing exact project, Reference, target-profile, rule-input, and rule-report identities.
- Atomic candidate publication with stale-current rejection, last-known-good preservation, old-view immutability, and no-change `Arc` preservation.
- Single-snapshot request execution; no operation can observe mixed generations.
- Explicit unknown-operation, unsupported-version, and stale-snapshot rejection.
- Deterministic repeated execution and serialization.

## Deliberately outside this crate

- Filesystem discovery, source acquisition, parsing, analysis, Reference persistence, and project mutation.
- Network listeners, daemon lifecycle, LSP/MCP transport, editor integration, authentication, and authorization.
- Runtime/client/combat/taint/secure-execution conclusions.
- Release installation, updates, rollback, and platform packaging.

## Next owner package

`apps/wow` must provide a bounded one-shot CLI that builds an exact project/Reference input, publishes one service snapshot, executes a registered operation, writes one canonical result, and maps structural failures to stable exit codes. Long-running transports remain later work.
