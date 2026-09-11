# `wow-service`

`wow-service` owns the stable E0-F in-process operation boundary. It captures one immutable evidence snapshot per request, exposes a versioned operation registry, and returns deterministic request/result envelopes. It is not a CLI, daemon, LSP server, MCP server, source loader, database, or live-client bridge.

## Implemented operation

`rules.evaluate@1` returns the already validated E0-E rule report bound to one service snapshot. The snapshot contains one project snapshot/generation, one Reference view, one target profile, and one canonical rule report.

A request must name the exact expected service snapshot. Stale snapshots, unknown operation IDs, and unsupported versions return explicit rejection envelopes with no result payload. Repeated execution against the same snapshot and request is byte-deterministic.

Candidate publication evaluates and content-addresses the complete rules input before acquiring the publication lock. A stale expected-current ID cannot move the current pointer. Successful publication swaps one `Arc`; previously captured read views remain immutable, and a no-change publication preserves the same `Arc`.

The service does not promote analyzer observations into WoW API, Secret, runtime-safety, combat, taint, or replacement authority.
