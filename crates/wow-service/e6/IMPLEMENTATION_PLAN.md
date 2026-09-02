# E6-B implementation plan

**Status:** normative order; implementation has not started.

0. Freeze implemented E0–E6-A prerequisite commits, fixtures, profiles, adapters, measured limits, and checksums.
1. Implement closed E6-B request/status/error/envelope primitives and exact selectors.
2. Implement provider configuration catalog validation and one-time selector resolution.
3. Implement credential-use authorization references and narrow provider session acquisition/close ports without exposing secrets or commands.
4. Implement durable query registration, dispatch receipts, cancellation, response-loss reconciliation, and no-blind-retry guards.
5. Integrate E6-A descriptor/capability/state/query/result validation without reproducing normalization logic.
6. Implement immutable result/artifact catalog publication, retention, fresh read-back validation, get/list/continuation/cache validation.
7. Implement project and reference locator-mapping owner ports plus exact mapping publication and negative-authority checks.
8. Implement immutable explicit selection requests/receipts/supersession without implicit ranking selection.
9. Reuse internal E3 exact view acquisition and invoke one `wow-context` operation with the exact mapped root.
10. Implement separate external Candidate sidecar and combined result publication without semantic evidence mixing.
11. Implement independent lane statuses, privacy/license/redaction, retention/audit, reverse close, startup recovery, and quarantine.
12. Activate thin `apps/wow/e6` only after service request/result bytes and exit mappings freeze.
13. Run E0–E6-A regressions; real adapter contract probes; synthetic stable/mutable/opaque provider cases; mapping ambiguity/no-authority/conflict; selection and context cases; response loss at every effect; cancellation; 1/2/N worker/order/storage variants; and measured resource benchmarks.
14. Populate all implementation/profile/adapter/vector/checksum fields and update manifests only after fresh passes.

No fake credential vault, provider process, generic MCP client, automatic selector, owner mapping, context fact injection, runtime claim, or CI placeholder is allowed. E7 transports and public distribution remain deferred.